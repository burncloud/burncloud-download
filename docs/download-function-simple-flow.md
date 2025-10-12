# download(url) 简化逻辑流程图

## 核心调用链（简化版）

```
用户调用: download("https://example.com/file.zip")
    ↓
1. lib.rs::download() - URL处理和路径构建
    ↓
2. lib.rs::download_to() - 获取全局管理器
    ↓
3. lib.rs::get_global_manager() - 单例管理器
    ↓ (如果首次调用)
4. PersistentAria2Manager::new() - 初始化管理器
    ├─ 数据库初始化
    ├─ aria2客户端连接
    ├─ 任务恢复
    └─ 后台监控启动
    ↓
5. PersistentAria2Manager::add_download() - 添加下载
    ├─ 重复检测
    ├─ 创建目录
    ├─ aria2添加任务
    ├─ 数据库持久化
    └─ 映射存储
    ↓
6. 返回 TaskId 给用户
```

## 主要组件关系

```
用户代码
    ↓
[便利函数层] lib.rs
    ↓
[管理器层] PersistentAria2Manager
    ├─ [下载引擎] Aria2DownloadManager → aria2守护进程
    └─ [持久化层] DownloadRepository → SQLite数据库
```

## 关键函数映射表

| 步骤 | 函数名 | 文件位置 | 主要作用 |
|------|--------|----------|----------|
| 1 | `download()` | src/lib.rs:138 | URL处理，提取文件名 |
| 2 | `download_to()` | src/lib.rs:176 | 调用管理器 |
| 3 | `get_global_manager()` | src/lib.rs:105 | 获取/创建管理器实例 |
| 4 | `PersistentAria2Manager::new()` | src/manager/persistent_aria2.rs:64 | 初始化所有组件 |
| 5 | `add_download()` | src/manager/persistent_aria2.rs:349 | 添加下载任务 |
| 6 | `create_new_download()` | src/manager/persistent_aria2.rs:210 | 实际创建下载 |

## 数据流转

```
URL字符串 → 文件名提取 → 路径构建 → 管理器调用 → 重复检测 → aria2任务 → 数据库记录 → TaskId返回
```