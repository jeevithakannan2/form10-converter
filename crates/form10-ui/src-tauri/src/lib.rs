mod commands;
mod dto;
mod state;

use state::AppState;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::import_source,
            commands::remove_source,
            commands::summarize,
            commands::preview_export,
            commands::export_workbook,
            commands::open_output,
            commands::reveal_output,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Form 10 Converter");
}
