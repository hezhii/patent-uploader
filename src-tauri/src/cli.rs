use clap::Parser;

/// 专利文件上传命令行工具
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// 服务器地址 (例如: http://localhost:3000)
    #[arg(short, long)]
    pub server: String,

    /// 用户账号
    #[arg(short = 'u', long)]
    pub username: String,

    /// 用户密码
    #[arg(short, long)]
    pub password: String,

    /// 输入目录路径（包含源 Excel 文件）
    #[arg(short, long)]
    pub input: String,

    /// 输出目录路径（存储转换后的文件）
    #[arg(short, long)]
    pub output: String,

    /// 是否仅上传有效发明专利
    #[arg(short = 'v', long, default_value_t = false)]
    pub only_valid_invention: bool,

    /// 列名映射（格式: "原列名:映射列名"，可多次指定）
    /// 例如: --column-mapping "申请号:申请号" --column-mapping "名称:专利名称"
    #[arg(short = 'm', long = "column-mapping", value_name = "MAPPING")]
    pub column_mappings: Vec<String>,
}

impl CliArgs {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
