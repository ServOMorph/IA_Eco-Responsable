# Signals — IA_Eco-Responsable   (MAJ 2026-10-25)

## Actions ouvertes
- [P1|ouvert] Faire l'essai sur le portable de référence (GTX 1650 Ti + Intel UHD, 8 Go de RAM) avec `/install_portable <utilisateur@ip>`, puis marquer la Phase 1 [FAIT]
  - fait quand: la commande va au bout sur le portable ; `controle.png` est valide ; le périphérique Vulkan choisi est la carte NVIDIA ; une image est générée depuis `app.exe` (raccourci du bureau) ; les seuils du garde-fou (5 000 Mo de RAM libre, 3 000 Mo de VRAM libre) sont confirmés ou ajustés ; les 3 contrôles de `tests_manuels.md` sont validés et leurs sections supprimées.
  - réf: `.claude/commands/install_portable.md`, `tests_manuels.md`, `roadmap_installateur.md` (Phase 1), `phase0/portable/activer_ssh.ps1`. Prérequis : connexion SSH non établie. `raph6@192.168.1.187` refusait la clé (publickey) ; il faut le nom d'utilisateur et l'IP affichés par `activer_ssh.ps1` exécuté en administrateur sur le portable (SSH relancé le 2026-10-22). Alternative sans SSH : copier par clé USB l'exécutable du test et `app.exe`. La commande n'a jamais été exécutée : premier usage à surveiller.
- [P2|ouvert] Comprendre les temps lents de premier passage sur le PC de développement (RTX 4060)
  - fait quand: la cause est identifiée (ou écartée) de 84 s (test de contrôle du 2026-10-22) et 102 s (premier portrait du 2026-10-24), alors que carré, paysage et portrait suivants prennent 12 à 17 s sur la même machine.
  - réf: `D:\Dev\ia-eco-test\sorties\controle.log`, `D:\Dev\ia-eco-test\sorties\image_*.log`, `phase0/resultats/`. Hypothèses non vérifiées : chargement à froid des modèles ou charge GPU passagère.
- [P2|ouvert] Vérifier sur le portable de test la fermeture complète de l'accès SSH et supprimer les résidus (reporté par l'utilisateur le 2026-10-22)
  - note: OpenSSH Server a été réinstallé sur le portable le 2026-10-22 pour l'essai de la Phase 1 ; la fermeture est à refaire une fois l'essai terminé.
  - fait quand: sur le portable, `Get-WindowsCapability -Online -Name "OpenSSH.Server*"` indique `NotPresent`, `Test-Path "$env:ProgramData\ssh\administrators_authorized_keys"` indique `False`, et `C:\Users\raph6\nettoyage_fini.txt` ainsi que `activer_ssh (1).ps1` (Téléchargements) sont supprimés.
  - réf: `phase0/portable/nettoyage_final.ps1`, `phase0/portable/desactiver_ssh.ps1`.
- [P2|ouvert] Décider du nettoyage sur le PC de développement des modèles de test, de la clé SSH dédiée et de l'installation d'essai de la Phase 1
  - fait quand: décision de l'utilisateur appliquée pour `D:\ServOMorph\ComfyUI-Qwen\models` (SD 1.5, SDXL Lightning, Z-Image, text encoder ajoutés pendant les essais), `phase0/moteurs`, `phase0/modeles`, `~/.ssh/id_ed25519_portable_test` et `D:\Dev\ia-eco-test` (environ 7,4 Go de modèles, moteur Vulkan, image de contrôle). Attention : `D:\Dev\ia-eco-test` sert aussi de racine par défaut de l'interface sur ce PC.
  - réf: [à préciser] (inventaire dans la synthèse de session du 2026-10-20).

## Dernière session (2026-10-25)
<!-- Écrasé intégralement par /close. Synthèse < 25 lignes. -->
# Session du 2026-10-25

## Décisions prises
- Réorientation : l'installation se fera en présence de la personne, avec son portable relié par SSH ; Claude lance l'installation, puis l'utilisateur montre la génération. Chat hors périmètre.
- Commande `/install_portable` : compile et copie un exécutable de test et `app.exe`, télécharge moteur et modèles sur le portable, génère l'image de contrôle, crée le raccourci du bureau, propose la fermeture du SSH.

## Livrables produits ou modifiés
- `.claude/commands/install_portable.md` : créé, jamais exécuté.
- `app/src-tauri/src/appli.rs`, `lib.rs`, `Cargo.toml` : interface branchée sur le cœur Rust (3 commandes Tauri, verrou de génération unique, racine déduite de `<dossier>\bin`).
- `app/src/` (index.html, main.js, styles.css), `tauri.conf.json` : écran de génération en français (description, 3 formats, chronomètre).
- `tests_manuels.md` : contrôle de l'interface ajouté puis supprimé après validation.

## Hypothèses validées / invalidées
- VALIDE : 37 tests automatiques passent en debug (30 unitaires, 7 de téléchargement) ; l'interface génère des images sur RTX 4060.
- VALIDE : durées sur RTX 4060 : carré 12 s, paysage 16 s, portrait 102 s puis 17 s.
- INVALIDE : `cargo test --release` complet (panic = "abort" du profil release) ; seule la cible `installation_reelle` se compile en release.
- EN ATTENTE : essai sur le portable ; cause des temps lents du premier passage ; portrait 512x768 et paysage 768x512 non mesurés sur GTX 1650 Ti.

## Prochaine étape exacte
Obtenir le nom d'utilisateur et l'IP du portable (`activer_ssh.ps1` en administrateur), puis lancer `/install_portable <utilisateur@ip>`.

## Question bloquante pour la session suivante
Aucune
