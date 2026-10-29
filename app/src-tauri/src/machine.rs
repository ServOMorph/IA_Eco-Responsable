use serde::Serialize;
use std::path::Path;

pub const VRAM_MIN_MO: u64 = 3900;
pub const RAM_MIN_MO: u64 = 7500;
pub const RAM_LIBRE_LANCEMENT_MO: u64 = 1500;
pub const VRAM_LIBRE_LANCEMENT_MO: u64 = 3000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Marque {
    Nvidia,
    Amd,
    Intel,
    Autre,
}

impl Marque {
    pub fn depuis_id(id: u32) -> Self {
        match id {
            0x10DE => Marque::Nvidia,
            0x1002 | 0x1022 => Marque::Amd,
            0x8086 => Marque::Intel,
            _ => Marque::Autre,
        }
    }

    pub fn depuis_nom(nom: &str) -> Self {
        let n = nom.to_lowercase();
        if n.contains("nvidia") || n.contains("geforce") || n.contains("quadro") {
            Marque::Nvidia
        } else if n.contains("amd") || n.contains("radeon") {
            Marque::Amd
        } else if n.contains("intel") {
            Marque::Intel
        } else {
            Marque::Autre
        }
    }

    pub fn rang(self) -> u8 {
        match self {
            Marque::Nvidia => 3,
            Marque::Amd => 2,
            Marque::Intel => 1,
            Marque::Autre => 0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Gpu {
    pub nom: String,
    pub marque: Marque,
    pub vram_mo: u64,
    pub vram_libre_mo: Option<u64>,
    #[serde(skip)]
    pub luid: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Machine {
    pub gpus: Vec<Gpu>,
    pub ram_totale_mo: u64,
    pub ram_dispo_mo: u64,
    pub disque_libre_mo: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Raison {
    GpuInsuffisant { meilleur_mo: u64 },
    RamInsuffisante { totale_mo: u64 },
    DisqueInsuffisant { libre_mo: u64, requis_mo: u64 },
}

impl Raison {
    pub fn message(&self) -> String {
        match self {
            Raison::GpuInsuffisant { meilleur_mo } if *meilleur_mo == 0 => {
                "Aucune carte graphique avec mémoire dédiée n'a été trouvée. Il faut une carte d'au moins 4 Go de mémoire vidéo.".into()
            }
            Raison::GpuInsuffisant { meilleur_mo } => format!(
                "Votre carte graphique dispose de {} Go de mémoire vidéo ; il en faut au moins 4 Go.",
                meilleur_mo / 1024
            ),
            Raison::RamInsuffisante { totale_mo } => format!(
                "Votre ordinateur dispose de {} Go de mémoire ; il en faut au moins 8 Go.",
                totale_mo / 1024
            ),
            Raison::DisqueInsuffisant { libre_mo, requis_mo } => format!(
                "Espace disque insuffisant : {} Go libres, {} Go nécessaires.",
                libre_mo / 1024,
                requis_mo / 1024
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Verdict {
    Compatible,
    NonTeste,
    Incompatible(Vec<Raison>),
}

pub fn meilleur_gpu(gpus: &[Gpu]) -> Option<&Gpu> {
    gpus.iter()
        .filter(|g| g.vram_mo >= VRAM_MIN_MO)
        .max_by_key(|g| (g.marque.rang(), g.vram_mo))
}

pub fn evaluer(m: &Machine, disque_requis_mo: u64) -> Verdict {
    let mut raisons = Vec::new();
    let meilleur = meilleur_gpu(&m.gpus);
    if meilleur.is_none() {
        raisons.push(Raison::GpuInsuffisant {
            meilleur_mo: m.gpus.iter().map(|g| g.vram_mo).max().unwrap_or(0),
        });
    }
    if m.ram_totale_mo < RAM_MIN_MO {
        raisons.push(Raison::RamInsuffisante { totale_mo: m.ram_totale_mo });
    }
    if m.disque_libre_mo < disque_requis_mo {
        raisons.push(Raison::DisqueInsuffisant {
            libre_mo: m.disque_libre_mo,
            requis_mo: disque_requis_mo,
        });
    }
    if !raisons.is_empty() {
        return Verdict::Incompatible(raisons);
    }
    match meilleur.map(|g| g.marque) {
        Some(Marque::Nvidia) => Verdict::Compatible,
        _ => Verdict::NonTeste,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refus {
    RamLibre { dispo_mo: u64 },
    VramLibre { dispo_mo: u64 },
}

impl Refus {
    pub fn message(&self) -> String {
        match self {
            Refus::RamLibre { dispo_mo } => format!(
                "Pas assez de mémoire disponible ({} Mo, {} Mo nécessaires). Fermez d'autres applications puis réessayez.",
                dispo_mo, RAM_LIBRE_LANCEMENT_MO
            ),
            Refus::VramLibre { dispo_mo } => format!(
                "Pas assez de mémoire vidéo disponible ({} Mo, {} Mo nécessaires). Fermez les applications qui utilisent la carte graphique (jeux, vidéo) puis réessayez.",
                dispo_mo, VRAM_LIBRE_LANCEMENT_MO
            ),
        }
    }
}

pub fn garde_fou_memoire(ram_dispo_mo: u64, vram_libre_mo: Option<u64>) -> Result<(), Refus> {
    if ram_dispo_mo < RAM_LIBRE_LANCEMENT_MO {
        return Err(Refus::RamLibre { dispo_mo: ram_dispo_mo });
    }
    if let Some(v) = vram_libre_mo {
        if v < VRAM_LIBRE_LANCEMENT_MO {
            return Err(Refus::VramLibre { dispo_mo: v });
        }
    }
    Ok(())
}

pub fn analyser_typeperf(sortie: &str) -> Vec<(String, u64)> {
    let lignes: Vec<&str> = sortie.lines().filter(|l| l.starts_with('"')).collect();
    if lignes.len() < 2 {
        return Vec::new();
    }
    let decouper = |l: &str| -> Vec<String> {
        l.trim().trim_matches('"').split("\",\"").map(|c| c.to_string()).collect()
    };
    let noms = decouper(lignes[0]);
    let valeurs = decouper(lignes[1]);
    noms.iter()
        .zip(valeurs.iter())
        .skip(1)
        .filter_map(|(n, v)| {
            let debut = n.find('(')? + 1;
            let fin = n[debut..].find(')')? + debut;
            let octets: f64 = v.trim().parse().ok()?;
            Some((n[debut..fin].to_lowercase(), (octets / (1024.0 * 1024.0)) as u64))
        })
        .collect()
}

#[cfg(windows)]
mod win {
    use super::*;
    use windows::core::PCWSTR;
    use windows::Win32::Graphics::Dxgi::{
        CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE,
    };
    use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

    pub fn gpus() -> Vec<Gpu> {
        let mut liste = Vec::new();
        let Ok(factory) = (unsafe { CreateDXGIFactory1::<IDXGIFactory1>() }) else {
            return liste;
        };
        let mut i = 0;
        while let Ok(adapter) = unsafe { factory.EnumAdapters1(i) } {
            i += 1;
            let Ok(desc) = (unsafe { adapter.GetDesc1() }) else { continue };
            if desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 != 0 {
                continue;
            }
            let fin = desc.Description.iter().position(|&c| c == 0).unwrap_or(desc.Description.len());
            let nom = String::from_utf16_lossy(&desc.Description[..fin]);
            liste.push(Gpu {
                marque: Marque::depuis_id(desc.VendorId),
                nom,
                vram_mo: desc.DedicatedVideoMemory as u64 / (1024 * 1024),
                vram_libre_mo: None,
                luid: format!(
                    "luid_0x{:08x}_0x{:08x}",
                    desc.AdapterLuid.HighPart as u32, desc.AdapterLuid.LowPart
                ),
            });
        }
        let usages = usages_vram_mo();
        for g in &mut liste {
            if let Some(u) = usages.iter().find(|(l, _)| l.starts_with(&g.luid)).map(|(_, v)| *v) {
                g.vram_libre_mo = Some(g.vram_mo.saturating_sub(u));
            }
        }
        liste
    }

    fn usages_vram_mo() -> Vec<(String, u64)> {
        use std::os::windows::process::CommandExt;
        let Ok(sortie) = std::process::Command::new("typeperf")
            .args([r"\GPU Adapter Memory(*)\Dedicated Usage", "-sc", "1"])
            .creation_flags(0x0800_0000)
            .output()
        else {
            return Vec::new();
        };
        super::analyser_typeperf(&String::from_utf8_lossy(&sortie.stdout))
    }

    pub fn ram() -> (u64, u64) {
        let mut s = MEMORYSTATUSEX {
            dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
            ..Default::default()
        };
        if unsafe { GlobalMemoryStatusEx(&mut s) }.is_ok() {
            (s.ullTotalPhys / (1024 * 1024), s.ullAvailPhys / (1024 * 1024))
        } else {
            (0, 0)
        }
    }

    pub fn disque_libre(chemin: &Path) -> u64 {
        let mut existant = chemin;
        while !existant.exists() {
            match existant.parent() {
                Some(p) => existant = p,
                None => break,
            }
        }
        let mut large: Vec<u16> = existant.as_os_str().to_string_lossy().encode_utf16().collect();
        large.push(0);
        let mut libre = 0u64;
        if unsafe { GetDiskFreeSpaceExW(PCWSTR(large.as_ptr()), Some(&mut libre), None, None) }.is_ok() {
            libre / (1024 * 1024)
        } else {
            0
        }
    }
}

#[cfg(windows)]
pub fn detecter(dossier_installation: &Path) -> Machine {
    let (totale, dispo) = win::ram();
    Machine {
        gpus: win::gpus(),
        ram_totale_mo: totale,
        ram_dispo_mo: dispo,
        disque_libre_mo: win::disque_libre(dossier_installation),
    }
}

#[cfg(not(windows))]
pub fn detecter(_dossier_installation: &Path) -> Machine {
    Machine { gpus: vec![], ram_totale_mo: 0, ram_dispo_mo: 0, disque_libre_mo: 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gpu(nom: &str, marque: Marque, vram: u64) -> Gpu {
        Gpu { nom: nom.into(), marque, vram_mo: vram, vram_libre_mo: None, luid: String::new() }
    }

    fn machine(gpus: Vec<Gpu>, ram: u64, disque: u64) -> Machine {
        Machine { gpus, ram_totale_mo: ram, ram_dispo_mo: ram, disque_libre_mo: disque }
    }

    #[test]
    fn portable_de_reference_est_compatible() {
        let m = machine(
            vec![gpu("Intel UHD", Marque::Intel, 128), gpu("GTX 1650 Ti", Marque::Nvidia, 4096)],
            8018,
            120_000,
        );
        assert_eq!(evaluer(&m, 12_000), Verdict::Compatible);
        assert_eq!(meilleur_gpu(&m.gpus).unwrap().marque, Marque::Nvidia);
    }

    #[test]
    fn igpu_seul_est_incompatible() {
        let m = machine(vec![gpu("Intel UHD", Marque::Intel, 128)], 16000, 120_000);
        match evaluer(&m, 12_000) {
            Verdict::Incompatible(r) => assert!(matches!(r[0], Raison::GpuInsuffisant { meilleur_mo: 128 })),
            v => panic!("{v:?}"),
        }
    }

    #[test]
    fn sans_gpu_message_dedie() {
        let m = machine(vec![], 16000, 120_000);
        match evaluer(&m, 12_000) {
            Verdict::Incompatible(r) => assert!(r[0].message().contains("Aucune carte")),
            v => panic!("{v:?}"),
        }
    }

    #[test]
    fn ram_et_disque_cumules() {
        let m = machine(vec![gpu("RTX", Marque::Nvidia, 8192)], 4096, 5000);
        match evaluer(&m, 12_000) {
            Verdict::Incompatible(r) => assert_eq!(r.len(), 2),
            v => panic!("{v:?}"),
        }
    }

    #[test]
    fn amd_ou_intel_discret_non_teste() {
        let m = machine(vec![gpu("Radeon RX 6600", Marque::Amd, 8192)], 16000, 120_000);
        assert_eq!(evaluer(&m, 12_000), Verdict::NonTeste);
    }

    #[test]
    fn seuils_aux_limites() {
        let ok = machine(vec![gpu("RTX", Marque::Nvidia, VRAM_MIN_MO)], RAM_MIN_MO, 12_000);
        assert_eq!(evaluer(&ok, 12_000), Verdict::Compatible);
        let ko = machine(vec![gpu("RTX", Marque::Nvidia, VRAM_MIN_MO - 1)], RAM_MIN_MO, 12_000);
        assert!(matches!(evaluer(&ko, 12_000), Verdict::Incompatible(_)));
    }

    #[test]
    fn nvidia_prefere_sur_double_gpu() {
        let g = vec![gpu("Radeon", Marque::Amd, 8192), gpu("RTX", Marque::Nvidia, 4096)];
        assert_eq!(meilleur_gpu(&g).unwrap().marque, Marque::Nvidia);
    }

    #[test]
    fn marque_depuis_nom() {
        assert_eq!(Marque::depuis_nom("NVIDIA GeForce RTX 4060"), Marque::Nvidia);
        assert_eq!(Marque::depuis_nom("AMD Radeon RX 6600"), Marque::Amd);
        assert_eq!(Marque::depuis_nom("Intel(R) UHD Graphics"), Marque::Intel);
    }

    #[test]
    fn analyse_typeperf() {
        let s = concat!(
            "\r\n",
            r#""(PDH-CSV 4.0)","\\PC\GPU Adapter Memory(luid_0x00000000_0x0000c5e3_phys_0)\Dedicated Usage","\\PC\GPU Adapter Memory(luid_0x00000000_0x0000c6a1_phys_0)\Dedicated Usage""#,
            "\r\n",
            r#""10/22/2026 14:00:00.000","6850277376.000000","134217728.000000""#,
            "\r\nCommande terminée.\r\n"
        );
        let u = analyser_typeperf(s);
        assert_eq!(u.len(), 2);
        assert_eq!(u[0], ("luid_0x00000000_0x0000c5e3_phys_0".to_string(), 6532));
        assert_eq!(u[1].1, 128);
        assert!(analyser_typeperf("Erreur").is_empty());
    }

    #[test]
    fn garde_fou() {
        assert!(garde_fou_memoire(6000, Some(3500)).is_ok());
        assert!(garde_fou_memoire(6000, None).is_ok());
        assert_eq!(garde_fou_memoire(1000, Some(3500)), Err(Refus::RamLibre { dispo_mo: 1000 }));
        assert_eq!(garde_fou_memoire(6000, Some(2000)), Err(Refus::VramLibre { dispo_mo: 2000 }));
    }

    #[cfg(windows)]
    #[test]
    fn detection_reelle_coherente() {
        let m = detecter(Path::new("D:\\"));
        assert!(m.ram_totale_mo > 1000);
        assert!(m.ram_dispo_mo <= m.ram_totale_mo);
        assert!(m.disque_libre_mo > 0);
        assert!(!m.gpus.is_empty());
    }
}
