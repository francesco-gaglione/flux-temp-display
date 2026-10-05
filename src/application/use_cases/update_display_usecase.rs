use std::sync::Arc;

use thiserror::Error;

use crate::{
    application::use_cases::read_temperatures_usecase::TemperaturesDto,
    domain::ports::display_port::{DisplayPort, DisplayPortError},
};

#[derive(Error, Debug)]
pub enum UpdateDisplayError {
    #[error(transparent)]
    Display(#[from] DisplayPortError),
}

pub struct UpdateDisplayUseCase {
    display_port: Arc<dyn DisplayPort>,
}

impl UpdateDisplayUseCase {
    pub fn new(display_port: Arc<dyn DisplayPort>) -> Self {
        Self { display_port }
    }

    /// Sends cached readings to keep the display visible between sensor polls.
    pub async fn execute(&self, readings: &TemperaturesDto) -> Result<(), UpdateDisplayError> {
        self.display_port
            .display_temperatures(readings.cpu.as_ref(), readings.gpu.as_ref())
            .await?;

        Ok(())
    }
}
