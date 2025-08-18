// Traits for persistence; sqlite implementation to follow

use crate::domain::models::Progress;

#[async_trait::async_trait]
pub trait ProgressRepo: Send + Sync {
    async fn get(&self, device_id: &str, book_id: &str) -> anyhow::Result<Option<Progress>>;
    async fn set(&self, progress: Progress) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
pub trait DeviceRepo: Send + Sync {
    async fn get_or_register(&self, fingerprint: &str) -> anyhow::Result<String>; // returns device_id
}
