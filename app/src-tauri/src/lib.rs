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
async fn generer(prompt: String, format: String) -> Result<appli::Image, String> {
    tauri::async_runtime::spawn_blocking(move || appli::generer(&prompt, &format))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn ouvrir_dossier_images() -> Result<(), String> {
    appli::ouvrir_dossier_images()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![etat, generer, ouvrir_dossier_images])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
