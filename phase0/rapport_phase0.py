import html
import json
import math
import shutil
import sys
from pathlib import Path

from PIL import Image, ImageChops

import analyse

RACINE = Path(__file__).parent
PORTABLE = RACINE / "sortie" / "portable"
SORTIE = RACINE / "sortie" / "rapport"
PROMPTS = json.loads(Path("D:/ServOMorph/Comparatif-Image/prompts.json").read_text(encoding="utf-8"))["prompts"]
REF_4060 = Path("D:/ServOMorph/Comparatif-Image/resultats/2026-10-01/zimage")

DESCRIPTIONS = {
    "b0_base": "Départ : CUDA, Q3_K, déchargement RAM, sans mmap",
    "b1_maxvram0": "CUDA, budget VRAM libre (max-vram 0)",
    "b2_fa_off": "CUDA, flash attention désactivée",
    "b3_convdirect": "CUDA, convolution directe",
    "b4_eager": "CUDA, chargement anticipé",
    "b5_mmap": "CUDA, mmap",
    "b6_q4_0": "CUDA, modèle Q4_0, sans mmap",
    "b7_q2k": "CUDA, modèle Q2_K, sans mmap",
    "b8_te_q3km": "CUDA, text encoder Q3_K_M, sans mmap",
    "b9_te_q4_0": "CUDA, text encoder Q4_0, sans mmap",
    "c0_mmap": "CUDA, mmap (référence du lot 2)",
    "c1_mmap_nooff": "CUDA, mmap, sans déchargement RAM",
    "c2_mmap_notile": "CUDA, mmap, sans tuilage VAE",
    "c3_mmap_eager": "CUDA, mmap, chargement anticipé",
    "c4_mmap_vulkan": "Vulkan, mmap, Q3_K",
    "c5_mmap_q4k": "CUDA, mmap, Q4_K",
    "c6_mmap_q4_0": "CUDA, mmap, Q4_0",
    "c7_mmap_q2k": "CUDA, mmap, Q2_K",
    "c8_mmap_q5_0": "CUDA, mmap, Q5_0",
    "c9_mmap_q6k": "CUDA, mmap, Q6_K",
    "d0_mmap_te_q5km": "CUDA, mmap, text encoder Q5_K_M",
    "d1_mmap_te_q6k": "CUDA, mmap, text encoder Q6_K",
    "d2_mmap_768": "CUDA, mmap, 768x768",
    "d3_mmap_1024": "CUDA, mmap, 1024x1024",
    "e0_vk_q6k": "Vulkan, mmap, Q6_K",
    "e1_vk_q4k": "Vulkan, mmap, Q4_K",
    "e2_vk_q5_0": "Vulkan, mmap, Q5_0",
    "e3_vk_q6k_fa_off": "Vulkan, Q6_K, flash attention désactivée",
    "e4_vk_q6k_fa_on": "Vulkan, Q6_K, flash attention forcée",
    "e5_vk_q6k_conv_on": "Vulkan, Q6_K, convolution directe",
    "e6_vk_q6k_conv_off": "Vulkan, Q6_K, convolution classique",
    "e7_vk_q6k_notile": "Vulkan, Q6_K, sans tuilage VAE",
    "e8_vk_q6k_nooff": "Vulkan, Q6_K, sans déchargement RAM",
    "e9_vk_q6k_eager": "Vulkan, Q6_K, chargement anticipé",
    "f0_vk_q6k_768": "Vulkan, Q6_K, 768x768",
    "f1_vk_q6k_1024": "Vulkan, Q6_K, 1024x1024",
    "g0_vk_q5_0": "Vulkan, Q5_0",
    "g1_vk_q8_0": "Vulkan, Q8_0",
    "g2_vk_q4_0": "Vulkan, Q4_0",
    "g3_vk_q3k": "Vulkan, Q3_K",
    "g4_vk_q5_0_768": "Vulkan, Q5_0, 768x768",
    "g5_vk_q5_0_1024": "Vulkan, Q5_0, 1024x1024",
    "g6_vk_q5_0_hires4": "Vulkan, Q5_0, 512 puis affinage 1024 (4 étapes)",
    "g7_vk_q5_0_hires8": "Vulkan, Q5_0, 512 puis affinage 1024 (8 étapes)",
    "g8_vk_q5_0_easycache": "Vulkan, Q5_0, cache de calcul easycache (risque qualité)",
}

CONFIGS = [
    ("ref", "Référence RTX 4060 (Q4_K_M, 1024)", None),
    ("q8_0", "Q8_0", "z_image_turbo-Q8_0"),
    ("q5_0", "Q5_0", "z_image_turbo-Q5_0"),
    ("q4_0", "Q4_0", "z_image_turbo-Q4_0"),
    ("q3k", "Q3_K", "z_image_turbo-Q3_K"),
]


def miniature(source, cible, largeur=768):
    cible.parent.mkdir(parents=True, exist_ok=True)
    with Image.open(source) as im:
        im = im.convert("RGB")
        if im.width > largeur:
            im = im.resize((largeur, round(im.height * largeur / im.width)), Image.LANCZOS)
        im.save(cible, "JPEG", quality=84, optimize=True)


def psnr(a, b):
    if a.size != b.size:
        return None
    d = ImageChops.difference(a, b)
    if d.getbbox() is None:
        return 99.0
    px = list(d.getdata())
    mse = sum(c * c for p in px for c in p) / (len(px) * 3)
    return 10 * math.log10(255 * 255 / mse)


def collecter_mesures():
    lignes = []
    for lot in ("lot1", "lot2", "lot3", "lot4"):
        f = PORTABLE / f"res_{lot}.jsonl"
        if not f.exists():
            continue
        for r in analyse.tableau(analyse.lire_jsonl(f)):
            lignes.append(r)
    base = Image.open(PORTABLE / "b0_base.png").convert("RGB")
    for r in lignes:
        p = PORTABLE / f"{r['label']}.png"
        r["psnr"] = psnr(base, Image.open(p).convert("RGB")) if p.exists() else None
        r["description"] = DESCRIPTIONS.get(r["label"], r["label"])
    return lignes


def collecter_images():
    f = PORTABLE / "res_lot5.jsonl"
    mesures = {}
    if f.exists():
        for r in analyse.tableau(analyse.lire_jsonl(f)):
            mesures[r["label"]] = r
    cartes = {}
    for p in PROMPTS:
        entree = {}
        ref = REF_4060 / f"{p['id']}.png"
        if ref.exists():
            miniature(ref, SORTIE / "images" / "ref" / f"{p['id']}.jpg")
            entree["ref"] = {"image": f"images/ref/{p['id']}.jpg", "temps": None}
        for cle, nom, _ in CONFIGS[1:]:
            lab = f"h_{cle}_{p['id']}"
            png = PORTABLE / f"{lab}.png"
            if png.exists() and lab in mesures:
                miniature(png, SORTIE / "images" / cle / f"{p['id']}.jpg")
                entree[cle] = {"image": f"images/{cle}/{p['id']}.jpg", "temps": mesures[lab]["duree_s"]}
        lab = f"h_q5_0_1024_{p['id']}"
        png = PORTABLE / f"{lab}.png"
        if png.exists() and lab in mesures:
            miniature(png, SORTIE / "images" / "q5_0_1024" / f"{p['id']}.jpg", 1024)
            entree["q5_0_1024"] = {"image": f"images/q5_0_1024/{p['id']}.jpg", "temps": mesures[lab]["duree_s"]}
        cartes[p["id"]] = entree
    return cartes


def barres(lignes):
    maxi = max(r["duree_s"] for r in lignes)
    sortie = []
    for r in lignes:
        enc = r.get("encodage_s") or 0
        gen = r.get("echantillonnage_s") or 0
        dec = r.get("decodage_s") or 0
        autre = max(r["duree_s"] - enc - gen - dec, 0)
        seg = lambda v, c: f'<i class="{c}" style="width:{100 * v / maxi:.2f}%"></i>'
        sortie.append(
            f'<div class="ligne"><span class="nom">{html.escape(r["description"])}</span>'
            f'<span class="barre">{seg(enc, "enc")}{seg(gen, "gen")}{seg(dec, "dec")}{seg(autre, "aut")}</span>'
            f'<span class="val">{r["duree_s"]:.0f} s</span></div>'
        )
    return "".join(sortie)


def tableau(lignes):
    out = []
    for r in lignes:
        ps = r.get("psnr")
        ecart = "n/a" if ps is None else ("identique" if ps >= 90 else f"{ps:.0f} dB")
        out.append(
            f'<tr><td>{html.escape(r["description"])}</td><td class="n">{r["w"]}x{r["h"]}</td>'
            f'<td class="n">{r["duree_s"]:.0f}</td><td class="n">{(r.get("encodage_s") or 0):.0f}</td>'
            f'<td class="n">{(r.get("echantillonnage_s") or 0):.0f}</td><td class="n">{(r.get("decodage_s") or 0):.0f}</td>'
            f'<td class="n">{r["vram_dediee_pic_mo"]}</td><td class="n">{r["ram_pic_mo"]}</td><td class="n">{ecart}</td></tr>'
        )
    return "".join(out)


CSS = """
:root { --bg:#f3f4f7; --surface:#ffffff; --ink:#14171f; --muted:#5b6272; --line:#d9dce4; --accent:#1f4fd8; --ok:#1c7a4a; --mid:#a8690b; --ko:#b3261e;
  --c-ref:#6b7280; --c-q8_0:#7a3fb5; --c-q5_0:#1f4fd8; --c-q4_0:#0f8a8a; --c-q3k:#b5651d; --c-q5_0_1024:#1f4fd8;
  --b-enc:#8aa4e8; --b-gen:#1f4fd8; --b-dec:#0f8a8a; --b-aut:#b8bcc8;
  --font-body:"Schibsted Grotesk",system-ui,sans-serif; --font-data:"IBM Plex Mono",ui-monospace,monospace; }
@media (prefers-color-scheme: dark) { :root:not([data-theme="light"]) { --bg:#0f1218; --surface:#181c25; --ink:#e8eaf0; --muted:#98a0b2; --line:#2b3140; --accent:#7ea2ff; --ok:#5fcf90; --mid:#e6b35a; --ko:#ff8a80;
  --c-ref:#9aa3b2; --c-q8_0:#c39bf0; --c-q5_0:#7ea2ff; --c-q4_0:#4fd1c5; --c-q3k:#f0a860; --c-q5_0_1024:#7ea2ff; --b-enc:#5b79c8; --b-gen:#7ea2ff; --b-dec:#4fd1c5; --b-aut:#4a5163; color-scheme: dark; } }
:root[data-theme="dark"] { --bg:#0f1218; --surface:#181c25; --ink:#e8eaf0; --muted:#98a0b2; --line:#2b3140; --accent:#7ea2ff; --ok:#5fcf90; --mid:#e6b35a; --ko:#ff8a80;
  --c-ref:#9aa3b2; --c-q8_0:#c39bf0; --c-q5_0:#7ea2ff; --c-q4_0:#4fd1c5; --c-q3k:#f0a860; --c-q5_0_1024:#7ea2ff; --b-enc:#5b79c8; --b-gen:#7ea2ff; --b-dec:#4fd1c5; --b-aut:#4a5163; color-scheme: dark; }
body { background:var(--bg); color:var(--ink); font-family:var(--font-body); font-size:15px; line-height:1.5; padding-block:20px 56px;
  padding-left:max(16px,env(safe-area-inset-left,0px)); padding-right:max(16px,env(safe-area-inset-right,0px)); }
main { max-width:1280px; margin:0 auto; display:grid; gap:32px; }
h1 { font-size:clamp(1.5rem,4vw,2.1rem); line-height:1.15; margin:0; text-wrap:balance; }
h2 { font-size:1.2rem; margin:0 0 10px; } h3 { font-size:1rem; margin:0; } p { margin:0; }
.sous { color:var(--muted); max-width:75ch; margin-top:8px; }
.constat { display:grid; gap:12px; grid-template-columns:repeat(auto-fit,minmax(220px,1fr)); }
.constat div { background:var(--surface); border:1px solid var(--line); border-radius:8px; padding:12px 14px; }
.constat b { display:block; font-size:1.5rem; font-family:var(--font-data); font-variant-numeric:tabular-nums; }
.constat span { color:var(--muted); font-size:13px; }
.legende { display:flex; flex-wrap:wrap; gap:6px 16px; font-size:13px; color:var(--muted); margin-bottom:10px; }
.legende i { display:inline-block; width:12px; height:12px; border-radius:2px; margin-right:6px; vertical-align:-1px; }
.ligne { display:grid; grid-template-columns:minmax(150px,34%) 1fr 56px; gap:10px; align-items:center; padding:3px 0; font-size:13px; }
.ligne .nom { min-width:0; overflow-wrap:anywhere; } .ligne .val { font-family:var(--font-data); text-align:right; font-variant-numeric:tabular-nums; }
.barre { display:flex; height:14px; background:color-mix(in srgb,var(--line) 40%,transparent); border-radius:3px; overflow:hidden; }
.barre i { display:block; height:100%; } .enc{background:var(--b-enc)} .gen{background:var(--b-gen)} .dec{background:var(--b-dec)} .aut{background:var(--b-aut)}
.tab { overflow-x:auto; border:1px solid var(--line); border-radius:8px; background:var(--surface); }
table { border-collapse:collapse; width:100%; min-width:820px; font-size:13px; }
th,td { text-align:left; padding:7px 10px; border-bottom:1px solid var(--line); vertical-align:top; }
th { font-size:11px; letter-spacing:.06em; text-transform:uppercase; color:var(--muted); font-weight:600; } td.n { font-family:var(--font-data); white-space:nowrap; font-variant-numeric:tabular-nums; }
tr:last-child td { border-bottom:0; }
.barre-puces { position:sticky; top:env(safe-area-inset-top,0px); z-index:5; background:var(--bg); padding-block:10px; border-bottom:1px solid var(--line); display:grid; gap:8px; }
body::before { content:""; position:fixed; inset:0 0 auto; height:env(safe-area-inset-top,0px); z-index:10; background:var(--bg); }
.puces { display:flex; flex-wrap:wrap; gap:8px; }
.puce { appearance:none; border:1.5px solid var(--line); background:var(--surface); color:var(--ink); font:inherit; font-size:14px; padding:7px 12px; border-radius:999px; cursor:pointer; min-height:40px; }
.puce[aria-pressed="true"] { border-color:var(--c); background:color-mix(in srgb,var(--c) 14%,var(--surface)); font-weight:600; }
.puce:focus-visible,.vignette:focus-visible,.fermer:focus-visible,.vote button:focus-visible,.copier:focus-visible { outline:2px solid var(--accent); outline-offset:2px; }
.aide { font-size:13px; color:var(--muted); }
.prompt { border-top:1px solid var(--line); padding-top:16px; display:grid; gap:10px; }
.prompt h3 small { font-weight:400; color:var(--muted); } .texte { color:var(--muted); font-size:14px; max-width:110ch; }
.grille { display:grid; gap:12px; grid-template-columns:repeat(auto-fill,minmax(min(100%,230px),1fr)); }
figure { margin:0; min-width:0; } figcaption { font-size:13px; display:flex; flex-wrap:wrap; gap:2px 10px; margin-bottom:5px; } figcaption b { color:var(--c); } figcaption span { font-family:var(--font-data); color:var(--muted); }
.vignette { display:block; width:100%; padding:0; border:1px solid var(--line); border-radius:6px; overflow:hidden; background:var(--surface); cursor:zoom-in; } .vignette img { display:block; width:100%; height:auto; }
.vide { border:1px dashed var(--line); border-radius:6px; padding:36px 10px; text-align:center; color:var(--muted); font-size:13px; background:var(--surface); }
.vote { display:flex; gap:6px; margin-top:6px; } .vote button { flex:1; appearance:none; border:1px solid var(--line); background:var(--surface); color:var(--ink); font:inherit; font-size:13px; padding:6px 4px; border-radius:6px; cursor:pointer; min-height:36px; }
.vote button[aria-pressed="true"][data-v="bon"] { background:color-mix(in srgb,var(--ok) 20%,var(--surface)); border-color:var(--ok); }
.vote button[aria-pressed="true"][data-v="moyen"] { background:color-mix(in srgb,var(--mid) 20%,var(--surface)); border-color:var(--mid); }
.vote button[aria-pressed="true"][data-v="mauvais"] { background:color-mix(in srgb,var(--ko) 20%,var(--surface)); border-color:var(--ko); }
.synth { display:grid; gap:10px; }
.copier { appearance:none; border:1px solid var(--accent); background:transparent; color:var(--accent); font:inherit; padding:8px 14px; border-radius:8px; min-height:40px; cursor:pointer; justify-self:start; }
.lightbox { position:fixed; inset:0; z-index:20; background:rgba(8,10,14,.94); display:grid; grid-template-rows:auto 1fr; padding:max(12px,env(safe-area-inset-top,0px)) max(12px,env(safe-area-inset-right,0px)) max(12px,env(safe-area-inset-bottom,0px)) max(12px,env(safe-area-inset-left,0px)); }
.lightbox[hidden] { display:none; } .lightbox .entete { color:#e8eaf0; display:flex; justify-content:space-between; align-items:center; gap:12px; font-size:14px; }
.fermer { appearance:none; border:1px solid #5b6272; background:transparent; color:#e8eaf0; font:inherit; padding:8px 14px; border-radius:8px; min-height:40px; cursor:pointer; }
.lightbox img { max-width:100%; max-height:100%; margin:auto; object-fit:contain; min-height:0; }
"""

JS = """
(function () {
  var D = JSON.parse(document.getElementById('donnees').textContent);
  var choisis = D.defaut.slice();
  var votes = {};
  try { votes = JSON.parse(localStorage.getItem('zimg-votes') || '{}') || {}; } catch (e) { votes = {}; }
  var barre = document.getElementById('puces'), zone = document.getElementById('prompts'), synth = document.getElementById('synth');
  var lb = document.getElementById('lightbox'), lbImg = lb.querySelector('img'), lbTitre = lb.querySelector('.titre');
  function sauve() { try { localStorage.setItem('zimg-votes', JSON.stringify(votes)); } catch (e) {} }
  function puces() {
    barre.innerHTML = '';
    D.configs.forEach(function (c) {
      var b = document.createElement('button'); b.type = 'button'; b.className = 'puce';
      b.style.setProperty('--c', 'var(--c-' + c.id + ')');
      b.setAttribute('aria-pressed', choisis.indexOf(c.id) >= 0 ? 'true' : 'false'); b.textContent = c.nom;
      b.addEventListener('click', function () {
        var i = choisis.indexOf(c.id);
        if (i >= 0) { if (choisis.length > 1) choisis.splice(i, 1); } else { choisis.push(c.id); }
        choisis.sort(function (a, z) { return D.ordre.indexOf(a) - D.ordre.indexOf(z); });
        puces(); grille();
      });
      barre.appendChild(b);
    });
  }
  function resume() {
    var lignes = [];
    D.configs.forEach(function (c) {
      var n = { bon: 0, moyen: 0, mauvais: 0 };
      Object.keys(votes).forEach(function (k) { if (k.indexOf(c.id + '|') === 0 && n[votes[k]] !== undefined) n[votes[k]]++; });
      var det = [];
      Object.keys(votes).forEach(function (k) { if (k.indexOf(c.id + '|') === 0) det.push(k.split('|')[1] + '=' + votes[k]); });
      lignes.push(c.nom + ' : ' + n.bon + ' bon, ' + n.moyen + ' moyen, ' + n.mauvais + ' mauvais' + (det.length ? '\\n   ' + det.join(', ') : ''));
    });
    synth.querySelector('pre').textContent = lignes.join('\\n');
  }
  function grille() {
    zone.innerHTML = '';
    D.prompts.forEach(function (p) {
      var s = document.createElement('section'); s.className = 'prompt';
      var h = document.createElement('h3'); h.appendChild(document.createTextNode(p.id + ' '));
      var sm = document.createElement('small'); sm.textContent = '(' + p.categorie + ')'; h.appendChild(sm);
      var t = document.createElement('p'); t.className = 'texte'; t.textContent = p.texte;
      var g = document.createElement('div'); g.className = 'grille';
      choisis.forEach(function (cid) {
        var c = D.configs.filter(function (x) { return x.id === cid; })[0];
        var e = (D.images[p.id] || {})[cid];
        var f = document.createElement('figure'); f.style.setProperty('--c', 'var(--c-' + cid + ')');
        var cap = document.createElement('figcaption'); var nom = document.createElement('b'); nom.textContent = c.nom; cap.appendChild(nom);
        if (e && e.temps) { var sp = document.createElement('span'); sp.textContent = e.temps.toFixed(0) + ' s'; cap.appendChild(sp); }
        f.appendChild(cap);
        if (e && e.image) {
          var bt = document.createElement('button'); bt.type = 'button'; bt.className = 'vignette'; bt.setAttribute('aria-label', 'Agrandir ' + c.nom + ' - ' + p.id);
          var im = document.createElement('img'); im.loading = 'lazy'; im.src = e.image; im.alt = c.nom + ' - ' + p.id; bt.appendChild(im);
          bt.addEventListener('click', function () { lbImg.src = e.image; lbTitre.textContent = c.nom + ' - ' + p.id; lb.hidden = false; lb.querySelector('.fermer').focus(); });
          f.appendChild(bt);
          if (cid !== 'ref') {
            var cle = cid + '|' + p.id, v = document.createElement('div'); v.className = 'vote';
            ['bon', 'moyen', 'mauvais'].forEach(function (val) {
              var b = document.createElement('button'); b.type = 'button'; b.dataset.v = val; b.textContent = val.charAt(0).toUpperCase() + val.slice(1);
              b.setAttribute('aria-pressed', votes[cle] === val ? 'true' : 'false');
              b.addEventListener('click', function () {
                if (votes[cle] === val) { delete votes[cle]; } else { votes[cle] = val; }
                sauve(); Array.prototype.forEach.call(v.children, function (x) { x.setAttribute('aria-pressed', votes[cle] === x.dataset.v ? 'true' : 'false'); }); resume();
              });
              v.appendChild(b);
            });
            f.appendChild(v);
          }
        } else { var vi = document.createElement('div'); vi.className = 'vide'; vi.textContent = 'Non généré'; f.appendChild(vi); }
        g.appendChild(f);
      });
      s.appendChild(h); s.appendChild(t); s.appendChild(g); zone.appendChild(s);
    });
  }
  synth.querySelector('.copier').addEventListener('click', function () {
    var txt = synth.querySelector('pre').textContent, bt = this;
    function ok() { bt.textContent = 'Copié'; setTimeout(function () { bt.textContent = 'Copier mon verdict'; }, 1500); }
    try { navigator.clipboard.writeText(txt).then(ok, function () { sel(); }); } catch (e) { sel(); }
    function sel() { var r = document.createRange(); r.selectNodeContents(synth.querySelector('pre')); var s = window.getSelection(); s.removeAllRanges(); s.addRange(r); }
  });
  lb.querySelector('.fermer').addEventListener('click', function () { lb.hidden = true; });
  lb.addEventListener('click', function (e) { if (e.target === lb) lb.hidden = true; });
  document.addEventListener('keydown', function (e) { if (e.key === 'Escape') lb.hidden = true; });
  puces(); grille(); resume();
})();
"""


def construire(lignes, cartes):
    configs = [{"id": c, "nom": n} for c, n, _ in CONFIGS] + [{"id": "q5_0_1024", "nom": "Q5_0 en 1024"}]
    charge = json.dumps({
        "configs": configs, "ordre": [c["id"] for c in configs], "defaut": ["ref", "q5_0", "q4_0"],
        "prompts": PROMPTS, "images": cartes,
    }, ensure_ascii=False).replace("</", "<\\/")
    base = next((r for r in lignes if r["label"] == "b0_base"), None)
    meilleur = min((r for r in lignes if r["w"] == 512 and r["psnr"] is not None and r["label"].startswith(("g0", "g2"))), key=lambda r: r["duree_s"], default=None)
    constat = ""
    if base and meilleur:
        constat = (
            f'<div class="constat"><div><b>{base["duree_s"]:.0f} s</b><span>Départ : CUDA, Q3_K, 512x512</span></div>'
            f'<div><b>{meilleur["duree_s"]:.0f} s</b><span>Meilleur essai : {html.escape(meilleur["description"])}</span></div>'
            f'<div><b>x{base["duree_s"] / meilleur["duree_s"]:.1f}</b><span>Gain de vitesse</span></div>'
            f'<div><b>{max(r["vram_dediee_pic_mo"] for r in lignes) / 1024:.1f} Go</b><span>VRAM dédiée maximale mesurée (carte : 4 Go)</span></div></div>'
        )
    return f"""<main>
<header><h1>Optimisation de Z-Image sur GTX 1650 Ti (4 Go)</h1>
<p class="sous">Portable MSI GF63, 8 Go de RAM, Windows 11. Même graine (42) et même prompt de test pour tous les essais de mesure. Les images ci-dessous servent à valider la qualité avant de choisir la configuration du futur installeur.</p></header>
{constat}
<section><h2>Où le temps est gagné</h2>
<div class="legende"><span><i class="enc" style="background:var(--b-enc)"></i>Encodage du texte</span><span><i style="background:var(--b-gen)"></i>Génération (8 étapes)</span><span><i style="background:var(--b-dec)"></i>Décodage de l'image</span><span><i style="background:var(--b-aut)"></i>Chargement et autres</span></div>
{barres(lignes)}</section>
<section><h2>Toutes les mesures</h2><p class="aide" style="margin-bottom:8px">L'écart d'image est mesuré par rapport à l'essai de départ (portrait, 512x512). « identique » signifie aucune différence de pixel : l'option accélère sans toucher à la qualité.</p>
<div class="tab"><table><tr><th>Essai</th><th>Taille</th><th>Total (s)</th><th>Encodage</th><th>Génération</th><th>Décodage</th><th>VRAM (Mo)</th><th>RAM (Mo)</th><th>Écart image</th></tr>{tableau(lignes)}</table></div></section>
<section><h2>Valider la qualité</h2>
<div class="barre-puces"><div class="puces" id="puces" role="group" aria-label="Configurations affichées"></div>
<p class="aide">Touche une configuration pour l'afficher ou la masquer, touche une image pour l'agrandir, puis note chaque image : bon, moyen ou mauvais. Tes notes restent dans ce navigateur.</p></div>
<div id="prompts"></div></section>
<section class="synth" id="synth"><h2>Ton verdict</h2><pre style="margin:0;font-family:var(--font-data);font-size:13px;white-space:pre-wrap"></pre><button type="button" class="copier">Copier mon verdict</button></section>
</main>
<div class="lightbox" id="lightbox" hidden><div class="entete"><span class="titre"></span><button type="button" class="fermer">Fermer</button></div><img alt=""></div>
<script type="application/json" id="donnees">{charge}</script>
<script>{JS}</script>"""


def ecrire():
    SORTIE.mkdir(parents=True, exist_ok=True)
    lignes = collecter_mesures()
    cartes = collecter_images()
    corps = construire(lignes, cartes)
    police = '<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500&family=Schibsted+Grotesk:wght@400;600;700&display=swap">'
    titre = "Optimisation Z-Image 4 Go"
    meta = '<meta name="claude:viewport-fit-sides" content="cover">'
    (SORTIE / "artefact.html").write_text(f"<title>{titre}</title>\n{meta}\n{police}\n<style>{CSS}</style>\n{corps}\n", encoding="utf-8")
    (SORTIE / "rapport_optimisation.html").write_text(
        '<!doctype html><html lang="fr"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">'
        f"<title>{titre}</title>{police}<style>body{{margin:0}}{CSS}</style></head><body>{corps}</body></html>", encoding="utf-8")
    print(f"{len(lignes)} essais, {sum(len(v) for v in cartes.values())} images -> {SORTIE}")


if __name__ == "__main__":
    ecrire()
