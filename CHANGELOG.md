# Changelog

## v0.5 — 2026-10-27

### Ajouté
- Interface : nombre d'images, historique, suppression, progression, annulation, indicateur de mémoire, nom du modèle, image de référence (écart fixe 0,50), journaux dans `journaux/`.
- `run.py` : lancement de l'application ; `.claude/commands/connect_portable.md`.
- `phase0/reference/` : évaluation de l'image de référence (114 générations).

### Modifié
- Décisions : modification d'image abandonnée ; `sd-server` rejeté.

## v0.4 — 2026-10-25

### Ajouté
- `.claude/commands/install_portable.md` : commande d'installation sur un portable distant par SSH (compilation, copie, téléchargement, image de contrôle, raccourci, fermeture du SSH). Jamais exécutée.
- Interface minimale de génération (`app/src/`, `app/src-tauri/src/appli.rs`) : description, trois formats, image affichée ; racine d'installation déduite de `<dossier>\bin`. 37 tests automatiques.
- Décision actée : installation assistée par SSH en présence de la personne ; chat hors périmètre.

### Modifié
- `Cargo.toml` : fonctionnalité `custom-protocol` de Tauri et dépendance `base64`.
- Mesures sur RTX 4060 : carré 12 s, paysage 16 s, portrait 102 s puis 17 s.

## v0.3 — 2026-10-22

### Modifié
- Accès SSH au portable de test relancé (OpenSSH réinstallé par `phase0/portable/activer_ssh.ps1`) pour l'essai de la Phase 1 ; connexion non encore établie, fermeture à refaire après l'essai.

## v0.2 — 2026-10-22

### Ajouté
- `app/` : projet Tauri + Rust. Cœur applicatif de la Phase 1 : détection de la machine et verdict, choix du backend Vulkan/CUDA depuis `sd-cli --list-devices`, téléchargement avec reprise et SHA-256, cycle de vie du moteur avec garde-fou mémoire, génération de contrôle. 31 tests automatiques, dont un test réel d'installation (`--ignored`).
- `tests_manuels.md` : file d'attente des contrôles manuels.
- Décisions actées : Tauri + Rust ; verdict Compatible / NonTesté / Incompatible ; garde-fou à 5 000 Mo de RAM libre et 3 000 Mo de VRAM libre.

### Corrigé
- VRAM libre : lecture via le compteur Windows « GPU Adapter Memory » (DXGI ne mesure que le processus appelant et laissait passer une génération vouée à l'échec).

## v0.1 — 2026-10-20

### Ajouté
- `roadmap_installateur.md` : roadmap de l'installateur Z-Image (phases 0 à 4), Phase 0 terminée.
- `phase0/` : banc de test de faisabilité sur machine réelle (mesure du temps, de la VRAM dédiée et de la RAM, analyse des journaux, rapport HTML de validation), mesures dans `phase0/resultats/`.
- Décisions actées : Windows seul, sans signature, interface en français, GPU de 4 Go minimum ; configuration retenue Z-Image Turbo Q5_0 avec stable-diffusion.cpp en Vulkan et `--mmap` (67 s en 512x512 sur GTX 1650 Ti, contre 275 à 310 s au départ).
