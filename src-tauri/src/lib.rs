mod commands;
mod excel;
mod utils;

use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_env_filter("patentupload=debug,tauri=info")
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            scan_excel_files,
            convert_excel_files,
            get_converted_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
