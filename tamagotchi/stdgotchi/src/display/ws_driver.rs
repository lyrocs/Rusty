use esp_idf_svc::hal::spi::{SpiDriver, SpiDeviceDriver, SpiDriverConfig, config::Config as SpiConfig};
use esp_idf_svc::hal::units::Hertz;
use sh8601_rs::{ControllerInterface, DMA_CHUNK_SIZE};

/// Waveshare AMOLED driver using ESP-IDF SPI
pub struct WaveshareAmoledDriver<'a> {
    spi: SpiDeviceDriver<'a, SpiDriver<'a>>,
}

impl<'a> WaveshareAmoledDriver<'a> {
    pub fn new(spi: SpiDeviceDriver<'a, SpiDriver<'a>>) -> Self {
        Self { spi }
    }
}

impl ControllerInterface for WaveshareAmoledDriver<'_> {
    type Error = esp_idf_svc::sys::EspError;

    fn send_command(&mut self, cmd: u8) -> Result<(), Self::Error> {
        // Command mode: send single byte
        self.spi.write(&[cmd])
    }

    fn send_data(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        // Data mode: send data bytes
        // For large transfers, split into chunks if needed
        if data.len() <= DMA_CHUNK_SIZE {
            self.spi.write(data)
        } else {
            for chunk in data.chunks(DMA_CHUNK_SIZE) {
                self.spi.write(chunk)?;
            }
            Ok(())
        }
    }

    fn send_data_repeated(&mut self, data: &[u8], repeat: usize) -> Result<(), Self::Error> {
        for _ in 0..repeat {
            self.send_data(data)?;
        }
        Ok(())
    }
}
