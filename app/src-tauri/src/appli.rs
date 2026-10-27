use crate::backend;
use crate::controle;
use crate::machine;
use crate::modeles;
use crate::moteur::{self, Demande, MESSAGE_ANNULE, Moteur, Phase, Reference};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const RACINE_DEFAUT: &str = "D:\\Dev\\ia-eco-test";
pub const NOMBRE_MAX: u32 = 4;
pub const ECART_REFERENCE: f32 = 0.5;
const REFERENCE_TAILLE_MAX: usize = 15 * 1024 * 1024;
pub const MODELE_AFFICHE: &str = "Z-Image Turbo (GGUF Q5_0)";
const DELAI_GENERATION: Duration = Duration::from_secs(900);

static EN_COURS: AtomicBool = AtomicBool::new(false);
static ANNULER: AtomicBool = AtomicBool::new(false);
static SUIVI: Mutex<Suivi> = Mutex::new(Suivi { journal: None, image: 0, total: 0 });

struct Suivi {
    journal: Option<PathBuf>,
    image: u32,
    total: u32,
}

struct Verrou;

impl Verrou {
    fn prendre() -> Option<Verrou> {
        if EN_COURS.swap(true, Ordering::SeqCst) {
            None
        } else {
            ANNULER.store(false, Ordering::SeqCst);
            Some(Verrou)
        }
    }
}

impl Drop for Verrou {
    fn drop(&mut self) {
        EN_COURS.store(false, Ordering::SeqCst);
        if let Ok(mut s) = SUIVI.lock() {
            *s = Suivi { journal: None, image: 0, total: 0 };
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Etat {
    pub installe: bool,
    pub racine: String,
    pub modele: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fiche {
    pub prompt: String,
    pub format: String,
    pub largeur: u32,
    pub hauteur: u32,
    pub graine: i64,
    pub duree_s: f64,
    pub date: u64,
    pub modele: String,
    pub chemin: String,
    #[serde(default)]
    pub reference: Option<String>,
    #[serde(default)]
    pub ecart: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct Image {
    pub donnees: String,
    pub fiche: Fiche,
}

#[derive(Debug, Serialize)]
pub struct Lot {
    pub images: Vec<Image>,
    pub interruption: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Progression {
    pub en_cours: bool,
    pub phase: &'static str,
    pub etape: u32,
    pub total: u32,
    pub image: u32,
    pub images_total: u32,
    pub pourcentage: u32,
}

#[derive(Debug, Serialize)]
pub struct Memoire {
    pub gpu: String,
    pub ram_dispo_mo: u64,
    pub ram_totale_mo: u64,
    pub vram_libre_mo: Option<u64>,
    pub vram_mo: u64,
    pub seuil_ram_mo: u64,
    pub seuil_vram_mo: u64,
    pub suffisante: bool,
    pub message: Option<String>,
}

pub fn racine_depuis_exe(exe: &Path) -> Option<PathBuf> {
    let dossier = exe.parent()?;
    if dossier.file_name()?.eq_ignore_ascii_case("bin") {
        dossier.parent().map(Path::to_path_buf)
    } else {
        None
    }
}

pub fn racine() -> PathBuf {
    if let Ok(r) = std::env::var("IA_ECO_RACINE") {
        return PathBuf::from(r);
    }
    if let Some(r) = std::env::current_exe().ok().and_then(|e| racine_depuis_exe(&e)) {
        return r;
    }
    PathBuf::from(RACINE_DEFAUT)
}

pub fn dossier_journaux(racine: &Path) -> PathBuf {
    racine.join("journaux")
}

pub fn migrer_anciens_fichiers(sorties: &Path, journaux: &Path) {
    let Ok(entrees) = fs::read_dir(sorties) else { return };
    for chemin in entrees.filter_map(|e| e.ok()).map(|e| e.path()) {
        let nom = chemin.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        let annexe = chemin.extension().is_some_and(|x| x == "json" || x == "log");
        if !annexe || !nom.starts_with("image_") {
            continue;
        }
        let reussie = chemin.with_extension("png").is_file();
        if chemin.extension().is_some_and(|x| x == "log") && reussie {
            let _ = fs::remove_file(&chemin);
        } else if fs::create_dir_all(journaux).is_ok() {
            let _ = fs::rename(&chemin, journaux.join(nom));
        }
    }
}

pub fn dimensions(format: &str) -> Option<(u32, u32)> {
    match format {
        "carre" => Some((512, 512)),
        "portrait" => Some((512, 768)),
        "paysage" => Some((768, 512)),
        _ => None,
    }
}

pub fn installation_complete(racine: &Path) -> bool {
    let vulkan = moteur::dossier_moteur(racine, backend::Backend::Vulkan).join("sd-cli.exe");
    let dossier = moteur::dossier_modeles(racine);
    vulkan.is_file()
        && modeles::MODELES
            .iter()
            .all(|f| fs::metadata(dossier.join(f.nom)).map(|m| m.len() == f.taille).unwrap_or(false))
}

pub fn etat() -> Etat {
    let racine = racine();
    Etat {
        installe: installation_complete(&racine),
        racine: racine.display().to_string(),
        modele: MODELE_AFFICHE.into(),
    }
}

fn nouvelle_graine() -> i64 {
    let n = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(1);
    (n % 2_147_483_647) as i64
}

fn maintenant() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn donnees_png(chemin: &Path) -> Result<String, String> {
    let octets = fs::read(chemin).map_err(|e| e.to_string())?;
    let encode = base64::engine::general_purpose::STANDARD.encode(octets);
    Ok(format!("data:image/png;base64,{encode}"))
}

pub fn dossier_references(racine: &Path) -> PathBuf {
    racine.join("references")
}

fn extension_image(octets: &[u8]) -> Option<&'static str> {
    if octets.starts_with(&[0x89, b'P', b'N', b'G']) {
        Some("png")
    } else if octets.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("jpg")
    } else if octets.starts_with(b"BM") {
        Some("bmp")
    } else {
        None
    }
}

pub fn importer_reference_dans(dossier: &Path, donnees: &str) -> Result<String, String> {
    let brut = donnees.split_once(',').map(|(_, d)| d).unwrap_or(donnees);
    let octets = base64::engine::general_purpose::STANDARD
        .decode(brut.trim())
        .map_err(|_| "Image de référence illisible.".to_string())?;
    if octets.len() > REFERENCE_TAILLE_MAX {
        return Err("L'image de référence est trop lourde (15 Mo maximum).".into());
    }
    let extension = extension_image(&octets)
        .ok_or("Format non pris en charge : utilisez une image PNG, JPEG ou BMP.")?;
    fs::create_dir_all(dossier).map_err(|e| e.to_string())?;
    let fichier = dossier.join(format!("ref_{}.{extension}", nouvelle_graine()));
    fs::write(&fichier, octets).map_err(|e| e.to_string())?;
    Ok(fichier.display().to_string())
}

pub fn importer_reference(donnees: &str) -> Result<String, String> {
    importer_reference_dans(&dossier_references(&racine()), donnees)
}

pub fn verifier_reference(chemin: &str, autorises: &[&Path]) -> Result<PathBuf, String> {
    let fichier = PathBuf::from(chemin).canonicalize().map_err(|_| "Image de référence introuvable.".to_string())?;
    let extension_ok = fichier
        .extension()
        .and_then(|x| x.to_str())
        .is_some_and(|x| ["png", "jpg", "jpeg", "bmp"].contains(&x.to_ascii_lowercase().as_str()));
    let dossier_ok = autorises
        .iter()
        .filter_map(|d| d.canonicalize().ok())
        .any(|d| fichier.starts_with(d));
    if extension_ok && dossier_ok && fichier.is_file() {
        Ok(fichier)
    } else {
        Err("Image de référence non autorisée.".into())
    }
}

fn generer_une(
    prompt: &str,
    format: &str,
    graine: i64,
    rang: u32,
    total: u32,
    reference: Option<&Path>,
) -> Result<Image, String> {
    let (largeur, hauteur) = dimensions(format).ok_or("Format d'image inconnu.")?;
    let racine = racine();
    if !installation_complete(&racine) {
        return Err("L'installation n'est pas terminée : le moteur ou les modèles sont absents.".into());
    }
    let machine = machine::detecter(&racine);
    let vulkan = moteur::dossier_moteur(&racine, backend::Backend::Vulkan).join("sd-cli.exe");
    let choix = backend::choisir_automatiquement(&vulkan, None, &machine.gpus).map_err(|e| e.message())?;
    let moteur = Moteur {
        dossier_moteur: moteur::dossier_moteur(&racine, choix.backend),
        dossier_modeles: moteur::dossier_modeles(&racine),
        choix,
    };
    let sortie = racine.join("sorties").join(format!("image_{graine}.png"));
    let journal = dossier_journaux(&racine).join(format!("image_{graine}.log"));
    let demande = Demande {
        prompt: prompt.into(),
        largeur,
        hauteur,
        graine,
        sortie: sortie.clone(),
        journal: journal.clone(),
        reference: reference.map(|r| Reference { image: r.to_path_buf(), ecart: ECART_REFERENCE }),
    };
    let generation = moteur.charger(&demande, &machine).map_err(|e| e.message())?;
    if let Ok(mut s) = SUIVI.lock() {
        *s = Suivi { journal: Some(journal.clone()), image: rang, total };
    }
    let duree = generation
        .attendre_avec(DELAI_GENERATION, &|| ANNULER.load(Ordering::SeqCst))
        .map_err(|e| e.message())?;
    if !controle::image_png_valide(&sortie) {
        return Err("L'image produite est invalide.".into());
    }
    let _ = fs::remove_file(&journal);
    let fiche = Fiche {
        prompt: prompt.into(),
        format: format.into(),
        largeur,
        hauteur,
        graine,
        duree_s: duree.as_secs_f64(),
        date: maintenant(),
        modele: MODELE_AFFICHE.into(),
        chemin: sortie.display().to_string(),
        reference: reference.and_then(|r| r.file_name()).map(|n| n.to_string_lossy().into_owned()),
        ecart: reference.map(|_| ECART_REFERENCE),
    };
    if let Ok(json) = serde_json::to_string_pretty(&fiche) {
        let _ = fs::write(dossier_journaux(&racine).join(format!("image_{graine}.json")), json);
    }
    Ok(Image { donnees: donnees_png(&sortie)?, fiche })
}

pub fn generer(prompt: &str, format: &str, nombre: u32, reference: Option<&str>) -> Result<Lot, String> {
    let prompt = prompt.trim();
    if prompt.is_empty() {
        return Err("Décrivez l'image à générer.".into());
    }
    dimensions(format).ok_or("Format d'image inconnu.")?;
    if nombre == 0 || nombre > NOMBRE_MAX {
        return Err(format!("Le nombre d'images doit être compris entre 1 et {NOMBRE_MAX}."));
    }
    let reference = match reference {
        Some(chemin) => {
            let racine = racine();
            let sorties = racine.join("sorties");
            let references = dossier_references(&racine);
            Some(verifier_reference(chemin, &[sorties.as_path(), references.as_path()])?)
        }
        None => None,
    };
    let _verrou = Verrou::prendre().ok_or("Une génération est déjà en cours.")?;

    let graine0 = nouvelle_graine();
    let mut images = Vec::new();
    let mut interruption = None;
    for i in 0..nombre {
        if ANNULER.load(Ordering::SeqCst) {
            interruption = Some(MESSAGE_ANNULE.to_string());
            break;
        }
        match generer_une(prompt, format, graine0 + i as i64, i + 1, nombre, reference.as_deref()) {
            Ok(image) => images.push(image),
            Err(e) => {
                interruption = Some(e);
                break;
            }
        }
    }
    if images.is_empty() {
        return Err(interruption.unwrap_or_else(|| "Aucune image produite.".into()));
    }
    Ok(Lot { images, interruption })
}

pub fn annuler() {
    if EN_COURS.load(Ordering::SeqCst) {
        ANNULER.store(true, Ordering::SeqCst);
    }
}

pub fn progression() -> Progression {
    let en_cours = EN_COURS.load(Ordering::SeqCst);
    let (journal, image, images_total) = match SUIVI.lock() {
        Ok(s) => (s.journal.clone(), s.image, s.total),
        Err(_) => (None, 0, 0),
    };
    let texte = journal.and_then(|j| fs::read_to_string(j).ok()).unwrap_or_default();
    let (phase, etape, total) = moteur::analyser_journal(&texte);
    let part = match phase {
        Phase::Demarrage | Phase::Chargement => 0.0,
        Phase::Calcul if total > 0 => 0.9 * f64::from(etape) / f64::from(total),
        Phase::Calcul => 0.0,
        Phase::Finalisation => 0.95,
    };
    let pourcentage = if images_total == 0 {
        0
    } else {
        let fait = f64::from(image.saturating_sub(1)) + part;
        ((fait / f64::from(images_total)) * 100.0).round() as u32
    };
    Progression {
        en_cours,
        phase: match phase {
            Phase::Demarrage => "demarrage",
            Phase::Chargement => "chargement",
            Phase::Calcul => "calcul",
            Phase::Finalisation => "finalisation",
        },
        etape,
        total,
        image,
        images_total,
        pourcentage,
    }
}

pub fn memoire() -> Memoire {
    let machine = machine::detecter(&racine());
    let gpu = machine::meilleur_gpu(&machine.gpus);
    let vram_libre_mo = gpu.and_then(|g| g.vram_libre_mo);
    let refus = machine::garde_fou_memoire(machine.ram_dispo_mo, vram_libre_mo).err();
    Memoire {
        gpu: gpu.map(|g| g.nom.clone()).unwrap_or_default(),
        ram_dispo_mo: machine.ram_dispo_mo,
        ram_totale_mo: machine.ram_totale_mo,
        vram_libre_mo,
        vram_mo: gpu.map(|g| g.vram_mo).unwrap_or(0),
        seuil_ram_mo: machine::RAM_LIBRE_LANCEMENT_MO,
        seuil_vram_mo: machine::VRAM_LIBRE_LANCEMENT_MO,
        suffisante: refus.is_none(),
        message: refus.map(|r| r.message()),
    }
}

pub fn historique_dans(dossier: &Path, limite: usize) -> Vec<Fiche> {
    let Ok(entrees) = fs::read_dir(dossier) else { return Vec::new() };
    let mut fiches: Vec<Fiche> = entrees
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension().is_some_and(|x| x == "json")
                && p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("image_"))
        })
        .filter_map(|p| serde_json::from_str::<Fiche>(&fs::read_to_string(p).ok()?).ok())
        .filter(|f| Path::new(&f.chemin).is_file())
        .collect();
    fiches.sort_by(|a, b| b.date.cmp(&a.date).then(b.graine.cmp(&a.graine)));
    fiches.truncate(limite);
    fiches
}

pub fn historique(limite: usize) -> Vec<Fiche> {
    let racine = racine();
    let journaux = dossier_journaux(&racine);
    migrer_anciens_fichiers(&racine.join("sorties"), &journaux);
    historique_dans(&journaux, limite)
}

pub fn charger_image_dans(dossier: &Path, chemin: &str) -> Result<String, String> {
    let dossier = dossier.canonicalize().map_err(|e| e.to_string())?;
    let fichier = PathBuf::from(chemin).canonicalize().map_err(|e| e.to_string())?;
    if !fichier.starts_with(&dossier) || !fichier.extension().is_some_and(|x| x == "png") {
        return Err("Fichier non autorisé.".into());
    }
    donnees_png(&fichier)
}

pub fn charger_image(chemin: &str) -> Result<String, String> {
    charger_image_dans(&racine().join("sorties"), chemin)
}

pub fn supprimer_image_dans(dossier: &Path, journaux: &Path, chemin: &str) -> Result<(), String> {
    let dossier = dossier.canonicalize().map_err(|e| e.to_string())?;
    let fichier = PathBuf::from(chemin).canonicalize().map_err(|e| e.to_string())?;
    let nom_valide = fichier.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("image_"));
    if fichier.parent() != Some(dossier.as_path()) || !fichier.extension().is_some_and(|x| x == "png") || !nom_valide {
        return Err("Fichier non autorisé.".into());
    }
    fs::remove_file(&fichier).map_err(|e| e.to_string())?;
    let nom = fichier.file_stem().unwrap_or_default();
    for extension in ["json", "log"] {
        let annexe = journaux.join(nom).with_extension(extension);
        if annexe.is_file() {
            fs::remove_file(annexe).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub fn supprimer_image(chemin: &str) -> Result<(), String> {
    if EN_COURS.load(Ordering::SeqCst) {
        return Err("Attendez la fin de la génération pour supprimer une image.".into());
    }
    let racine = racine();
    supprimer_image_dans(&racine.join("sorties"), &dossier_journaux(&racine), chemin)
}

pub fn ouvrir_dossier_images() -> Result<(), String> {
    let dossier = racine().join("sorties");
    fs::create_dir_all(&dossier).map_err(|e| e.to_string())?;
    Command::new("explorer").arg(&dossier).spawn().map(|_| ()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fiche(chemin: &Path, graine: i64, date: u64) -> Fiche {
        Fiche {
            prompt: "un chat".into(),
            format: "carre".into(),
            largeur: 512,
            hauteur: 512,
            graine,
            duree_s: 1.0,
            date,
            modele: MODELE_AFFICHE.into(),
            chemin: chemin.display().to_string(),
            reference: None,
            ecart: None,
        }
    }

    fn dossier_temp(nom: &str) -> PathBuf {
        let d = std::env::temp_dir().join(nom);
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn racine_deduite_du_dossier_bin() {
        assert_eq!(
            racine_depuis_exe(Path::new("C:\\IA_Eco\\bin\\app.exe")),
            Some(PathBuf::from("C:\\IA_Eco"))
        );
        assert_eq!(racine_depuis_exe(Path::new("C:\\IA_Eco\\BIN\\app.exe")), Some(PathBuf::from("C:\\IA_Eco")));
        assert_eq!(racine_depuis_exe(Path::new("C:\\x\\target\\debug\\app.exe")), None);
    }

    #[test]
    fn formats_connus_et_inconnus() {
        assert_eq!(dimensions("carre"), Some((512, 512)));
        assert_eq!(dimensions("portrait"), Some((512, 768)));
        assert_eq!(dimensions("paysage"), Some((768, 512)));
        assert_eq!(dimensions("autre"), None);
    }

    #[test]
    fn description_vide_refusee() {
        assert!(generer("   ", "carre", 1, None).unwrap_err().contains("Décrivez"));
    }

    #[test]
    fn format_inconnu_refuse() {
        assert!(generer("un chat", "triangle", 1, None).is_err());
    }

    #[test]
    fn nombre_d_images_borne() {
        assert!(generer("un chat", "carre", 0, None).unwrap_err().contains("entre 1 et"));
        assert!(generer("un chat", "carre", NOMBRE_MAX + 1, None).unwrap_err().contains("entre 1 et"));
    }

    #[test]
    fn installation_absente_detectee() {
        assert!(!installation_complete(Path::new("C:\\dossier_inexistant_ia_eco")));
    }

    #[test]
    fn historique_trie_et_limite() {
        let d = dossier_temp("ia_eco_historique");
        for (graine, date) in [(1_i64, 100_u64), (2, 300), (3, 200)] {
            let png = d.join(format!("image_{graine}.png"));
            fs::write(&png, b"png").unwrap();
            fs::write(png.with_extension("json"), serde_json::to_string(&fiche(&png, graine, date)).unwrap()).unwrap();
        }
        let orpheline = d.join("image_9.json");
        fs::write(&orpheline, serde_json::to_string(&fiche(&d.join("image_9.png"), 9, 999)).unwrap()).unwrap();
        fs::write(d.join("image_8.json"), "pas du json").unwrap();
        let h = historique_dans(&d, 10);
        assert_eq!(h.iter().map(|f| f.graine).collect::<Vec<_>>(), vec![2, 3, 1]);
        assert_eq!(historique_dans(&d, 2).len(), 2);
        assert!(historique_dans(&d.join("absent"), 5).is_empty());
    }

    #[test]
    fn chargement_limite_au_dossier_des_images() {
        let d = dossier_temp("ia_eco_chargement");
        let dehors = dossier_temp("ia_eco_chargement_dehors");
        let ok = d.join("image_1.png");
        let hors = dehors.join("image_2.png");
        let texte = d.join("note.txt");
        fs::write(&ok, b"png").unwrap();
        fs::write(&hors, b"png").unwrap();
        fs::write(&texte, b"x").unwrap();
        assert!(charger_image_dans(&d, &ok.display().to_string()).unwrap().starts_with("data:image/png;base64,"));
        assert!(charger_image_dans(&d, &hors.display().to_string()).is_err());
        assert!(charger_image_dans(&d, &texte.display().to_string()).is_err());
        assert!(charger_image_dans(&d, &d.join("absent.png").display().to_string()).is_err());
    }

    #[test]
    #[ignore = "génère une image avec l'installation réelle ; lancer avec --ignored"]
    fn generation_reelle_portrait() {
        let lot = generer("a red apple on a wooden table, soft daylight", "portrait", 1, None).unwrap();
        assert!(lot.images[0].donnees.starts_with("data:image/png;base64,"));
        eprintln!("{} s : {}", lot.images[0].fiche.duree_s, lot.images[0].fiche.chemin);
    }

    #[test]
    fn suppression_limitee_aux_images_generees() {
        let d = dossier_temp("ia_eco_suppression");
        let dehors = dossier_temp("ia_eco_suppression_dehors");
        let j = dossier_temp("ia_eco_suppression_journaux");
        let png = d.join("image_1.png");
        fs::write(&png, b"png").unwrap();
        fs::write(j.join("image_1.json"), b"{}").unwrap();
        fs::write(j.join("image_1.log"), b"log").unwrap();
        let autre = d.join("controle.png");
        fs::write(&autre, b"png").unwrap();
        let hors = dehors.join("image_2.png");
        fs::write(&hors, b"png").unwrap();

        assert!(supprimer_image_dans(&d, &j, &autre.display().to_string()).is_err());
        assert!(supprimer_image_dans(&d, &j, &hors.display().to_string()).is_err());
        assert!(supprimer_image_dans(&d, &j, &d.join("absent.png").display().to_string()).is_err());
        assert!(autre.is_file() && hors.is_file());

        supprimer_image_dans(&d, &j, &png.display().to_string()).unwrap();
        assert!(!png.exists());
        assert!(!j.join("image_1.json").exists());
        assert!(!j.join("image_1.log").exists());
    }

    #[test]
    fn migration_des_anciens_fichiers() {
        let s = dossier_temp("ia_eco_migration_sorties");
        let j = std::env::temp_dir().join("ia_eco_migration_journaux");
        let _ = fs::remove_dir_all(&j);
        for nom in ["image_1.png", "image_1.json", "image_1.log", "image_2.log", "controle.log"] {
            fs::write(s.join(nom), b"x").unwrap();
        }
        migrer_anciens_fichiers(&s, &j);
        assert!(s.join("image_1.png").is_file());
        assert!(s.join("controle.log").is_file());
        assert!(j.join("image_1.json").is_file());
        assert!(!s.join("image_1.log").exists() && !j.join("image_1.log").exists());
        assert!(j.join("image_2.log").is_file());
        assert!(!s.join("image_1.json").exists() && !s.join("image_2.log").exists());
    }

    #[test]
    #[ignore = "supprime image_999999999 dans les sorties réelles ; lancer avec --ignored"]
    fn suppression_reelle() {
        let png = racine().join("sorties").join("image_999999999.png");
        assert!(png.is_file());
        supprimer_image(&png.display().to_string()).unwrap();
        assert!(!png.exists());
    }

    #[test]
    fn reference_importee_et_verifiee() {
        let d = dossier_temp("ia_eco_references");
        let png = base64::engine::general_purpose::STANDARD.encode([0x89, b'P', b'N', b'G', 1, 2, 3]);
        let chemin = importer_reference_dans(&d, &format!("data:image/png;base64,{png}")).unwrap();
        assert!(chemin.ends_with(".png") && Path::new(&chemin).is_file());
        assert!(verifier_reference(&chemin, &[d.as_path()]).is_ok());

        let texte = base64::engine::general_purpose::STANDARD.encode(b"pas une image");
        assert!(importer_reference_dans(&d, &texte).unwrap_err().contains("PNG, JPEG ou BMP"));
        assert!(importer_reference_dans(&d, "%%%").is_err());
    }

    #[test]
    fn reference_limitee_aux_dossiers_autorises() {
        let d = dossier_temp("ia_eco_reference_ok");
        let dehors = dossier_temp("ia_eco_reference_dehors");
        let ok = d.join("a.png");
        let hors = dehors.join("b.png");
        let texte = d.join("c.txt");
        for f in [&ok, &hors, &texte] {
            fs::write(f, b"x").unwrap();
        }
        assert!(verifier_reference(&ok.display().to_string(), &[d.as_path()]).is_ok());
        assert!(verifier_reference(&hors.display().to_string(), &[d.as_path()]).is_err());
        assert!(verifier_reference(&texte.display().to_string(), &[d.as_path()]).is_err());
        assert!(verifier_reference(&d.join("absent.png").display().to_string(), &[d.as_path()]).is_err());
    }

    #[test]
    fn verrou_exclusif() {
        let a = Verrou::prendre();
        assert!(a.is_some());
        assert!(Verrou::prendre().is_none());
        drop(a);
        assert!(Verrou::prendre().is_some());
    }
}
