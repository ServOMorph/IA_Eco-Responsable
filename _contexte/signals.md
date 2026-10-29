# Signals — IA_Eco-Responsable   (MAJ 2026-10-29)

## Actions ouvertes
- [P1|ouvert] Terminer l'essai sur le portable de référence (GTX 1650 Ti + Intel UHD, 8 Go de RAM), puis marquer la Phase 1 [FAIT]
  - fait quand: depuis `app.exe` (raccourci du bureau) une image est générée et son temps, la VRAM et la RAM libre sont relevés ; le seuil de RAM libre (1 500 Mo) et celui de VRAM libre (3 000 Mo) sont confirmés ou ajustés ; la génération avec image de référence est essayée (temps, VRAM, garde-fou) ; les contrôles de `tests_manuels.md` sont validés par l'utilisateur et leurs sections supprimées.
  - réf: `tests_manuels.md`, `roadmap_installateur.md` (Phase 1), `app/src-tauri/src/machine.rs` (seuils). Acquis le 2026-10-29 : installation complète et génération de contrôle réussies (63,7 s, carte NVIDIA) dans `C:\Users\raph6\Documents\ServOMorph\Test_Ia_Eco-Responsable` ; l'utilisateur indique que la génération depuis l'application fonctionne, sans relevé chiffré.
- [P2|ouvert] Comprendre les temps lents de premier passage sur le PC de développement (RTX 4060)
  - fait quand: la cause est identifiée (ou écartée) de 84 s (test de contrôle du 2026-10-22) et 102 s (premier portrait du 2026-10-24), alors que carré, paysage et portrait suivants prennent 12 à 17 s sur la même machine.
  - réf: `D:\Dev\ia-eco-test\sorties\controle.log`, `D:\Dev\ia-eco-test\sorties\image_*.log`, `phase0/resultats/`. Hypothèses non vérifiées : chargement à froid des modèles ou charge GPU passagère.
- [P2|ouvert] Fermer l'accès SSH du portable de test et supprimer les résidus
  - note: SSH rouvert le 2026-10-29 pour l'essai (`activer_ssh.ps1`, qui affiche une erreur système 1376 non bloquante ; cause non identifiée, hypothèse : copie du Bureau différente de celle du dépôt). Le dossier d'essai occupe environ 7,5 Go.
  - fait quand: sur le portable, `Get-WindowsCapability -Online -Name "OpenSSH.Server*"` indique `NotPresent`, `Test-Path "$env:ProgramData\ssh\administrators_authorized_keys"` indique `False`, `C:\Users\raph6\nettoyage_fini.txt`, `Downloads\activer_ssh (1).ps1` et les scripts temporaires `C:\Users\raph6\*.ps1` et `chk*.out` sont supprimés ; la décision sur le dossier `Test_Ia_Eco-Responsable` est prise et appliquée.
  - réf: `phase0/portable/desactiver_ssh.ps1` (copié dans `...\Test_Ia_Eco-Responsable\bin\`), `phase0/portable/nettoyage_final.ps1`.
- [P2|ouvert] Décider du nettoyage sur le PC de développement des modèles de test, de la clé SSH dédiée et de l'installation d'essai de la Phase 1
  - fait quand: décision de l'utilisateur appliquée pour `D:\ServOMorph\ComfyUI-Qwen\models` (SD 1.5, SDXL Lightning, Z-Image, text encoder ajoutés pendant les essais), `phase0/moteurs`, `phase0/modeles`, `~/.ssh/id_ed25519_portable_test` et `D:\Dev\ia-eco-test` (environ 7,4 Go de modèles, moteur Vulkan, image de contrôle). Attention : `D:\Dev\ia-eco-test` sert aussi de racine par défaut de l'interface sur ce PC.
  - réf: [à préciser] (inventaire dans la synthèse de session du 2026-10-20).

## Dernière session (2026-10-29)
<!-- Écrasé intégralement par /close. Synthèse < 25 lignes. -->
# Session du 2026-10-29

## Décisions prises
- Seuil de RAM libre du garde-fou abaissé de 5 000 à 1 500 Mo (décision de l'utilisateur, après refus à 5 000 puis à 2 500 Mo).
- Interface refaite pour PC (deux colonnes, charte du site SérénIA Tech, logo et liens du site et du mail) ; fenêtre 1280x900.
- Installation d'essai du portable placée dans `C:\Users\raph6\Documents\ServOMorph\Test_Ia_Eco-Responsable`.

## Livrables produits ou modifiés
- `app/src/` (index.html, styles.css, main.js, assets/) : nouvelle interface, polices et logo ; `tauri.conf.json` : taille de fenêtre.
- `app/src-tauri/src/machine.rs`, `moteur.rs` : seuil de RAM et tests adaptés (40 tests unitaires et 7 de téléchargement passent).
- `app/src-tauri/tests/installation_reelle.rs` : variable `IA_ECO_IGNORER_GARDE_RAM` (essai seulement).

## Hypothèses validées / invalidées
- VALIDE : `/install_portable` va au bout sur le portable ; contrôle 63,7 s en 512x512, périphérique Vulkan 0 = NVIDIA, image conforme (pomme).
- VALIDE : génération possible avec environ 2 900 Mo de RAM libre ; le seuil de 5 000 Mo était inatteignable (Windows et services occupent environ 4 Go sur 8).
- VALIDE (code) : un processus `sd-cli` par image, le modèle est libéré après chaque image.
- EN ATTENTE : RAM libre mesurée juste après une génération ; image de référence sur le portable ; ouverture des liens dans l'application ; seuil de 1 500 Mo sans relevé de temps ni de VRAM.

## Prochaine étape exacte
Générer depuis `app.exe` sur le portable en relevant temps, VRAM et RAM libre, essayer l'image de référence, puis valider `tests_manuels.md` et clore la Phase 1.

## Question bloquante pour la session suivante
Aucune
