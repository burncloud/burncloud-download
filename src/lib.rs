use burncloud_database_download::DownloadDB;
use burncloud_download_aria2::{quick_start, Aria2Manager};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("数据库错误: {0}")]
    Database(#[from] burncloud_database::DatabaseError),
    #[error("Aria2错误: {0}")]
    Aria2(#[from] burncloud_download_aria2::Aria2Error),
}

pub type Result<T> = std::result::Result<T, DownloadError>;

pub struct DownloadManager {
    aria2: Aria2Manager,
    db: DownloadDB,
}

impl DownloadManager {
    pub async fn new() -> Result<Self> {
        let aria2 = quick_start().await?;
        let db = DownloadDB::new().await?;
        let manager = Self { aria2, db };
        manager.restore_incomplete_downloads().await?;
        Ok(manager)
    }

    pub async fn add_download(&self, url: &str) -> Result<String> {
        let client = self.aria2.create_rpc_client().ok_or_else(||
            DownloadError::Aria2(burncloud_download_aria2::Aria2Error::RpcError("客户端未就绪".to_string())))?;

        let options = burncloud_download_aria2::DownloadOptions {
            dir: Some("./downloads".to_string()),
            out: None,
            split: None,
            max_connection_per_server: None,
            continue_download: Some(true),
        };

        let gid = client.add_uri(vec![url.to_string()], Some(options)).await?;
        self.db.add(&gid, vec![url.to_string()]).await?;
        Ok(gid)
    }

    pub async fn get_status(&self, gid: &str) -> Result<burncloud_download_aria2::DownloadStatus> {
        let client = self.aria2.create_rpc_client().ok_or_else(||
            DownloadError::Aria2(burncloud_download_aria2::Aria2Error::RpcError("客户端未就绪".to_string())))?;

        let status = client.tell_status(gid).await?;

        // 同步状态到数据库
        self.db.update_status(gid, &status.status).await?;
        let completed: i64 = status.completed_length.parse().unwrap_or(0);
        let speed: i64 = status.download_speed.parse().unwrap_or(0);
        self.db.update_progress(gid, completed, speed).await?;

        Ok(status)
    }

    pub async fn pause(&self, gid: &str) -> Result<()> {
        let client = self.aria2.create_rpc_client().ok_or_else(||
            DownloadError::Aria2(burncloud_download_aria2::Aria2Error::RpcError("客户端未就绪".to_string())))?;

        client.pause(gid).await?;
        self.db.update_status(gid, "paused").await?;
        Ok(())
    }

    pub async fn resume(&self, gid: &str) -> Result<()> {
        let client = self.aria2.create_rpc_client().ok_or_else(||
            DownloadError::Aria2(burncloud_download_aria2::Aria2Error::RpcError("客户端未就绪".to_string())))?;

        client.unpause(gid).await?;
        self.db.update_status(gid, "active").await?;
        Ok(())
    }

    pub async fn remove(&self, gid: &str) -> Result<()> {
        let client = self.aria2.create_rpc_client().ok_or_else(||
            DownloadError::Aria2(burncloud_download_aria2::Aria2Error::RpcError("客户端未就绪".to_string())))?;

        client.remove(gid).await?;
        self.db.delete(gid).await?;
        Ok(())
    }

    async fn restore_incomplete_downloads(&self) -> Result<Vec<String>> {
        let client = self.aria2.create_rpc_client().ok_or_else(||
            DownloadError::Aria2(burncloud_download_aria2::Aria2Error::RpcError("客户端未就绪".to_string())))?;

        let incomplete = self.db.list(Some("active")).await?;
        let mut restored = Vec::new();

        for download in incomplete {
            let uris: Vec<String> = serde_json::from_str(&download.uris).unwrap_or_default();
            if !uris.is_empty() {
                let options = burncloud_download_aria2::DownloadOptions {
                    dir: Some("./downloads".to_string()),
                    out: None,
                    split: None,
                    max_connection_per_server: None,
                    continue_download: Some(true),
                };
                if let Ok(gid) = client.add_uri(uris, Some(options)).await {
                    restored.push(gid);
                }
            }
        }

        Ok(restored)
    }
}