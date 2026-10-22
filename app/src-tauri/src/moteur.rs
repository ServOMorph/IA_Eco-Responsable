use crate::backend::{Backend, Choix};
use crate::machine::{self, Machine, Refus};
use crate::modeles;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct Demande {
    pub prompt: String,
    pub largeur: u32,
    pub hauteur: u32,
    pub graine: i64,
    pub sortie: PathBuf,
}

#[derive(Debug)]
pub enum ErreurMoteur {
    Memoire(Refus),
    MemoireVideoMoteur,
    Fichier(String),
    Lancement(String),
    Echec { code: Option<i32>, journal: PathBuf },
    Delai,
}

impl ErreurMoteur {
    pub fn message(&self) -> String {
        match self {
            ErreurMoteur::Memoire(r) => r.message(),
            ErreurMoteur::MemoireVideoMoteur => "La carte graphique n'a pas assez de mémoire disponible. Fermez les applications qui l'utilisent (navigateurs, jeux, vidéo) puis réessayez.".into(),
            ErreurMoteur::Fichier(e) => format!("Fichier manquant ou illisible : {e}. Relancez l'installation."),
            ErreurMoteur::Lancement(e) => format!("Le moteur n'a pas pu démarrer : {e}"),
            ErreurMoteur::Echec { code, .. } => format!(
                "La génération a échoué (code {}). Un journal est disponible pour le support.",
                code.map(|c| c.to_string()).unwrap_or_else(|| "inconnu".into())
            ),
            ErreurMoteur::Delai => "La génération a dépassé le délai maximal et a été arrêtée.".into(),
        }
    }
}

pub struct Moteur {
    pub dossier_moteur: PathBuf,
    pub dossier_modeles: PathBuf,
    pub choix: Choix,
}

impl Moteur {
    pub fn executable(&self) -> PathBuf {
        self.dossier_moteur.join("sd-cli.exe")
    }

    pub fn arguments(&self, d: &Demande) -> Vec<String> {
        let m = &self.dossier_modeles;
        vec![
            "--diffusion-model".into(),
            m.join(modeles::DIFFUSION.nom).to_string_lossy().into_owned(),
            "--vae".into(),
            m.join(modeles::VAE.nom).to_string_lossy().into_owned(),
            "--llm".into(),
            m.join(modeles::ENCODEUR.nom).to_string_lossy().into_owned(),
            "-p".into(),
            d.prompt.clone(),
            "--cfg-scale".into(),
            "1.0".into(),
            "--steps".into(),
            "8".into(),
            "-W".into(),
            d.largeur.to_string(),
            "-H".into(),
            d.hauteur.to_string(),
            "-s".into(),
            d.graine.to_string(),
            "--offload-to-cpu".into(),
            "--vae-tiling".into(),
            "--mmap".into(),
            "--backend".into(),
            self.choix.peripherique.clone(),
            "-o".into(),
            d.sortie.to_string_lossy().into_owned(),
        ]
    }

    pub fn verifier_garde_fou(&self, machine: &Machine) -> Result<(), ErreurMoteur> {
        let vram_libre = machine
            .gpus
            .iter()
            .find(|g| g.nom == self.choix.gpu.nom)
            .and_then(|g| g.vram_libre_mo);
        machine::garde_fou_memoire(machine.ram_dispo_mo, vram_libre).map_err(ErreurMoteur::Memoire)
    }

    pub fn verifier_fichiers(&self) -> Result<(), ErreurMoteur> {
        let exe = self.executable();
        if !exe.is_file() {
            return Err(ErreurMoteur::Fichier(exe.display().to_string()));
        }
        for f in modeles::MODELES {
            let p = self.dossier_modeles.join(f.nom);
            match fs::metadata(&p) {
                Ok(m) if m.len() == f.taille => {}
                _ => return Err(ErreurMoteur::Fichier(p.display().to_string())),
            }
        }
        Ok(())
    }

    pub fn charger(&self, d: &Demande, machine: &Machine) -> Result<Generation, ErreurMoteur> {
        self.verifier_fichiers()?;
        self.verifier_garde_fou(machine)?;
        if let Some(parent) = d.sortie.parent() {
            fs::create_dir_all(parent).map_err(|e| ErreurMoteur::Fichier(e.to_string()))?;
        }
        let journal = d.sortie.with_extension("log");
        let out = File::create(&journal).map_err(|e| ErreurMoteur::Fichier(e.to_string()))?;
        let err = out.try_clone().map_err(|e| ErreurMoteur::Fichier(e.to_string()))?;
        let mut cmd = Command::new(self.executable());
        cmd.args(self.arguments(d))
            .current_dir(&self.dossier_moteur)
            .stdin(Stdio::null())
            .stdout(Stdio::from(out))
            .stderr(Stdio::from(err));
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x0800_0000);
        }
        let enfant = cmd.spawn().map_err(|e| ErreurMoteur::Lancement(e.to_string()))?;
        Ok(Generation { enfant, debut: Instant::now(), sortie: d.sortie.clone(), journal })
    }
}

pub struct Generation {
    enfant: Child,
    debut: Instant,
    pub sortie: PathBuf,
    pub journal: PathBuf,
}

impl Generation {
    pub fn attendre(mut self, delai: Duration) -> Result<Duration, ErreurMoteur> {
        loop {
            match self.enfant.try_wait() {
                Ok(Some(statut)) => {
                    return if statut.success() && self.sortie.is_file() {
                        Ok(self.debut.elapsed())
                    } else if journal_indique_memoire(&self.journal) {
                        Err(ErreurMoteur::MemoireVideoMoteur)
                    } else {
                        Err(ErreurMoteur::Echec { code: statut.code(), journal: self.journal.clone() })
                    };
                }
                Ok(None) => {}
                Err(e) => return Err(ErreurMoteur::Lancement(e.to_string())),
            }
            if self.debut.elapsed() > delai {
                self.decharger();
                return Err(ErreurMoteur::Delai);
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    }

    pub fn decharger(&mut self) {
        let _ = self.enfant.kill();
        let _ = self.enfant.wait();
    }

    pub fn en_cours(&mut self) -> bool {
        matches!(self.enfant.try_wait(), Ok(None))
    }
}

pub fn journal_indique_memoire(journal: &Path) -> bool {
    fs::read_to_string(journal)
        .map(|t| t.contains("cannot make enough memory") || t.contains("out of device memory"))
        .unwrap_or(false)
}

pub fn dossier_moteur(racine: &Path, backend: Backend) -> PathBuf {
    racine.join("moteur").join(match backend {
        Backend::Vulkan => "vulkan",
        Backend::Cuda => "cuda",
    })
}

pub fn dossier_modeles(racine: &Path) -> PathBuf {
    racine.join("modeles")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::{Gpu, Marque};

    fn moteur() -> Moteur {
        Moteur {
            dossier_moteur: PathBuf::from("C:\\app\\moteur\\vulkan"),
            dossier_modeles: PathBuf::from("C:\\app\\modeles"),
            choix: Choix {
                backend: Backend::Vulkan,
                peripherique: "Vulkan1".into(),
                gpu: Gpu { nom: "RTX".into(), marque: Marque::Nvidia, vram_mo: 4096, vram_libre_mo: Some(3500), luid: String::new() },
            },
        }
    }

    fn demande() -> Demande {
        Demande { prompt: "un chat".into(), largeur: 512, hauteur: 512, graine: 42, sortie: PathBuf::from("C:\\o\\a.png") }
    }

    #[test]
    fn arguments_reprennent_la_configuration_validee() {
        let a = moteur().arguments(&demande());
        for opt in ["--offload-to-cpu", "--vae-tiling", "--mmap"] {
            assert!(a.iter().any(|x| x == opt), "{opt} absent");
        }
        let pos = a.iter().position(|x| x == "--backend").unwrap();
        assert_eq!(a[pos + 1], "Vulkan1");
        let pos = a.iter().position(|x| x == "--steps").unwrap();
        assert_eq!(a[pos + 1], "8");
        assert!(a.contains(&"un chat".to_string()));
    }

    #[test]
    fn garde_fou_refuse_si_ram_basse() {
        let m = Machine { gpus: vec![moteur().choix.gpu], ram_totale_mo: 8000, ram_dispo_mo: 3000, disque_libre_mo: 100_000 };
        assert!(matches!(moteur().verifier_garde_fou(&m), Err(ErreurMoteur::Memoire(Refus::RamLibre { .. }))));
    }

    #[test]
    fn garde_fou_refuse_si_vram_basse() {
        let mut g = moteur().choix.gpu;
        g.vram_libre_mo = Some(1000);
        let m = Machine { gpus: vec![g], ram_totale_mo: 8000, ram_dispo_mo: 6000, disque_libre_mo: 100_000 };
        assert!(matches!(moteur().verifier_garde_fou(&m), Err(ErreurMoteur::Memoire(Refus::VramLibre { .. }))));
    }

    #[test]
    fn garde_fou_accepte() {
        let m = Machine { gpus: vec![moteur().choix.gpu], ram_totale_mo: 8000, ram_dispo_mo: 6000, disque_libre_mo: 100_000 };
        assert!(moteur().verifier_garde_fou(&m).is_ok());
    }

    #[test]
    fn journal_memoire_detecte() {
        let d = std::env::temp_dir().join("ia_eco_journal");
        fs::create_dir_all(&d).unwrap();
        let j = d.join("a.log");
        fs::write(&j, "[W] model manager cannot make enough memory available on Vulkan0").unwrap();
        assert!(journal_indique_memoire(&j));
        fs::write(&j, "[E] autre erreur").unwrap();
        assert!(!journal_indique_memoire(&j));
    }

    #[test]
    fn fichiers_manquants_detectes() {
        assert!(matches!(moteur().verifier_fichiers(), Err(ErreurMoteur::Fichier(_))));
    }
}
