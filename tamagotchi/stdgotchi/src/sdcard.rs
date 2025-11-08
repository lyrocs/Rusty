// SD Card Support Module
//!
//! Provides SD card initialization and file operations for ESP32-S3.
//! Uses embedded-sdmmc library over SPI.

use embedded_sdmmc::{TimeSource, Timestamp, VolumeIdx, VolumeManager};
use std::error::Error;

/// Trait for SD card operations (for type erasure in ECS)
pub trait SdCardOps {
    fn is_mounted(&self) -> bool;
    fn save_to_file(&mut self, filename: &str, data: &str) -> Result<(), Box<dyn Error>>;
    fn load_from_file(&mut self, filename: &str) -> Result<String, Box<dyn Error>>;
    fn file_exists(&mut self, filename: &str) -> bool;
}

/// Simple TimeSource implementation for SD card file timestamps
pub struct SimpleTimeSource;

impl TimeSource for SimpleTimeSource {
    fn get_timestamp(&self) -> Timestamp {
        // Use system time if available
        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(duration) => {
                let secs = duration.as_secs();
                // Convert to FAT timestamp format
                let year = ((secs / 31536000) + 1970) as u16;
                let year_since_1970 = (year - 1970) as u8;

                Timestamp {
                    year_since_1970,
                    zero_indexed_month: 0,
                    zero_indexed_day: 0,
                    hours: 0,
                    minutes: 0,
                    seconds: 0,
                }
            }
            Err(_) => Timestamp {
                year_since_1970: 55,   // 2025
                zero_indexed_month: 10, // November
                zero_indexed_day: 7,    // 8th
                hours: 12,
                minutes: 0,
                seconds: 0,
            },
        }
    }
}

/// SD card resource for ECS
pub struct SdCardResource<SPI, CS, D>
where
    SPI: embedded_hal::spi::SpiBus,
    CS: embedded_hal::digital::OutputPin,
    D: embedded_hal::delay::DelayNs,
{
    pub volume_mgr: VolumeManager<embedded_sdmmc::SdCard<embedded_hal_bus::spi::ExclusiveDevice<SPI, CS, D>, D>, SimpleTimeSource, 4, 4, 1>,
    pub mounted: bool,
}

impl<SPI, CS, D> SdCardOps for SdCardResource<SPI, CS, D>
where
    SPI: embedded_hal::spi::SpiBus,
    CS: embedded_hal::digital::OutputPin,
    D: embedded_hal::delay::DelayNs,
{
    fn is_mounted(&self) -> bool {
        self.mounted
    }

    fn save_to_file(&mut self, filename: &str, data: &str) -> Result<(), Box<dyn Error>> {
        use embedded_sdmmc::Mode;

        log::info!("Saving to {}", filename);

        // Open volume
        let mut volume = self.volume_mgr.open_volume(VolumeIdx(0))
            .map_err(|e| format!("Failed to open volume: {:?}", e))?;

        // Open root directory
        let mut root_dir = volume.open_root_dir()
            .map_err(|e| format!("Failed to open root directory: {:?}", e))?;

        // Create or truncate file
        let mut file = root_dir.open_file_in_dir(filename, Mode::ReadWriteCreateOrTruncate)
            .map_err(|e| format!("Failed to open file: {:?}", e))?;

        // Write data
        file.write(data.as_bytes())
            .map_err(|e| format!("Failed to write data: {:?}", e))?;

        log::info!("Saved {} bytes to {}", data.len(), filename);
        Ok(())
    }

    fn load_from_file(&mut self, filename: &str) -> Result<String, Box<dyn Error>> {
        use embedded_sdmmc::Mode;

        log::info!("Loading from {}", filename);

        // Open volume
        log::info!("Opening volume...");
        let mut volume = self.volume_mgr.open_volume(VolumeIdx(0))
            .map_err(|e| {
                log::error!("Failed to open volume: {:?}", e);
                format!("Failed to open volume: {:?}", e)
            })?;

        log::info!("Volume opened, opening root directory...");
        // Open root directory
        let mut root_dir = volume.open_root_dir()
            .map_err(|e| {
                log::error!("Failed to open root directory: {:?}", e);
                format!("Failed to open root directory: {:?}", e)
            })?;

        log::info!("Root directory opened, opening file {}...", filename);
        // Open file
        let mut file = root_dir.open_file_in_dir(filename, Mode::ReadOnly)
            .map_err(|e| {
                log::warn!("File {} not found or cannot be opened: {:?}", filename, e);
                format!("Failed to open file: {:?}", e)
            })?;

        log::info!("File opened, reading contents...");
        // Read file
        let mut buffer = Vec::new();
        let mut chunk = [0u8; 512];
        let mut chunks_read = 0;

        loop {
            match file.read(&mut chunk) {
                Ok(0) => {
                    log::info!("Reached end of file after {} chunks", chunks_read);
                    break;
                }
                Ok(n) => {
                    buffer.extend_from_slice(&chunk[..n]);
                    chunks_read += 1;
                    if chunks_read % 10 == 0 {
                        log::info!("Read {} chunks ({} bytes)...", chunks_read, buffer.len());
                    }
                }
                Err(e) => {
                    log::error!("Failed to read file after {} chunks: {:?}", chunks_read, e);
                    return Err(format!("Failed to read file: {:?}", e).into());
                }
            }
        }

        let data = String::from_utf8(buffer)
            .map_err(|e| format!("Invalid UTF-8: {:?}", e))?;

        log::info!("Loaded {} bytes from {}", data.len(), filename);
        Ok(data)
    }

    fn file_exists(&mut self, filename: &str) -> bool {
        use embedded_sdmmc::Mode;

        // Try to open the file - if successful, it exists
        match self.volume_mgr.open_volume(VolumeIdx(0)) {
            Ok(mut volume) => {
                match volume.open_root_dir() {
                    Ok(mut root_dir) => {
                        root_dir.open_file_in_dir(filename, Mode::ReadOnly).is_ok()
                    }
                    Err(_) => false,
                }
            }
            Err(_) => false,
        }
    }
}

impl<SPI, CS> SdCardResource<SPI, CS, FreeRtosDelay>
where
    SPI: embedded_hal::spi::SpiBus,
    CS: embedded_hal::digital::OutputPin,
{
    /// Create a new SD card resource with FreeRtos delay
    pub fn new(
        spi: SPI,
        cs: CS,
    ) -> Result<Self, Box<dyn Error>> {
        log::info!("Initializing SD card...");

        // Create SPI device with CS control
        use embedded_hal_bus::spi::ExclusiveDevice;
        let delay = FreeRtosDelay;
        let spi_device = ExclusiveDevice::new(spi, cs, FreeRtosDelay)
            .map_err(|_| "Failed to create SPI device")?;

        // Create SD card
        let sd_card = embedded_sdmmc::SdCard::new(spi_device, delay);

        // Create volume manager
        let time_source = SimpleTimeSource;
        let volume_mgr = VolumeManager::new(sd_card, time_source);

        log::info!("SD card initialized successfully");

        Ok(Self {
            volume_mgr,
            mounted: true,
        })
    }
}

/// Copy-able delay wrapper for FreeRtos
#[derive(Clone, Copy)]
pub struct FreeRtosDelay;

impl embedded_hal::delay::DelayNs for FreeRtosDelay {
    fn delay_ns(&mut self, ns: u32) {
        let ms = (ns / 1_000_000).max(1);
        esp_idf_svc::hal::delay::FreeRtos.delay_ms(ms);
    }
}

/// Get the default save file path
/// Using 8.3 filename format for FAT filesystem compatibility
pub fn get_save_path() -> &'static str {
    "SAVE.JSN"
}
