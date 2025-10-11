# manager/basic.rs - 基础下载管理器

## 结构体

### BasicDownloadManager
- **位置**: src/manager/basic.rs:18
- **说明**: 基础下载管理器实现，用于演示和测试目的，提供模拟下载功能

#### 字段
- `tasks: Arc<RwLock<HashMap<TaskId, DownloadTask>>>` - 按ID存储的所有任务
- `progress: Arc<RwLock<HashMap<TaskId, DownloadProgress>>>` - 任务进度跟踪
- `mock_data: Arc<RwLock<HashMap<TaskId, MockDownloadData>>>` - 模拟下载仿真数据

### MockDownloadData
- **位置**: src/manager/basic.rs:28
- **说明**: 用于模拟下载进度的数据结构

#### 字段
- `start_time: Instant` - 下载开始时间
- `total_size: u64` - 总文件大小
- `download_speed: u64` - 下载速度（字节/秒）

## 实现方法

### new()
- **位置**: src/manager/basic.rs:36
- **功能**: 创建新的基础下载管理器实例
- **返回值**: `Self`
- **说明**: 初始化所有内部HashMap和Arc包装器

### update_task_progress(task_id)
- **位置**: src/manager/basic.rs:45
- **功能**: 更新任务的进度信息（内部方法）
- **参数**: `task_id: TaskId` - 要更新的任务ID
- **返回值**: `Result<()>`
- **说明**: 根据模拟数据计算已下载字节数、剩余时间等进度信息

### start_mock_download(task_id)
- **位置**: src/manager/basic.rs:93
- **功能**: 为任务启动模拟下载仿真
- **参数**: `task_id: TaskId` - 任务ID
- **说明**: 创建模拟10MB文件以1MB/s速度下载的仿真数据

## DownloadManager trait 实现

### add_download(url, target_path)
- **位置**: src/manager/basic.rs:123
- **功能**: 添加新的下载任务
- **参数**:
  - `url: String` - 下载URL
  - `target_path: PathBuf` - 目标保存路径
- **返回值**: `Result<TaskId>`
- **说明**: 创建新任务并启动模拟下载

### pause_download(task_id)
- **位置**: src/manager/basic.rs:137
- **功能**: 暂停下载任务
- **参数**: `task_id: TaskId` - 任务ID
- **返回值**: `Result<()>`
- **说明**: 检查任务状态并更新为暂停状态，停止模拟仿真

### resume_download(task_id)
- **位置**: src/manager/basic.rs:154
- **功能**: 恢复暂停的下载任务
- **参数**: `task_id: TaskId` - 任务ID
- **返回值**: `Result<()>`
- **说明**: 检查任务状态并恢复下载，重新启动模拟仿真

### cancel_download(task_id)
- **位置**: src/manager/basic.rs:171
- **功能**: 取消下载任务
- **参数**: `task_id: TaskId` - 任务ID
- **返回值**: `Result<()>`
- **说明**: 从所有集合中移除任务相关数据

### get_progress(task_id)
- **位置**: src/manager/basic.rs:180
- **功能**: 获取任务的下载进度
- **参数**: `task_id: TaskId` - 任务ID
- **返回值**: `Result<DownloadProgress>`
- **说明**: 先更新进度再返回最新的进度信息

### get_task(task_id)
- **位置**: src/manager/basic.rs:190
- **功能**: 获取任务的详细信息
- **参数**: `task_id: TaskId` - 任务ID
- **返回值**: `Result<DownloadTask>`
- **说明**: 更新进度以确保任务状态是最新的

### list_tasks()
- **位置**: src/manager/basic.rs:200
- **功能**: 列出所有下载任务
- **返回值**: `Result<Vec<DownloadTask>>`
- **说明**: 返回所有任务的克隆副本

### active_download_count()
- **位置**: src/manager/basic.rs:205
- **功能**: 获取当前活跃下载任务数量
- **返回值**: `Result<usize>`
- **说明**: 统计处于活跃状态的任务数量

## 重复检测方法

### find_duplicate_task(url, target_path)
- **位置**: src/manager/basic.rs:215
- **功能**: 查找重复的下载任务
- **参数**:
  - `url: &str` - URL地址
  - `target_path: &Path` - 目标路径
- **返回值**: `Result<Option<TaskId>>`
- **说明**: 在内存中查找URL和路径完全匹配的任务

### add_download_with_policy(url, target_path, policy)
- **位置**: src/manager/basic.rs:234
- **功能**: 根据重复策略添加下载任务
- **参数**:
  - `url: &str` - URL地址
  - `target_path: &Path` - 目标路径
  - `policy: DuplicatePolicy` - 重复处理策略
- **返回值**: `Result<DuplicateResult>`
- **说明**: 检查重复后根据策略决定是否创建新任务或重用现有任务

### verify_task_validity(task_id)
- **位置**: src/manager/basic.rs:264
- **功能**: 验证任务的有效性
- **参数**: `task_id: &TaskId` - 任务ID引用
- **返回值**: `Result<bool>`
- **说明**: 对于基础管理器，只检查任务是否存在

### get_duplicate_candidates(url, target_path)
- **位置**: src/manager/basic.rs:271
- **功能**: 获取重复候选任务
- **参数**:
  - `url: &str` - URL地址
  - `target_path: &Path` - 目标路径
- **返回值**: `Result<Vec<TaskId>>`
- **说明**: 查找所有可能的重复任务候选者

## 特征实现

### Default
- **位置**: src/manager/basic.rs:115
- **说明**: 提供默认实例创建，调用new()方法

## 依赖项

- `std::collections::HashMap` - 哈希映射容器
- `std::path::{Path, PathBuf}` - 路径处理
- `std::sync::Arc` - 原子引用计数
- `tokio::sync::RwLock` - 异步读写锁
- `tokio::time::Instant` - 时间点
- `async_trait::async_trait` - 异步特征宏
- `anyhow::Result` - 错误处理