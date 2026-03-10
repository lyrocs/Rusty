// SD Card support — SPI-based via embedded-sdmmc.
//
// Hardware (Waveshare ESP32-C6-Touch-LCD-1.83):
//   SCK  = GPIO1  (shared with display)
//   MOSI = GPIO2  (shared with display)
//   MISO = GPIO16
//   CS   = GPIO17 (direct GPIO, active-low)

use embedded_sdmmc::{SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use std::error::Error;

// ─── Delay implementation ─────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct FreeRtosDelay;

impl embedded_hal::delay::DelayNs for FreeRtosDelay {
    fn delay_ns(&mut self, ns: u32) {
        use std::time::Duration;
        let dur = Duration::from_nanos(ns as u64).max(Duration::from_micros(1));
        std::thread::sleep(dur);
    }
}

// ─── Fake time source ─────────────────────────────────────────────────────────

pub struct SimpleTimeSource;

impl TimeSource for SimpleTimeSource {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 55, // 2025
            zero_indexed_month: 2,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

// ─── SD card resource ─────────────────────────────────────────────────────────

pub struct SdCardResource<DEV>
where
    DEV: embedded_hal::spi::SpiDevice,
{
    volume_mgr: VolumeManager<SdCard<DEV, FreeRtosDelay>, SimpleTimeSource, 4, 4, 1>,
}

impl<DEV> SdCardResource<DEV>
where
    DEV: embedded_hal::spi::SpiDevice<Error: core::fmt::Debug>,
{
    pub fn new(mut spi_device: DEV) -> Result<Self, Box<dyn Error>> {
        log::info!("SD: power-on wake-up sequence…");
        // Send 80 dummy clock cycles with CS high (card must see clocks before CMD0).
        let _ = spi_device.write(&[0xFF; 10]);
        std::thread::sleep(std::time::Duration::from_millis(10));

        let sd_card = SdCard::new(spi_device, FreeRtosDelay);

        match sd_card.num_bytes() {
            Ok(bytes) => log::info!("SD: {} MB detected", bytes / 1024 / 1024),
            Err(e) => return Err(format!("SD card not responding: {:?}", e).into()),
        }

        let volume_mgr = VolumeManager::new(sd_card, SimpleTimeSource);
        log::info!("SD: ready");
        Ok(Self { volume_mgr })
    }

    /// Read an entire file from the SD root directory into a Vec<u8>.
    pub fn read_file(&mut self, filename: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        use embedded_sdmmc::Mode;
        let mut volume = self.volume_mgr.open_volume(VolumeIdx(0))
            .map_err(|e| format!("open_volume: {:?}", e))?;
        let mut root = volume.open_root_dir()
            .map_err(|e| format!("open_root_dir: {:?}", e))?;
        let mut file = root.open_file_in_dir(filename, Mode::ReadOnly)
            .map_err(|e| format!("open {filename}: {:?}", e))?;

        let mut buf = Vec::new();
        let mut chunk = [0u8; 512];
        loop {
            match file.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => buf.extend_from_slice(&chunk[..n]),
                Err(e) => return Err(format!("read: {:?}", e).into()),
            }
        }
        Ok(buf)
    }

    /// List the root directory contents to the log (equivalent of `ls /`).
    pub fn ls_root(&mut self) {
        let mut volume = match self.volume_mgr.open_volume(VolumeIdx(0)) {
            Ok(v) => v,
            Err(e) => { log::error!("SD ls: open_volume failed: {:?}", e); return; }
        };
        let mut root = match volume.open_root_dir() {
            Ok(d) => d,
            Err(e) => { log::error!("SD ls: open_root_dir failed: {:?}", e); return; }
        };

        log::info!("── SD card / ────────────────────");
        let _ = root.iterate_dir(|entry| {
            let marker = if entry.attributes.is_directory() { "/" } else { "" };
            log::info!("  {}{}", entry.name, marker);
        });
        log::info!("─────────────────────────────────");
    }
}
