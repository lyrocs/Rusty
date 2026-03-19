//! Storage subsystem: SD card file operations.
//!
//! Merges the best of both projects: rustymon's simple API and stdgotchi's
//! path handling, range reads, and write support.

use crate::error::{Result, RustyError};
use embedded_sdmmc::{SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};

/// Copy-able delay for SD card operations.
#[derive(Clone, Copy)]
pub struct FreeRtosDelay;

impl embedded_hal::delay::DelayNs for FreeRtosDelay {
    fn delay_ns(&mut self, ns: u32) {
        if ns >= 1_000_000 {
            let ms = ns / 1_000_000;
            esp_idf_svc::hal::delay::FreeRtos.delay_ms(ms);
        } else if ns >= 1_000 {
            let us = ns / 1_000;
            esp_idf_svc::hal::delay::FreeRtos.delay_us(us);
        } else if ns > 0 {
            esp_idf_svc::hal::delay::FreeRtos.delay_us(1);
        }
    }
}

use embedded_hal::delay::DelayNs;

struct SimpleTimeSource;

impl TimeSource for SimpleTimeSource {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 56, // 2026
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

/// High-level SD card storage.
///
/// Hides all SPI generics behind a type-erased interface.
pub struct Storage {
    inner: Box<dyn StorageOps>,
}

trait StorageOps {
    fn read_file(&mut self, path: &str) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error>>;
    fn write_file(&mut self, path: &str, data: &[u8]) -> std::result::Result<(), Box<dyn std::error::Error>>;
    fn read_range(&mut self, path: &str, offset: u32, length: usize) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error>>;
    fn file_exists(&mut self, path: &str) -> bool;
    fn ls_root(&mut self);
}

struct SdCardStorage<DEV>
where
    DEV: embedded_hal::spi::SpiDevice,
{
    volume_mgr: VolumeManager<SdCard<DEV, FreeRtosDelay>, SimpleTimeSource, 4, 4, 1>,
}

impl<DEV> SdCardStorage<DEV>
where
    DEV: embedded_hal::spi::SpiDevice<Error: core::fmt::Debug>,
{
    fn new(mut spi_device: DEV) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        log::info!("SD: power-on wake-up...");
        FreeRtosDelay.delay_ms(10);
        let _ = spi_device.write(&[0xFF; 10]);
        FreeRtosDelay.delay_ms(10);

        let sd_card = SdCard::new(spi_device, FreeRtosDelay);

        match sd_card.num_bytes() {
            Ok(bytes) => log::info!("SD: {} MB detected", bytes / 1024 / 1024),
            Err(e) => return Err(format!("SD card not responding: {:?}", e).into()),
        }

        let volume_mgr = VolumeManager::new(sd_card, SimpleTimeSource);
        log::info!("SD: ready");
        Ok(Self { volume_mgr })
    }
}

impl<DEV> StorageOps for SdCardStorage<DEV>
where
    DEV: embedded_hal::spi::SpiDevice<Error: core::fmt::Debug>,
{
    fn read_file(&mut self, path: &str) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error>> {
        use embedded_sdmmc::Mode;

        let path = path.trim_start_matches('/');
        let parts: Vec<&str> = path.split('/').collect();

        let mut volume = self.volume_mgr.open_volume(VolumeIdx(0))
            .map_err(|e| format!("open_volume: {:?}", e))?;
        let mut root = volume.open_root_dir()
            .map_err(|e| format!("open_root_dir: {:?}", e))?;

        let read_chunks = |file: &mut embedded_sdmmc::File<_, _, _, _, _>| -> std::result::Result<Vec<u8>, Box<dyn std::error::Error>> {
            let mut buf = Vec::new();
            let mut chunk = [0u8; 2048];
            loop {
                match file.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(n) => buf.extend_from_slice(&chunk[..n]),
                    Err(e) => return Err(format!("read: {:?}", e).into()),
                }
            }
            Ok(buf)
        };

        match parts.len() {
            1 => {
                let mut file = root.open_file_in_dir(parts[0], Mode::ReadOnly)
                    .map_err(|e| format!("open {}: {:?}", parts[0], e))?;
                read_chunks(&mut file)
            }
            2 => {
                let mut dir = root.open_dir(parts[0])
                    .map_err(|e| format!("open dir {}: {:?}", parts[0], e))?;
                let mut file = dir.open_file_in_dir(parts[1], Mode::ReadOnly)
                    .map_err(|e| format!("open {}: {:?}", parts[1], e))?;
                read_chunks(&mut file)
            }
            3 => {
                let mut dir1 = root.open_dir(parts[0])
                    .map_err(|e| format!("open dir {}: {:?}", parts[0], e))?;
                let mut dir2 = dir1.open_dir(parts[1])
                    .map_err(|e| format!("open dir {}: {:?}", parts[1], e))?;
                let mut file = dir2.open_file_in_dir(parts[2], Mode::ReadOnly)
                    .map_err(|e| format!("open {}: {:?}", parts[2], e))?;
                read_chunks(&mut file)
            }
            _ => Err("Path too deep (max 3 levels)".into()),
        }
    }

    fn write_file(&mut self, path: &str, data: &[u8]) -> std::result::Result<(), Box<dyn std::error::Error>> {
        use embedded_sdmmc::Mode;

        let path = path.trim_start_matches('/');

        let mut volume = self.volume_mgr.open_volume(VolumeIdx(0))
            .map_err(|e| format!("open_volume: {:?}", e))?;
        let mut root = volume.open_root_dir()
            .map_err(|e| format!("open_root_dir: {:?}", e))?;
        let mut file = root.open_file_in_dir(path, Mode::ReadWriteCreateOrTruncate)
            .map_err(|e| format!("open {}: {:?}", path, e))?;

        file.write(data)
            .map_err(|e| format!("write: {:?}", e))?;

        Ok(())
    }

    fn read_range(&mut self, path: &str, offset: u32, length: usize) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error>> {
        use embedded_sdmmc::Mode;

        let path = path.trim_start_matches('/');

        let mut volume = self.volume_mgr.open_volume(VolumeIdx(0))
            .map_err(|e| format!("open_volume: {:?}", e))?;
        let mut root = volume.open_root_dir()
            .map_err(|e| format!("open_root_dir: {:?}", e))?;
        let mut file = root.open_file_in_dir(path, Mode::ReadOnly)
            .map_err(|e| format!("open {}: {:?}", path, e))?;

        file.seek_from_start(offset)
            .map_err(|e| format!("seek: {:?}", e))?;

        let mut buffer = vec![0u8; length];
        let mut total = 0;
        while total < length {
            match file.read(&mut buffer[total..]) {
                Ok(0) => { buffer.truncate(total); break; }
                Ok(n) => total += n,
                Err(e) => return Err(format!("read: {:?}", e).into()),
            }
        }
        Ok(buffer)
    }

    fn file_exists(&mut self, path: &str) -> bool {
        use embedded_sdmmc::Mode;
        let path = path.trim_start_matches('/');

        self.volume_mgr.open_volume(VolumeIdx(0)).ok().and_then(|mut vol| {
            vol.open_root_dir().ok().map(|mut root| {
                root.open_file_in_dir(path, Mode::ReadOnly).is_ok()
            })
        }).unwrap_or(false)
    }

    fn ls_root(&mut self) {
        let mut volume = match self.volume_mgr.open_volume(VolumeIdx(0)) {
            Ok(v) => v,
            Err(e) => { log::error!("SD ls: {:?}", e); return; }
        };
        let mut root = match volume.open_root_dir() {
            Ok(d) => d,
            Err(e) => { log::error!("SD ls: {:?}", e); return; }
        };

        log::info!("── SD card / ──");
        let _ = root.iterate_dir(|entry| {
            let marker = if entry.attributes.is_directory() { "/" } else { "" };
            log::info!("  {}{}", entry.name, marker);
        });
    }
}

impl Storage {
    /// Create a new Storage from an SPI device connected to an SD card.
    pub fn new<DEV>(spi_device: DEV) -> Result<Self>
    where
        DEV: embedded_hal::spi::SpiDevice<Error: core::fmt::Debug> + 'static,
    {
        let inner = SdCardStorage::new(spi_device)
            .map_err(|e| RustyError::Storage(e.to_string()))?;
        Ok(Self {
            inner: Box::new(inner),
        })
    }

    /// Read an entire file as bytes.
    pub fn read_file(&mut self, path: &str) -> Result<Vec<u8>> {
        self.inner.read_file(path).map_err(|e| RustyError::Storage(e.to_string()))
    }

    /// Read a file as a UTF-8 string.
    pub fn read_text(&mut self, path: &str) -> Result<String> {
        let bytes = self.read_file(path)?;
        String::from_utf8(bytes).map_err(|e| RustyError::Storage(e.to_string()))
    }

    /// Read a byte range from a file.
    pub fn read_range(&mut self, path: &str, offset: u32, length: usize) -> Result<Vec<u8>> {
        self.inner.read_range(path, offset, length).map_err(|e| RustyError::Storage(e.to_string()))
    }

    /// Write text to a file (create or truncate).
    pub fn write_text(&mut self, path: &str, data: &str) -> Result<()> {
        self.inner.write_file(path, data.as_bytes()).map_err(|e| RustyError::Storage(e.to_string()))
    }

    /// Write raw bytes to a file (create or truncate).
    pub fn write_bytes(&mut self, path: &str, data: &[u8]) -> Result<()> {
        self.inner.write_file(path, data).map_err(|e| RustyError::Storage(e.to_string()))
    }

    /// Check if a file exists.
    pub fn exists(&mut self, path: &str) -> bool {
        self.inner.file_exists(path)
    }

    /// Load and deserialize JSON from a file.
    pub fn load_json<T: serde::de::DeserializeOwned>(&mut self, path: &str) -> Result<T> {
        let text = self.read_text(path)?;
        serde_json::from_str(&text).map_err(|e| RustyError::Config(e.to_string()))
    }

    /// Serialize and save JSON to a file.
    pub fn save_json<T: serde::Serialize>(&mut self, path: &str, data: &T) -> Result<()> {
        let json = serde_json::to_string(data).map_err(|e| RustyError::Config(e.to_string()))?;
        self.write_text(path, &json)
    }

    /// Load a .spr sprite file from SD card.
    pub fn load_sprite(&mut self, path: &str) -> Result<crate::sprite::Sprite> {
        let bytes = self.read_file(path)?;
        crate::sprite::Sprite::from_bytes(&bytes).map_err(RustyError::Sprite)
    }

    /// List root directory contents to the log.
    pub fn ls_root(&mut self) {
        self.inner.ls_root();
    }
}
