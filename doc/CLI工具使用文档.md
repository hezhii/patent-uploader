# 命令行工具使用文档

## 概述

PatentUpload 现在支持命令行工具模式，无需图形界面即可完成专利文件的扫描、转换和上传。

## 构建

### 构建 CLI 工具

```bash
cd src-tauri
cargo build --release --bin patent-cli
```

构建完成后，可执行文件位于：
- macOS/Linux: `src-tauri/target/release/patent-cli`
- Windows: `src-tauri/target/release/patent-cli.exe`

### 构建 GUI 应用（原有功能）

```bash
pnpm tauri build
```

## 使用方法

### 命令行参数

```bash
patent-cli [OPTIONS]
```

#### 必需参数

- `-s, --server <SERVER>` - 服务器地址，例如: `http://localhost:3000`
- `-u, --username <USERNAME>` - 用户账号
- `-p, --password <PASSWORD>` - 用户密码
- `-i, --input <INPUT>` - 输入目录路径（包含源 Excel 文件）
- `-o, --output <OUTPUT>` - 输出目录路径（存储转换后的文件，仅在配置列映射时需要）

#### 可选参数

- `-v, --only-valid-invention` - 是否仅上传有效发明专利（默认: false）
- `-m, --column-mapping <MAPPING>` - 列名映射（格式: "原列名:映射列名"，可多次指定）
  - **如果不指定列映射，将直接上传原始 Excel 文件，无需进行列转换**
  - **注意：当指定列映射时，必须同时指定 `-o` 输出目录**

### 使用示例

#### 1. 直接上传（无列映射，不进行转换）

**新功能：** 如果不需要对 Excel 列进行转换，可以不指定 `--column-mapping` 参数，工具将直接上传原始文件。

```bash
./patent-cli \
  --server http://localhost:3000 \
  --username admin \
  --password admin123 \
  --input /path/to/input \
  --output /path/to/output
```

**注意：** 当不配置列映射时，`--output` 参数将被忽略，因为不需要生成转换后的文件。

#### 2. 仅上传有效发明专利（直接上传模式）

```bash
./patent-cli \
  --server http://localhost:3000 \
  --username admin \
  --password admin123 \
  --input /path/to/input \
  --output /path/to/output \
  --only-valid-invention
```

#### 3. 使用自定义列映射（转换模式）

**指定列映射时，工具会先转换 Excel 文件，然后上传转换后的文件。**

```bash
./patent-cli \
  --server http://localhost:3000 \
  --username admin \
  --password admin123 \
  --input /path/to/input \
  --output /path/to/output \
  --column-mapping "申请号:申请号" \
  --column-mapping "申请日:申请日" \
  --column-mapping "名称:专利名称" \
  --column-mapping "类型:专利类型" \
  --column-mapping "法律状态:法律状态" \
  --column-mapping "申请人:申请人"
```

#### 4. 使用短参数

```bash
./patent-cli \
  -s http://localhost:3000 \
  -u admin \
  -p admin123 \
  -i /path/to/input \
  -o /path/to/output \
  -v \
  -m "申请号:申请号" \
  -m "名称:专利名称"
```

## 执行流程

CLI 工具会按以下步骤执行：

1. **登录认证** - 使用提供的用户名和密码登录服务器，获取访问令牌
2. **扫描文件** - 扫描输入目录中的所有 Excel 文件
3. **转换文件** - 将源 Excel 文件转换为标准格式，保存到输出目录
4. **上传文件** - 依次上传转换后的文件到服务器

## 输出示例

```
=== 专利文件上传工具 ===
服务器地址: http://localhost:3000
用户名: admin
输入目录: /path/to/input
输出目录: /path/to/output
仅上传有效发明专利: false
列名映射: 6 组

[1/4] 正在登录...
✓ 登录成功

[2/4] 正在扫描输入目录...
✓ 发现 5 个 Excel 文件

[3/4] 正在转换文件...
  使用列映射:
    申请号 -> 申请号
    申请日 -> 申请日
    名称 -> 专利名称
    类型 -> 专利类型
    法律状态 -> 法律状态
    申请人 -> 申请人
✓ 成功转换 5 个文件

[4/4] 正在上传文件...
  [1/5] 正在上传 file1.xlsx ... ✓ (修改: 10, 新增: 5)
  [2/5] 正在上传 file2.xlsx ... ✓ (修改: 8, 新增: 12)
  [3/5] 正在上传 file3.xlsx ... ✓ (修改: 15, 新增: 3)
  [4/5] 正在上传 file4.xlsx ... ✓ (修改: 6, 新增: 9)
  [5/5] 正在上传 file5.xlsx ... ✓ (修改: 11, 新增: 7)

=== 上传完成 ===
成功: 5 个文件
失败: 0 个文件
```

## 错误处理

- 如果登录失败，工具会立即退出并显示错误信息
- 如果扫描或转换失败，会显示详细的错误信息
- 如果某个文件上传失败，会继续处理其他文件，最后汇总成功和失败的数量

## 日志

CLI 工具使用 `tracing` 记录日志。可以通过环境变量控制日志级别：

```bash
RUST_LOG=patentupload=debug ./patent-cli [OPTIONS]
```

日志级别：
- `error` - 仅错误信息
- `warn` - 警告及以上
- `info` - 信息及以上（默认）
- `debug` - 调试及以上
- `trace` - 所有日志

## 配置列映射

### 命令行参数方式（推荐）

使用 `--column-mapping` 或 `-m` 参数指定列映射，可以多次使用该参数来指定多组映射：

```bash
./patent-cli \
  --server http://localhost:3000 \
  --username admin \
  --password admin123 \
  --input /path/to/input \
  --output /path/to/output \
  --column-mapping "申请号:申请号" \
  --column-mapping "申请日:申请日" \
  --column-mapping "名称:专利名称"
```

**映射格式**：`"原列名:映射列名"`
- 冒号前是 Excel 文件中的原始列名
- 冒号后是要映射到的目标列名
- 每个映射使用一个 `--column-mapping` 参数

### 默认映射方式

如果不指定 `--column-mapping` 参数，工具会使用默认的列映射。默认映射在 `src-tauri/src/bin/patent-cli.rs` 的 `get_default_mappings()` 函数中定义。

如需修改默认映射，请编辑该函数：

```rust
fn get_default_mappings() -> Vec<ColumnMapping> {
    vec![
        ColumnMapping {
            original: "申请号".to_string(),
            mapped: "申请号".to_string(),
        },
        // 添加更多映射...
    ]
}
```

修改后需要重新构建工具。

## 注意事项

1. 确保输入和输出目录存在且有读写权限
2. 服务器地址需要包含协议（http:// 或 https://）
3. 账号密码需要有上传专利文件的权限
4. 转换过程会在输出目录创建新文件，不会修改输入目录的原始文件

## 集成到 CI/CD

可以将 CLI 工具集成到自动化流程中：

```bash
#!/bin/bash
set -e

# 构建 CLI 工具
cd src-tauri
cargo build --release --bin patent-cli

# 运行上传任务
./target/release/patent-cli \
  --server "$SERVER_URL" \
  --username "$USERNAME" \
  --password "$PASSWORD" \
  --input "$INPUT_DIR" \
  --output "$OUTPUT_DIR" \
  --only-valid-invention

echo "专利文件上传完成"
```
