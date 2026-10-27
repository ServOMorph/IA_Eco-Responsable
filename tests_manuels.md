# Tests manuels en attente

- Génération de contrôle réussie de bout en bout (Phase 1) : sur le PC de développement (RTX 4060), fermer navigateurs et applications utilisant la carte graphique pour libérer au moins 3 Go de VRAM, puis lancer `cargo test --test installation_reelle -- --ignored --nocapture` dans `app/src-tauri` (variable `IA_ECO_RACINE` pour changer le dossier, défaut `D:\Dev\ia-eco-test`). Attendu : image `sorties/controle.png` valide.
- Même test sur le portable de référence (GTX 1650 Ti + Intel UHD, 8 Go de RAM) : vérifier que le périphérique Vulkan choisi est bien la carte NVIDIA et que l'image est produite en environ 67 s.
- Vérifier sur le portable que le garde-fou ne refuse pas à tort avec 8 Go de RAM (seuil de RAM libre 5 000 Mo, à ajuster selon les mesures).
- Image de référence sur le portable de référence : charger une image, générer, relever le temps, la VRAM et l'acceptation par le garde-fou ; essayer aussi un format paysage avec une référence carrée (résultat moins fiable sur le PC de développement).
- Interface après compilation du 2026-10-27 : vérifier l'affichage de la phrase d'aide sous la référence (sans curseur).
