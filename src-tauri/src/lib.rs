use tauri::Manager;

#[tauri::command]
fn app_status(app: tauri::AppHandle) -> Result<String, String> {
    let package = app.package_info();
    Ok(format!("Local workspace initialized · v{}", package.version))
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![app_status])
        .setup(|app| {
            let _app_data_dir = app.path().app_data_dir()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running MTG Local-First application");
}
