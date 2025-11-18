use calamine::{open_workbook, Data, Reader, Xlsx};
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;
use crate::commands::types::{ColumnMapping, ScanResult};

/// 扫描目录中的所有 Excel 文件
pub async fn scan_directory(source_path: &str) -> Result<ScanResult, Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("开始扫描目录: {}", source_path);
    
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
                let file_path = path.to_string_lossy().to_string();
                tracing::debug!("发现 Excel 文件: {}", file_path);
                files.push(file_path);
            }
        }
    }

    tracing::info!("扫描完成，共发现 {} 个文件，总大小: {} bytes", files.len(), total_size);

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
    tracing::info!("开始转换文件，源路径: {}, 目标路径: {}, 映射规则数: {}", source_path, target_path, mappings.len());
    
    // 创建映射 HashMap
    let mapping_map: HashMap<String, String> = mappings
        .iter()
        .map(|m| {
            tracing::debug!("列映射: {} -> {}", m.original, m.mapped);
            (m.original.clone(), m.mapped.clone())
        })
        .collect();

    // 扫描文件
    let scan_result = scan_directory(source_path).await?;
    let mut converted_files = Vec::new();

    tracing::info!("准备转换 {} 个文件", scan_result.files.len());

    // 转换每个文件
    for (index, file_path) in scan_result.files.iter().enumerate() {
        tracing::info!("正在转换文件 {}/{}: {}", index + 1, scan_result.files.len(), file_path);
        match convert_single_file(file_path, source_path, target_path, &mapping_map).await {
            Ok(converted_path) => {
                tracing::info!("文件转换成功: {}", converted_path);
                converted_files.push(converted_path);
            }
            Err(e) => {
                tracing::error!("文件转换失败 {}: {}", file_path, e);
                return Err(e);
            }
        }
    }

    tracing::info!("所有文件转换完成，共 {} 个文件", converted_files.len());

    Ok(converted_files)
}

/// 转换单个 Excel 文件
async fn convert_single_file(
    file_path: &str,
    source_root: &str,
    target_root: &str,
    mappings: &HashMap<String, String>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    tracing::debug!("转换单个文件: {}", file_path);
    
    // 计算目标文件路径，保持目录结构
    let relative_path = Path::new(file_path)
        .strip_prefix(source_root)
        .map_err(|e| {
            tracing::error!("路径结构错误: {}", e);
            format!("Invalid path structure: {}", e)
        })?;
    
    let target_path = Path::new(target_root).join(relative_path);
    
    tracing::debug!("目标文件路径: {}", target_path.display());
    
    // 确保目标目录存在
    if let Some(parent) = target_path.parent() {
        tracing::debug!("创建目标目录: {}", parent.display());
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
    
    tracing::debug!("文件处理完成: {}", target_path_buf_clone.display());
    
    Ok(target_path_buf_clone.to_string_lossy().to_string())
}

/// 处理 Excel 文件内容
fn process_excel_file(
    source_path: &str,
    target_path: &Path,
    mappings: &HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::debug!("处理 Excel 文件: {} -> {}", source_path, target_path.display());
    
    // 打开源文件
    let mut workbook: Xlsx<_> = open_workbook(source_path)
        .map_err(|e| {
            tracing::error!("打开 Excel 文件失败: {}", e);
            e
        })?;
    
    // 使用 xlsxwriter 创建新文件
    let new_workbook = xlsxwriter::Workbook::new(target_path.to_str().unwrap())
        .map_err(|e| {
            tracing::error!("创建新 Excel 文件失败: {}", e);
            e
        })?;
    
    let sheet_names = workbook.sheet_names().to_owned();
    tracing::debug!("工作表数量: {}", sheet_names.len());
    
    // 处理每个工作表
    for sheet_name in sheet_names {
        tracing::debug!("处理工作表: {}", sheet_name);
        
        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            let mut worksheet = new_workbook.add_worksheet(Some(&sheet_name))?;
            
            let mut row_index = 0u32;
            let row_count = range.rows().count();
            tracing::debug!("工作表 {} 共 {} 行", sheet_name, row_count);
            
            // 处理所有行
            for row in range.rows() {
                if row_index == 0 {
                    // 处理表头行
                    let mut mapped_count = 0;
                    for (col_index, cell) in row.iter().enumerate() {
                        let header_text = cell_to_string(cell);
                        let mapped_header = mappings.get(&header_text).unwrap_or(&header_text);
                        if mapped_header != &header_text {
                            mapped_count += 1;
                            tracing::debug!("映射列名: {} -> {}", header_text, mapped_header);
                        }
                        worksheet.write_string(row_index, col_index as u16, mapped_header, None)?;
                    }
                    if mapped_count > 0 {
                        tracing::info!("工作表 {} 应用了 {} 个列映射", sheet_name, mapped_count);
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
    
    new_workbook.close()
        .map_err(|e| {
            tracing::error!("关闭 Excel 文件失败: {}", e);
            e
        })?;
    
    tracing::debug!("Excel 文件处理完成");
    
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