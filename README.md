# IA_Eco-Responsable

Interface open source pour installer et lancer des IA locales (chat, image) en un clic, adaptées à la machine de l'utilisateur, pensée pour les novices.

## État actuel
Phase 0 (faisabilité) terminée : Z-Image Turbo génère une image 512x512 en 67 s sur un portable à GPU de 4 Go de VRAM et 8 Go de RAM (GTX 1650 Ti), avec stable-diffusion.cpp en Vulkan.
Phase 1 (cœur applicatif) en cours : détection de la machine, choix du backend, téléchargement avec reprise, cycle de vie du moteur et génération de contrôle sont écrits en Rust (Tauri) et testés ; interface de génération pour PC (3 formats, 1 à 4 images, historique, annulation, image de référence, charte SérénIA Tech) ; 40 tests unitaires. Déploiement actuel : installation assistée par SSH via `/install_portable`, essayée sur le portable de référence le 2026-10-29 (contrôle en 63,7 s). Reste : relevés depuis l'application sur le portable et validation des seuils du garde-fou.

## Cible du MVP
Windows, interface en français, GPU de 4 Go de VRAM minimum, installation guidée depuis un lien de téléchargement.

## Structure
- `roadmap_installateur.md` : phases et résultats de la phase 0.
- `app/` : projet Tauri + Rust ; cœur applicatif dans `app/src-tauri/src/`, tests dans `app/src-tauri/tests/` (`cargo test` ; le test réel `installation_reelle` se lance avec `-- --ignored`).
- `.claude/commands/install_portable.md` : commande d'installation sur un portable distant par SSH.
- `tests_manuels.md` : contrôles manuels en attente.
- `phase0/` : banc de test de faisabilité (scripts de mesure, analyse, rapport) et mesures dans `phase0/resultats/`. Les moteurs, modèles et images générées sont exclus de git.
- `_contexte/` : contexte et signaux de session.
