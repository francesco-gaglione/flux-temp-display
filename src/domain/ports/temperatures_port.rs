use async_trait::async_trait;
use thiserror::Error;

use crate::domain::entities::temperature::Temperature;

#[derive(Error, Debug)]
pub enum TemperaturePortError {
    #[error("Generic error: {0}")]
    GenericError(String),
}

#[async_trait]
pub trait TemperaturePort {
    async fn read_cpu_temp(&self) -> Result<Temperature, TemperaturePortError>;
    async fn read_gpu_temp(&self) -> Result<Temperature, TemperaturePortError>;
}
