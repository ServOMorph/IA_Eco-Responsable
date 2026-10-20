# Signals — IA_Eco-Responsable   (MAJ 2026-10-20)

## Actions ouvertes
- [P1|ouvert] Démarrer la Phase 1 (cœur applicatif) de la roadmap, après `/compact` et confirmation écrite de l'utilisateur
  - fait quand: la Phase 1 de `roadmap_installateur.md` est marquée [FAIT] (détection machine, choix du backend, téléchargement des modèles, cycle de vie du moteur, génération de contrôle, tests passés).
  - réf: `roadmap_installateur.md` (Cadre, Phase 1), `phase0/resultats/`.
- [P2|ouvert] Vérifier sur le portable de test la fermeture complète de l'accès SSH et supprimer les résidus
  - fait quand: sur le portable, `Get-WindowsCapability -Online -Name "OpenSSH.Server*"` indique `NotPresent`, `Test-Path "$env:ProgramData\ssh\administrators_authorized_keys"` indique `False`, et `C:\Users\raph6\nettoyage_fini.txt` ainsi que `activer_ssh (1).ps1` (Téléchargements) sont supprimés.
  - réf: `phase0/portable/nettoyage_final.ps1`, `phase0/portable/desactiver_ssh.ps1`.
- [P2|ouvert] Décider du nettoyage sur le PC de développement des modèles de test et de la clé SSH dédiée
  - fait quand: décision de l'utilisateur appliquée pour `D:\ServOMorph\ComfyUI-Qwen\models` (SD 1.5, SDXL Lightning, Z-Image, text encoder ajoutés pendant les essais), `phase0/moteurs`, `phase0/modeles` et `~/.ssh/id_ed25519_portable_test`.
  - réf: [à préciser] (inventaire dans la synthèse de session du 2026-10-20).

## Dernière session (2026-10-20)
<!-- Écrasé intégralement par /close. Synthèse < 25 lignes. -->
# Session du 2026-10-20

## Décisions prises
- MVP : Windows seul, sans signature, interface en français ; GPU de 4 Go de VRAM minimum, toutes marques.
- Configuration retenue : Z-Image Turbo Q5_0, Qwen3-4B Q4_K_M, stable-diffusion.cpp Vulkan, options `--mmap --offload-to-cpu --vae-tiling`.
- Go de la phase 0 : 67 s par image en 512x512 jugé acceptable.
- Essais sur la machine réelle (accès SSH temporaire) plutôt qu'en simulation ; accès fermé et dossier de test supprimé.

## Livrables produits ou modifiés
- `roadmap_installateur.md` : créée, Phase 0 [FAIT] avec résultats.
- `phase0/` : banc de test (`mesure.ps1`, `analyse.py`, `rapport_phase0.py`, `portable/*`), mesures dans `phase0/resultats/`.
- Rapport de validation publié (artefact privé) : https://claude.ai/artifact/JA7z2SZBmLkCMGfzZDzj7U
- `README.md`, `CHANGELOG.md`, `_contexte/` : mis à jour.

## Hypothèses validées / invalidées
- VALIDE : Z-Image tourne sur GTX 1650 Ti 4 Go (VRAM 2,8 Go, RAM 4,7 Go), 67 s en 512x512 (275-310 s au départ).
- VALIDE : `--mmap` divise le temps par 2,3 sans changer l'image ; Vulkan plus rapide que CUDA sur cette carte.
- INVALIDE : une quantification plus petite accélère nettement -> le temps varie peu avec la quantification.
- EN ATTENTE : support Vulkan sur GPU AMD et Intel ; mode serveur du moteur ; qualité de Q8_0, Q3_K et du 1024 (non notés).

## Prochaine étape exacte
Faire `/compact`, confirmer par écrit, puis démarrer la Phase 1 (détection machine, backend, téléchargement, cycle de vie du moteur).

## Question bloquante pour la session suivante
Aucune
