use crate::domain::{
    entities::temperature::Temperature,
    ports::temperatures_port::{TemperaturePort, TemperaturePortError},
};
use async_trait::async_trait;
use std::{fs, path::Path};

type HwmonSensor = (String, String);
type HwmonDevice = (String, String, Vec<HwmonSensor>);

pub struct TemperaturePortImpl {}

impl TemperaturePortImpl {
    fn read_temperature(path: &Path) -> Result<Temperature, TemperaturePortError> {
        let value = fs::read_to_string(path).map_err(|error| {
            TemperaturePortError::GenericError(format!(
                "Unable to read temperature from {}: {error}",
                path.display()
            ))
        })?;

        let milli_degrees: i64 = value.trim().parse().map_err(|error| {
            TemperaturePortError::GenericError(format!(
                "Invalid temperature in {}: {error}",
                path.display()
            ))
        })?;
        let degrees = milli_degrees / 1000;
        let degrees = u16::try_from(degrees).map_err(|_| {
            TemperaturePortError::GenericError(format!(
                "Temperature out of range in {}: {degrees} °C",
                path.display()
            ))
        })?;

        Ok(Temperature::new(degrees))
    }

    fn hwmon_sensors() -> Vec<HwmonDevice> {
        let mut devices = Vec::new();
        let Ok(entries) = fs::read_dir("/sys/class/hwmon") else {
            return devices;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let name = fs::read_to_string(path.join("name")).unwrap_or_default();
            let mut sensors = Vec::new();
            let Ok(files) = fs::read_dir(&path) else {
                continue;
            };
            for file in files.flatten() {
                let input = file.path();
                let Some(filename) = input.file_name().and_then(|name| name.to_str()) else {
                    continue;
                };
                if !filename.starts_with("temp") || !filename.ends_with("_input") {
                    continue;
                }
                let label_path = input.with_file_name(filename.replace("_input", "_label"));
                let label = fs::read_to_string(label_path).unwrap_or_default();
                sensors.push((
                    label.trim().to_lowercase(),
                    input.to_string_lossy().into_owned(),
                ));
            }
            devices.push((
                path.to_string_lossy().into_owned(),
                name.trim().to_lowercase(),
                sensors,
            ));
        }
        devices
    }
}

#[async_trait]
impl TemperaturePort for TemperaturePortImpl {
    async fn read_cpu_temp(&self) -> Result<Temperature, TemperaturePortError> {
        let thermal_dir = Path::new("/sys/class/thermal");
        let entries = fs::read_dir(thermal_dir).map_err(|error| {
            TemperaturePortError::GenericError(format!("Unable to find CPU thermal zones: {error}"))
        })?;

        let mut zones = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if !name.starts_with("thermal_zone") {
                continue;
            }

            let zone_type = fs::read_to_string(path.join("type")).unwrap_or_default();
            let zone_type = zone_type.trim().to_lowercase();
            let priority = if ["x86_pkg_temp", "cpu", "cpu-thermal", "soc"]
                .iter()
                .any(|keyword| zone_type.contains(keyword))
            {
                0
            } else {
                1
            };
            zones.push((priority, path.join("temp")));
        }
        zones.sort_by_key(|(priority, _)| *priority);

        for (_, path) in zones {
            if let Ok(temperature) = Self::read_temperature(&path) {
                return Ok(temperature);
            }
        }

        // CPU temperature drivers commonly expose their readings through hwmon
        // instead of /sys/class/thermal (e.g. coretemp and k10temp).
        let mut cpu_sensors = Vec::new();
        for (_, name, sensors) in Self::hwmon_sensors() {
            if !["coretemp", "k10temp", "zenpower", "cpu"]
                .iter()
                .any(|driver| name.contains(driver))
            {
                continue;
            }
            for (label, path) in sensors {
                let priority = if label.contains("package") || label.contains("tdie") {
                    0
                } else if label.contains("tctl") {
                    1
                } else {
                    2
                };
                cpu_sensors.push((priority, path));
            }
        }
        cpu_sensors.sort_by_key(|(priority, _)| *priority);
        for (_, path) in cpu_sensors {
            if let Ok(temperature) = Self::read_temperature(Path::new(&path)) {
                return Ok(temperature);
            }
        }

        Err(TemperaturePortError::GenericError(
            "No readable CPU thermal zone found".to_string(),
        ))
    }

    async fn read_gpu_temp(&self) -> Result<Temperature, TemperaturePortError> {
        let devices = Self::hwmon_sensors();
        let mut gpu_sensors = Vec::new();
        for (device_path, name, sensors) in devices {
            if !name.contains("gpu") && !name.contains("nvidia") && !name.contains("amdgpu") {
                continue;
            }
            // On hybrid AMD systems each GPU has its own amdgpu hwmon device.
            // Prefer the one with the largest VRAM, which is usually the
            // discrete GPU rather than the integrated GPU.
            let vram_bytes =
                fs::read_to_string(Path::new(&device_path).join("device/mem_info_vram_total"))
                    .ok()
                    .and_then(|value| value.trim().parse::<u64>().ok())
                    .unwrap_or(0);
            for (label, path) in sensors {
                // Prefer the GPU edge/core reading; hotspot/junction sensors
                // are deliberately fallback-only because they run hotter.
                let priority = if label.contains("edge") || label.contains("gpu core") {
                    0
                } else if label.contains("hotspot") || label.contains("junction") {
                    2
                } else {
                    1
                };
                let number = Path::new(&path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .and_then(|name| name.strip_prefix("temp"))
                    .and_then(|name| name.strip_suffix("_input"))
                    .and_then(|number| number.parse::<u8>().ok())
                    .unwrap_or(u8::MAX);
                gpu_sensors.push((priority, std::cmp::Reverse(vram_bytes), number, path));
            }
        }

        gpu_sensors.sort_by_key(|(priority, vram, number, _)| (*priority, *vram, *number));
        for (_, _, _, path) in gpu_sensors {
            if let Ok(temperature) = Self::read_temperature(Path::new(&path)) {
                return Ok(temperature);
            }
        }

        Err(TemperaturePortError::GenericError(
            "No readable GPU temperature sensor found in /sys/class/hwmon".to_string(),
        ))
    }
}
