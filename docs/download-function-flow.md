# download(url) 函数调用逻辑图

## 概述

本文档详细描述了通过 `download(url)` 函数调用整个程序的完整逻辑流程，包括所有涉及的函数、模块和依赖关系。

## 1. 入口函数调用链

### 1.1 用户调用
```rust
// 用户代码
let task_id = download("https://example.com/file.zip").await?;
```

### 1.2 函数调用流程图

```
用户调用
    ↓
download(url) [src/lib.rs:138]
    ↓ [URL处理]
    │ 1. url.as_ref() - 获取字符串引用
    │ 2. url_str.split('/').next_back() - 从URL提取文件名
    │ 3. PathBuf::from("./data").join(filename) - 构建目标路径
    ↓
download_to(url_str, target_path) [src/lib.rs:176]
    ↓ [获取全局管理器]
    │ 1. get_global_manager().await?
    │     ↓
    │     GLOBAL_MANAGER.get_or_init() [src/lib.rs:106]
    │         ↓ [如果未初始化]
    │         PersistentAria2Manager::new().await? [src/manager/persistent_aria2.rs:64]
    │             ↓ [调用详细配置版本]
    │             PersistentAria2Manager::new_with_config() [src/manager/persistent_aria2.rs:73]
    │                 ↓ [数据库初始化]
    │                 │ Database::new_default_initialized().await [burncloud-database-download]
    │                 │ DownloadRepository::new(db) [burncloud-database-download]
    │                 │ repository.initialize().await [数据库表结构]
    │                 ↓ [Aria2初始化]
    │                 │ Aria2DownloadManager::new(rpc_url, secret).await [burncloud-download-aria2]
    │                 ↓ [任务恢复]
    │                 │ restore_tasks().await [src/manager/persistent_aria2.rs:121]
    │                 │     ↓
    │                 │     repository.list_tasks().await [查询数据库中的所有任务]
    │                 │     ↓ [对每个未完成的任务]
    │                 │     restore_single_task(&task).await [src/manager/persistent_aria2.rs:163]
    │                 │         ↓
    │                 │         Aria2DownloadManager::add_download() [重新添加到aria2]
    │                 │         get_gid_for_task() [获取aria2 GID]
    │                 │         store_task_mapping() [存储任务映射]
    │                 ↓ [启动后台持久化]
    │                 start_persistence_poller().await [src/manager/persistent_aria2.rs:241]
    │                     ↓ [后台任务]
    │                     tokio::spawn() [启动定时器]
    │                         ↓ [每秒循环]
    │                         interval(Duration::from_secs(1))
    │                             ↓
    │                             Aria2DownloadManager::get_task() [检查任务状态]
    │                             repository.save_task() [保存任务状态]
    │                             ↓ [每5秒]
    │                             Aria2DownloadManager::get_progress() [获取进度]
    │                             repository.save_progress() [保存进度]
    ↓ [使用管理器添加下载]
    │ manager.add_download(url, target_path).await
    │     ↓
    │     PersistentAria2Manager::add_download() [src/manager/persistent_aria2.rs:349]
    │         ↓ [重复检测]
    │         add_download_with_policy(&url, &target_path, DuplicatePolicy::default()) [src/manager/persistent_aria2.rs:479]
    │             ↓
    │             find_duplicate_task(url, target_path).await [src/manager/persistent_aria2.rs:444]
    │                 ↓ [检查活跃任务]
    │                 │ Aria2DownloadManager::list_tasks().await [检查aria2中的任务]
    │                 │ ↓ [检查URL和路径匹配]
    │                 │ task.url == url && task.target_path == target_path
    │                 ↓ [检查数据库任务]
    │                 │ repository.list_tasks().await [检查数据库中的所有任务]
    │                 │ ↓ [检查URL和路径匹配]
    │                 │ task.url == url && task.target_path == target_path
    │                 ↓
    │                 return Option<TaskId> [返回重复任务ID或None]
    │             ↓ [根据重复检测结果]
    │             match duplicate_result:
    │                 DuplicateResult::ExistingTask => return existing_task_id
    │                 _ => create_new_download(url, target_path) [src/manager/persistent_aria2.rs:210]
    │                     ↓ [创建目标目录]
    │                     │ tokio::fs::create_dir_all(parent).await
    │                     ↓ [添加到aria2]
    │                     │ Aria2DownloadManager::add_download(&*self.aria2, url, target_path).await
    │                     │     ↓ [burncloud-download-aria2 内部流程]
    │                     │     aria2_client.add_uri() [RPC调用到aria2守护进程]
    │                     │     ↓
    │                     │     aria2守护进程接收任务并开始下载
    │                     ↓ [获取任务信息]
    │                     │ Aria2DownloadManager::get_task(&*self.aria2, task_id).await
    │                     ↓ [保存到数据库]
    │                     │ repository.save_task(&task).await
    │                     │     ↓
    │                     │     SQLite INSERT/UPDATE [数据库持久化]
    │                     ↓ [存储GID映射]
    │                     │ get_gid_for_task(task_id).await [src/manager/persistent_aria2.rs:182]
    │                     │ store_task_mapping(task_id, gid).await [src/manager/persistent_aria2.rs:195]
    │                     ↓
    │                     return TaskId [返回任务ID]
    ↓
    return TaskId [返回给用户]
```

## 2. 主要组件和模块

### 2.1 核心模块

| 模块 | 位置 | 作用 |
|------|------|------|
| `lib.rs` | src/lib.rs | 提供便利函数API |
| `PersistentAria2Manager` | src/manager/persistent_aria2.rs | 主要下载管理器 |
| `Aria2DownloadManager` | burncloud-download-aria2 | aria2 RPC客户端封装 |
| `DownloadRepository` | burncloud-database-download | 数据库持久化层 |
| `Database` | burncloud-database-download | SQLite数据库封装 |

### 2.2 关键数据结构

| 类型 | 定义位置 | 用途 |
|------|----------|------|
| `TaskId` | burncloud-download-types | 唯一任务标识符 |
| `DownloadTask` | burncloud-download-types | 下载任务信息 |
| `DownloadProgress` | burncloud-download-types | 下载进度信息 |
| `DownloadStatus` | burncloud-download-types | 任务状态枚举 |

## 3. 详细函数调用说明

### 3.1 download() 函数 [src/lib.rs:138]

**功能**: 简化的下载函数，自动提取文件名并下载到 `./data/` 目录

**内部处理**:
1. 转换 URL 参数为字符串引用
2. 从 URL 中提取文件名（`url.split('/').next_back()`）
3. 构建目标路径（`./data/文件名`）
4. 调用 `download_to()`

### 3.2 download_to() 函数 [src/lib.rs:176]

**功能**: 指定目标路径的下载函数

**内部处理**:
1. 获取全局管理器实例
2. 调用管理器的 `add_download()` 方法

### 3.3 get_global_manager() 函数 [src/lib.rs:105]

**功能**: 获取或初始化全局 PersistentAria2Manager 实例

**内部处理**:
1. 使用 `OnceLock` 确保单例模式
2. 如果未初始化，创建新的 `PersistentAria2Manager`
3. 返回 Arc 包装的管理器实例

### 3.4 PersistentAria2Manager::new() [src/manager/persistent_aria2.rs:64]

**功能**: 创建持久化aria2管理器

**初始化流程**:
1. **数据库初始化**:
   - 创建 SQLite 数据库连接
   - 初始化表结构（download_tasks, download_progress等）

2. **Aria2客户端初始化**:
   - 连接到 aria2 RPC 服务（默认 localhost:6800）
   - 验证连接和认证

3. **任务恢复**:
   - 从数据库查询未完成的任务
   - 重新添加到 aria2 守护进程
   - 恢复暂停状态

4. **后台服务启动**:
   - 启动定时器（每秒检查状态，每5秒保存进度）
   - 监听关闭信号

### 3.5 PersistentAria2Manager::add_download() [src/manager/persistent_aria2.rs:349]

**功能**: 添加新下载任务（带重复检测）

**处理流程**:
1. **重复检测**:
   - 检查 aria2 中的活跃任务
   - 检查数据库中的所有任务
   - 根据策略决定是否重用现有任务

2. **创建新任务**:
   - 确保目标目录存在
   - 调用 aria2 API 添加下载
   - 保存任务信息到数据库
   - 存储 TaskId 到 GID 的映射

### 3.6 后台持久化循环 [src/manager/persistent_aria2.rs:248]

**功能**: 定期保存任务状态和进度

**执行逻辑**:
```rust
loop {
    // 每秒执行
    for task_id in active_tasks {
        // 获取当前任务状态
        current_task = aria2.get_task(task_id)
        // 保存状态变化
        repository.save_task(current_task)

        // 每5秒保存进度
        if poll_count % 5 == 0 {
            progress = aria2.get_progress(task_id)
            repository.save_progress(task_id, progress)
        }
    }
}
```

## 4. 依赖关系图

```
用户代码
    ↓
burncloud-download (lib.rs)
    ↓
PersistentAria2Manager
    ├─→ burncloud-download-aria2 (Aria2DownloadManager)
    │    ↓
    │    aria2 守护进程 (外部进程)
    │
    └─→ burncloud-database-download (DownloadRepository)
         ↓
         SQLite 数据库文件
```

## 5. 错误处理流程

### 5.1 初始化失败
- 数据库连接失败 → 返回错误给用户
- aria2连接失败 → 返回错误给用户
- 任务恢复失败 → 标记任务为失败状态

### 5.2 下载过程错误
- aria2添加任务失败 → 返回错误
- 数据库保存失败 → 记录日志但不阻止下载
- 网络错误 → 由aria2处理重试逻辑

## 6. 性能考虑

### 6.1 并发处理
- 全局管理器使用 Arc + RwLock 支持多线程访问
- 后台持久化使用独立的 tokio 任务
- 数据库操作使用连接池

### 6.2 内存管理
- 任务映射表只存储活跃任务
- 完成的任务会从内存中清理
- 使用 Arc 避免大对象复制

## 7. 配置参数

| 参数 | 默认值 | 说明 |
|------|--------|------|
| ARIA2_RPC_URL | http://localhost:6800/jsonrpc | aria2 RPC地址 |
| ARIA2_RPC_SECRET | "burncloud" | aria2 RPC密钥 |
| PROGRESS_SAVE_INTERVAL_SECS | 5 | 进度保存间隔（秒） |
| STATUS_POLL_INTERVAL_SECS | 1 | 状态检查间隔（秒） |

这个完整的调用流程图展示了从用户调用 `download(url)` 到最终下载开始的所有环节，包括初始化、重复检测、持久化和后台监控等关键步骤。