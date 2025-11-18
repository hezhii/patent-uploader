# 更新日志

## 2025-11-18 (第三次更新)

### 🔧 修复文件上传问题

#### 问题描述
服务器端报错 `FILE_ENDED`，表示接收到的文件数据不完整或损坏。错误发生在 unzipper 解析 Excel 文件时。

#### 根本原因
JavaScript 端通过 `FormData` 和 `XMLHttpRequest` 上传文件，但在 Tauri 应用中，这种方式可能无法正确读取本地文件的完整内容，导致上传的文件数据不完整。

#### 解决方案
**将文件上传逻辑从 JavaScript 迁移到 Rust**

1. **添加 Rust HTTP 客户端依赖**
   - 在 `src-tauri/Cargo.toml` 中添加 `reqwest` (支持 multipart 和 json)
   - 使用 Rust 的文件系统 API 读取完整文件内容

2. **创建 Rust 上传命令**
   - 新建 `src-tauri/src/commands/upload.rs`
   - 实现 `upload_file` 命令：
     - 读取本地文件完整内容
     - 构建 multipart/form-data 请求
     - 发送到服务器并处理响应
   - 注册到 `mod.rs` 和 `lib.rs`

3. **修改 TypeScript 上传逻辑**
   - 在 `useFileUpload.ts` 中调用 Tauri 命令替代 HTTP 请求
   - 从 File 对象的 `filePath` 属性获取文件路径
   - 传递 token 和 serverUrl 给 Rust 端

4. **清理旧代码**
   - 从 `httpClient.ts` 中移除 `uploadFile` 方法
   - 保留 `login` 和 `testConnection` 方法（仍使用 fetch）
   - 添加 `getToken()` 方法供上传使用

#### 修改的文件
- `src-tauri/Cargo.toml` - 添加 reqwest 依赖
- `src-tauri/src/commands/upload.rs` - 新建上传命令
- `src-tauri/src/commands/mod.rs` - 导出上传模块
- `src-tauri/src/lib.rs` - 注册上传命令
- `src/composables/useFileUpload.ts` - 使用 Tauri 命令上传
- `src/utils/httpClient.ts` - 移除上传方法，保留登录

#### 优势
- ✅ 文件读取更可靠：Rust 原生文件系统 API
- ✅ 数据完整性：避免 JavaScript 环境的文件访问限制
- ✅ 更好的错误处理：Rust 的 Result 类型
- ✅ 性能提升：直接在 Rust 层处理文件 I/O

## 2025-11-18 (第二次更新)

### ✅ 新增功能

#### 1. 服务器配置和列名映射日志增强
- **改进位置**: 
  - `src/components/ServerConfig.vue`
  - `src/components/ColumnMapping.vue`
- **新增日志**:
  - 服务器配置保存、测试连接（成功/失败）
  - 列映射添加、删除、导入、导出
  - 所有关键操作的详细记录

#### 2. 修复前端日志显示问题
- **问题**: 日志显示"暂无日志记录"
- **原因**: useLogger 每次调用都创建新实例，导致日志不共享
- **解决方案**: 将 useLogger 改为单例模式，使用全局日志存储
- **效果**: 所有组件现在共享同一个日志实例，日志能正确显示

#### 3. Rust 日志集成到前端 ⭐
- **实现方式**: 
  - 创建自定义 `TauriLayer` (src-tauri/src/utils/logger.rs)
  - 使用 Tauri event 系统发送日志到前端
  - 前端通过 `listen('rust-log')` 接收并显示
- **新增文件**:
  - `src-tauri/src/utils/logger.rs` - 自定义 tracing layer
- **修改文件**:
  - `src-tauri/src/lib.rs` - 配置 logger
  - `src/App.vue` - 监听 Rust 日志事件
- **日志格式**: `[Rust] 消息内容`
- **效果**: 前端可以实时看到 Rust 层的所有日志

#### 4. 修复文件上传问题 🔧
**问题1: 上传失败提示"请先登录"**
- **原因**: 上传前未进行身份验证
- **解决方案**: 在 `startUpload` 前自动调用登录
- **流程**: 读取配置 → 登录 → 上传文件

**问题2: 上传卡住无响应**
- **原因**: 单个文件失败导致整个流程中断
- **解决方案**: 
  - 使用 try-catch 包裹单文件上传
  - 失败后记录日志并继续下一个
  - 添加暂停/继续功能
- **改进**: 
  - 添加 `isPaused` 状态
  - 实现 `pauseUpload` 函数
  - 上传循环中检查暂停状态

#### 5. 步骤展开/收起功能 🎨
- **实现位置**: `src/App.vue`
- **新增状态**: `expandedSteps` - 记录每个步骤的展开状态
- **交互方式**: 点击步骤标题展开/收起
- **自动收起**: 
  - 完成服务器配置后，自动收起步骤1，展开步骤2
  - 完成列名映射后，自动收起步骤2，展开步骤3
  - 完成文件转换后，自动收起步骤3，展开步骤4
- **视觉效果**: 使用 ▼ (展开) 和 ▶ (收起) 图标
- **优化**: 
  - 移除子组件中的重复标题
  - 统一步骤卡片样式

### 🔧 技术细节

#### 新增的 Rust 代码
```rust
// src-tauri/src/utils/logger.rs
pub struct TauriLayer {
    app_handle: AppHandle,
}

impl<S> Layer<S> for TauriLayer {
    fn on_event(&self, event: &tracing::Event, _ctx: Context) {
        // 将 Rust 日志发送到前端
        let _ = self.app_handle.emit("rust-log", &log_event);
    }
}
```

#### 前端日志监听
```typescript
// App.vue
unlistenRustLog = await listen('rust-log', (event) => {
  const { level, message } = event.payload;
  addLog(level, `[Rust] ${message}`);
});
```

#### 文件上传流程改进
```typescript
async function startUpload() {
  // 1. 自动登录
  await httpClient.login(username, password);
  
  // 2. 逐个上传，失败不中断
  for (let i = 0; i < files.length; i++) {
    try {
      await uploadSingleFile(i);
    } catch (error) {
      logger.error(`文件失败，继续下一个`);
    }
  }
}
```

### 📋 文件修改清单
```
前端:
- src/App.vue - 添加步骤展开/收起、Rust 日志监听
- src/components/ServerConfig.vue - 添加日志、移除标题
- src/components/ColumnMapping.vue - 添加日志、移除标题
- src/components/FileUpload.vue - 更新暂停功能
- src/composables/useLogger.ts - 改为单例模式
- src/composables/useFileUpload.ts - 添加自动登录、暂停功能

后端:
- src-tauri/src/utils/logger.rs - 新增自定义 logger
- src-tauri/src/utils/mod.rs - 导出 TauriLayer
- src-tauri/src/lib.rs - 配置日志系统
```

### 🎯 功能验证

#### 测试步骤
1. ✅ **日志显示测试**
   - 打开应用，检查是否显示"系统已启动"日志
   - 进行各种操作，确认日志实时更新

2. ✅ **Rust 日志测试**
   - 扫描文件，查看是否显示 `[Rust]` 开头的日志
   - 转换文件，验证详细的处理日志

3. ✅ **上传功能测试**
   - 配置服务器后直接上传
   - 验证自动登录流程
   - 测试暂停/继续功能

4. ✅ **步骤展开测试**
   - 点击步骤标题，验证展开/收起
   - 完成步骤，验证自动收起和展开下一步

### 🐛 已修复的问题
1. ✅ 前端显示"暂无日志记录"
2. ✅ 上传文件时提示"请先登录"
3. ✅ 上传失败后卡住无响应
4. ✅ Rust 日志无法在前端查看
5. ✅ 所有步骤同时展开，界面混乱

### 💡 改进建议
1. 可以为步骤添加完成标记（✓）
2. 可以添加更多的日志级别过滤选项
3. 可以实现日志搜索功能
4. 可以添加上传队列管理（优先级、排序等）

---

## 2025-11-18 (第一次更新)

### ✅ 已完成功能

#### 1. 修复文件夹选择权限问题
- **问题**: 选择文件夹时报错 "dialog.open not allowed"
- **解决方案**: 在 `src-tauri/capabilities/default.json` 中添加了以下权限：
  - `dialog:default`
  - `dialog:allow-open`
  - `dialog:allow-save`
- **影响**: 用户现在可以正常选择源文件夹和目标文件夹

#### 2. 添加跳过列名映射功能
- **功能**: 允许用户跳过列名映射配置步骤
- **实现位置**: `src/components/ColumnMapping.vue`
- **使用方法**: 
  - 在列名映射配置界面，点击"跳过配置"按钮
  - 系统将使用空的映射配置继续下一步
- **适用场景**: 当 Excel 文件的列名已经符合要求时，无需进行映射

#### 3. 实现配置持久化保存
- **功能**: 服务器配置和列名映射配置会自动保存到本地
- **实现位置**: 
  - `src/stores/index.ts` - 数据存储逻辑
  - `src/App.vue` - 调用保存方法
- **保存机制**:
  - 使用 localStorage 进行本地持久化
  - 服务器配置保存键: `patent-upload-server-config`
  - 列名映射保存键: `patent-upload-column-mappings`
- **效果**: 关闭程序后重新打开，之前的配置会自动加载

#### 4. 完善前端日志记录
- **改进位置**:
  - `src/composables/useFileOperations.ts` - 文件操作日志
  - `src/composables/useFileUpload.ts` - 文件上传日志
- **新增日志内容**:
  - 文件夹选择操作（成功/失败/取消）
  - 文件扫描进度（开始/完成/文件数量/总大小）
  - 文件转换进度（每个文件的转换状态）
  - 文件上传进度（每个文件的上传状态）
  - 所有关键操作的成功和失败信息

#### 5. 完善后端日志记录
- **改进位置**: `src-tauri/src/excel/mod.rs`
- **新增日志内容**:
  - 目录扫描过程（发现的每个文件）
  - 文件转换进度（处理中的文件、工作表、映射规则）
  - 列映射应用情况（实际应用的映射数量）
  - 文件创建和保存操作
  - 错误详细信息
- **日志级别**:
  - `info`: 关键操作节点
  - `debug`: 详细处理过程
  - `error`: 错误信息

#### 6. 实现日志导出功能
- **功能**: 将所有日志导出为 txt 文件
- **实现位置**:
  - 前端: `src/composables/useLogger.ts`
  - 后端: `src-tauri/src/commands/file_operations.rs` - `save_log_file` 命令
- **使用方法**:
  1. 在日志查看器中点击"导出日志"按钮
  2. 选择保存位置和文件名
  3. 日志会以文本格式保存，包含时间戳和日志级别
- **日志格式**: `[时间] [级别] 消息内容`

### 📋 技术细节

#### 文件修改清单
```
前端文件:
- src/App.vue - 添加配置保存逻辑
- src/components/ColumnMapping.vue - 添加跳过按钮
- src/components/ServerConfig.vue - 保持不变（已有配置逻辑）
- src/composables/useLogger.ts - 实现日志导出
- src/composables/useFileOperations.ts - 添加详细日志
- src/composables/useFileUpload.ts - 添加详细日志
- src/stores/index.ts - 持久化存储（已有实现）

后端文件:
- src-tauri/capabilities/default.json - 添加对话框权限
- src-tauri/src/lib.rs - 注册新命令
- src-tauri/src/commands/file_operations.rs - 添加日志保存命令
- src-tauri/src/excel/mod.rs - 完善日志记录
```

#### 新增的 Tauri 命令
- `save_log_file(path: String, content: String)` - 保存日志文件到指定路径

### 🔄 待优化功能

#### 7. 前端集成 Rust 日志（可选）
- **目标**: 将 Rust 层的 tracing 日志实时传递到前端显示
- **挑战**: Tauri v2 中实现日志流传输需要额外配置
- **当前状态**: 
  - Rust 日志已完善，可通过终端查看
  - 前端日志独立运行，记录 JavaScript 层操作
- **未来改进**: 可以通过 Tauri 的 event 系统实现日志同步

### 📝 使用说明

#### 启动应用
```bash
# 开发模式
pnpm tauri dev

# 构建生产版本
pnpm tauri build
```

#### 查看后端日志
在开发模式下，Rust 日志会输出到终端。日志级别配置在 `src-tauri/src/lib.rs` 中：
```rust
tracing_subscriber::fmt()
    .with_env_filter("patentupload=debug,tauri=info")
    .init();
```

#### 配置文件位置
- **浏览器存储**: localStorage (浏览器开发工具 > Application > Local Storage)
- **配置键**:
  - `patent-upload-server-config` - 服务器配置
  - `patent-upload-column-mappings` - 列名映射
  - `patent-upload-settings` - 应用设置

### 🐛 已修复的问题
1. ✅ 文件夹选择权限错误
2. ✅ 配置不能持久化保存
3. ✅ 日志信息不完整
4. ✅ 无法导出日志

### 🎯 测试建议
1. 测试文件夹选择功能是否正常
2. 测试配置保存和加载（关闭重开应用）
3. 测试跳过列名映射功能
4. 测试日志导出功能
5. 检查日志是否记录了所有关键操作
