const { invoke } = window.__TAURI__.core;

const el = (id) => document.getElementById(id);

let minuteur = null;

function afficherAlerte(message) {
  const alerte = el("alerte");
  alerte.textContent = message;
  alerte.hidden = !message;
}

function demarrerChrono() {
  const debut = Date.now();
  const maj = () => {
    const s = Math.floor((Date.now() - debut) / 1000);
    el("statut").textContent = `Génération en cours… ${s} s (comptez environ 1 à 2 minutes). Ne fermez pas la fenêtre.`;
  };
  maj();
  minuteur = setInterval(maj, 1000);
}

function arreterChrono() {
  clearInterval(minuteur);
  minuteur = null;
}

async function generer(evenement) {
  evenement.preventDefault();
  afficherAlerte("");
  const format = document.querySelector('input[name="format"]:checked').value;
  el("bouton").disabled = true;
  demarrerChrono();
  try {
    const image = await invoke("generer", { prompt: el("description").value, format });
    el("image").src = image.donnees;
    el("legende").textContent = `Générée en ${Math.round(image.duree_s)} s. Enregistrée : ${image.chemin}`;
    el("resultat").hidden = false;
    el("statut").textContent = "Terminé. Cliquez sur Générer pour une nouvelle variante.";
  } catch (erreur) {
    el("statut").textContent = "";
    afficherAlerte(String(erreur));
  } finally {
    arreterChrono();
    el("bouton").disabled = false;
  }
}

window.addEventListener("DOMContentLoaded", async () => {
  el("formulaire").addEventListener("submit", generer);
  el("dossier").addEventListener("click", () => {
    invoke("ouvrir_dossier_images").catch((e) => afficherAlerte(String(e)));
  });
  try {
    const etat = await invoke("etat");
    if (!etat.installe) {
      afficherAlerte(`L'installation n'est pas terminée (dossier : ${etat.racine}).`);
      el("bouton").disabled = true;
    }
  } catch (erreur) {
    afficherAlerte(String(erreur));
  }
});
