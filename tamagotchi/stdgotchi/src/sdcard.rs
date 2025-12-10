// SD Card Support Module
//!
//! Provides SD card initialization and file operations for ESP32-C6.
//! Uses embedded-sdmmc library over SPI.

use embedded_hal::delay::DelayNs;
use embedded_sdmmc::{TimeSource, Timestamp, VolumeIdx, VolumeManager};
use std::error::Error;

/// Trait for SD card operations (for type erasure in ECS)
pub trait SdCardOps {
    fn is_mounted(&self) -> bool;
    fn save_to_file(&mut self, filename: &str, data: &str) -> Result<(), Box<dyn Error>>;
    fn load_from_file(&mut self, filename: &str) -> Result<String, Box<dyn Error>>;
    fn load_binary_file(&mut self, filename: &str) -> Result<Vec<u8>, Box<dyn Error>>;
    fn load_binary_range(&mut self, filename: &str, offset: u32, length: usize) -> Result<Vec<u8>, Box<dyn Error>>;
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

/// Copy-able delay wrapper for FreeRtos
#[derive(Clone, Copy)]
pub struct FreeRtosDelay;

impl embedded_hal::delay::DelayNs for FreeRtosDelay {
    fn delay_ns(&mut self, ns: u32) {
        // Use microsecond precision instead of rounding to milliseconds
        // This is critical for SD card performance
        if ns >= 1_000_000 {
            // 1ms or more - use millisecond delay
            let ms = ns / 1_000_000;
            esp_idf_svc::hal::delay::FreeRtos.delay_ms(ms);
        } else if ns >= 1_000 {
            // 1us or more - use microsecond delay
            let us = ns / 1_000;
            esp_idf_svc::hal::delay::FreeRtos.delay_us(us);
        } else if ns > 0 {
            // Less than 1us - use minimum 1us delay
            esp_idf_svc::hal::delay::FreeRtos.delay_us(1);
        }
        // If ns == 0, no delay needed
    }
}

/// SD card resource with concrete type for MutexDevice
pub struct SdCardResource<DEV>
where
    DEV: embedded_hal::spi::SpiDevice,
{
    pub volume_mgr: VolumeManager<embedded_sdmmc::SdCard<DEV, FreeRtosDelay>, SimpleTimeSource, 4, 4, 1>,
    pub mounted: bool,
}

impl<DEV> SdCardResource<DEV>
where
    DEV: embedded_hal::spi::SpiDevice<Error: std::fmt::Debug>,
{
    /// Create a new SD card resource from an existing SPI device
    pub fn new(mut spi_device: DEV) -> Result<Self, Box<dyn Error>> {
        log::info!("Initializing SD card...");

        // SD card power-on sequence:
        // 1. Wait at least 1ms after power stabilization
        // 2. Send at least 74 clock cycles with CS HIGH (deselected)
        // 3. Then the card is ready for CMD0

        // Wait for power stabilization
        FreeRtosDelay.delay_ms(10);

        // Send dummy bytes to generate clock cycles
        // The SPI device will keep CS high between transactions when we're not in a transaction
        // So we do a transaction with dummy data to generate clocks
        log::info!("Sending SD card wake-up clocks...");
        let dummy = [0xFF; 10]; // 80 clock cycles (10 bytes * 8 bits)
        let _ = spi_device.write(&dummy); // Ignore errors, card might not respond yet

        // Small delay after wake-up sequence
        FreeRtosDelay.delay_ms(10);

        let delay = FreeRtosDelay;

        // Create SD card with the provided SPI device
        let sd_card = embedded_sdmmc::SdCard::new(spi_device, delay);

        // Try to get card size to verify the card is present
        // This triggers the initialization sequence
        match sd_card.num_bytes() {
            Ok(size) => {
                log::info!("SD card detected: {} MB", size / 1024 / 1024);
            }
            Err(e) => {
                log::error!("SD card not responding: {:?}", e);
                return Err(format!("SD card error: {:?}", e).into());
            }
        }

        // Create volume manager
        let time_source = SimpleTimeSource;
        let volume_mgr = VolumeManager::new(sd_card, time_source);

        log::info!("SD card ready");

        Ok(Self {
            volume_mgr,
            mounted: true,
        })
    }
}

impl<DEV> SdCardOps for SdCardResource<DEV>
where
    DEV: embedded_hal::spi::SpiDevice<Error: std::fmt::Debug>,
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

    fn load_binary_file(&mut self, filename: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        use embedded_sdmmc::Mode;

        log::info!("Loading binary file from {}", filename);

        // Open volume
        let mut volume = self.volume_mgr.open_volume(VolumeIdx(0))
            .map_err(|e| {
                log::error!("Failed to open volume: {:?}", e);
                format!("Failed to open volume: {:?}", e)
            })?;

        // Open root directory
        let mut root_dir = volume.open_root_dir()
            .map_err(|e| {
                log::error!("Failed to open root directory: {:?}", e);
                format!("Failed to open root directory: {:?}", e)
            })?;

        // Parse path - remove leading slash and split into parts
        let path = filename.trim_start_matches('/');
        let parts: Vec<&str> = path.split('/').collect();

        if parts.is_empty() {
            return Err("Empty filename".into());
        }

        log::info!("Path parts: {:?}", parts);

        // Helper function to read file in chunks
        let read_file_chunks = |file: &mut embedded_sdmmc::File<_, _, _, _, _>| -> Result<Vec<u8>, Box<dyn Error>> {
            let mut buffer = Vec::new();
            let mut chunk = [0u8; 2048];
            let mut chunks_read = 0;

            loop {
                match file.read(&mut chunk) {
                    Ok(0) => {
                        log::info!("Loaded {} bytes from binary file {}", buffer.len(), filename);
                        break;
                    }
                    Ok(n) => {
                        buffer.extend_from_slice(&chunk[..n]);
                        chunks_read += 1;
                        if chunks_read % 10 == 0 {
                            log::debug!("Read {} chunks ({} bytes)...", chunks_read, buffer.len());
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to read binary file after {} chunks: {:?}", chunks_read, e);
                        return Err(format!("Failed to read file: {:?}", e).into());
                    }
                }
            }
            Ok(buffer)
        };

        // Handle different directory depths and read file immediately
        let buffer = match parts.len() {
            1 => {
                // File in root directory
                let file_name = parts[0];
                log::info!("Opening file in root: {}", file_name);
                let mut file = root_dir.open_file_in_dir(file_name, Mode::ReadOnly)
                    .map_err(|e| {
                        log::warn!("File {} not found: {:?}", file_name, e);
                        format!("Failed to open file {}: {:?}", file_name, e)
                    })?;
                read_file_chunks(&mut file)?
            }
            2 => {
                // File in one subdirectory (e.g., /DIR/FILE.GIF)
                let dir_name = parts[0];
                let file_name = parts[1];
                log::info!("Opening directory: {} for file: {}", dir_name, file_name);
                let mut dir = root_dir.open_dir(dir_name)
                    .map_err(|e| {
                        log::error!("Failed to open directory {}: {:?}", dir_name, e);
                        format!("Failed to open directory {}: {:?}", dir_name, e)
                    })?;
                let mut file = dir.open_file_in_dir(file_name, Mode::ReadOnly)
                    .map_err(|e| {
                        log::warn!("File {} not found in {}: {:?}", file_name, dir_name, e);
                        format!("Failed to open file {}: {:?}", file_name, e)
                    })?;
                read_file_chunks(&mut file)?
            }
            3 => {
                // File in two subdirectories (e.g., /SPRITES/ENEMY/FILE.GIF)
                let dir1_name = parts[0];
                let dir2_name = parts[1];
                let file_name = parts[2];
                log::info!("Opening path: {}/{}/{}", dir1_name, dir2_name, file_name);

                let mut dir1 = root_dir.open_dir(dir1_name)
                    .map_err(|e| {
                        log::error!("Failed to open directory {}: {:?}", dir1_name, e);
                        format!("Failed to open directory {}: {:?}", dir1_name, e)
                    })?;

                let mut dir2 = dir1.open_dir(dir2_name)
                    .map_err(|e| {
                        log::error!("Failed to open directory {}: {:?}", dir2_name, e);
                        format!("Failed to open directory {}: {:?}", dir2_name, e)
                    })?;

                let mut file = dir2.open_file_in_dir(file_name, Mode::ReadOnly)
                    .map_err(|e| {
                        log::warn!("File {} not found in {}/{}: {:?}", file_name, dir1_name, dir2_name, e);
                        format!("Failed to open file {}: {:?}", file_name, e)
                    })?;
                read_file_chunks(&mut file)?
            }
            _ => {
                return Err(format!("Path too deep (max 2 subdirectories): {}", filename).into());
            }
        };

        Ok(buffer)
    }

    fn load_binary_range(&mut self, filename: &str, offset: u32, length: usize) -> Result<Vec<u8>, Box<dyn Error>> {
        use embedded_sdmmc::Mode;

        log::info!("Loading binary range from {} (offset: {}, length: {})", filename, offset, length);

        // Open volume
        let mut volume = self.volume_mgr.open_volume(VolumeIdx(0))
            .map_err(|e| {
                log::error!("Failed to open volume: {:?}", e);
                format!("Failed to open volume: {:?}", e)
            })?;

        // Open root directory
        let mut root_dir = volume.open_root_dir()
            .map_err(|e| {
                log::error!("Failed to open root directory: {:?}", e);
                format!("Failed to open root directory: {:?}", e)
            })?;

        // Parse path - remove leading slash and split into parts
        let path = filename.trim_start_matches('/');
        let parts: Vec<&str> = path.split('/').collect();

        if parts.is_empty() {
            return Err("Empty filename".into());
        }

        // Helper function to read range from file
        let read_file_range = |file: &mut embedded_sdmmc::File<_, _, _, _, _>| -> Result<Vec<u8>, Box<dyn Error>> {
            // Seek to offset
            file.seek_from_start(offset)
                .map_err(|e| format!("Failed to seek to offset {}: {:?}", offset, e))?;

            // Read exactly 'length' bytes (or less if EOF)
            let mut buffer = vec![0u8; length];
            let mut total_read = 0;

            while total_read < length {
                match file.read(&mut buffer[total_read..]) {
                    Ok(0) => {
                        // EOF reached - truncate buffer and return
                        buffer.truncate(total_read);
                        log::info!("Read {} bytes from {} (EOF reached)", total_read, filename);
                        return Ok(buffer);
                    }
                    Ok(n) => {
                        total_read += n;
                    }
                    Err(e) => {
                        log::error!("Failed to read range after {} bytes: {:?}", total_read, e);
                        return Err(format!("Failed to read file: {:?}", e).into());
                    }
                }
            }

            log::info!("Read {} bytes from {}", total_read, filename);
            Ok(buffer)
        };

        // Handle different directory depths
        let buffer = match parts.len() {
            1 => {
                // File in root
                log::info!("Opening file in root: {}", parts[0]);
                let mut file = root_dir.open_file_in_dir(parts[0], Mode::ReadOnly)
                    .map_err(|e| {
                        log::error!("Failed to open file {}: {:?}", parts[0], e);
                        format!("Failed to open file: {:?}", e)
                    })?;
                read_file_range(&mut file)?
            }
            2 => {
                // File in subdirectory
                log::info!("Opening directory: {}", parts[0]);
                let mut dir = root_dir.open_dir(parts[0])
                    .map_err(|e| {
                        log::error!("Failed to open directory {}: {:?}", parts[0], e);
                        format!("Failed to open directory: {:?}", e)
                    })?;

                log::info!("Opening file in directory: {}", parts[1]);
                let mut file = dir.open_file_in_dir(parts[1], Mode::ReadOnly)
                    .map_err(|e| {
                        log::error!("Failed to open file {}: {:?}", parts[1], e);
                        format!("Failed to open file: {:?}", e)
                    })?;
                read_file_range(&mut file)?
            }
            3 => {
                // File in nested subdirectory
                log::info!("Opening directory: {}", parts[0]);
                let mut dir1 = root_dir.open_dir(parts[0])
                    .map_err(|e| {
                        log::error!("Failed to open directory {}: {:?}", parts[0], e);
                        format!("Failed to open directory: {:?}", e)
                    })?;

                log::info!("Opening subdirectory: {}", parts[1]);
                let mut dir2 = dir1.open_dir(parts[1])
                    .map_err(|e| {
                        log::error!("Failed to open subdirectory {}: {:?}", parts[1], e);
                        format!("Failed to open subdirectory: {:?}", e)
                    })?;

                log::info!("Opening file in subdirectory: {}", parts[2]);
                let mut file = dir2.open_file_in_dir(parts[2], Mode::ReadOnly)
                    .map_err(|e| {
                        log::error!("Failed to open file {}: {:?}", parts[2], e);
                        format!("Failed to open file: {:?}", e)
                    })?;
                read_file_range(&mut file)?
            }
            _ => {
                return Err(format!("Path depth {} not supported (max 3 levels)", parts.len()).into());
            }
        };

        Ok(buffer)
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

/// Get the default save file path
/// Using 8.3 filename format for FAT filesystem compatibility
pub fn get_save_path() -> &'static str {
    "SAVE.JSN"
}
