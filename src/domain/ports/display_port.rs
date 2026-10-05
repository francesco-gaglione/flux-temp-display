use async_trait::async_trait;
use thiserror::Error;

use crate::domain::entities::temperature::Temperature;

#[derive(Error, Debug)]
pub enum DisplayPortError {
    #[error("Display communication error: {0}")]
    GenericError(String),
}

#[async_trait]
pub trait DisplayPort: Send + Sync {
    async fn display_temperatures(
        &self,
        cpu: Option<&Temperature>,
        gpu: Option<&Temperature>,
    ) -> Result<(), DisplayPortError>;
}
