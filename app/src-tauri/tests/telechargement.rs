use app_lib::modeles::{self, ErreurTelechargement, Fichier, Options};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone, Copy)]
enum Mode {
    CoupePuisReprise,
    IgnoreRange,
    Normal,
}

fn contenu(taille: usize) -> Vec<u8> {
    (0..taille).map(|i| ((i * 31 + 7) % 251) as u8).collect()
}

fn sha(o: &[u8]) -> String {
    Sha256::digest(o).iter().map(|b| format!("{b:02x}")).collect()
}

fn serveur(corps: Vec<u8>, mode: Mode) -> (String, Arc<Mutex<Vec<Option<u64>>>>) {
    let ecoute = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/f.bin", ecoute.local_addr().unwrap());
    let journal = Arc::new(Mutex::new(Vec::new()));
    let j = journal.clone();
    std::thread::spawn(move || {
        let mut n = 0;
        for flux in ecoute.incoming() {
            let Ok(mut s) = flux else { break };
            let mut tampon = [0u8; 4096];
            let lu = s.read(&mut tampon).unwrap_or(0);
            let requete = String::from_utf8_lossy(&tampon[..lu]).to_string();
            let debut = requete
                .lines()
                .find_map(|l| l.to_lowercase().strip_prefix("range: bytes=").map(|v| v.trim_end_matches('-').parse::<u64>().unwrap()));
            j.lock().unwrap().push(debut);
            n += 1;
            let (statut, depart) = match (mode, debut) {
                (Mode::IgnoreRange, _) => ("200 OK", 0usize),
                (_, Some(d)) => ("206 Partial Content", d as usize),
                _ => ("200 OK", 0usize),
            };
            let reste = &corps[depart..];
            let entetes = format!(
                "HTTP/1.1 {statut}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                reste.len()
            );
            let _ = s.write_all(entetes.as_bytes());
            let envoyer = match (mode, n) {
                (Mode::CoupePuisReprise, 1) => reste.len() / 3,
                _ => reste.len(),
            };
            let _ = s.write_all(&reste[..envoyer]);
        }
    });
    (url, journal)
}

fn dossier(nom: &str) -> PathBuf {
    let d = std::env::temp_dir().join("ia_eco_tests").join(nom);
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn fichier(url: String, corps: &[u8], sha256: String) -> Fichier {
    Fichier {
        nom: "f.bin",
        url: Box::leak(url.into_boxed_str()),
        taille: corps.len() as u64,
        sha256: Box::leak(sha256.into_boxed_str()),
    }
}

fn options() -> Options {
    Options { tentatives: 4, pause: Duration::from_millis(50) }
}

#[test]
fn reprise_apres_coupure() {
    let corps = contenu(3_000_000);
    let (url, journal) = serveur(corps.clone(), Mode::CoupePuisReprise);
    let f = fichier(url, &corps, sha(&corps));
    let d = dossier("reprise");
    let chemin = modeles::telecharger(&f, &d, &options(), &|_, _| {}, &AtomicBool::new(false)).unwrap();
    assert_eq!(std::fs::read(chemin).unwrap(), corps);
    let requetes = journal.lock().unwrap().clone();
    assert_eq!(requetes[0], None);
    assert!(requetes[1].unwrap() > 0, "la reprise doit demander un Range");
    assert!(!d.join("f.bin.part").exists());
}

#[test]
fn serveur_ignorant_range_recommence() {
    let corps = contenu(600_000);
    let (url, _) = serveur(corps.clone(), Mode::IgnoreRange);
    let f = fichier(url, &corps, sha(&corps));
    let d = dossier("sans_range");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("f.bin.part"), &corps[..100_000]).unwrap();
    let chemin = modeles::telecharger(&f, &d, &options(), &|_, _| {}, &AtomicBool::new(false)).unwrap();
    assert_eq!(std::fs::read(chemin).unwrap(), corps);
}

#[test]
fn fichier_corrompu_supprime() {
    let corps = contenu(400_000);
    let (url, _) = serveur(corps.clone(), Mode::Normal);
    let f = fichier(url, &corps, "0".repeat(64));
    let d = dossier("corrompu");
    let e = modeles::telecharger(&f, &d, &options(), &|_, _| {}, &AtomicBool::new(false)).unwrap_err();
    assert!(matches!(e, ErreurTelechargement::Integrite { .. }));
    assert!(!d.join("f.bin").exists());
    assert!(!d.join("f.bin.part").exists());
}

#[test]
fn fichier_deja_present_non_retelecharge() {
    let corps = contenu(200_000);
    let (url, journal) = serveur(corps.clone(), Mode::Normal);
    let f = fichier(url, &corps, sha(&corps));
    let d = dossier("present");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("f.bin"), &corps).unwrap();
    modeles::telecharger(&f, &d, &options(), &|_, _| {}, &AtomicBool::new(false)).unwrap();
    assert!(journal.lock().unwrap().is_empty());
}

#[test]
fn annulation() {
    let corps = contenu(200_000);
    let (url, _) = serveur(corps.clone(), Mode::Normal);
    let f = fichier(url, &corps, sha(&corps));
    let d = dossier("annule");
    let e = modeles::telecharger(&f, &d, &options(), &|_, _| {}, &AtomicBool::new(true)).unwrap_err();
    assert!(matches!(e, ErreurTelechargement::Annule));
}

#[test]
fn serveur_injoignable_message_clair() {
    let corps = contenu(1000);
    let ecoute = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/f.bin", ecoute.local_addr().unwrap());
    drop(ecoute);
    let f = fichier(url, &corps, sha(&corps));
    let d = dossier("injoignable");
    let e = modeles::telecharger(&f, &d, &Options { tentatives: 2, pause: Duration::from_millis(10) }, &|_, _| {}, &AtomicBool::new(false)).unwrap_err();
    assert!(e.message().contains("connexion"));
}

#[test]
fn espace_disque_insuffisant() {
    assert!(matches!(modeles::verifier_espace(1000, 9000), Err(ErreurTelechargement::Espace { .. })));
    assert!(modeles::verifier_espace(9000, 9000).is_ok());
}
