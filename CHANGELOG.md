# Changelog

## v0.1 — 2026-10-20

### Ajouté
- `roadmap_installateur.md` : roadmap de l'installateur Z-Image (phases 0 à 4), Phase 0 terminée.
- `phase0/` : banc de test de faisabilité sur machine réelle (mesure du temps, de la VRAM dédiée et de la RAM, analyse des journaux, rapport HTML de validation), mesures dans `phase0/resultats/`.
- Décisions actées : Windows seul, sans signature, interface en français, GPU de 4 Go minimum ; configuration retenue Z-Image Turbo Q5_0 avec stable-diffusion.cpp en Vulkan et `--mmap` (67 s en 512x512 sur GTX 1650 Ti, contre 275 à 310 s au départ).
