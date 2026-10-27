import json
import subprocess
import sys
import time
from pathlib import Path

import numpy as np
from PIL import Image

RACINE = Path(r"D:\Dev\ia-eco-test")
MOTEUR = RACINE / "moteur" / "vulkan" / "sd-cli.exe"
MODELES = RACINE / "modeles"
SORTIE = RACINE / "eval_reference"
RESULTATS = Path(__file__).resolve().parent / "resultats"
PERIPHERIQUE = "Vulkan0"

REFERENCES = [
    ("theiere", "a purple teapot with golden stripes on a bright red table, white wall in the background, studio photo"),
    ("voiture", "a yellow sports car parked on a bright green grass field under a deep blue sky, wide shot"),
    ("maison", "a small blue house with an orange roof on a snowy hill, grey sky"),
    ("fleur", "a single pink flower in a black vase on a white background, minimalist photo"),
    ("phare", "a red and white striped lighthouse on a dark rocky coast at sunset, orange sky"),
    ("chat", "a black cat sitting on a bright yellow sofa, indoor, side view"),
]
PROMPTS_NEUTRES = [
    "an oil painting of a still life, dramatic lighting",
    "a watercolor illustration, soft pastel colors",
]
ECARTS = [0.3, 0.5, 0.7, 0.85, 0.9, 0.95, 1.0]
FORMATS = {"carre": (512, 512), "portrait": (512, 768), "paysage": (768, 512)}
CAS_FORMAT = [("theiere", "portrait"), ("theiere", "paysage"), ("maison", "portrait"), ("maison", "paysage")]
ECARTS_FORMAT = [0.5, 0.7]


def commande(prompt, largeur, hauteur, graine, sortie, reference=None, ecart=None):
    a = [
        str(MOTEUR),
        "--diffusion-model", str(MODELES / "z_image_turbo-Q5_0.gguf"),
        "--vae", str(MODELES / "ae.safetensors"),
        "--llm", str(MODELES / "Qwen3-4B-Instruct-2507-Q4_K_M.gguf"),
        "-p", prompt,
        "--cfg-scale", "1.0",
        "--steps", "8",
        "-W", str(largeur),
        "-H", str(hauteur),
        "-s", str(graine),
        "--offload-to-cpu", "--vae-tiling", "--mmap",
        "--backend", PERIPHERIQUE,
        "-o", str(sortie),
    ]
    if reference is not None:
        a += ["-i", str(reference), "--strength", f"{ecart:.2f}"]
    return a


def generer_image(prompt, format_, graine, sortie, reference=None, ecart=None):
    if sortie.is_file() and sortie.stat().st_size > 0:
        return None
    sortie.parent.mkdir(parents=True, exist_ok=True)
    largeur, hauteur = FORMATS[format_]
    debut = time.time()
    journal = sortie.with_suffix(".log")
    with open(journal, "w", encoding="utf-8", errors="replace") as f:
        code = subprocess.run(
            commande(prompt, largeur, hauteur, graine, sortie, reference, ecart),
            cwd=MOTEUR.parent, stdout=f, stderr=subprocess.STDOUT, stdin=subprocess.DEVNULL,
        ).returncode
    duree = time.time() - debut
    if code != 0 or not sortie.is_file():
        print(f"ECHEC {sortie.name} code={code}", flush=True)
        return {"echec": True, "fichier": sortie.name, "duree": duree}
    journal.unlink(missing_ok=True)
    print(f"ok {sortie.name} {duree:.1f} s", flush=True)
    return {"echec": False, "fichier": sortie.name, "duree": duree}


def plan():
    cas = []
    for i, (nom, prompt) in enumerate(REFERENCES):
        cas.append({"type": "reference", "ref": nom, "prompt": prompt, "format": "carre", "graine": 1000 + i,
                    "sortie": SORTIE / "refs" / f"{nom}_carre.png"})
    graine = 5000
    for nom, _ in REFERENCES:
        for p, prompt in enumerate(PROMPTS_NEUTRES):
            graine += 1
            base = {"ref": nom, "prompt": prompt, "format": "carre", "graine": graine, "p": p}
            cas.append({**base, "type": "temoin", "sortie": SORTIE / "temoins" / f"{nom}_p{p}.png"})
            for e in ECARTS:
                cas.append({**base, "type": "avec", "ecart": e,
                            "sortie": SORTIE / "avec" / f"{nom}_p{p}_e{int(e * 100):03d}.png"})
    for nom, fmt in CAS_FORMAT:
        graine += 1
        base = {"ref": nom, "prompt": PROMPTS_NEUTRES[0], "format": fmt, "graine": graine, "p": 0}
        cas.append({**base, "type": "temoin_format", "sortie": SORTIE / "format" / f"{nom}_{fmt}_temoin.png"})
        for e in ECARTS_FORMAT:
            cas.append({**base, "type": "avec_format", "ecart": e,
                        "sortie": SORTIE / "format" / f"{nom}_{fmt}_e{int(e * 100):03d}.png"})
    return cas


def generer():
    cas = plan()
    refs = {c["ref"]: c["sortie"] for c in cas if c["type"] == "reference"}
    mesures = []
    for n, c in enumerate(cas, 1):
        print(f"[{n}/{len(cas)}] ", end="", flush=True)
        reference = refs[c["ref"]] if c["type"] in ("avec", "avec_format") else None
        r = generer_image(c["prompt"], c["format"], c["graine"], c["sortie"], reference, c.get("ecart"))
        if r:
            mesures.append(r)
    RESULTATS.mkdir(parents=True, exist_ok=True)
    (RESULTATS / "durees.json").write_text(json.dumps(mesures, indent=1), encoding="utf-8")
    echecs = [m for m in mesures if m["echec"]]
    print(f"termine : {len(mesures)} generations, {len(echecs)} echecs", flush=True)


def charger(chemin, taille=32):
    return np.asarray(Image.open(chemin).convert("RGB").resize((taille, taille), Image.LANCZOS), dtype=np.float64)


def correlation_structure(a, b):
    ga = a.mean(axis=2).ravel()
    gb = b.mean(axis=2).ravel()
    ga, gb = ga - ga.mean(), gb - gb.mean()
    d = np.linalg.norm(ga) * np.linalg.norm(gb)
    return float(ga @ gb / d) if d > 0 else 0.0


def similarite_couleur(a, b):
    ga = np.asarray(Image.fromarray(a.astype(np.uint8)).resize((8, 8), Image.BOX), dtype=np.float64)
    gb = np.asarray(Image.fromarray(b.astype(np.uint8)).resize((8, 8), Image.BOX), dtype=np.float64)
    return float(1.0 - np.abs(ga - gb).mean() / 255.0)


def analyser():
    cas = plan()
    noms = [n for n, _ in REFERENCES]
    refs = {n: charger(SORTIE / "refs" / f"{n}_carre.png") for n in noms}
    lignes = []
    manquants = [c["sortie"].name for c in cas if not c["sortie"].is_file()]
    if manquants:
        print(f"{len(manquants)} images manquantes : {manquants[:5]}...")

    def evaluer(c):
        sortie = charger(c["sortie"])
        propre = {
            "structure": correlation_structure(sortie, refs[c["ref"]]),
            "couleur": similarite_couleur(sortie, refs[c["ref"]]),
        }
        autres_s = [correlation_structure(sortie, refs[n]) for n in noms]
        autres_c = [similarite_couleur(sortie, refs[n]) for n in noms]
        propre["id_structure"] = int(int(np.argmax(autres_s)) == noms.index(c["ref"]))
        propre["id_couleur"] = int(int(np.argmax(autres_c)) == noms.index(c["ref"]))
        propre["id_les_deux"] = int(
            int(np.argmax(np.array(autres_s) + np.array(autres_c))) == noms.index(c["ref"])
        )
        propre["structure_autres"] = float(np.mean([s for n, s in zip(noms, autres_s) if n != c["ref"]]))
        propre["couleur_autres"] = float(np.mean([s for n, s in zip(noms, autres_c) if n != c["ref"]]))
        return propre

    groupes = {}
    for c in cas:
        if c["type"] not in ("temoin", "avec") or not c["sortie"].is_file():
            continue
        cle = "temoin" if c["type"] == "temoin" else f"e{c['ecart']:.2f}"
        groupes.setdefault(cle, []).append({**evaluer(c), "ref": c["ref"], "p": c["p"]})

    def moyenne(liste, champ):
        return float(np.mean([x[champ] for x in liste])) if liste else float("nan")

    ordre = ["temoin"] + [f"e{e:.2f}" for e in ECARTS]
    resume = {}
    for cle in ordre:
        g = groupes.get(cle, [])
        resume[cle] = {
            "n": len(g),
            "structure": moyenne(g, "structure"),
            "structure_autres": moyenne(g, "structure_autres"),
            "couleur": moyenne(g, "couleur"),
            "couleur_autres": moyenne(g, "couleur_autres"),
            "id_structure": moyenne(g, "id_structure"),
            "id_couleur": moyenne(g, "id_couleur"),
            "id_les_deux": moyenne(g, "id_les_deux"),
        }

    par_ref = {}
    for cle in ["temoin", "e0.50"]:
        for x in groupes.get(cle, []):
            par_ref.setdefault(x["ref"], {}).setdefault(cle, []).append(x["structure"])

    format_ = []
    for c in cas:
        if c["type"] in ("temoin_format", "avec_format") and c["sortie"].is_file():
            sortie = charger(c["sortie"])
            format_.append({
                "ref": c["ref"], "format": c["format"], "ecart": c.get("ecart"),
                "structure": correlation_structure(sortie, refs[c["ref"]]),
                "couleur": similarite_couleur(sortie, refs[c["ref"]]),
            })

    RESULTATS.mkdir(parents=True, exist_ok=True)
    (RESULTATS / "analyse.json").write_text(
        json.dumps({"resume": resume, "format": format_, "manquants": manquants}, indent=1), encoding="utf-8"
    )

    out = ["# Evaluation des images de reference", "", f"Images analysees : {sum(r['n'] for r in resume.values())} (carre) + {len(format_)} (formats)", ""]
    out.append("| Condition | n | structure (propre ref) | structure (autres refs) | couleur (propre) | couleur (autres) | id structure | id couleur | id combine |")
    out.append("|---|---|---|---|---|---|---|---|---|")
    for cle in ordre:
        r = resume[cle]
        out.append(
            f"| {cle} | {r['n']} | {r['structure']:.2f} | {r['structure_autres']:.2f} | {r['couleur']:.2f} | "
            f"{r['couleur_autres']:.2f} | {r['id_structure']:.0%} | {r['id_couleur']:.0%} | {r['id_les_deux']:.0%} |"
        )
    out += ["", f"Hasard pour l'identification : {1 / len(noms):.0%}.", "", "## Formats differents de la reference", ""]
    out.append("| Ref | Format | Ecart | structure | couleur |")
    out.append("|---|---|---|---|---|")
    for x in format_:
        e = "temoin" if x["ecart"] is None else f"{x['ecart']:.2f}"
        out.append(f"| {x['ref']} | {x['format']} | {e} | {x['structure']:.2f} | {x['couleur']:.2f} |")
    (RESULTATS / "rapport.md").write_text("\n".join(out), encoding="utf-8")
    print("\n".join(out))


if __name__ == "__main__":
    mode = sys.argv[1] if len(sys.argv) > 1 else ""
    if mode == "generer":
        generer()
    elif mode == "analyser":
        analyser()
    elif mode == "plan":
        c = plan()
        print(len(c), "generations")
    else:
        print("usage : python evaluer_reference.py generer|analyser|plan")
