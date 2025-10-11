# manager/persistent_aria2.rs - 持久化Aria2下载管理器

## 模块概述

持久化Aria2下载管理器将`Aria2DownloadManager`与`DownloadRepository`集成，提供下载任务和进度的自动数据库持久化功能。

## 配置常量

- `ARIA2_RPC_URL`: "http://localhost:6800/jsonrpc" - Aria2 RPC服务地址
- `ARIA2_RPC_SECRET`: "burncloud" - Aria2 RPC密钥
- `PROGRESS_SAVE_INTERVAL_SECS`: 5 - 进度保存间隔（秒）
- `STATUS_POLL_INTERVAL_SECS`: 1 - 状态轮询间隔（秒）

## 结构体

### PersistentAria2Manager
- **位置**: src/manager/persistent_aria2.rs:54
- **说明**: 集成Aria2与数据库持久化的下载管理器

#### 字段
- `aria2: Arc<Aria2DownloadManager>` - Aria2下载管理器实例
- `repository: Arc<DownloadRepository>` - 数据库仓库
- `task_mapping: Arc<RwLock<HashMap<TaskId, String>>>` - TaskId到Aria2 GID的映射
- `persistence_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>` - 持久化任务句柄
- `shutdown: Arc<tokio::sync::Notify>` - 关闭通知器

## 公共方法

### new()
- **位置**: src/manager/persistent_aria2.rs:64
- **功能**: 使用默认配置创建新的持久化下载管理器
- **返回值**: `Result<Self>`
- **说明**: 调用`new_with_config`使用默认参数

### new_with_config(rpc_url, secret, db_path)
- **位置**: src/manager/persistent_aria2.rs:73
- **功能**: 使用自定义配置创建持久化下载管理器
- **参数**:
  - `rpc_url: String` - Aria2 RPC URL
  - `secret: String` - RPC密钥
  - `db_path: Option<PathBuf>` - 数据库路径（可选）
- **返回值**: `Result<Self>`
- **说明**: 初始化数据库、Aria2管理器，恢复任务并启动持久化轮询器

### shutdown()
- **位置**: src/manager/persistent_aria2.rs:328
- **功能**: 优雅地关闭管理器
- **返回值**: `Result<()>`
- **说明**: 通知关闭、等待持久化轮询器结束并最终保存所有任务

## 私有方法

### restore_tasks()
- **位置**: src/manager/persistent_aria2.rs:121
- **功能**: 在启动时从数据库恢复未完成的任务
- **返回值**: `Result<()>`
- **说明**: 查询数据库中的任务，过滤已完成的，尝试恢复到Aria2中

### restore_single_task(task)
- **位置**: src/manager/persistent_aria2.rs:163
- **功能**: 恢复单个任务到Aria2
- **参数**: `task: &DownloadTask` - 要恢复的任务
- **返回值**: `Result<String>` - 新的Aria2 GID
- **说明**: 重新添加下载到Aria2并应用原始状态

### get_gid_for_task(task_id)
- **位置**: src/manager/persistent_aria2.rs:182
- **功能**: 获取任务对应的Aria2 GID
- **参数**: `task_id: TaskId` - 任务ID
- **返回值**: `Result<String>` - Aria2 GID
- **说明**: 从Aria2管理器获取任务的GID标识符

### store_task_mapping(task_id, gid)
- **位置**: src/manager/persistent_aria2.rs:195
- **功能**: 存储TaskId到Aria2 GID的映射
- **参数**:
  - `task_id: TaskId` - 任务ID
  - `gid: String` - Aria2 GID
- **说明**: 维护任务ID与Aria2内部GID的对应关系

### remove_task_mapping(task_id)
- **位置**: src/manager/persistent_aria2.rs:202
- **功能**: 移除任务映射
- **参数**: `task_id: TaskId` - 任务ID
- **说明**: 清理不再需要的映射关系

### create_new_download(url, target_path)
- **位置**: src/manager/persistent_aria2.rs:210
- **功能**: 创建新下载任务的内部方法（不检查重复）
- **参数**:
  - `url: String` - 下载URL
  - `target_path: PathBuf` - 目标路径
- **返回值**: `Result<TaskId>`
- **说明**: 确保目标目录存在，添加到Aria2，保存到数据库，存储GID映射

### start_persistence_poller()
- **位置**: src/manager/persistent_aria2.rs:241
- **功能**: 启动后台持久化轮询器
- **说明**: 每秒检查任务状态变化，每5秒保存进度到数据库

### save_all_tasks()
- **位置**: src/manager/persistent_aria2.rs:307
- **功能**: 保存所有当前任务到数据库
- **返回值**: `Result<()>`
- **说明**: 批量保存任务和进度信息，通常在关闭时调用

## DownloadManager trait 实现

### add_download(url, target_path)
- **位置**: src/manager/persistent_aria2.rs:349
- **功能**: 添加新的下载任务
- **参数**:
  - `url: String` - 下载URL
  - `target_path: PathBuf` - 目标路径
- **返回值**: `Result<TaskId>`
- **说明**: 使用默认重复策略，检查重复后决定是否创建新任务

### pause_download(task_id)
- **位置**: src/manager/persistent_aria2.rs:371
- **功能**: 暂停下载任务
- **参数**: `task_id: TaskId` - 任务ID
- **返回值**: `Result<()>`
- **说明**: 在Aria2中暂停并立即更新数据库状态

### resume_download(task_id)
- **位置**: src/manager/persistent_aria2.rs:387
- **功能**: 恢复下载任务
- **参数**: `task_id: TaskId` - 任务ID
- **返回值**: `Result<()>`
- **说明**: 在Aria2中恢复并立即更新数据库状态

### cancel_download(task_id)
- **位置**: src/manager/persistent_aria2.rs:403
- **功能**: 取消下载任务
- **参数**: `task_id: TaskId` - 任务ID
- **返回值**: `Result<()>`
- **说明**: 在Aria2中取消，从数据库删除任务和进度，移除映射

### get_progress(task_id)
- **位置**: src/manager/persistent_aria2.rs:423
- **功能**: 获取下载进度
- **参数**: `task_id: TaskId` - 任务ID
- **返回值**: `Result<DownloadProgress>`
- **说明**: 始终从Aria2获取最新数据

### get_task(task_id)
- **位置**: src/manager/persistent_aria2.rs:428
- **功能**: 获取任务信息
- **参数**: `task_id: TaskId` - 任务ID
- **返回值**: `Result<DownloadTask>`
- **说明**: 始终从Aria2获取最新数据

### list_tasks()
- **位置**: src/manager/persistent_aria2.rs:433
- **功能**: 列出所有任务
- **返回值**: `Result<Vec<DownloadTask>>`
- **说明**: 从Aria2获取最新状态的任务列表

### active_download_count()
- **位置**: src/manager/persistent_aria2.rs:438
- **功能**: 获取活跃下载数量
- **返回值**: `Result<usize>`
- **说明**: 代理到Aria2管理器

## 重复检测方法

### find_duplicate_task(url, target_path)
- **位置**: src/manager/persistent_aria2.rs:444
- **功能**: 查找重复任务
- **参数**:
  - `url: &str` - URL地址
  - `target_path: &Path` - 目标路径
- **返回值**: `Result<Option<TaskId>>`
- **说明**: 先检查活跃任务，再检查数据库中的所有任务

### add_download_with_policy(url, target_path, policy)
- **位置**: src/manager/persistent_aria2.rs:479
- **功能**: 根据策略添加下载
- **参数**:
  - `url: &str` - URL地址
  - `target_path: &Path` - 目标路径
  - `policy: DuplicatePolicy` - 重复策略
- **返回值**: `Result<DuplicateResult>`
- **说明**: 检查重复，根据策略决定重用现有任务或创建新任务，自动恢复暂停/失败的任务

### verify_task_validity(task_id)
- **位置**: src/manager/persistent_aria2.rs:536
- **功能**: 验证任务有效性
- **参数**: `task_id: &TaskId` - 任务ID引用
- **返回值**: `Result<bool>`
- **说明**: 检查Aria2中的活跃任务和数据库中的任务，对已完成任务验证文件是否存在

### get_duplicate_candidates(url, target_path)
- **位置**: src/manager/persistent_aria2.rs:558
- **功能**: 获取重复候选任务
- **参数**:
  - `url: &str` - URL地址
  - `target_path: &Path` - 目标路径
- **返回值**: `Result<Vec<TaskId>>`
- **说明**: 从活跃任务和数据库任务中查找所有匹配的候选者

## Drop trait 实现

### drop()
- **位置**: src/manager/persistent_aria2.rs:590
- **功能**: 析构时的最后保存
- **说明**: 尽力保存所有任务到数据库（无法await，在后台执行）

## 核心特性

1. **自动任务恢复**: 启动时从数据库恢复未完成的任务
2. **定期进度保存**: 每5秒保存任务进度到数据库
3. **状态同步**: 每秒检查并保存任务状态变化
4. **重复检测**: 智能检测重复下载并根据策略处理
5. **优雅关闭**: 关闭时保存所有任务状态
6. **错误恢复**: 对恢复失败的任务标记为失败状态

## 依赖项

- `burncloud_download_aria2::Aria2DownloadManager` - Aria2下载引擎
- `burncloud_database_download::{DownloadRepository, Database}` - 数据库层
- `burncloud_download_types` - 核心类型定义
- `crate::models` - 重复检测模型
- `async_trait::async_trait` - 异步特征支持
- `anyhow::Result` - 错误处理
- `tokio` - 异步运行时