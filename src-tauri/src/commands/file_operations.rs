use tauri::command;
use crate::commands::types::{ColumnMapping, ScanResult};
use crate::excel;

/// 扫描 Excel 文件
#[command]
pub async fn scan_excel_files(source_path: String) -> Result<ScanResult, String> {
    tracing::info!("开始扫描目录: {}", source_path);
    
    excel::scan_directory(&source_path)
        .await
        .map_err(|e| {
            tracing::error!("扫描文件失败: {}", e);
            e.to_string()
        })
}

/// 转换 Excel 文件
#[command]
pub async fn convert_excel_files(
    source_path: String,
    target_path: String,
    mappings: Vec<ColumnMapping>,
) -> Result<Vec<String>, String> {
    tracing::info!("开始转换文件: {} -> {}, 映射数量: {}", source_path, target_path, mappings.len());
    
    excel::convert_files(&source_path, &target_path, &mappings)
        .await
        .map_err(|e| {
            tracing::error!("转换文件失败: {}", e);
            e.to_string()
        })
}

/// 获取转换后的文件列表
#[command]
pub async fn get_converted_files(target_path: String) -> Result<Vec<String>, String> {
    excel::get_excel_files(&target_path)
        .await
        .map_err(|e| e.to_string())
}

/// 保存日志文件
#[command]
pub async fn save_log_file(path: String, content: String) -> Result<(), String> {
    tracing::info!("保存日志文件到: {}", path);
    
    tokio::fs::write(&path, content)
        .await
        .map_err(|e| {
            tracing::error!("保存日志文件失败: {}", e);
            e.to_string()
        })?;
    
    tracing::info!("日志文件保存成功");
    Ok(())
}