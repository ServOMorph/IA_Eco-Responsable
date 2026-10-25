# Roadmap — Installateur Z-Image pour novices

## Cadre
- Objectif : un programme Windows téléchargé depuis un site, qui installe Z-Image Turbo en quelques clics et génère des images en local, pensé pour des novices.
- Modèle retenu : Z-Image Turbo GGUF Q5_0 (diffusion), Qwen3-4B-Instruct-2507 Q4_K_M (text encoder), VAE `ae.safetensors`. Choix validé le 2026-10-20 sur le portable cible (voir Phase 0).
- Exigence minimale utilisateur : GPU avec au moins 4 Go de VRAM, quelle que soit la marque. Le programme détecte la machine et refuse l'installation sous ce seuil.
- Machine cible de référence (config fournie le 2026-10-18) : MSI GF63 Thin 10SCSR, Intel Core i5-10500H, 8 Go de RAM (7,83 Go utilisables), NVIDIA GeForce GTX 1650 Ti Max-Q 4 Go (carte Turing) + Intel UHD Graphics (double GPU), disque 477 Go dont environ 118 Go libres, Windows 11 Famille 25H2 (build 26200).
- Objectif de diffusion : fonctionner sur le plus de PC possible, donc plusieurs backends GPU dès la conception ; le support annoncé se limite à ce qui a été testé.
- Interface : Tauri + Rust (choisi le 2026-10-22), projet dans `app/`.
- Moteur : stable-diffusion.cpp (`sd-cli`, build master-973-f0aac52), backend Vulkan retenu, CUDA en alternative ; options `--mmap --offload-to-cpu --vae-tiling`. Validé en phase 0.

## Décisions ouvertes
- Windows seul pour le MVP : décidé (2026-10-18).
- Signature du programme : non (décidé 2026-10-18). L'avertissement SmartScreen sera traité par un guide à l'écran.
- Langue de l'interface : français seul (décidé 2026-10-18).
- RAM minimale exigée : 8 Go (pic mesuré 4,7 Go sur le portable cible, 7,83 Go utilisables ; non testé sous 8 Go).
- Backend sur GPU AMD et Intel : non testé (seul NVIDIA l'a été) ; support annoncé limité à ce qui est testé.
- Mode serveur du moteur (`sd-server.exe`) : non testé. Le chargement par image coûte environ 4 s avec `--mmap`, donc le bouton Charger/Décharger reste à définir (voir Phase 2).

## Phase 0 — Faisabilité du moteur [FAIT]
- Installer stable-diffusion.cpp (build CUDA) et faire tourner Z-Image Turbo GGUF Q4_K_M sur les 15 prompts de `prompts.json`, même graine.
- Mesurer la VRAM dédiée réelle du processus (compteurs Windows « GPU Process Memory », contexte CUDA inclus), pas seulement le pic de l'allocateur.
- Tester les options de réduction mémoire du moteur (déchargement CPU du text encoder, VAE en tuiles) sous un plafond équivalent à 4 Go.
- Vérifier si le moteur permet de garder le modèle en mémoire et de le décharger à la demande (mode serveur ou équivalent). Sinon, évaluer le coût d'un chargement par image.
- Tester la build Vulkan sur cette machine pour valider un second backend.
- Comparer résultats, temps et mémoire avec le run ComfyUI du 2026-10-01.
- Gate : franchi le 2026-10-20 (go de l'utilisateur : 67 s par image en 512x512 acceptable ; aucun plantage sur les 15 prompts).
- Résultats sur le portable cible (GTX 1650 Ti Max-Q 4 Go, 8 Go de RAM), données dans `phase0/resultats/` :
  - Départ (CUDA, Q3_K, sans mmap) : 275 à 310 s en 512x512. Retenu (Vulkan, Q5_0, mmap) : 67 s en 512x512, 125 s en 768x768, 233 s en 1024x1024.
  - Mémoire mesurée : VRAM dédiée 2,8 Go maximum, RAM 4,7 Go maximum (compteurs Windows).
  - `--mmap` a divisé le temps par 2,3 à image strictement identique ; Vulkan a gagné environ 30 % de plus ; la taille de quantification influe peu sur le temps (Q4_0 60 s, Q5_0 67 s, Q3_K 82 s, Q6_K 101 s, Q8_0 104 s en Vulkan).
  - Sans effet : flash attention, convolution directe, chargement anticipé, budget VRAM. Le déchargement en RAM est utile (132 s sans lui). Affinage 512 vers 1024 : 186 s, peu rentable. `easycache` : gain faible, risque qualité, écarté.
  - Qualité (notes de l'utilisateur, 15 prompts) : Q5_0 13 bon et 1 mauvais sur 14 notés ; Q4_0 11 bon, 1 moyen, 3 mauvais ; Q8_0, Q3_K, 1024 non notés.
  - Texte long dans l'image : 2 graines sur 6 donnent un texte exact, les autres ont une ou deux fautes de lettres. Limite du modèle, pas de la quantification.
  - Températures GPU autour de 85 °C pendant les séries.

**⏸ Checkpoint** — Demander à l'utilisateur de faire `/compact` avant de continuer.
Attendre sa réponse écrite. Ne pas commencer la phase suivante sans confirmation.

## Phase 1 — Cœur applicatif [EN COURS]
- Détection de la machine : GPU, VRAM (méthode fiable), RAM, disque libre, pilote ; verdict compatible / lent / non compatible.
- Sélection automatique du backend (Vulkan par défaut, CUDA en alternative) ; sur double GPU (NVIDIA + Intel intégré), forcer l'usage du GPU NVIDIA.
- Gestionnaire de modèles : téléchargement avec reprise et vérification d'intégrité, contrôle de l'espace disque.
- Cycle de vie du moteur : charger, décharger, garde-fou mémoire avant chaque chargement.
- Génération de contrôle à la fin de l'installation.
- Tests unitaires et tests d'intégration inclus dans la phase.
- État au 2026-10-22 : code et 31 tests automatiques en place dans `app/src-tauri/` ; installation et génération de contrôle réussies sur le PC de développement (RTX 4060, 84 s). Écarts : verdict « lent » remplacé par « NonTesté » (pas de mesure) ; pilote vérifié indirectement par la présence d'un périphérique Vulkan ; VRAM libre lue via le compteur Windows « GPU Adapter Memory ».
- État au 2026-10-25 : 37 tests automatiques. Réorientation (2026-10-25) : installation assistée par SSH, en présence de la personne, via la commande `.claude/commands/install_portable.md` (jamais exécutée). Une interface minimale de génération (description, 3 formats, image affichée) existe déjà (`app/src/`, `app/src-tauri/src/appli.rs`) ; sur RTX 4060 : carré 12 s, paysage 16 s, portrait 102 s puis 17 s.
- Reste avant [FAIT] : essai sur le portable de référence (double GPU, 8 Go de RAM) avec `/install_portable`, confirmation des seuils du garde-fou, voir `tests_manuels.md`.

**⏸ Checkpoint** — Demander à l'utilisateur de faire `/compact` avant de continuer.
Attendre sa réponse écrite. Ne pas commencer la phase suivante sans confirmation.

## Phase 2 — Interface [TODO]
- Note 2026-10-25 : un écran minimal est déjà livré (voir Phase 1) ; la suite (onglets Modèles et Installation, paramètres avancés, boutons Charger/Décharger) est à revoir selon la réorientation vers l'installation assistée.
- Onglet Modèles : liste (Z-Image seul), bouton Installer.
- Onglet Installation : guidage pas à pas, messages en langage simple.
- Onglet Génération : texte, format (carré, portrait, paysage), qualité, variante (graine), boutons Charger et Décharger, affichage de l'état mémoire.
- Paramètres avancés masqués par défaut.
- Proposer « Regénérer avec une autre variante » : le texte long dans l'image n'est exact que dans une partie des générations.
- Définir le sens des boutons Charger et Décharger (chargement par image d'environ 4 s avec mmap ; mode serveur non testé).

**⏸ Checkpoint** — Demander à l'utilisateur de faire `/compact` avant de continuer.
Attendre sa réponse écrite. Ne pas commencer la phase suivante sans confirmation.

## Phase 3 — Empaquetage et diffusion [TODO]
- Note 2026-10-25 : la réorientation vers l'installation assistée par SSH rend cette phase probablement sans objet à court terme ; décision de l'utilisateur à prendre.
- Installeur Windows, signature selon la décision ouverte, lien de téléchargement sur le site.
- Écran d'aide en cas d'avertissement SmartScreen si le programme n'est pas signé.

**⏸ Checkpoint** — Demander à l'utilisateur de faire `/compact` avant de continuer.
Attendre sa réponse écrite. Ne pas commencer la phase suivante sans confirmation.

## Phase 4 — Validation sur machines réelles [TODO]
- Test d'installation par l'ami, sans aide, sur son PC.
- Test sur au moins un PC avec GPU d'une autre marque, si disponible.
- Contrôles manuels consignés dans `tests_manuels.md`.

**⏸ Checkpoint** — Demander à l'utilisateur de faire `/compact` avant de continuer.
Attendre sa réponse écrite. Ne pas commencer la phase suivante sans confirmation.
