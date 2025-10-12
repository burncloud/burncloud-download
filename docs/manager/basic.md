# manager/basic.rs - 基础下载管理器

## 结构体

### BasicDownloadManager
- **位置**: src/manager/basic.rs:17
- **说明**: 基础下载管理器实现，用于测试和最小功能实现，提供基本任务管理而不包含实际下载功能

#### 字段
- `tasks: Arc<RwLock<HashMap<TaskId, DownloadTask>>>` - 按ID存储的所有任务
- `progress: Arc<RwLock<HashMap<TaskId, DownloadProgress>>>` - 任务进度跟踪

## 实现方法

### new()
- **位置**: src/manager/basic.rs:25
- **功能**: 创建新的基础下载管理器实例
- **返回值**: `Self`
- **说明**: 初始化所有内部HashMap和Arc包装器

## DownloadManager trait 实现

### add_download(url, target_path)
- **位置**: src/manager/basic.rs:41
- **功能**: 添加新的下载任务
- **参数**:
  - `url: String` - 下载URL
  - `target_path: PathBuf` - 目标保存路径
- **返回值**: `Result<TaskId>`
- **说明**: 创建新任务并设置为等待状态，初始化基本进度信息

### pause_download(task_id)
- **位置**: src/manager/basic.rs:62
- **功能**: 暂停下载任务
- **参数**: `task_id: TaskId` - 任务ID
- **返回值**: `Result<()>`
- **说明**: 检查任务状态并更新为暂停状态

### resume_download(task_id)
- **位置**: src/manager/basic.rs:76
- **功能**: 恢复暂停的下载任务
- **参数**: `task_id: TaskId` - 任务ID
- **返回值**: `Result<()>`
- **说明**: 检查任务状态并恢复下载状态

### cancel_download(task_id)
- **位置**: src/manager/basic.rs:90
- **功能**: 取消下载任务
- **参数**: `task_id: TaskId` - 任务ID
- **返回值**: `Result<()>`
- **说明**: 从所有集合中移除任务相关数据

### get_progress(task_id)
- **位置**: src/manager/basic.rs:98
- **功能**: 获取任务的下载进度
- **参数**: `task_id: TaskId` - 任务ID
- **返回值**: `Result<DownloadProgress>`
- **说明**: 返回任务的当前进度信息

### get_task(task_id)
- **位置**: src/manager/basic.rs:105
- **功能**: 获取任务的详细信息
- **参数**: `task_id: TaskId` - 任务ID
- **返回值**: `Result<DownloadTask>`
- **说明**: 返回任务的详细信息

### list_tasks()
- **位置**: src/manager/basic.rs:112
- **功能**: 列出所有下载任务
- **返回值**: `Result<Vec<DownloadTask>>`
- **说明**: 返回所有任务的克隆副本

### active_download_count()
- **位置**: src/manager/basic.rs:117
- **功能**: 获取当前活跃下载任务数量
- **返回值**: `Result<usize>`
- **说明**: 统计处于活跃状态的任务数量

## 重复检测方法

### find_duplicate_task(url, target_path)
- **位置**: src/manager/basic.rs:127
- **功能**: 查找重复的下载任务
- **参数**:
  - `url: &str` - URL地址
  - `target_path: &Path` - 目标路径
- **返回值**: `Result<Option<TaskId>>`
- **说明**: 在内存中查找URL和路径完全匹配的任务

### add_download_with_policy(url, target_path, policy)
- **位置**: src/manager/basic.rs:146
- **功能**: 根据重复策略添加下载任务
- **参数**:
  - `url: &str` - URL地址
  - `target_path: &Path` - 目标路径
  - `policy: DuplicatePolicy` - 重复处理策略
- **返回值**: `Result<DuplicateResult>`
- **说明**: 检查重复后根据策略决定是否创建新任务或重用现有任务

### verify_task_validity(task_id)
- **位置**: src/manager/basic.rs:176
- **功能**: 验证任务的有效性
- **参数**: `task_id: &TaskId` - 任务ID引用
- **返回值**: `Result<bool>`
- **说明**: 对于基础管理器，只检查任务是否存在

### get_duplicate_candidates(url, target_path)
- **位置**: src/manager/basic.rs:183
- **功能**: 获取重复候选任务
- **参数**:
  - `url: &str` - URL地址
  - `target_path: &Path` - 目标路径
- **返回值**: `Result<Vec<TaskId>>`
- **说明**: 查找所有可能的重复任务候选者

## 特征实现

### Default
- **位置**: src/manager/basic.rs:33
- **说明**: 提供默认实例创建，调用new()方法

## 依赖项

- `std::collections::HashMap` - 哈希映射容器
- `std::path::{Path, PathBuf}` - 路径处理
- `std::sync::Arc` - 原子引用计数
- `tokio::sync::RwLock` - 异步读写锁
- `async_trait::async_trait` - 异步特征宏
- `anyhow::Result` - 错误处理

## 重要说明

BasicDownloadManager 是一个最小化的实现，主要用于：
- 测试和开发时的基础功能验证
- 作为其他下载管理器的参考实现
- 提供基本的任务状态管理

**注意**: 此管理器不执行实际的下载操作。生产环境请使用 `PersistentAria2Manager`。