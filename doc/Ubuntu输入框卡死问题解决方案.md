# Ubuntu 24 输入框卡死问题解决方案

## 问题描述

在 Ubuntu 24 中，使用 Tauri 构建的应用程序在 build 后运行时，在服务器配置输入框中输入会导致整个应用卡死无响应。开发模式下运行正常。

## 问题原因

这是 Tauri/WebView2 在 Linux 系统上的已知问题，主要原因包括：

1. **输入法框架冲突**：IBus/Fcitx 等输入法框架与 WebView 的交互存在问题
2. **WebKit 合成模式**：WebKit 的某些渲染特性在某些环境下不稳定
3. **X11/Wayland 兼容性**：显示服务器的差异可能导致输入处理问题

## 解决方案

### 方案 1：使用启动脚本（推荐）

项目根目录提供了 `patentupload.sh` 启动脚本，该脚本会自动设置所需的环境变量：

```bash
# 进入项目根目录
cd /path/to/PatentUpload

# 使用启动脚本运行
./patentupload.sh
```

启动脚本做了以下配置：
- 设置 `GTK_IM_MODULE=xim` 和 `QT_IM_MODULE=xim` 使用简单的输入法模块
- 设置 `XMODIFIERS=@im=none` 禁用 X 输入法扩展
- 设置 `WEBKIT_DISABLE_COMPOSITING_MODE=1` 禁用 WebKit 合成模式

### 方案 2：手动设置环境变量

如果不想使用脚本，可以在终端手动设置环境变量后启动：

```bash
export GTK_IM_MODULE=xim
export QT_IM_MODULE=xim
export XMODIFIERS=@im=none
export WEBKIT_DISABLE_COMPOSITING_MODE=1

# 启动应用
./patentupload
```

### 方案 3：创建 Desktop 文件

对于已安装的应用，可以修改 `.desktop` 文件添加环境变量：

```bash
# 编辑 desktop 文件
sudo nano /usr/share/applications/patentupload.desktop
```

修改 `Exec` 行：
```ini
[Desktop Entry]
Name=专利文件导入工具
Exec=env GTK_IM_MODULE=xim QT_IM_MODULE=xim XMODIFIERS=@im=none WEBKIT_DISABLE_COMPOSITING_MODE=1 patentupload
Icon=patentupload
Type=Application
Categories=Utility;
```

### 方案 4：如果仍有问题

如果上述方案仍无法解决问题，可以尝试：

1. **强制使用 X11**（针对 Wayland 用户）：
```bash
export GDK_BACKEND=x11
```

2. **禁用 DMA-BUF 渲染器**：
```bash
export WEBKIT_DISABLE_DMABUF_RENDERER=1
```

3. **临时禁用输入法**：
```bash
# 临时关闭 ibus
ibus exit

# 或关闭 fcitx
fcitx-remote -e
```

## 项目代码改进

除了环境变量配置，项目在 `tauri.conf.json` 中也做了一些改进：

```json
{
  "app": {
    "windows": [{
      "webviewOptions": {
        "windowEffects": {
          "linux": {
            "disableAutomaticWindowTabbing": true
          }
        }
      }
    }],
    "withGlobalTauri": true
  }
}
```

## 验证修复

修复后，可以通过以下步骤验证：

1. 使用启动脚本运行应用
2. 在"服务器配置"标签页的输入框中输入文字
3. 确认输入流畅，没有卡顿或卡死
4. 切换中英文输入法测试
5. 测试其他输入框（用户名、密码等）

## 相关链接

- [Tauri Issue: Input freeze on Linux](https://github.com/tauri-apps/tauri/issues/9109)
- [WebKitGTK Input Issues](https://github.com/tauri-apps/wry/issues/1009)
- [Linux Input Method Issues](https://github.com/tauri-apps/tauri/issues/4243)

## 总结

这是一个 Tauri 在 Linux 平台上的已知限制，通过适当的环境变量配置可以有效解决。推荐使用提供的启动脚本来确保应用正常运行。
