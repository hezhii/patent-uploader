mod commands;
mod excel;
mod utils;

use commands::*;
use utils::TauriLayer;
use tracing_subscriber::prelude::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 初始化日志系统，包含自定义的 Tauri layer
            let app_handle = app.handle().clone();
            
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer())
                .with(tracing_subscriber::EnvFilter::new("patentupload=debug,tauri=info"))
                .with(TauriLayer::new(app_handle))
                .init();
            
            tracing::info!("应用启动");
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_excel_files,
            convert_excel_files,
            get_converted_files,
            save_log_file,
            upload_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
