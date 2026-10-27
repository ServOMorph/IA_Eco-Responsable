---
description: Donne la commande PowerShell à lancer sur le portable pour ouvrir SSH, puis se connecte au portable
argument-hint: [utilisateur@ip]
---

# /connect_portable [utilisateur@ip]

Arguments : `$ARGUMENTS`

## 1. Donner la commande (toujours, à chaque exécution)

Afficher à l'utilisateur, sans autre préambule, cette commande à lancer dans PowerShell **en administrateur** sur le portable :

```powershell
powershell -ExecutionPolicy Bypass -File "C:\Users\raph6\Desktop\activer_ssh.ps1"
```

Rappeler en une ligne : le script affiche en fin d'exécution le nom d'utilisateur et l'adresse IP.

## 2. Connexion

- Si `utilisateur@ip` est fourni en argument : tester `ssh -i ~/.ssh/id_ed25519_portable_test -o BatchMode=yes -o ConnectTimeout=8 -o StrictHostKeyChecking=accept-new <utilisateur@ip> "hostname; whoami"`.
- Sinon : attendre que l'utilisateur donne `utilisateur@ip` (valeur précédente connue, non garantie : `raph6@192.168.1.187`).
- Succès : le dire en une ligne (nom de machine et compte).
- `Permission denied (publickey)` : s'arrêter, ne pas insister, redonner la commande de l'étape 1.
