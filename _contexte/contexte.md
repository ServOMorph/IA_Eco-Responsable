# Contexte — IA_Eco-Responsable

## Objectif (immuable sauf décision explicite)
Interface open source pour installer et lancer des IA locales (chat, image) en un clic, adaptées à la machine de l'utilisateur, pensée pour les novices.

## Stack / contraintes techniques (stable, rarement modifié)
- Cible MVP : Windows, interface en français, GPU de 4 Go de VRAM minimum (toutes marques), 8 Go de RAM, sans signature de code.
- Génération d'image : Z-Image Turbo GGUF Q5_0 + Qwen3-4B-Instruct-2507 Q4_K_M + VAE `ae.safetensors`, via stable-diffusion.cpp (`sd-cli`, build master-973-f0aac52), backend Vulkan (CUDA en alternative), options `--mmap --offload-to-cpu --vae-tiling`.
- Interface et cœur : Tauri + Rust (choisi le 2026-10-22), projet dans `app/`. Chaîne de build sur D:\Dev (Rust via `RUSTUP_HOME`/`CARGO_HOME`, Build Tools C++).
- Chat : hors périmètre immédiat, l'image est le besoin prioritaire.

## État actuel (réécrit intégralement à chaque /close)
Phase 0 terminée. Phase 1 en cours : cœur Rust et interface complète (3 formats, 1 à 4 images, historique, suppression, progression, annulation, mémoire, image de référence à écart fixe 0,50) fonctionnent sur la RTX 4060 ; 40 tests unitaires.
Déploiement : installation assistée par SSH via `/install_portable` (jamais exécutée).
Reste pour clore la Phase 1 : essai sur le portable de référence (connexion SSH à établir, voir `signals.md` et `tests_manuels.md`).
Nettoyages du portable et du PC de développement en attente.

## Décisions structurantes (append only — 10 entrées max, 5 lignes max/entrée, archiver au-delà)
- 2026-10-17 : Initialisation du protocole vibecoding.
- 2026-10-20 : Périmètre MVP : Windows seul, sans signature (guide SmartScreen à l'écran), français seul, GPU de 4 Go de VRAM minimum toutes marques ; l'image est prioritaire sur le chat.
- 2026-10-20 : Configuration retenue : Z-Image Turbo Q5_0, stable-diffusion.cpp Vulkan, `--mmap --offload-to-cpu --vae-tiling`. Mesure : 67 s en 512x512, 125 s en 768x768, 233 s en 1024x1024 sur GTX 1650 Ti. Détail dans `roadmap_installateur.md` (Phase 0).
- 2026-10-20 : Q5_0 préféré à Q4_0 après validation visuelle (13 bon sur 14 contre 11 sur 15, pour 7 s de plus par image).
- 2026-10-20 : Le texte long dans l'image n'est exact que pour une partie des graines : l'interface proposera de regénérer avec une autre variante.
- 2026-10-20 : Les mesures de faisabilité se font sur la machine réelle (accès SSH temporaire, retiré ensuite) et non en simulation.
- 2026-10-22 : Stack Tauri + Rust retenue. Verdict machine Compatible / NonTesté / Incompatible (pas de « lent » sans mesure). Garde-fou avant lancement : 5 000 Mo de RAM libre, 3 000 Mo de VRAM libre (VRAM libre lue via le compteur Windows « GPU Adapter Memory », DXGI inutilisable).
- 2026-10-25 : Réorientation : installation assistée par SSH sur le portable de la personne, en sa présence (commande `/install_portable`), plutôt qu'un installateur grand public. Interface minimale de génération livrée en avance sur la Phase 2. Chat hors périmètre.
- 2026-10-27 : Image de référence gardée avec un écart fixe à 0,50 ; modification d'image abandonnée ; `sd-server` rejeté (7,7 Go de RAM) au profit de `sd-cli`.
