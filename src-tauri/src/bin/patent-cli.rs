use anyhow::{Context, Result};
use patentupload_lib::cli::CliArgs;
use patentupload_lib::excel;
use patentupload_lib::commands::types::ColumnMapping;
use reqwest;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing_subscriber::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    success: bool,
    data: Option<LoginData>,
    message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginData {
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UploadResult {
    success: bool,
    data: Option<UploadData>,
    message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UploadData {
    #[serde(rename = "modifiedCount")]
    modified_count: i32,
    #[serde(rename = "upsertedCount")]
    upserted_count: i32,
    #[serde(rename = "excelCount")]
    excel_count: i32,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::new("patentupload=info"))
        .init();

    // 解析命令行参数
    let args = CliArgs::parse_args();

    println!("=== 专利文件上传工具 ===");
    println!("服务器地址: {}", args.server);
    println!("用户名: {}", args.username);
    println!("输入目录: {}", args.input);
    println!("输出目录: {}", args.output);
    println!("仅上传有效发明专利: {}", args.only_valid_invention);
    println!();

    // 步骤 1: 登录获取 token
    println!("[1/4] 正在登录...");
    let token = login(&args.server, &args.username, &args.password).await?;
    println!("✓ 登录成功");
    println!();

    // 步骤 2: 扫描输入目录
    println!("[2/4] 正在扫描输入目录...");
    let scan_result = excel::scan_directory(&args.input)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    println!("✓ 发现 {} 个 Excel 文件", scan_result.file_count);
    println!();

    // 步骤 3: 转换文件
    println!("[3/4] 正在转换文件...");
    
    // 使用默认的列映射（根据你的业务逻辑设置）
    let mappings = get_default_mappings();
    
    let converted_files = excel::convert_files(&args.input, &args.output, &mappings)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    
    println!("✓ 成功转换 {} 个文件", converted_files.len());
    println!();

    // 步骤 4: 上传文件
    println!("[4/4] 正在上传文件...");
    let mut success_count = 0;
    let mut fail_count = 0;
    
    for (index, file_path) in converted_files.iter().enumerate() {
        print!("  [{}/{}] 正在上传 {} ... ", 
            index + 1, 
            converted_files.len(), 
            Path::new(file_path).file_name().unwrap().to_string_lossy()
        );
        
        match upload_file(
            file_path,
            &args.server,
            &token,
            args.only_valid_invention
        ).await {
            Ok(result) => {
                if result.success {
                    success_count += 1;
                    if let Some(data) = result.data {
                        println!("✓ (修改: {}, 新增: {})", 
                            data.modified_count, 
                            data.upserted_count
                        );
                    } else {
                        println!("✓");
                    }
                } else {
                    fail_count += 1;
                    println!("✗ {}", result.message.unwrap_or_else(|| "未知错误".to_string()));
                }
            }
            Err(e) => {
                fail_count += 1;
                println!("✗ {}", e);
            }
        }
    }
    
    println!();
    println!("=== 上传完成 ===");
    println!("成功: {} 个文件", success_count);
    println!("失败: {} 个文件", fail_count);

    Ok(())
}

/// 登录并获取 token
async fn login(server_url: &str, username: &str, password: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let login_url = format!("{}/auth/admin/login", server_url.trim_end_matches('/'));
    
    let request = LoginRequest {
        username: username.to_string(),
        password: password.to_string(),
    };
    
    let response = client
        .post(&login_url)
        .json(&request)
        .send()
        .await
        .context("发送登录请求失败")?;
    
    if !response.status().is_success() {
        anyhow::bail!("登录失败: HTTP {}", response.status());
    }
    
    let result: LoginResponse = response.json().await.context("解析登录响应失败")?;
    
    if result.success && result.data.is_some() {
        Ok(result.data.unwrap().token)
    } else {
        anyhow::bail!("登录失败: {}", result.message.unwrap_or_else(|| "未知错误".to_string()));
    }
}

/// 上传单个文件
async fn upload_file(
    file_path: &str,
    server_url: &str,
    token: &str,
    only_valid_invention: bool,
) -> Result<UploadResult> {
    let path = Path::new(file_path);
    if !path.exists() {
        anyhow::bail!("文件不存在: {}", file_path);
    }
    
    // 读取文件内容
    let file_content = tokio::fs::read(&path)
        .await
        .context("读取文件失败")?;
    
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file.xlsx")
        .to_string();
    
    // 创建 HTTP 客户端
    let client = reqwest::Client::new();
    
    // 构建 multipart form
    let form = reqwest::multipart::Form::new()
        .part(
            "file",
            reqwest::multipart::Part::bytes(file_content)
                .file_name(file_name.clone())
                .mime_str("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
                .context("设置 MIME 类型失败")?,
        );
    
    // 构建上传 URL
    let upload_url = format!(
        "{}/admin/patent/import?onlyValidInvention={}", 
        server_url.trim_end_matches('/'),
        only_valid_invention
    );
    
    // 发送请求
    let response = client
        .post(&upload_url)
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await
        .context("发送上传请求失败")?;
    
    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "无法获取错误信息".to_string());
        anyhow::bail!("上传失败 [{}]: {}", status, error_text);
    }
    
    // 解析响应
    let result: UploadResult = response.json().await.context("解析响应失败")?;
    
    Ok(result)
}

/// 获取默认的列映射
/// 根据你的业务需求调整这些映射
fn get_default_mappings() -> Vec<ColumnMapping> {
    vec![
        ColumnMapping {
            original: "申请号".to_string(),
            mapped: "申请号".to_string(),
        },
        ColumnMapping {
            original: "申请日".to_string(),
            mapped: "申请日".to_string(),
        },
        ColumnMapping {
            original: "名称".to_string(),
            mapped: "名称".to_string(),
        },
        ColumnMapping {
            original: "类型".to_string(),
            mapped: "类型".to_string(),
        },
        ColumnMapping {
            original: "法律状态".to_string(),
            mapped: "法律状态".to_string(),
        },
        ColumnMapping {
            original: "申请人".to_string(),
            mapped: "申请人".to_string(),
        },
    ]
}
