---
description: Installe Z-Image (moteur + modèles) sur un portable distant via SSH et lance la génération de contrôle
argument-hint: <utilisateur@ip> [dossier_installation]
---

# /install_portable <utilisateur@ip> [dossier_installation]

Installe sur un portable Windows distant, par SSH depuis ce PC, le moteur stable-diffusion.cpp et les 3 modèles Z-Image, lance la génération de contrôle, puis dépose l'application (interface de génération) et son raccourci sur le bureau. Le travail est fait par le cœur Rust de la Phase 1 (`app/src-tauri/tests/installation_reelle.rs`) : détection de la machine, garde-fou, téléchargement avec reprise et vérification SHA-256, génération. Ne pas réinventer l'installation à la main.

Arguments : `$ARGUMENTS`
- `utilisateur@ip` : obligatoire (affichés par `activer_ssh.ps1` sur le portable).
- `dossier_installation` : optionnel, défaut `C:\IA_Eco` sur le portable (variable `IA_ECO_RACINE`).

Clé SSH : `~/.ssh/id_ed25519_portable_test`. Options SSH communes : `-i ~/.ssh/id_ed25519_portable_test -o BatchMode=yes -o StrictHostKeyChecking=accept-new -o ServerAliveInterval=30`.
Les commandes distantes passent par `powershell -NoProfile -Command "..."` (le shell par défaut d'OpenSSH sous Windows est `cmd`).

## 0. Prérequis (arrêter et le dire si un point manque)

- Argument fourni, sinon demander.
- Sur le portable, la personne a consenti à l'accès SSH et a exécuté `phase0/portable/activer_ssh.ps1` en administrateur (SSH par clé uniquement, réseau local uniquement).
- Les deux machines sont sur le même réseau local.
- Le portable est branché sur secteur, et la personne a fermé navigateurs et applications lourdes : le garde-fou exige 5 000 Mo de RAM libre et 3 000 Mo de VRAM libre, seuils non encore validés sur un portable de 8 Go.

## 1. Connexion et pré-contrôle

1. `ssh <options> <utilisateur@ip> "echo ok"`. En cas de `Permission denied (publickey)` : s'arrêter, ne pas insister. La clé publique attendue est celle de `phase0/portable/activer_ssh.ps1`.
2. Relever sur le portable, en une commande PowerShell : GPU (`Win32_VideoController` : nom, `AdapterRAM`), RAM totale et libre, espace libre du lecteur d'installation, version de Windows.
3. Afficher ce relevé à l'utilisateur. Refuser de continuer si aucun GPU NVIDIA ou si l'espace libre est inférieur à environ 9 Go. Le support annoncé est limité à ce qui a été testé (NVIDIA, au moins 4 Go de VRAM, 8 Go de RAM) : tout autre cas se signale à l'utilisateur avant de poursuivre.

## 2. Compilation du test (sur ce PC)

Dans `app/src-tauri`, deux compilations release :
- `cargo test --release --test installation_reelle --no-run` (ne pas lancer `cargo test --release` sans `--test` : le profil `panic = "abort"` fait échouer les autres cibles de test ; les 37 tests automatiques se lancent en debug avec `cargo test`).
- `cargo build --release` : produit `target\release\app.exe`, l'interface.
- Si `cargo` est introuvable : `RUSTUP_HOME=D:\Dev\rustup`, `CARGO_HOME=D:\Dev\cargo`, `D:\Dev\cargo\bin` dans le PATH.
- Relever dans la sortie le chemin de l'exécutable (`target\release\deps\installation_reelle-<hash>.exe`).
- Compiler le test en release : la vérification SHA-256 de 7,4 Go est trop lente en debug.

## 3. Copie sur le portable

1. Créer le dossier d'installation et `<dossier>\bin` sur le portable.
2. `scp` de l'exécutable du test vers `<dossier>\bin\installation_reelle.exe`.
3. `scp` de `target\release\app.exe` vers `<dossier>\bin\app.exe`. L'application déduit le dossier d'installation de son emplacement (`<dossier>\bin`) : ne pas la placer ailleurs.
4. `scp` de `phase0/portable/desactiver_ssh.ps1` vers `<dossier>\bin\` (pour la fermeture en fin de séance).

## 4. Installation et génération de contrôle

Le test télécharge environ 7,4 Go puis génère `sorties\controle.png`. Les sessions SSH tuent les processus enfants à la déconnexion : garder la session ouverte pendant toute l'exécution.

1. Lancer en arrière-plan local (Bash avec `run_in_background`) une commande `ssh` qui exécute, sur le portable, `$env:IA_ECO_RACINE='<dossier>'; & '<dossier>\bin\installation_reelle.exe' --ignored --nocapture 2>&1 | Tee-Object '<dossier>\installation.log'`.
2. Suivre l'avancement par une seconde connexion SSH qui lit la fin de `<dossier>\installation.log` (pas de boucle serrée : un contrôle toutes les 2 à 5 minutes). Le téléchargement reprend si la connexion Internet du portable coupe ; si le test échoue sur le réseau, le relancer (reprise automatique).
3. En fin de test, lire dans le journal : la machine détectée, le périphérique Vulkan choisi, le rapport (durée, taille).

## 5. Raccourci et contrôle du résultat

- Créer sur le bureau de l'utilisateur du portable un raccourci « Générateur d'images » vers `<dossier>\bin\app.exe` (PowerShell : `WScript.Shell`, `CreateShortcut`, dossier `[Environment]::GetFolderPath('Desktop')`). Une application lancée par SSH ne s'affiche pas sur le bureau de la personne : c'est elle qui double-clique sur le raccourci.

- Récupérer `<dossier>\sorties\controle.png` par `scp` dans le scratchpad, puis le lire avec Read pour contrôle visuel (une pomme rouge sur une table en bois).
- Contrôler que le périphérique Vulkan choisi est la carte NVIDIA et non l'Intel intégré.
- Comparer la durée au repère de 67 s mesuré sur la GTX 1650 Ti (indication, pas critère).
- Si le garde-fou refuse (RAM ou VRAM libre insuffisante) : le dire avec les valeurs mesurées, demander à la personne de fermer des applications, relancer. Ne jamais contourner ni modifier les seuils sans accord de l'utilisateur.

## 6. Compte rendu et fermeture

- Rendre un compte rendu court : machine, espace utilisé, durée, résultat, anomalies. Distinguer ce qui a été observé de ce qui est supposé.
- Si l'essai concerne le portable de référence, rappeler d'inscrire le résultat dans `tests_manuels.md` (contrôles à valider par l'utilisateur) ; ne pas supprimer ces sections soi-même.
- Fermeture SSH : demander à l'utilisateur s'il veut la faire maintenant. Si oui, lui indiquer d'exécuter `<dossier>\bin\desactiver_ssh.ps1` en administrateur sur le portable (service arrêté, règle pare-feu et clé retirées). Cette fermeture coupe la connexion en cours : la faire en dernier. Le composant OpenSSH Server reste installé (désactivé) ; `nettoyage_final.ps1` contient des chemins propres à ce PC et ne doit pas être utilisé sur le portable d'un tiers.

## Limites actuelles

- L'interface (`app.exe`) est minimale : description, trois formats (carré 512x512 mesuré ; portrait 512x768 et paysage 768x512 non mesurés), bouton Générer, image affichée. Elle n'a pas encore généré d'image réelle (le garde-fou a refusé lors du seul essai, VRAM libre insuffisante sur ce PC) : première génération depuis l'interface à surveiller sur le portable.
- Backend Vulkan uniquement (les tests CUDA ne sont pas câblés dans le test d'installation).
- Aucune exécution de cette commande n'a encore été faite sur un portable : premier usage = essai à surveiller.
