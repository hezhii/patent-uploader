use anyhow::{Context, Result};
use patentupload_lib::cli::CliArgs;
use patentupload_lib::excel;
use patentupload_lib::commands::types::ColumnMapping;
use reqwest;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing_subscriber::prelude::*;
use tracing_appender;
use chrono::Local;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use tokio::time::timeout;

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

#[derive(Debug)]
struct FailedFile {
    path: String,
    reason: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 生成日志文件名
    let log_filename = Local::now().format("patent_upload_%Y%m%d%H%M.log").to_string();
    
    // 创建文件日志 appender
    let file_appender = tracing_appender::rolling::never(".", &log_filename);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    // 初始化日志系统 - 同时输出到终端和文件
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
        )
        .with(tracing_subscriber::EnvFilter::new("patentupload=info"))
        .init();

    tracing::info!("日志文件: {}", log_filename);

    // 解析命令行参数
    let args = CliArgs::parse_args();

    println!("=== 专利文件上传工具 ===");
    println!("服务器地址: {}", args.server);
    println!("用户名: {}", args.username);
    println!("输入目录: {}", args.input);
    println!("输出目录: {}", args.output);
    println!("仅上传有效发明专利: {}", args.only_valid_invention);
    if !args.column_mappings.is_empty() {
        println!("列名映射: {} 组", args.column_mappings.len());
    }
    println!();
    
    tracing::info!("========== 开始执行专利上传任务 ==========");
    tracing::info!("服务器地址: {}", args.server);
    tracing::info!("用户名: {}", args.username);
    tracing::info!("输入目录: {}", args.input);
    tracing::info!("输出目录: {}", args.output);
    tracing::info!("仅上传有效发明专利: {}", args.only_valid_invention);
    if !args.column_mappings.is_empty() {
        tracing::info!("列名映射: {} 组", args.column_mappings.len());
    }

    // 创建共享的 HTTP 客户端（配置连接池和超时）
    let http_client = reqwest::Client::builder()
        .pool_max_idle_per_host(2)  // 限制每个主机的空闲连接数
        .pool_idle_timeout(Duration::from_secs(30))  // 空闲连接超时
        .timeout(Duration::from_secs(300))  // 请求超时 5 分钟
        .connect_timeout(Duration::from_secs(10))  // 连接超时
        .build()
        .context("创建 HTTP 客户端失败")?;
    
    tracing::info!("已创建 HTTP 客户端（连接池配置: 每主机最大空闲连接数=2）");

    // 步骤 1: 登录获取 token
    println!("[1/4] 正在登录...");
    tracing::info!("========== 步骤 1/4: 登录 ==========");
    let token = login(&http_client, &args.server, &args.username, &args.password).await?;
    println!("✓ 登录成功");
    println!();

    // 步骤 2: 扫描输入目录
    println!("[2/4] 正在扫描输入目录...");
    tracing::info!("========== 步骤 2/4: 扫描目录 ==========");
    tracing::info!("扫描目录: {}", args.input);
    let scan_result = excel::scan_directory(&args.input)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    println!("✓ 发现 {} 个 Excel 文件", scan_result.file_count);
    tracing::info!("发现 {} 个 Excel 文件", scan_result.file_count);
    println!();

    // 步骤 3: 转换文件（如果需要）
    let files_to_upload = if !args.column_mappings.is_empty() {
        println!("[3/4] 正在转换文件...");
        tracing::info!("========== 步骤 3/4: 转换文件 ==========");
        
        // 解析列映射：仅在命令行提供参数时使用
        let parsed = parse_column_mappings(&args.column_mappings)?;
        println!("  使用列映射:");
        for mapping in &parsed {
            println!("    {} -> {}", mapping.original, mapping.mapped);
            tracing::info!("列映射: {} -> {}", mapping.original, mapping.mapped);
        }
        
        let converted_files = excel::convert_files(&args.input, &args.output, &parsed)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        
        println!("✓ 成功转换 {} 个文件", converted_files.len());
        tracing::info!("成功转换 {} 个文件", converted_files.len());
        println!();
        
        converted_files
    } else {
        println!("[3/4] 跳过文件转换（未配置列映射）...");
        tracing::info!("========== 步骤 3/4: 跳过转换 ==========");
        println!("✓ 将直接上传原始文件");
        tracing::info!("未配置列映射，将直接上传原始文件");
        println!();
        
        // 直接使用扫描到的原文件
        scan_result.files.clone()
    };

    // 步骤 4: 上传文件
    println!("[4/4] 正在上传文件...");
    tracing::info!("========== 步骤 4/4: 上传文件 ==========");
    tracing::info!("开始上传 {} 个文件", files_to_upload.len());
    
    let mut success_count = 0;
    let mut fail_count = 0;
    let mut failed_files: Vec<FailedFile> = Vec::new();
    
    // 创建进度条
    let pb = ProgressBar::new(files_to_upload.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-")
    );
    
    for (index, file_path) in files_to_upload.iter().enumerate() {
        let full_path = std::fs::canonicalize(file_path)
            .unwrap_or_else(|_| Path::new(file_path).to_path_buf());
        let display_path = full_path.display().to_string();
        
        pb.set_message(format!("正在上传 {}", display_path));
        tracing::info!("[{}/{}] 开始上传: {}", index + 1, files_to_upload.len(), display_path);
        
        // 带超时的上传 (10分钟)
        let upload_future = upload_file(
            &http_client,
            file_path,
            &args.server,
            &token,
            args.only_valid_invention
        );
        
        match timeout(Duration::from_secs(600), upload_future).await {
            Ok(Ok(result)) => {
                if result.success {
                    success_count += 1;
                    if let Some(data) = result.data {
                        let msg = format!("✓ {} (修改: {}, 新增: {})", 
                            display_path,
                            data.modified_count, 
                            data.upserted_count
                        );
                        pb.println(&msg);
                        tracing::info!("{}", msg);
                    } else {
                        let msg = format!("✓ {}", display_path);
                        pb.println(&msg);
                        tracing::info!("{}", msg);
                    }
                    
                    // 上传成功后等待3秒再继续下一个
                    if index + 1 < files_to_upload.len() {
                        tracing::info!("等待 3 秒后继续下一个文件...");
                        tokio::time::sleep(Duration::from_secs(3)).await;
                    }
                } else {
                    fail_count += 1;
                    let reason = result.message.unwrap_or_else(|| "未知错误".to_string());
                    let msg = format!("✗ {} - {}", display_path, reason);
                    pb.println(&msg);
                    tracing::error!("{}", msg);
                    failed_files.push(FailedFile {
                        path: display_path,
                        reason,
                    });
                }
            }
            Ok(Err(e)) => {
                fail_count += 1;
                let reason = e.to_string();
                let msg = format!("✗ {} - {}", display_path, reason);
                pb.println(&msg);
                tracing::error!("{}", msg);
                failed_files.push(FailedFile {
                    path: display_path,
                    reason,
                });
            }
            Err(_) => {
                fail_count += 1;
                let reason = "上传超时 (10分钟)".to_string();
                let msg = format!("✗ {} - {}", display_path, reason);
                pb.println(&msg);
                tracing::error!("{}", msg);
                failed_files.push(FailedFile {
                    path: display_path,
                    reason,
                });
            }
        }
        
        pb.inc(1);
    }
    
    pb.finish_and_clear();
    
    println!();
    println!("=== 上传完成 ===");
    println!("成功: {} 个文件", success_count);
    println!("失败: {} 个文件", fail_count);
    
    tracing::info!("上传完成 - 成功: {}, 失败: {}", success_count, fail_count);
    
    // 输出失败文件列表
    if !failed_files.is_empty() {
        println!();
        println!("失败的文件:");
        for failed in &failed_files {
            println!("  - {} (原因: {})", failed.path, failed.reason);
            tracing::error!("失败文件: {} - {}", failed.path, failed.reason);
        }
    }
    
    println!();
    println!("日志已保存至: {}", log_filename);

    Ok(())
}

/// 登录并获取 token
async fn login(client: &reqwest::Client, server_url: &str, username: &str, password: &str) -> Result<String> {
    tracing::info!("正在连接服务器: {}", server_url);
    
    let login_url = format!("{}/auth/admin/login", server_url.trim_end_matches('/'));
    
    let request = LoginRequest {
        username: username.to_string(),
        password: password.to_string(),
    };
    
    tracing::info!("发送登录请求...");
    let response = client
        .post(&login_url)
        .json(&request)
        .send()
        .await
        .context("发送登录请求失败")?;
    
    if !response.status().is_success() {
        let error = format!("登录失败: HTTP {}", response.status());
        tracing::error!("{}", error);
        anyhow::bail!(error);
    }
    
    let result: LoginResponse = response.json().await.context("解析登录响应失败")?;
    
    if result.success && result.data.is_some() {
        tracing::info!("登录成功，已获取 token");
        Ok(result.data.unwrap().token)
    } else {
        let error = format!("登录失败: {}", result.message.unwrap_or_else(|| "未知错误".to_string()));
        tracing::error!("{}", error);
        anyhow::bail!(error);
    }
}

/// 上传单个文件
async fn upload_file(
    client: &reqwest::Client,
    file_path: &str,
    server_url: &str,
    token: &str,
    only_valid_invention: bool,
) -> Result<UploadResult> {
    let path = Path::new(file_path);
    if !path.exists() {
        let error = format!("文件不存在: {}", file_path);
        tracing::error!("{}", error);
        anyhow::bail!(error);
    }
    
    tracing::debug!("读取文件内容: {}", file_path);
    // 读取文件内容
    let file_content = tokio::fs::read(&path)
        .await
        .context("读取文件失败")?;
    
    tracing::debug!("文件大小: {} 字节", file_content.len());
    
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file.xlsx")
        .to_string();
    
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
    
    tracing::debug!("发送上传请求: {}", upload_url);
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
        let error = format!("上传失败 [{}]: {}", status, error_text);
        tracing::error!("{}", error);
        anyhow::bail!(error);
    }
    
    tracing::debug!("解析上传响应");
    // 解析响应
    let result: UploadResult = response.json().await.context("解析响应失败")?;
    
    Ok(result)
}

/// 解析命令行传入的列映射参数
/// 格式: "原列名:映射列名"
fn parse_column_mappings(mappings: &[String]) -> Result<Vec<ColumnMapping>> {
    let mut result = Vec::new();
    
    for mapping_str in mappings {
        let parts: Vec<&str> = mapping_str.split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!(
                "列映射格式错误: '{}'。正确格式: '原列名:映射列名'",
                mapping_str
            );
        }
        
        let original = parts[0].trim();
        let mapped = parts[1].trim();
        
        if original.is_empty() || mapped.is_empty() {
            anyhow::bail!(
                "列名不能为空: '{}'",
                mapping_str
            );
        }
        
        result.push(ColumnMapping {
            original: original.to_string(),
            mapped: mapped.to_string(),
        });
    }
    
    Ok(result)
}