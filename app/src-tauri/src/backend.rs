use crate::machine::{Gpu, Marque, VRAM_MIN_MO};
use serde::Serialize;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Backend {
    Vulkan,
    Cuda,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Peripherique {
    pub nom: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Choix {
    pub backend: Backend,
    pub peripherique: String,
    pub gpu: Gpu,
}

#[derive(Debug)]
pub enum ErreurBackend {
    Lancement(String),
    AucunPeripherique,
}

impl ErreurBackend {
    pub fn message(&self) -> String {
        match self {
            ErreurBackend::Lancement(e) => format!("Impossible d'interroger le moteur graphique : {e}"),
            ErreurBackend::AucunPeripherique => {
                "Aucune carte graphique utilisable par le moteur. Mettez à jour le pilote de votre carte graphique.".into()
            }
        }
    }
}

pub fn analyser_peripheriques(sortie: &str) -> Vec<Peripherique> {
    sortie
        .lines()
        .filter_map(|l| {
            let (nom, desc) = l.split_once('\t')?;
            let nom = nom.trim();
            if nom.is_empty() || nom.eq_ignore_ascii_case("cpu") {
                return None;
            }
            Some(Peripherique { nom: nom.to_string(), description: desc.trim().to_string() })
        })
        .collect()
}

pub fn lister_peripheriques(sd_cli: &Path) -> Result<Vec<Peripherique>, ErreurBackend> {
    let sortie = Command::new(sd_cli)
        .arg("--list-devices")
        .current_dir(sd_cli.parent().unwrap_or(Path::new(".")))
        .output()
        .map_err(|e| ErreurBackend::Lancement(e.to_string()))?;
    Ok(analyser_peripheriques(&String::from_utf8_lossy(&sortie.stdout)))
}

fn normaliser(s: &str) -> String {
    s.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect()
}

fn correspond(periph: &Peripherique, gpu: &Gpu) -> bool {
    let a = normaliser(&periph.description);
    let b = normaliser(&gpu.nom);
    if a.is_empty() || b.is_empty() {
        return false;
    }
    a.contains(&b) || b.contains(&a)
}

pub fn choisir(backend: Backend, periphs: &[Peripherique], gpus: &[Gpu]) -> Result<Choix, ErreurBackend> {
    periphs
        .iter()
        .filter_map(|p| {
            gpus.iter()
                .filter(|g| g.vram_mo >= VRAM_MIN_MO && correspond(p, g))
                .max_by_key(|g| g.vram_mo)
                .map(|g| (p, g))
        })
        .max_by_key(|(_, g)| (g.marque.rang(), g.vram_mo))
        .map(|(p, g)| Choix { backend, peripherique: p.nom.clone(), gpu: g.clone() })
        .ok_or(ErreurBackend::AucunPeripherique)
}

pub fn choisir_automatiquement(
    sd_vulkan: &Path,
    sd_cuda: Option<&Path>,
    gpus: &[Gpu],
) -> Result<Choix, ErreurBackend> {
    let vulkan = lister_peripheriques(sd_vulkan).and_then(|p| choisir(Backend::Vulkan, &p, gpus));
    match vulkan {
        Ok(c) => Ok(c),
        Err(e) => {
            if let Some(cuda) = sd_cuda {
                if gpus.iter().any(|g| g.marque == Marque::Nvidia && g.vram_mo >= VRAM_MIN_MO) {
                    let p = lister_peripheriques(cuda)?;
                    return choisir(Backend::Cuda, &p, gpus);
                }
            }
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gpu(nom: &str, marque: Marque, vram: u64) -> Gpu {
        Gpu { nom: nom.into(), marque, vram_mo: vram, vram_libre_mo: None, luid: String::new() }
    }

    #[test]
    fn analyse_sortie_moteur() {
        let s = "Vulkan0\tNVIDIA GeForce RTX 4060\nCPU\tAMD Ryzen 7 5700X 8-Core Processor   \n";
        let p = analyser_peripheriques(s);
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].nom, "Vulkan0");
        assert_eq!(p[0].description, "NVIDIA GeForce RTX 4060");
    }

    #[test]
    fn analyse_ignore_bruit() {
        assert!(analyser_peripheriques("ggml_vulkan: Found 1 Vulkan devices:\nload_backend: x\n").is_empty());
    }

    #[test]
    fn double_gpu_force_nvidia() {
        let periphs = vec![
            Peripherique { nom: "Vulkan0".into(), description: "Intel(R) UHD Graphics".into() },
            Peripherique { nom: "Vulkan1".into(), description: "NVIDIA GeForce GTX 1650 with Max-Q Design".into() },
        ];
        let gpus = vec![
            gpu("Intel(R) UHD Graphics", Marque::Intel, 4096),
            gpu("NVIDIA GeForce GTX 1650 with Max-Q Design", Marque::Nvidia, 4096),
        ];
        let c = choisir(Backend::Vulkan, &periphs, &gpus).unwrap();
        assert_eq!(c.peripherique, "Vulkan1");
        assert_eq!(c.gpu.marque, Marque::Nvidia);
    }

    #[test]
    fn igpu_sous_seuil_jamais_choisi() {
        let periphs = vec![Peripherique { nom: "Vulkan0".into(), description: "Intel(R) UHD Graphics".into() }];
        let gpus = vec![gpu("Intel(R) UHD Graphics", Marque::Intel, 128)];
        assert!(matches!(choisir(Backend::Vulkan, &periphs, &gpus), Err(ErreurBackend::AucunPeripherique)));
    }

    #[test]
    fn aucun_peripherique() {
        assert!(matches!(choisir(Backend::Vulkan, &[], &[]), Err(ErreurBackend::AucunPeripherique)));
    }
}
