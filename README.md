# IA_Eco-Responsable

Interface open source pour installer et lancer des IA locales (chat, image) en un clic, adaptées à la machine de l'utilisateur, pensée pour les novices.

## État actuel
Phase 0 (faisabilité) terminée : Z-Image Turbo génère une image 512x512 en 67 s sur un portable à GPU de 4 Go de VRAM et 8 Go de RAM (GTX 1650 Ti), avec stable-diffusion.cpp en Vulkan.
Prochaine étape : Phase 1 de `roadmap_installateur.md` (détection de la machine, téléchargement des modèles, cycle de vie du moteur).

## Cible du MVP
Windows, interface en français, GPU de 4 Go de VRAM minimum, installation guidée depuis un lien de téléchargement.

## Structure
- `roadmap_installateur.md` : phases et résultats de la phase 0.
- `phase0/` : banc de test de faisabilité (scripts de mesure, analyse, rapport) et mesures dans `phase0/resultats/`. Les moteurs, modèles et images générées sont exclus de git.
- `_contexte/` : contexte et signaux de session.
