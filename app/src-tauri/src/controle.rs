use crate::backend::{self, Backend, Choix};
use crate::machine::{self, Machine, Verdict};
use crate::modeles::{self, Fichier, Options};
use crate::moteur::{self, Demande, Moteur};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
pub struct Rapport {
    pub duree_s: f64,
    pub taille_octets: u64,
    pub image: PathBuf,
}

pub const PROMPT_CONTROLE: &str = "a red apple on a wooden table, soft daylight";
pub const DELAI_CONTROLE: Duration = Duration::from_secs(900);

pub fn image_png_valide(chemin: &Path) -> bool {
    const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    match fs::read(chemin) {
        Ok(o) => o.len() > 10_000 && o[..8] == SIGNATURE,
        Err(_) => false,
    }
}

pub fn installer_moteur(
    racine: &Path,
    backend: Backend,
    options: &Options,
    progression: &dyn Fn(&str, u64, u64),
    annuler: &AtomicBool,
) -> Result<PathBuf, String> {
    let dest = moteur::dossier_moteur(racine, backend);
    if dest.join("sd-cli.exe").is_file() {
        return Ok(dest);
    }
    let archives: Vec<Fichier> = match backend {
        Backend::Vulkan => vec![modeles::MOTEUR_VULKAN],
        Backend::Cuda => vec![modeles::MOTEUR_CUDA, modeles::CUDART],
    };
    let dossier_dl = racine.join("telechargements");
    for a in &archives {
        let chemin = modeles::telecharger(a, &dossier_dl, options, &|p, t| progression(a.nom, p, t), annuler)
            .map_err(|e| e.message())?;
        modeles::extraire_zip(&chemin, &dest)?;
        let _ = fs::remove_file(&chemin);
    }
    if dest.join("sd-cli.exe").is_file() {
        Ok(dest)
    } else {
        Err("sd-cli.exe introuvable après extraction du moteur".into())
    }
}

pub fn installer_modeles(
    racine: &Path,
    options: &Options,
    progression: &dyn Fn(&str, u64, u64),
    annuler: &AtomicBool,
) -> Result<(), String> {
    let dossier = moteur::dossier_modeles(racine);
    for f in modeles::MODELES {
        modeles::telecharger(&f, &dossier, options, &|p, t| progression(f.nom, p, t), annuler)
            .map_err(|e| e.message())?;
    }
    Ok(())
}

pub fn preparer(racine: &Path, machine: &Machine, sd_cuda: Option<&Path>) -> Result<Choix, String> {
    let requis = modeles::espace_requis_mo(sd_cuda.is_some());
    match machine::evaluer(machine, requis) {
        Verdict::Incompatible(raisons) => {
            Err(raisons.iter().map(|r| r.message()).collect::<Vec<_>>().join(" "))
        }
        _ => {
            let vulkan = moteur::dossier_moteur(racine, Backend::Vulkan).join("sd-cli.exe");
            backend::choisir_automatiquement(&vulkan, sd_cuda, &machine.gpus).map_err(|e| e.message())
        }
    }
}

pub fn generation_de_controle(moteur: &Moteur, sortie: &Path) -> Result<Rapport, String> {
    let machine = machine::detecter(sortie);
    let demande = Demande {
        prompt: PROMPT_CONTROLE.into(),
        largeur: 512,
        hauteur: 512,
        graine: 42,
        sortie: sortie.to_path_buf(),
    };
    let generation = moteur.charger(&demande, &machine).map_err(|e| e.message())?;
    let duree = generation.attendre(DELAI_CONTROLE).map_err(|e| e.message())?;
    if !image_png_valide(sortie) {
        return Err("L'image de contrôle produite est invalide.".into());
    }
    let taille = fs::metadata(sortie).map(|m| m.len()).unwrap_or(0);
    Ok(Rapport { duree_s: duree.as_secs_f64(), taille_octets: taille, image: sortie.to_path_buf() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_invalide_refuse() {
        let d = std::env::temp_dir().join("ia_eco_test_png");
        fs::create_dir_all(&d).unwrap();
        let f = d.join("faux.png");
        fs::write(&f, vec![0u8; 20_000]).unwrap();
        assert!(!image_png_valide(&f));
        let mut ok = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        ok.extend(vec![0u8; 20_000]);
        fs::write(&f, ok).unwrap();
        assert!(image_png_valide(&f));
        fs::write(&f, vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]).unwrap();
        assert!(!image_png_valide(&f));
        assert!(!image_png_valide(&d.join("absent.png")));
    }

    #[test]
    fn preparer_refuse_machine_incompatible() {
        let m = Machine { gpus: vec![], ram_totale_mo: 16000, ram_dispo_mo: 8000, disque_libre_mo: 200_000 };
        let e = preparer(Path::new("C:\\x"), &m, None).unwrap_err();
        assert!(e.contains("Aucune carte"));
    }
}
