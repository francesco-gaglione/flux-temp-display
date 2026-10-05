use std::time::Duration;

use async_trait::async_trait;
use rusb::{Direction, GlobalContext, TransferType};

use crate::domain::{
    entities::temperature::Temperature,
    ports::display_port::{DisplayPort, DisplayPortError},
};

const VENDOR_ID: u16 = 0x2022;
const PRODUCT_ID: u16 = 0x0522;
const UNKNOWN: u8 = 0xEE;

pub struct DisplayPortImpl {
    handle: rusb::DeviceHandle<GlobalContext>,
    endpoint: u8,
}

impl DisplayPortImpl {
    pub fn new() -> Result<Self, DisplayPortError> {
        let handle = rusb::open_device_with_vid_pid(VENDOR_ID, PRODUCT_ID).ok_or_else(|| {
            DisplayPortError::GenericError(format!(
                "USB display {VENDOR_ID:04x}:{PRODUCT_ID:04x} not found or permission denied"
            ))
        })?;

        if handle.kernel_driver_active(0).unwrap_or(false) {
            handle.detach_kernel_driver(0).map_err(|error| {
                DisplayPortError::GenericError(format!(
                    "Unable to detach USB kernel driver: {error}"
                ))
            })?;
        }
        handle.claim_interface(0).map_err(|error| {
            DisplayPortError::GenericError(format!("Unable to claim USB interface 0: {error}"))
        })?;

        let endpoint = handle
            .device()
            .config_descriptor(0)
            .map_err(|error| {
                DisplayPortError::GenericError(format!("Unable to inspect USB display: {error}"))
            })?
            .interfaces()
            .flat_map(|interface| interface.descriptors())
            .flat_map(|descriptor| descriptor.endpoint_descriptors())
            .find(|endpoint| {
                endpoint.transfer_type() == TransferType::Interrupt
                    && endpoint.direction() == Direction::Out
            })
            .map(|endpoint| endpoint.address())
            .unwrap_or(0x03);

        Ok(Self { handle, endpoint })
    }

    fn encode_temperature(temperature: Option<&Temperature>) -> [u8; 3] {
        match temperature {
            Some(temperature) => {
                let value = temperature.temperature().min(99);
                [(value / 10) as u8, (value % 10) as u8, 0]
            }
            None => [UNKNOWN; 3],
        }
    }

    fn packet(cpu: Option<&Temperature>, gpu: Option<&Temperature>) -> [u8; 12] {
        let mut packet = [0x55, 0xAA, 0x01, 0x01, 0x06, 0, 0, 0, 0, 0, 0, 0];
        packet[5..8].copy_from_slice(&Self::encode_temperature(cpu));
        packet[8..11].copy_from_slice(&Self::encode_temperature(gpu));
        packet[11] = packet[..11]
            .iter()
            .fold(0u8, |checksum, byte| checksum.wrapping_add(*byte));
        packet
    }
}

#[async_trait]
impl DisplayPort for DisplayPortImpl {
    async fn display_temperatures(
        &self,
        cpu: Option<&Temperature>,
        gpu: Option<&Temperature>,
    ) -> Result<(), DisplayPortError> {
        let packet = Self::packet(cpu, gpu);
        self.handle
            .write_interrupt(self.endpoint, &packet, Duration::from_secs(1))
            .map(|_| ())
            .map_err(|error| {
                DisplayPortError::GenericError(format!("USB display write failed: {error}"))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::DisplayPortImpl;
    use crate::domain::entities::temperature::Temperature;

    #[test]
    fn encodes_packet_and_checksum() {
        let cpu = Temperature::new(42);
        let gpu = Temperature::new(30);
        let packet = DisplayPortImpl::packet(Some(&cpu), Some(&gpu));
        assert_eq!(&packet[..11], &[0x55, 0xAA, 1, 1, 6, 4, 2, 0, 3, 0, 0]);
        assert_eq!(
            packet[11],
            packet[..11]
                .iter()
                .fold(0u8, |sum, byte| sum.wrapping_add(*byte))
        );
    }

    #[test]
    fn marks_unavailable_temperature() {
        let packet = DisplayPortImpl::packet(None, None);
        assert_eq!(&packet[5..11], &[0xEE; 6]);
    }
}
