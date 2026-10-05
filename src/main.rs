mod application;
mod domain;
mod infrastructure;

use std::{error::Error, sync::Arc, time::Duration};

use application::use_cases::{
    read_temperatures_usecase::{ReadTemperaturesUseCase, TemperaturesDto},
    update_display_usecase::UpdateDisplayUseCase,
};
use domain::{
    entities::temperature::Temperature,
    ports::{display_port::DisplayPort, temperatures_port::TemperaturePort},
};
use infrastructure::ports::{
    display_port_impl::DisplayPortImpl, temperatures_port_impl::TemperaturePortImpl,
};

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("flux_temp_display: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn Error>> {
    let temperature_port: Arc<dyn TemperaturePort> = Arc::new(TemperaturePortImpl {});
    let display_port: Arc<dyn DisplayPort> = Arc::new(DisplayPortImpl::new()?);
    let read_temperatures = ReadTemperaturesUseCase::new(temperature_port);
    let update_display = UpdateDisplayUseCase::new(display_port.clone());

    let mut readings = read_temperatures.execute().await;
    log_sensor_issues(&readings);
    update_display.execute(&readings).await?;

    // Read sensors every second and refresh the display twice per second
    // because its firmware does not retain a packet indefinitely.
    let now = tokio::time::Instant::now();
    let mut read_interval =
        tokio::time::interval_at(now + Duration::from_secs(1), Duration::from_secs(1));
    let mut display_interval =
        tokio::time::interval_at(now + Duration::from_millis(500), Duration::from_millis(500));
    let mut interrupt = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;

    loop {
        tokio::select! {
            _ = read_interval.tick() => {
                readings = read_temperatures.execute().await;
                log_sensor_issues(&readings);
                if let Err(error) = update_display.execute(&readings).await {
                    eprintln!("Unable to update display: {error}");
                }
            },
            _ = display_interval.tick() => {
                if let Err(error) = update_display.execute(&readings).await {
                    eprintln!("Unable to update display: {error}");
                }
            },
            _ = interrupt.recv() => break,
            _ = terminate.recv() => break,
        }
    }

    let zero = Temperature::new(0);
    display_port
        .display_temperatures(Some(&zero), Some(&zero))
        .await?;
    Ok(())
}

fn log_sensor_issues(readings: &TemperaturesDto) {
    if let Some(error) = &readings.issues.cpu {
        eprintln!("CPU temperature unavailable: {error}");
    }
    if let Some(error) = &readings.issues.gpu {
        eprintln!("GPU temperature unavailable: {error}");
    }
}
