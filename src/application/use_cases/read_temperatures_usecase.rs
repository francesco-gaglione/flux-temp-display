use std::sync::Arc;

use crate::domain::{
    entities::temperature::Temperature, ports::temperatures_port::TemperaturePort,
};

#[derive(Debug)]
pub struct TemperaturesDto {
    pub cpu: Option<Temperature>,
    pub gpu: Option<Temperature>,
    pub issues: TemperatureReadIssues,
}

#[derive(Debug, Default)]
pub struct TemperatureReadIssues {
    pub cpu: Option<String>,
    pub gpu: Option<String>,
}

pub struct ReadTemperaturesUseCase {
    temperature_port: Arc<dyn TemperaturePort>,
}

impl ReadTemperaturesUseCase {
    pub fn new(temperature_port: Arc<dyn TemperaturePort>) -> Self {
        Self { temperature_port }
    }

    pub async fn execute(&self) -> TemperaturesDto {
        let (cpu, cpu_error) = match self.temperature_port.read_cpu_temp().await {
            Ok(temperature) => (Some(temperature), None),
            Err(error) => (None, Some(error.to_string())),
        };
        let (gpu, gpu_error) = match self.temperature_port.read_gpu_temp().await {
            Ok(temperature) => (Some(temperature), None),
            Err(error) => (None, Some(error.to_string())),
        };

        TemperaturesDto {
            cpu,
            gpu,
            issues: TemperatureReadIssues {
                cpu: cpu_error,
                gpu: gpu_error,
            },
        }
    }
}
