use std::sync::Mutex;
use tauri::Manager;

pub mod backup;
pub mod commands;
pub mod domain;
pub mod error;
pub mod integrations;
pub mod repositories;
pub mod services;
pub mod storage;

use storage::Database;

#[tauri::command]
fn app_status(app: tauri::AppHandle) -> Result<String, String> {
    let package = app.package_info();
    Ok(format!(
        "Local workspace initialized · v{}",
        package.version
    ))
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            app_status,
            commands::list_owned_cards,
            commands::search_owned_cards,
            commands::add_owned_card,
            commands::add_owned_catalog_card,
            commands::remove_owned_card,
            commands::update_owned_card,
            commands::catalog_import_mtgjson,
            commands::catalog_clear,
            commands::catalog_search
        ])
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            let database = Database::open(&app_data_dir.join("collection.sqlite"))?;
            app.manage(Mutex::new(database));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running MTG Local-First application");
}
