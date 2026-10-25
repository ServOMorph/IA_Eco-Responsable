use crate::backend;
use crate::controle;
use crate::machine;
use crate::modeles;
use crate::moteur::{self, Demande, Moteur};
use base64::Engine;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const RACINE_DEFAUT: &str = "D:\\Dev\\ia-eco-test";
const DELAI_GENERATION: Duration = Duration::from_secs(900);

static EN_COURS: AtomicBool = AtomicBool::new(false);

struct Verrou;

impl Verrou {
    fn prendre() -> Option<Verrou> {
        if EN_COURS.swap(true, Ordering::SeqCst) { None } else { Some(Verrou) }
    }
}

impl Drop for Verrou {
    fn drop(&mut self) {
        EN_COURS.store(false, Ordering::SeqCst);
    }
}

#[derive(Debug, Serialize)]
pub struct Etat {
    pub installe: bool,
    pub racine: String,
}

#[derive(Debug, Serialize)]
pub struct Image {
    pub donnees: String,
    pub duree_s: f64,
    pub chemin: String,
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
    Etat { installe: installation_complete(&racine), racine: racine.display().to_string() }
}

fn nouvelle_graine() -> i64 {
    let n = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(1);
    (n % 2_147_483_647) as i64
}

pub fn generer(prompt: &str, format: &str) -> Result<Image, String> {
    let prompt = prompt.trim();
    if prompt.is_empty() {
        return Err("Décrivez l'image à générer.".into());
    }
    let (largeur, hauteur) = dimensions(format).ok_or("Format d'image inconnu.")?;
    let _verrou = Verrou::prendre().ok_or("Une génération est déjà en cours.")?;

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
    let graine = nouvelle_graine();
    let sortie = racine.join("sorties").join(format!("image_{graine}.png"));
    let demande = Demande { prompt: prompt.into(), largeur, hauteur, graine, sortie: sortie.clone() };
    let generation = moteur.charger(&demande, &machine).map_err(|e| e.message())?;
    let duree = generation.attendre(DELAI_GENERATION).map_err(|e| e.message())?;
    if !controle::image_png_valide(&sortie) {
        return Err("L'image produite est invalide.".into());
    }
    let octets = fs::read(&sortie).map_err(|e| e.to_string())?;
    let encode = base64::engine::general_purpose::STANDARD.encode(octets);
    Ok(Image {
        donnees: format!("data:image/png;base64,{encode}"),
        duree_s: duree.as_secs_f64(),
        chemin: sortie.display().to_string(),
    })
}

pub fn ouvrir_dossier_images() -> Result<(), String> {
    let dossier = racine().join("sorties");
    fs::create_dir_all(&dossier).map_err(|e| e.to_string())?;
    Command::new("explorer").arg(&dossier).spawn().map(|_| ()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(generer("   ", "carre").unwrap_err().contains("Décrivez"));
    }

    #[test]
    fn format_inconnu_refuse() {
        assert!(generer("un chat", "triangle").is_err());
    }

    #[test]
    fn installation_absente_detectee() {
        assert!(!installation_complete(Path::new("C:\\dossier_inexistant_ia_eco")));
    }

    #[test]
    #[ignore = "génère une image avec l'installation réelle ; lancer avec --ignored"]
    fn generation_reelle_portrait() {
        let image = generer("a red apple on a wooden table, soft daylight", "portrait").unwrap();
        assert!(image.donnees.starts_with("data:image/png;base64,"));
        eprintln!("{} s : {}", image.duree_s, image.chemin);
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
