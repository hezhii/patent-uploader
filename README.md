# 专利文件导入工具 (Patent Upload Tool)

基于 Tauri + Vue 3 + TypeScript 构建的桌面应用程序，用于批量导入专利文件到服务器。

## 功能特性

- 📁 批量扫描和选择专利文件
- 📊 Excel 文件预览和列名映射
- 🔐 服务器连接配置和身份验证
- 📤 多文件并发上传，支持进度跟踪
- 🎯 支持仅导入有效发明专利选项
- 📝 详细的操作日志记录
- 💾 配置自动保存

## 技术栈

- **前端**: Vue 3 + TypeScript + Vite
- **后端**: Rust + Tauri
- **UI**: Tailwind CSS
- **状态管理**: Pinia
- **HTTP**: Reqwest (Rust) + Fetch API

## 开发环境要求

- Node.js 18+
- Rust 1.70+
- pnpm (推荐) 或 npm

## 安装和运行

### 开发模式

```bash
# 安装依赖
pnpm install

# 启动开发服务器
pnpm tauri dev
```

### 构建生产版本

```bash
# 构建
pnpm tauri build
```

构建完成后，可执行文件位于 `src-tauri/target/release/`

## Linux (Ubuntu) 使用说明

### Ubuntu 24 输入框卡死问题

如果在 Ubuntu 24 中遇到输入框输入时应用卡死的问题，请使用提供的启动脚本：

```bash
# 给启动脚本添加执行权限（首次使用）
chmod +x patentupload.sh

# 使用启动脚本运行
./patentupload.sh
```

**问题原因**: 这是 Tauri/WebView 与 Linux 输入法框架的兼容性问题。

详细的解决方案请参考：[Ubuntu输入框卡死问题解决方案](./doc/Ubuntu输入框卡死问题解决方案.md)

## 使用说明

1. **服务器配置**: 配置服务器地址、用户名和密码
2. **扫描文件**: 选择包含专利文件的文件夹
3. **列名映射**: 配置 Excel 列名与服务器字段的映射关系
4. **上传选项**: 选择是否仅导入有效发明专利
5. **开始上传**: 批量上传文件并查看进度

## 开发文档

- [项目需求](./doc/项目需求.md)
- [项目设计方案](./doc/项目设计方案.md)
- [打包部署文档](./doc/打包部署文档.md)
- [Ubuntu输入框卡死问题解决方案](./doc/Ubuntu输入框卡死问题解决方案.md)

## IDE 推荐配置

- [VS Code](https://code.visualstudio.com/)
- [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar)
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## 许可证

MIT

