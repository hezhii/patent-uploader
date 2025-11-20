#!/bin/bash
# 专利文件导入工具启动脚本
# 用于解决 Ubuntu 24 中输入法导致的输入框卡死问题

# 禁用输入法模块，避免与 WebView 冲突
export GTK_IM_MODULE=xim
export QT_IM_MODULE=xim
export XMODIFIERS=@im=none

# 禁用 WebKit 合成模式，提高稳定性
export WEBKIT_DISABLE_COMPOSITING_MODE=1

# 强制使用 X11 而非 Wayland（可选，如果使用 Wayland 仍有问题）
# export GDK_BACKEND=x11

# 设置 WebView 使用硬件加速（可选）
# export WEBKIT_DISABLE_DMABUF_RENDERER=1

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 启动应用程序
# 如果是开发模式，使用相对路径
if [ -f "$SCRIPT_DIR/src-tauri/target/debug/patentupload" ]; then
    echo "启动开发版本..."
    exec "$SCRIPT_DIR/src-tauri/target/debug/patentupload"
# 如果是生产环境，查找已安装的程序
elif command -v patentupload &> /dev/null; then
    echo "启动已安装版本..."
    exec patentupload
# 或使用 release 构建
elif [ -f "$SCRIPT_DIR/src-tauri/target/release/patentupload" ]; then
    echo "启动 Release 版本..."
    exec "$SCRIPT_DIR/src-tauri/target/release/patentupload"
else
    echo "错误：找不到 patentupload 可执行文件"
    echo "请先构建项目：cd src-tauri && cargo build --release"
    exit 1
fi
