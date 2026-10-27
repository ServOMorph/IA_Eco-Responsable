# Signals — IA_Eco-Responsable   (MAJ 2026-10-27)

## Actions ouvertes
- [P1|ouvert] Faire l'essai sur le portable de référence (GTX 1650 Ti + Intel UHD, 8 Go de RAM) avec `/install_portable <utilisateur@ip>`, puis marquer la Phase 1 [FAIT]
  - fait quand: la commande va au bout sur le portable ; `controle.png` est valide ; le périphérique Vulkan choisi est la carte NVIDIA ; une image est générée depuis `app.exe` (raccourci du bureau) ; les seuils du garde-fou (5 000 Mo de RAM libre, 3 000 Mo de VRAM libre) sont confirmés ou ajustés ; la génération avec image de référence est essayée (temps, VRAM, garde-fou) ; les contrôles de `tests_manuels.md` sont validés et leurs sections supprimées.
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

## Dernière session (2026-10-27)
<!-- Écrasé intégralement par /close. Synthèse < 25 lignes. -->
# Session du 2026-10-27

## Décisions prises
- Image de référence conservée avec un écart fixe à 0,50 (plus de curseur) ; fonction de modification d'image abandonnée.
- Mode serveur `sd-server` rejeté (environ 7,7 Go de RAM résidents, incompatible avec un portable de 8 Go) ; `sd-cli` conservé.

## Livrables produits ou modifiés
- `app/` : nombre d'images (1 à 4), historique, suppression, progression, annulation, indicateur de mémoire, nom du modèle, journaux dans `journaux/` (log effacé si succès), image de référence : compilé en release, 40 tests unitaires et 7 de téléchargement passent.
- `run.py` : lance l'application (`--build` pour compiler).
- `phase0/reference/evaluer_reference.py` et `resultats/` : évaluation de 114 générations ; modification d'image : scripts et rapports supprimés.
- `.claude/commands/connect_portable.md` : créé.

## Hypothèses validées / invalidées
- VALIDE : la référence est suivie de 0,30 à 0,85 (structure 0,99 à 0,71), ignorée dès 0,90 ; 0,50 et 0,60 donnent le même résultat.
- INVALIDE : modifier un élément de la référence de façon fiable (8 cas sur 12 en jugement visuel, échec sur ciel et grandes surfaces).
- EN ATTENTE : référence sur GTX 1650 Ti ; paysage avec une référence carrée moins fiable (maison : 0,39) sur 2 cas ; affichage de la nouvelle phrase d'aide non contrôlé.

## Prochaine étape exacte
Obtenir le nom d'utilisateur et l'IP du portable (`activer_ssh.ps1`), puis `/connect_portable` et `/install_portable <utilisateur@ip>`.

## Question bloquante pour la session suivante
Aucune
