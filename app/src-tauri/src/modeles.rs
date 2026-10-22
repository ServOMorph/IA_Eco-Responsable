use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct Fichier {
    pub nom: &'static str,
    pub url: &'static str,
    pub taille: u64,
    pub sha256: &'static str,
}

pub const DIFFUSION: Fichier = Fichier {
    nom: "z_image_turbo-Q5_0.gguf",
    url: "https://huggingface.co/leejet/Z-Image-Turbo-GGUF/resolve/main/z_image_turbo-Q5_0.gguf",
    taille: 4_542_547_904,
    sha256: "12b4a5c064cf8a0fa45d665c5b399d43effbcbeb37748d028d0d0d0fc34c2255",
};

pub const ENCODEUR: Fichier = Fichier {
    nom: "Qwen3-4B-Instruct-2507-Q4_K_M.gguf",
    url: "https://huggingface.co/unsloth/Qwen3-4B-Instruct-2507-GGUF/resolve/main/Qwen3-4B-Instruct-2507-Q4_K_M.gguf",
    taille: 2_497_281_120,
    sha256: "3605803b982cb64aead44f6c1b2ae36e3acdb41d8e46c8a94c6533bc4c67e597",
};

pub const VAE: Fichier = Fichier {
    nom: "ae.safetensors",
    url: "https://huggingface.co/Comfy-Org/z_image_turbo/resolve/main/split_files/vae/ae.safetensors",
    taille: 335_304_388,
    sha256: "afc8e28272cd15db3919bacdb6918ce9c1ed22e96cb12c4d5ed0fba823529e38",
};

pub const MOTEUR_VULKAN: Fichier = Fichier {
    nom: "sd-vulkan.zip",
    url: "https://github.com/leejet/stable-diffusion.cpp/releases/download/master-973-f0aac52/sd-master-f0aac52-bin-win-vulkan-x64.zip",
    taille: 29_363_455,
    sha256: "64ae41edf331723871ff6925f6f9d8bb46f6407d49e51f2073ef11f646765cb2",
};

pub const MOTEUR_CUDA: Fichier = Fichier {
    nom: "sd-cuda12.zip",
    url: "https://github.com/leejet/stable-diffusion.cpp/releases/download/master-973-f0aac52/sd-master-f0aac52-bin-win-cuda12-x64.zip",
    taille: 343_638_522,
    sha256: "68a7b08b87925a39dc1661d872a7625e633df90dc6e3c32165e7cb8815b7febf",
};

pub const CUDART: Fichier = Fichier {
    nom: "cudart.zip",
    url: "https://github.com/leejet/stable-diffusion.cpp/releases/download/master-973-f0aac52/cudart-sd-bin-win-cu12-x64.zip",
    taille: 563_452_046,
    sha256: "fe20366827d357c00797eebb58244dddab7fd9a348d70090c3871004c320f38d",
};

pub const MODELES: [Fichier; 3] = [DIFFUSION, ENCODEUR, VAE];

pub fn espace_requis_mo(backend_cuda: bool) -> u64 {
    let moteur = if backend_cuda { MOTEUR_CUDA.taille + CUDART.taille } else { MOTEUR_VULKAN.taille };
    let modeles: u64 = MODELES.iter().map(|f| f.taille).sum();
    let archives_et_extraction = moteur * 2;
    (modeles + archives_et_extraction + 1_000_000_000) / (1024 * 1024)
}

#[derive(Debug)]
pub enum ErreurTelechargement {
    Espace { libre_mo: u64, requis_mo: u64 },
    Reseau(String),
    Disque(String),
    Integrite { attendu: String, obtenu: String },
    Annule,
}

impl ErreurTelechargement {
    pub fn message(&self) -> String {
        match self {
            ErreurTelechargement::Espace { libre_mo, requis_mo } => format!(
                "Espace disque insuffisant : {} Go libres, {} Go nécessaires.",
                libre_mo / 1024,
                requis_mo / 1024
            ),
            ErreurTelechargement::Reseau(e) => {
                format!("Le téléchargement a échoué (connexion). Vérifiez votre connexion Internet puis relancez : il reprendra où il s'est arrêté. Détail : {e}")
            }
            ErreurTelechargement::Disque(e) => format!("Écriture impossible sur le disque : {e}"),
            ErreurTelechargement::Integrite { .. } => {
                "Le fichier téléchargé est corrompu. Il a été supprimé ; relancez le téléchargement.".into()
            }
            ErreurTelechargement::Annule => "Téléchargement annulé.".into(),
        }
    }
}

pub fn sha256_fichier(chemin: &Path) -> std::io::Result<String> {
    let mut f = File::open(chemin)?;
    let mut h = Sha256::new();
    let mut tampon = vec![0u8; 1 << 20];
    loop {
        let n = f.read(&mut tampon)?;
        if n == 0 {
            break;
        }
        h.update(&tampon[..n]);
    }
    Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

pub fn est_present(fichier: &Fichier, dossier: &Path) -> bool {
    let chemin = dossier.join(fichier.nom);
    match fs::metadata(&chemin) {
        Ok(m) if m.len() == fichier.taille => {
            if fs::read_to_string(marqueur(fichier, dossier)).map(|s| s.trim() == fichier.sha256).unwrap_or(false) {
                return true;
            }
            let valide = sha256_fichier(&chemin).map(|s| s == fichier.sha256).unwrap_or(false);
            if valide {
                let _ = fs::write(marqueur(fichier, dossier), fichier.sha256);
            }
            valide
        }
        _ => false,
    }
}

fn marqueur(fichier: &Fichier, dossier: &Path) -> PathBuf {
    dossier.join(format!("{}.ok", fichier.nom))
}

pub struct Options {
    pub tentatives: u32,
    pub pause: Duration,
}

impl Default for Options {
    fn default() -> Self {
        Options { tentatives: 6, pause: Duration::from_secs(3) }
    }
}

pub fn telecharger(
    fichier: &Fichier,
    dossier: &Path,
    options: &Options,
    progression: &dyn Fn(u64, u64),
    annuler: &AtomicBool,
) -> Result<PathBuf, ErreurTelechargement> {
    fs::create_dir_all(dossier).map_err(|e| ErreurTelechargement::Disque(e.to_string()))?;
    let final_ = dossier.join(fichier.nom);
    let partiel = dossier.join(format!("{}.part", fichier.nom));
    if est_present(fichier, dossier) {
        progression(fichier.taille, fichier.taille);
        return Ok(final_);
    }
    let mut essai = 0;
    loop {
        if annuler.load(Ordering::Relaxed) {
            return Err(ErreurTelechargement::Annule);
        }
        match tenter(fichier, &partiel, progression, annuler) {
            Ok(()) => break,
            Err(ErreurTelechargement::Reseau(e)) => {
                essai += 1;
                if essai >= options.tentatives {
                    return Err(ErreurTelechargement::Reseau(e));
                }
                std::thread::sleep(options.pause);
            }
            Err(autre) => return Err(autre),
        }
    }
    let obtenu = sha256_fichier(&partiel).map_err(|e| ErreurTelechargement::Disque(e.to_string()))?;
    if obtenu != fichier.sha256 {
        let _ = fs::remove_file(&partiel);
        return Err(ErreurTelechargement::Integrite { attendu: fichier.sha256.into(), obtenu });
    }
    fs::rename(&partiel, &final_).map_err(|e| ErreurTelechargement::Disque(e.to_string()))?;
    let _ = fs::write(marqueur(fichier, dossier), fichier.sha256);
    Ok(final_)
}

fn tenter(
    fichier: &Fichier,
    partiel: &Path,
    progression: &dyn Fn(u64, u64),
    annuler: &AtomicBool,
) -> Result<(), ErreurTelechargement> {
    let deja = fs::metadata(partiel).map(|m| m.len()).unwrap_or(0);
    if deja > fichier.taille {
        let _ = fs::remove_file(partiel);
        return tenter(fichier, partiel, progression, annuler);
    }
    if deja == fichier.taille {
        return Ok(());
    }
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(15))
        .timeout_read(Duration::from_secs(30))
        .build();
    let mut requete = agent.get(fichier.url);
    if deja > 0 {
        requete = requete.set("Range", &format!("bytes={deja}-"));
    }
    let reponse = match requete.call() {
        Ok(r) => r,
        Err(ureq::Error::Status(416, _)) => {
            let _ = fs::remove_file(partiel);
            return Err(ErreurTelechargement::Reseau("reprise refusée par le serveur (416)".into()));
        }
        Err(e) => return Err(ErreurTelechargement::Reseau(e.to_string())),
    };
    let reprise = reponse.status() == 206;
    let mut sortie = if reprise {
        OpenOptions::new().append(true).open(partiel)
    } else {
        File::create(partiel)
    }
    .map_err(|e| ErreurTelechargement::Disque(e.to_string()))?;
    let mut position = if reprise { deja } else { 0 };
    let mut lecteur = reponse.into_reader();
    let mut tampon = vec![0u8; 256 * 1024];
    loop {
        if annuler.load(Ordering::Relaxed) {
            return Err(ErreurTelechargement::Annule);
        }
        let n = lecteur.read(&mut tampon).map_err(|e| ErreurTelechargement::Reseau(e.to_string()))?;
        if n == 0 {
            break;
        }
        sortie.write_all(&tampon[..n]).map_err(|e| ErreurTelechargement::Disque(e.to_string()))?;
        position += n as u64;
        progression(position, fichier.taille);
    }
    sortie.flush().map_err(|e| ErreurTelechargement::Disque(e.to_string()))?;
    if position != fichier.taille {
        return Err(ErreurTelechargement::Reseau(format!(
            "flux interrompu à {position} octets sur {}",
            fichier.taille
        )));
    }
    Ok(())
}

pub fn verifier_espace(libre_mo: u64, requis_mo: u64) -> Result<(), ErreurTelechargement> {
    if libre_mo < requis_mo {
        Err(ErreurTelechargement::Espace { libre_mo, requis_mo })
    } else {
        Ok(())
    }
}

pub fn extraire_zip(archive: &Path, destination: &Path) -> Result<(), String> {
    let f = File::open(archive).map_err(|e| e.to_string())?;
    let mut z = zip::ZipArchive::new(f).map_err(|e| e.to_string())?;
    fs::create_dir_all(destination).map_err(|e| e.to_string())?;
    z.extract(destination).map_err(|e| e.to_string())
}
