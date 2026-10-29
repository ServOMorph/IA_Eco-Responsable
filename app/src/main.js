const { invoke } = window.__TAURI__.core;

const el = (id) => document.getElementById(id);

const MESSAGE_ANNULE = "Génération annulée.";
const LIMITE_HISTORIQUE = 12;
const FORMATS = { carre: "Carré", portrait: "Portrait", paysage: "Paysage" };

const TAILLE_REFERENCE_MAX = 10 * 1024 * 1024;

let minuteur = null;
let enCours = false;
let reference = null;

function afficherAlerte(message) {
  const alerte = el("alerte");
  alerte.textContent = message;
  alerte.hidden = !message;
}

function enMo(valeur) {
  return `${(valeur / 1024).toFixed(1).replace(".", ",")} Go`;
}

async function actualiserMemoire() {
  if (enCours) return;
  try {
    const m = await invoke("memoire");
    el("memoire-ram").textContent =
      `Mémoire vive : ${enMo(m.ram_dispo_mo)} libres sur ${enMo(m.ram_totale_mo)} (minimum ${enMo(m.seuil_ram_mo)})`;
    el("memoire-vram").textContent =
      m.vram_libre_mo === null
        ? `Mémoire vidéo : ${m.gpu || "carte non détectée"}, valeur libre indisponible`
        : `Mémoire vidéo : ${enMo(m.vram_libre_mo)} libres sur ${enMo(m.vram_mo)} (minimum ${enMo(m.seuil_vram_mo)}) — ${m.gpu}`;
    const avis = el("memoire-avis");
    avis.textContent = m.message || "";
    avis.hidden = m.suffisante;
  } catch (erreur) {
    el("memoire-ram").textContent = `Mesure de la mémoire impossible : ${erreur}`;
  }
}

function texteProgression(p, secondes) {
  const prefixe = p.images_total > 1 ? `Image ${p.image}/${p.images_total} — ` : "";
  const phases = {
    demarrage: "Démarrage",
    chargement: "Chargement du modèle",
    calcul: `Calcul de l'image (étape ${p.etape}/${p.total})`,
    finalisation: "Finalisation",
  };
  return `${prefixe}${phases[p.phase] || "En cours"} — ${secondes} s. Ne fermez pas la fenêtre.`;
}

function demarrerSuivi() {
  const debut = Date.now();
  el("barre").hidden = false;
  el("barre").value = 0;
  const maj = async () => {
    const secondes = Math.floor((Date.now() - debut) / 1000);
    try {
      const p = await invoke("progression");
      if (!enCours) return;
      el("barre").value = p.pourcentage;
      el("statut").textContent = texteProgression(p, secondes);
    } catch {
      el("statut").textContent = `En cours — ${secondes} s.`;
    }
  };
  maj();
  minuteur = setInterval(maj, 800);
}

function arreterSuivi() {
  clearInterval(minuteur);
  minuteur = null;
  el("barre").hidden = true;
}

function definirEnCours(valeur) {
  enCours = valeur;
  el("bouton").disabled = valeur;
  el("annuler").hidden = !valeur;
  el("annuler").disabled = false;
  el("etat-modele").textContent = valeur
    ? "Modèle : chargé en mémoire (génération en cours)"
    : "Modèle : libéré de la mémoire";
}

function legendeFiche(fiche) {
  const base = `${FORMATS[fiche.format] || fiche.format}, graine ${fiche.graine}, ${Math.round(fiche.duree_s)} s`;
  if (!fiche.reference) return base;
  return `${base}, avec référence`;
}

function formatProche(largeur, hauteur) {
  const rapport = largeur / hauteur;
  if (rapport > 1.2) return "paysage";
  if (rapport < 0.83) return "portrait";
  return "carre";
}

function definirReference(chemin, apercu) {
  reference = chemin ? { chemin, apercu } : null;
  el("reference-choisie").hidden = !reference;
  el("reference-aide").hidden = !reference;
  if (reference) {
    el("reference-apercu").src = apercu;
    const sonde = new Image();
    sonde.onload = () => {
      const radio = document.querySelector(
        `input[name="format"][value="${formatProche(sonde.naturalWidth, sonde.naturalHeight)}"]`,
      );
      if (radio) radio.checked = true;
    };
    sonde.src = apercu;
  } else {
    el("fichier-reference").value = "";
  }
}

function lireFichier(fichier) {
  return new Promise((resolve, reject) => {
    const lecteur = new FileReader();
    lecteur.onload = () => resolve(lecteur.result);
    lecteur.onerror = () => reject(new Error("Lecture du fichier impossible."));
    lecteur.readAsDataURL(fichier);
  });
}

async function choisirReference(evenement) {
  const fichier = evenement.target.files[0];
  if (!fichier) return;
  afficherAlerte("");
  if (fichier.size > TAILLE_REFERENCE_MAX) {
    afficherAlerte("L'image de référence est trop lourde (10 Mo maximum).");
    definirReference(null);
    return;
  }
  try {
    const donnees = await lireFichier(fichier);
    const chemin = await invoke("importer_reference", { donnees });
    definirReference(chemin, donnees);
  } catch (erreur) {
    definirReference(null);
    afficherAlerte(String(erreur));
  }
}

function utiliserCommeReference(fiche, apercu) {
  definirReference(fiche.chemin, apercu);
  window.scrollTo({ top: 0, behavior: "smooth" });
}

function creerBoutonSuppression(fiche, apres) {
  const bouton = document.createElement("button");
  bouton.type = "button";
  bouton.className = "secondaire petit danger";
  bouton.textContent = "Supprimer";
  let armee = null;
  const desarmer = () => {
    clearTimeout(armee);
    armee = null;
    bouton.textContent = "Supprimer";
  };
  bouton.addEventListener("click", async () => {
    if (armee === null) {
      bouton.textContent = "Confirmer la suppression";
      armee = setTimeout(desarmer, 4000);
      return;
    }
    desarmer();
    bouton.disabled = true;
    try {
      await invoke("supprimer_image", { chemin: fiche.chemin });
      apres();
    } catch (erreur) {
      bouton.disabled = false;
      afficherAlerte(String(erreur));
    }
  });
  return bouton;
}

function retirerFigure(figure, fiche) {
  figure.remove();
  document.querySelectorAll(".vignette").forEach((v) => {
    if (v.dataset.chemin === fiche.chemin) v.remove();
  });
  el("historique-vide").hidden = document.querySelectorAll(".vignette").length > 0;
}

function creerFigure(donnees, fiche) {
  const figure = document.createElement("figure");
  const image = document.createElement("img");
  image.src = donnees;
  image.alt = fiche.prompt;
  const legende = document.createElement("figcaption");
  legende.textContent = `${legendeFiche(fiche)}. Enregistrée : ${fiche.chemin}`;
  const supprimer = creerBoutonSuppression(fiche, () => retirerFigure(figure, fiche));
  const comme = document.createElement("button");
  comme.type = "button";
  comme.className = "secondaire petit";
  comme.textContent = "Utiliser comme référence";
  comme.addEventListener("click", () => utiliserCommeReference(fiche, donnees));
  const actions = document.createElement("div");
  actions.className = "ligne actions";
  actions.append(comme, supprimer);
  figure.append(image, legende, actions);
  return figure;
}

function afficherImages(liste) {
  const zone = el("resultats");
  zone.replaceChildren(...liste.map((i) => creerFigure(i.donnees, i.fiche)));
}

function reutiliser(fiche) {
  el("description").value = fiche.prompt;
  const radio = document.querySelector(`input[name="format"][value="${fiche.format}"]`);
  if (radio) radio.checked = true;
  window.scrollTo({ top: 0, behavior: "smooth" });
}

function creerVignette(fiche) {
  const bloc = document.createElement("div");
  bloc.className = "vignette";
  bloc.dataset.chemin = fiche.chemin;
  const image = document.createElement("img");
  image.alt = fiche.prompt;
  image.title = "Afficher cette image";
  image.addEventListener("click", () => {
    if (image.src) el("resultats").replaceChildren(creerFigure(image.src, fiche));
  });
  const texte = document.createElement("p");
  texte.textContent = fiche.prompt;
  texte.title = `${fiche.prompt}\n${legendeFiche(fiche)}\n${new Date(fiche.date * 1000).toLocaleString("fr-FR")}`;
  const bouton = document.createElement("button");
  bouton.type = "button";
  bouton.className = "secondaire petit";
  bouton.textContent = "Réutiliser";
  bouton.addEventListener("click", () => reutiliser(fiche));
  const supprimer = creerBoutonSuppression(fiche, () => {
    document.querySelectorAll("#resultats figure").forEach((f) => {
      if (f.querySelector("figcaption")?.textContent.includes(fiche.chemin)) f.remove();
    });
    bloc.remove();
    el("historique-vide").hidden = document.querySelectorAll(".vignette").length > 0;
  });
  const comme = document.createElement("button");
  comme.type = "button";
  comme.className = "secondaire petit";
  comme.textContent = "Comme référence";
  comme.addEventListener("click", () => {
    if (image.src) utiliserCommeReference(fiche, image.src);
  });
  bloc.append(image, texte, bouton, comme, supprimer);
  return { bloc, image };
}

async function actualiserHistorique() {
  try {
    const fiches = await invoke("historique", { limite: LIMITE_HISTORIQUE });
    el("historique-vide").hidden = fiches.length > 0;
    const vignettes = fiches.map(creerVignette);
    el("historique").replaceChildren(...vignettes.map((v) => v.bloc));
    for (let i = 0; i < fiches.length; i++) {
      try {
        vignettes[i].image.src = await invoke("charger_image", { chemin: fiches[i].chemin });
      } catch {
        vignettes[i].bloc.classList.add("indisponible");
      }
    }
  } catch (erreur) {
    afficherAlerte(String(erreur));
  }
}

async function generer(evenement) {
  evenement.preventDefault();
  afficherAlerte("");
  const format = document.querySelector('input[name="format"]:checked').value;
  const nombre = Number(el("nombre").value);
  definirEnCours(true);
  demarrerSuivi();
  try {
    const lot = await invoke("generer", {
      prompt: el("description").value,
      format,
      nombre,
      reference: reference ? reference.chemin : null,
    });
    afficherImages(lot.images);
    if (lot.interruption === MESSAGE_ANNULE) {
      el("statut").textContent = `Annulé après ${lot.images.length} image(s) sur ${nombre}.`;
    } else if (lot.interruption) {
      el("statut").textContent = "";
      afficherAlerte(`${lot.images.length} image(s) sur ${nombre} produite(s). ${lot.interruption}`);
    } else {
      el("statut").textContent = "Terminé. Cliquez sur Générer pour une nouvelle variante.";
    }
  } catch (erreur) {
    const message = String(erreur);
    if (message === MESSAGE_ANNULE) {
      el("statut").textContent = MESSAGE_ANNULE;
    } else {
      el("statut").textContent = "";
      afficherAlerte(message);
    }
  } finally {
    arreterSuivi();
    definirEnCours(false);
    actualiserHistorique();
    actualiserMemoire();
  }
}

window.addEventListener("DOMContentLoaded", async () => {
  el("formulaire").addEventListener("submit", generer);
  el("annuler").addEventListener("click", () => {
    el("annuler").disabled = true;
    el("statut").textContent = "Annulation en cours…";
    invoke("annuler").catch((e) => afficherAlerte(String(e)));
  });
  el("actualiser").addEventListener("click", actualiserMemoire);
  document.querySelectorAll("a[data-externe]").forEach((lien) => {
    lien.addEventListener("click", (evenement) => {
      evenement.preventDefault();
      invoke("plugin:opener|open_url", { url: lien.href }).catch((e) => afficherAlerte(String(e)));
    });
  });
  el("fichier-reference").addEventListener("change", choisirReference);
  el("reference-retirer").addEventListener("click", () => definirReference(null));
  el("dossier").addEventListener("click", () => {
    invoke("ouvrir_dossier_images").catch((e) => afficherAlerte(String(e)));
  });
  try {
    const etat = await invoke("etat");
    el("nom-modele").textContent = etat.modele;
    if (!etat.installe) {
      afficherAlerte(`L'installation n'est pas terminée (dossier : ${etat.racine}).`);
      el("bouton").disabled = true;
    }
  } catch (erreur) {
    afficherAlerte(String(erreur));
  }
  actualiserMemoire();
  actualiserHistorique();
});
