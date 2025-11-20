use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResult {
    pub success: bool,
    pub data: Option<UploadData>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadData {
    #[serde(rename = "modifiedCount")]
    pub modified_count: i32,
    #[serde(rename = "upsertedCount")]
    pub upserted_count: i32,
    #[serde(rename = "excelCount")]
    pub excel_count: i32,
}

/// 上传文件到服务器
#[tauri::command]
pub async fn upload_file(
    file_path: String,
    server_url: String,
    token: String,
    only_valid_invention: bool,
) -> Result<UploadResult, String> {
    tracing::info!("开始上传文件: {}, 仅导入有效发明专利: {}", file_path, only_valid_invention);
    
    // 检查文件是否存在
    let path = Path::new(&file_path);
    if !path.exists() {
        let error_msg = format!("文件不存在: {}", file_path);
        tracing::error!("{}", error_msg);
        return Err(error_msg);
    }
    
    // 读取文件内容
    let file_content = tokio::fs::read(&file_path)
        .await
        .map_err(|e| {
            let error_msg = format!("读取文件失败: {}", e);
            tracing::error!("{}", error_msg);
            error_msg
        })?;
    
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file.xlsx")
        .to_string();
    
    tracing::info!("文件大小: {} bytes, 文件名: {}", file_content.len(), file_name);
    
    // 创建 HTTP 客户端
    let client = reqwest::Client::new();
    
    // 构建 multipart form
    let form = reqwest::multipart::Form::new()
        .part(
            "file",
            reqwest::multipart::Part::bytes(file_content)
                .file_name(file_name.clone())
                .mime_str("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
                .map_err(|e| format!("设置 MIME 类型失败: {}", e))?,
        );
    
    // 构建上传 URL
    let upload_url = format!(
        "{}/admin/patent/import?onlyValidInvention={}", 
        server_url.trim_end_matches('/'),
        only_valid_invention
    );
    tracing::info!("上传 URL: {}", upload_url);
    
    // 发送请求
    let response = client
        .post(&upload_url)
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await
        .map_err(|e| {
            let error_msg = format!("发送请求失败: {}", e);
            tracing::error!("{}", error_msg);
            error_msg
        })?;
    
    let status = response.status();
    tracing::info!("响应状态: {}", status);
    
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "无法获取错误信息".to_string());
        let error_msg = format!("上传失败 [{}]: {}", status, error_text);
        tracing::error!("{}", error_msg);
        return Err(error_msg);
    }
    
    // 解析响应
    let result: UploadResult = response.json().await.map_err(|e| {
        let error_msg = format!("解析响应失败: {}", e);
        tracing::error!("{}", error_msg);
        error_msg
    })?;
    
    if result.success {
        tracing::info!("文件上传成功: {}", file_name);
    } else {
        tracing::warn!("文件上传返回失败: {:?}", result.message);
    }
    
    Ok(result)
}
