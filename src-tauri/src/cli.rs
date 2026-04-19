use clap::{Parser, ValueEnum};

/// 导入模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum ImportMode {
    /// 上传所有专利
    #[default]
    All,
    /// 仅上传发明专利
    #[value(name = "invention-only")]
    InventionOnly,
    /// 仅上传有效发明专利
    #[value(name = "valid-invention-only")]
    ValidInventionOnly,
}

impl std::fmt::Display for ImportMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportMode::All => write!(f, "all"),
            ImportMode::InventionOnly => write!(f, "inventionOnly"),
            ImportMode::ValidInventionOnly => write!(f, "validInventionOnly"),
        }
    }
}

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

    /// 导入模式: all (所有专利) / invention-only (仅发明专利) / valid-invention-only (仅有效发明专利)
    #[arg(short = 'v', long, default_value_t = ImportMode::All, value_name = "MODE")]
    pub import_mode: ImportMode,

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
