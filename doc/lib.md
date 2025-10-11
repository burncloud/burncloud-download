# lib.rs - BurnCloud 下载管理器核心库

## 模块概述

BurnCloud Download Manager 是一个统一的下载管理接口，为 BurnCloud 平台提供下载功能。

## 核心功能

- 简单的下载 API，自动集成 aria2
- 持久化下载，支持应用重启后继续下载
- 进度监控和任务生命周期管理
- 默认存储到 `./data/` 目录，支持自定义路径
- 自动数据库持久化和恢复

## 公共函数

### get_global_manager()
- **位置**: src/lib.rs:105
- **功能**: 获取或初始化全局下载管理器
- **返回值**: `Result<std::sync::Arc<PersistentAria2Manager>>`
- **说明**: 使用单例模式确保全局只有一个管理器实例

### download(url)
- **位置**: src/lib.rs:138
- **功能**: 简单下载函数，将文件下载到默认的 ./data/ 目录
- **参数**:
  - `url: S` - 要下载的URL，其中 S 实现了 AsRef<str>
- **返回值**: `Result<TaskId>` - 下载任务的唯一标识符
- **说明**: 自动从URL中提取文件名，如果无法提取则使用"download"作为默认名

### download_to(url, target_path)
- **位置**: src/lib.rs:176
- **功能**: 下载文件到指定路径
- **参数**:
  - `url: S` - 要下载的URL
  - `target_path: P` - 目标保存路径
- **返回值**: `Result<TaskId>` - 下载任务的唯一标识符
- **说明**: 提供更精确的下载控制，可以指定具体的保存位置

### get_download_progress(task_id)
- **位置**: src/lib.rs:191
- **功能**: 获取下载任务的进度信息
- **参数**: `task_id: TaskId` - 下载任务的唯一标识符
- **返回值**: `Result<DownloadProgress>` - 当前进度信息
- **说明**: 用于监控下载进度，包含已下载字节数等信息

### get_download_task(task_id)
- **位置**: src/lib.rs:203
- **功能**: 获取下载任务的详细信息
- **参数**: `task_id: TaskId` - 下载任务的唯一标识符
- **返回值**: `Result<DownloadTask>` - 完整的任务信息，包括状态
- **说明**: 提供比进度信息更完整的任务状态数据

### pause_download(task_id)
- **位置**: src/lib.rs:212
- **功能**: 暂停下载任务
- **参数**: `task_id: TaskId` - 下载任务的唯一标识符
- **返回值**: `Result<()>`
- **说明**: 暂停指定的下载任务，可以稍后恢复

### resume_download(task_id)
- **位置**: src/lib.rs:221
- **功能**: 恢复暂停的下载任务
- **参数**: `task_id: TaskId` - 下载任务的唯一标识符
- **返回值**: `Result<()>`
- **说明**: 恢复之前暂停的下载任务

### cancel_download(task_id)
- **位置**: src/lib.rs:230
- **功能**: 取消下载任务
- **参数**: `task_id: TaskId` - 下载任务的唯一标识符
- **返回值**: `Result<()>`
- **说明**: 永久取消下载任务，无法恢复

### list_downloads()
- **位置**: src/lib.rs:239
- **功能**: 列出所有下载任务
- **返回值**: `Result<Vec<DownloadTask>>` - 所有下载任务的列表
- **说明**: 获取系统中所有下载任务的完整列表

### active_download_count()
- **位置**: src/lib.rs:248
- **功能**: 获取当前活跃下载任务数量
- **返回值**: `Result<usize>` - 活跃下载任务的数量
- **说明**: 用于监控系统负载，了解当前有多少个下载任务正在进行

## 类型定义

### Result<T>
- **位置**: src/lib.rs:95
- **定义**: `std::result::Result<T, anyhow::Error>`
- **说明**: 下载操作的结果类型别名，使用 anyhow 进行错误处理

## 全局变量

### GLOBAL_MANAGER
- **位置**: src/lib.rs:102
- **类型**: `OnceLock<Mutex<Option<std::sync::Arc<PersistentAria2Manager>>>>`
- **说明**: 全局管理器实例，用于便利函数的单例模式实现

## 重新导出

该模块重新导出了以下重要类型和特征：
- `DownloadTask`, `DownloadProgress`, `DownloadStatus`, `TaskId` (来自 burncloud-download-types)
- `DownloadManager`, `DownloadEventHandler` (特征)
- `TaskQueueManager`, `BasicDownloadManager`, `PersistentAria2Manager` (管理器实现)
- 重复检测相关类型和服务
- `DownloadError` (错误类型)