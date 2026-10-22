# Signals — IA_Eco-Responsable   (MAJ 2026-10-22)

## Actions ouvertes
- [P1|ouvert] Terminer la Phase 1 : essai du cœur applicatif sur le portable de référence (GTX 1650 Ti + Intel UHD, 8 Go de RAM)
  - fait quand: sur le portable, le test `installation_reelle` produit une image valide, le périphérique Vulkan choisi est la carte NVIDIA, et les seuils du garde-fou (5 000 Mo de RAM libre, 3 000 Mo de VRAM libre) sont confirmés ou ajustés ; la Phase 1 de `roadmap_installateur.md` est alors marquée [FAIT].
  - réf: `tests_manuels.md`, `app/src-tauri/tests/installation_reelle.rs`, `roadmap_installateur.md` (Phase 1). Le portable n'a plus d'accès SSH : prévoir un moyen de transfert.
- [P2|ouvert] Comprendre le temps de 84 s de la génération de contrôle sur le PC de développement (RTX 4060)
  - fait quand: la cause est identifiée (ou écartée) d'un temps supérieur aux 67 s mesurés sur la GTX 1650 Ti, moins puissante.
  - réf: `D:\Dev\ia-eco-test\sorties\controle.log`, `phase0/resultats/`.
- [P2|ouvert] Vérifier sur le portable de test la fermeture complète de l'accès SSH et supprimer les résidus (reporté par l'utilisateur le 2026-10-22)
  - fait quand: sur le portable, `Get-WindowsCapability -Online -Name "OpenSSH.Server*"` indique `NotPresent`, `Test-Path "$env:ProgramData\ssh\administrators_authorized_keys"` indique `False`, et `C:\Users\raph6\nettoyage_fini.txt` ainsi que `activer_ssh (1).ps1` (Téléchargements) sont supprimés.
  - réf: `phase0/portable/nettoyage_final.ps1`, `phase0/portable/desactiver_ssh.ps1`.
- [P2|ouvert] Décider du nettoyage sur le PC de développement des modèles de test, de la clé SSH dédiée et de l'installation d'essai de la Phase 1
  - fait quand: décision de l'utilisateur appliquée pour `D:\ServOMorph\ComfyUI-Qwen\models` (SD 1.5, SDXL Lightning, Z-Image, text encoder ajoutés pendant les essais), `phase0/moteurs`, `phase0/modeles`, `~/.ssh/id_ed25519_portable_test` et `D:\Dev\ia-eco-test` (environ 7,4 Go de modèles, moteur Vulkan, image de contrôle).
  - réf: [à préciser] (inventaire dans la synthèse de session du 2026-10-20).

## Dernière session (2026-10-22)
<!-- Écrasé intégralement par /close. Synthèse < 25 lignes. -->
# Session du 2026-10-22

## Décisions prises
- Interface et cœur applicatif : Tauri + Rust (choisi par l'utilisateur) ; chaîne installée sur D:\Dev (Rust, Build Tools C++).
- Verdict machine : Compatible / NonTesté / Incompatible ; « lent » écarté faute de mesure.
- Garde-fou mémoire avant chaque lancement : 5 000 Mo de RAM libre et 3 000 Mo de VRAM libre (marges sur les pics de la phase 0).

## Livrables produits ou modifiés
- `app/` : projet Tauri ; cœur dans `app/src-tauri/src/` (machine, backend, modeles, moteur, controle). Tests dans le code et `app/src-tauri/tests/`.
- `tests_manuels.md` : créé (3 contrôles en attente).
- `roadmap_installateur.md`, `README.md`, `CHANGELOG.md`, `_contexte/` : mis à jour.

## Hypothèses validées / invalidées
- VALIDE : installation complète (moteur, 3 modèles) et génération de contrôle par le code, sur RTX 4060 : PNG valide, 84 s.
- VALIDE : 31 tests automatiques passent (24 unitaires, 7 sur le téléchargement avec reprise).
- INVALIDE : la VRAM libre via DXGI (usage du seul processus) -> pivot vers le compteur Windows « GPU Adapter Memory » (typeperf).
- EN ATTENTE : essai sur le portable (double GPU, 8 Go de RAM) ; temps de 84 s inexpliqué.
- Le serveur ComfyUI (port 8189) du PC de développement a été arrêté pour libérer 5 Go de VRAM ; à relancer si besoin.

## Prochaine étape exacte
Faire l'essai sur le portable de référence (voir `tests_manuels.md`), puis marquer la Phase 1 [FAIT] ; demander `/compact` avant la Phase 2.

## Question bloquante pour la session suivante
Aucune
