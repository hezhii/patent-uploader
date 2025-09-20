use std::path::Path;

/// 格式化文件大小
pub fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// 检查文件是否为 Excel 文件
pub fn is_excel_file<P: AsRef<Path>>(path: P) -> bool {
    if let Some(extension) = path.as_ref().extension() {
        matches!(extension.to_string_lossy().to_lowercase().as_str(), "xlsx" | "xls")
    } else {
        false
    }
}

/// 生成日志条目
pub fn create_log_entry(level: &str, message: &str) -> String {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
    format!("[{}] [{}] {}", timestamp, level.to_uppercase(), message)
}