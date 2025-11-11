use calamine::{open_workbook, Data, Reader, Xlsx};
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;
use crate::commands::types::{ColumnMapping, ScanResult};

/// 扫描目录中的所有 Excel 文件
pub async fn scan_directory(source_path: &str) -> Result<ScanResult, Box<dyn std::error::Error + Send + Sync>> {
    let mut files = Vec::new();
    let mut total_size = 0u64;

    for entry in WalkDir::new(source_path) {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(extension) = path.extension() {
            if extension == "xlsx" || extension == "xls" {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }
                files.push(path.to_string_lossy().to_string());
            }
        }
    }

    Ok(ScanResult {
        file_count: files.len(),
        total_size,
        files,
    })
}

/// 转换 Excel 文件
pub async fn convert_files(
    source_path: &str,
    target_path: &str,
    mappings: &[ColumnMapping],
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    // 创建映射 HashMap
    let mapping_map: HashMap<String, String> = mappings
        .iter()
        .map(|m| (m.original.clone(), m.mapped.clone()))
        .collect();

    // 扫描文件
    let scan_result = scan_directory(source_path).await?;
    let mut converted_files = Vec::new();

    // 转换每个文件
    for file_path in scan_result.files {
        let converted_path = convert_single_file(&file_path, source_path, target_path, &mapping_map).await?;
        converted_files.push(converted_path);
    }

    Ok(converted_files)
}

/// 转换单个 Excel 文件
async fn convert_single_file(
    file_path: &str,
    source_root: &str,
    target_root: &str,
    mappings: &HashMap<String, String>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // 计算目标文件路径，保持目录结构
    let relative_path = Path::new(file_path)
        .strip_prefix(source_root)
        .map_err(|e| format!("Invalid path structure: {}", e))?;
    
    let target_path = Path::new(target_root).join(relative_path);
    
    // 确保目标目录存在
    if let Some(parent) = target_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // 克隆数据以便移动到 blocking task 中
    let file_path = file_path.to_string();
    let target_path_buf = target_path.to_path_buf();
    let target_path_buf_clone = target_path_buf.clone();
    let mappings = mappings.clone();

    // 处理 Excel 文件
    tokio::task::spawn_blocking(move || {
        process_excel_file(&file_path, &target_path_buf, &mappings)
    }).await??;
    
    Ok(target_path_buf_clone.to_string_lossy().to_string())
}

/// 处理 Excel 文件内容
fn process_excel_file(
    source_path: &str,
    target_path: &Path,
    mappings: &HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 打开源文件
    let mut workbook: Xlsx<_> = open_workbook(source_path)?;
    
    // 使用 xlsxwriter 创建新文件
    let new_workbook = xlsxwriter::Workbook::new(target_path.to_str().unwrap())?;
    
    // 处理每个工作表
    for sheet_name in workbook.sheet_names().to_owned() {
        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            let mut worksheet = new_workbook.add_worksheet(Some(&sheet_name))?;
            
            let mut row_index = 0u32;
            
            // 处理所有行
            for row in range.rows() {
                if row_index == 0 {
                    // 处理表头行
                    for (col_index, cell) in row.iter().enumerate() {
                        let header_text = cell_to_string(cell);
                        let mapped_header = mappings.get(&header_text).unwrap_or(&header_text);
                        worksheet.write_string(row_index, col_index as u16, mapped_header, None)?;
                    }
                } else {
                    // 处理数据行
                    for (col_index, cell) in row.iter().enumerate() {
                        let cell_text = cell_to_string(cell);
                        worksheet.write_string(row_index, col_index as u16, &cell_text, None)?;
                    }
                }
                
                row_index += 1;
            }
        }
    }
    
    new_workbook.close()?;
    Ok(())
}

/// 将 Data 转换为字符串
fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::String(s) => s.clone(),
        Data::Int(i) => i.to_string(),
        Data::Float(f) => f.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => dt.to_string(),
        Data::DateTimeIso(dt) => dt.to_string(),
        Data::DurationIso(d) => d.to_string(),
        Data::Error(e) => format!("ERROR: {:?}", e),
        Data::Empty => String::new(),
    }
}

/// 获取目录下的所有 Excel 文件路径
pub async fn get_excel_files(directory: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let scan_result = scan_directory(directory).await?;
    Ok(scan_result.files)
}