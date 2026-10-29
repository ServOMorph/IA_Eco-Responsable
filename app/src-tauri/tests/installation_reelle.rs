use app_lib::backend::Backend;
use app_lib::controle;
use app_lib::machine;
use app_lib::modeles::Options;
use app_lib::moteur::{self, Moteur};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

fn racine() -> PathBuf {
    PathBuf::from(std::env::var("IA_ECO_RACINE").unwrap_or_else(|_| "D:\\Dev\\ia-eco-test".into()))
}

#[test]
#[ignore = "télécharge environ 7,4 Go et génère une image ; lancer avec --ignored"]
fn installation_et_generation_de_controle() {
    let racine = racine();
    let annuler = AtomicBool::new(false);
    let dernier = Mutex::new((String::new(), 0u64));
    let suivi = |nom: &str, p: u64, t: u64| {
        let mut d = dernier.lock().unwrap();
        let palier = p * 10 / t.max(1);
        if d.0 != nom || d.1 != palier {
            eprintln!("{nom} : {} %", palier * 10);
            *d = (nom.to_string(), palier);
        }
    };
    let options = Options::default();

    let mut machine = machine::detecter(&racine);
    eprintln!("{machine:#?}");
    if std::env::var("IA_ECO_IGNORER_GARDE_RAM").is_ok() {
        eprintln!("garde-fou RAM neutralise pour l'essai : RAM libre reelle {} Mo", machine.ram_dispo_mo);
        machine.ram_dispo_mo = machine.ram_dispo_mo.max(machine::RAM_LIBRE_LANCEMENT_MO);
    }

    controle::installer_moteur(&racine, Backend::Vulkan, &options, &suivi, &annuler).unwrap();
    controle::installer_modeles(&racine, &options, &suivi, &annuler).unwrap();

    let choix = controle::preparer(&racine, &machine, None).unwrap();
    eprintln!("choix : {:?} {}", choix.backend, choix.peripherique);

    let moteur = Moteur {
        dossier_moteur: moteur::dossier_moteur(&racine, choix.backend),
        dossier_modeles: moteur::dossier_modeles(&racine),
        choix,
    };
    let sortie = racine.join("sorties").join("controle.png");
    let _ = std::fs::remove_file(&sortie);
    if std::env::var("IA_ECO_IGNORER_GARDE_RAM").is_ok() {
        let demande = moteur::Demande {
            prompt: controle::PROMPT_CONTROLE.into(),
            largeur: 512,
            hauteur: 512,
            graine: 42,
            sortie: sortie.clone(),
            journal: sortie.with_extension("log"),
            reference: None,
        };
        let generation = moteur.charger(&demande, &machine).unwrap();
        let duree = generation.attendre(controle::DELAI_CONTROLE).unwrap();
        assert!(controle::image_png_valide(&sortie));
        eprintln!("generation de controle : {:.1} s", duree.as_secs_f64());
        return;
    }
    let rapport = controle::generation_de_controle(&moteur, &sortie).unwrap();
    eprintln!("rapport : {rapport:?}");
    assert!(rapport.taille_octets > 10_000);
}
