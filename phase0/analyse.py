import json
import re
import subprocess
import sys
from pathlib import Path

CLE = str(Path.home() / ".ssh" / "id_ed25519_portable_test")
HOTE = "raph6@192.168.1.187"
LOCAL = Path(__file__).parent / "sortie" / "portable"
ANSI = re.compile(r"\x1b\[[0-9;?]*[A-Za-z]")


def scp(distant, local):
    subprocess.run(["scp", "-i", CLE, "-o", "BatchMode=yes", f"{HOTE}:C:/TestZImage/{distant}", Path(local).name],
                   capture_output=True, timeout=300, cwd=LOCAL)


def recuperer(lot, labels):
    LOCAL.mkdir(parents=True, exist_ok=True)
    scp(f"res_{lot}.jsonl", LOCAL / f"res_{lot}.jsonl")
    for label in labels:
        for ext in ("out", "err", "png"):
            scp(f"sortie/{label}.{ext}", LOCAL / f"{label}.{ext}")


def lire_jsonl(chemin):
    lignes = []
    texte = Path(chemin).read_bytes()
    for enc in ("utf-8-sig", "utf-16"):
        try:
            texte = texte.decode(enc)
            break
        except Exception:
            continue
    for ligne in texte.splitlines():
        ligne = ligne.strip().lstrip("\ufeff")
        if ligne.startswith("{"):
            lignes.append(json.loads(ligne))
    return lignes


def nettoyer(texte):
    return ANSI.sub("", texte).replace("\r", "\n")


def chronos(label):
    texte = ""
    for ext in ("out", "err"):
        f = LOCAL / f"{label}.{ext}"
        if f.exists():
            texte += nettoyer(f.read_text(encoding="utf-8", errors="replace")) + "\n"

    def trouve(motif):
        m = re.findall(motif, texte)
        return float(m[-1]) if m else None

    chargements = [float(x) for x in re.findall(r"loading tensors completed, taking ([0-9.]+)s", texte)]
    return {
        "encodage_s": trouve(r"get_learned_condition completed, taking ([0-9.]+)s"),
        "echantillonnage_s": trouve(r"sampling completed, taking ([0-9.]+)s"),
        "decodage_s": trouve(r"decode_first_stage completed, taking ([0-9.]+)s"),
        "generation_s": trouve(r"generate_image completed in ([0-9.]+)s"),
        "chargements_n": len(chargements),
        "chargement_max_s": max(chargements) if chargements else None,
        "segments": len(re.findall(r"segment", texte, flags=re.I)),
    }


def tableau(lignes):
    sortie = []
    for r in lignes:
        c = chronos(r["label"])
        sortie.append({**r, **c})
    return sortie


if __name__ == "__main__":
    lot = sys.argv[1]
    brut = lire_jsonl(LOCAL / f"res_{lot}.jsonl") if (LOCAL / f"res_{lot}.jsonl").exists() else []
    labels = [r["label"] for r in brut]
    recuperer(lot, labels)
    lignes = tableau(lire_jsonl(LOCAL / f"res_{lot}.jsonl"))
    print(f"{'label':16} {'total':>6} {'encod':>6} {'echant':>7} {'decod':>6} {'VRAMp':>6} {'RAM':>6} {'MHz':>5} {'T':>3} {'bridage'}")
    for r in lignes:
        f = lambda v: "n/a" if v is None else f"{v:.0f}"
        print(f"{r['label']:16} {f(r['duree_s']):>6} {f(r['encodage_s']):>6} {f(r['echantillonnage_s']):>7} {f(r['decodage_s']):>6} "
              f"{r['vram_dediee_pic_mo']:>6} {r['ram_pic_mo']:>6} {r['horloge_moy_mhz']:>5} {r['temp_max_c']:>3.0f} {r['raisons_bridage']}")
