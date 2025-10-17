# Rust 下载功能

## 概述
基于 Aria2 的 Rust 下载系统，集成数据库存储。

## 架构
- `burncloud-download`: Rust 下载核心
- `burncloud-database-download`: 下载数据存储
- `burncloud-download-aria2`: Aria2 引擎实现

## 核心组件

### 下载管理器
```rust
pub struct DownloadManager {
    aria2_client: Aria2Client,
    database: DatabaseClient,
}

impl DownloadManager {
    pub async fn add_download(&self, url: &str) -> Result<String>;
    pub async fn get_status(&self, gid: &str) -> Result<DownloadStatus>;
    pub async fn pause(&self, gid: &str) -> Result<()>;
    pub async fn resume(&self, gid: &str) -> Result<()>;
    pub async fn remove(&self, gid: &str) -> Result<()>;
}
```

### 数据模型
```rust
#[derive(Serialize, Deserialize)]
pub struct DownloadTask {
    pub gid: String,
    pub url: String,
    pub status: DownloadStatus,
    pub progress: f64,
    pub total_length: u64,
    pub completed_length: u64,
    pub download_speed: u64,
}
```

## 使用示例

### 添加下载
```rust
let manager = DownloadManager::new().await?;
let gid = manager.add_download("https://example.com/file.zip").await?;
println!("下载任务 ID: {}", gid);
```

### 查询状态
```rust
let status = manager.get_status(&gid).await?;
println!("进度: {:.2}%", status.progress);
```

### 控制下载
```rust
manager.pause(&gid).await?;   // 暂停
manager.resume(&gid).await?;  // 恢复
manager.remove(&gid).await?;  // 移除
```

## 配置
```toml
[download]
aria2_host = "localhost"
aria2_port = 6800
max_concurrent = 5
download_dir = "./downloads"

[database]
url = "sqlite://downloads.db"
```

## 依赖
- `tokio`: 异步运行时
- `reqwest`: HTTP 客户端
- `serde`: 序列化
- `sqlx`: 数据库访问