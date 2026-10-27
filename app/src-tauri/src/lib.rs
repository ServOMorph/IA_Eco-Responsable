pub mod appli;
pub mod backend;
pub mod controle;
pub mod machine;
pub mod modeles;
pub mod moteur;

#[tauri::command]
fn etat() -> appli::Etat {
    appli::etat()
}

#[tauri::command]
async fn generer(
    prompt: String,
    format: String,
    nombre: u32,
    reference: Option<String>,
) -> Result<appli::Lot, String> {
    tauri::async_runtime::spawn_blocking(move || {
        appli::generer(&prompt, &format, nombre, reference.as_deref())
    })
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn importer_reference(donnees: String) -> Result<String, String> {
    appli::importer_reference(&donnees)
}

#[tauri::command]
fn annuler() {
    appli::annuler();
}

#[tauri::command]
fn progression() -> appli::Progression {
    appli::progression()
}

#[tauri::command]
async fn memoire() -> Result<appli::Memoire, String> {
    tauri::async_runtime::spawn_blocking(appli::memoire).await.map_err(|e| e.to_string())
}

#[tauri::command]
fn historique(limite: usize) -> Vec<appli::Fiche> {
    appli::historique(limite)
}

#[tauri::command]
fn charger_image(chemin: String) -> Result<String, String> {
    appli::charger_image(&chemin)
}

#[tauri::command]
fn supprimer_image(chemin: String) -> Result<(), String> {
    appli::supprimer_image(&chemin)
}

#[tauri::command]
fn ouvrir_dossier_images() -> Result<(), String> {
    appli::ouvrir_dossier_images()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            etat,
            generer,
            importer_reference,
            annuler,
            progression,
            memoire,
            historique,
            charger_image,
            supprimer_image,
            ouvrir_dossier_images
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
