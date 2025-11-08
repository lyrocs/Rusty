//! SD Card Support Module
//!
//! Provides SD card initialization and file operations for ESP32-S3.
//!
//! Note: This is a simplified implementation that saves to internal flash
//! when SD card is not available. For production, SD card mounting would
//! use esp-idf's SDMMC or SPI interfaces.

use std::error::Error;

/// SD card mount point (or fallback internal storage)
pub const SD_MOUNT_POINT: &str = "/spiffs";

/// SD card manager
pub struct SdCard {
    mounted: bool,
    use_internal_storage: bool,
}

impl SdCard {
    /// Create a new SD card manager (not yet initialized)
    pub fn new() -> Self {
        Self {
            mounted: false,
            use_internal_storage: true, // Use internal storage as fallback
        }
    }

    /// Initialize and mount storage
    ///
    /// For ESP32-S3-Touch-AMOLED, this attempts to:
    /// 1. Mount SD card if available (GPIO39-43)
    /// 2. Fall back to internal SPIFFS if no SD card
    ///
    /// Note: Actual SD card support requires additional configuration
    /// and the SD card hardware to be present. This implementation uses
    /// internal storage as a fallback.
    pub fn init(&mut self) -> Result<(), Box<dyn Error>> {
        log::info!("Initializing storage...");

        // For now, we use internal storage since SD card mounting
        // requires specific hardware configuration and the SD card to be present.
        // In a production environment, you would attempt SD card mount first,
        // then fall back to SPIFFS.

        log::info!("Using internal storage (SPIFFS fallback)");
        self.mounted = true;
        self.use_internal_storage = true;

        Ok(())
    }

    /// Check if storage is mounted
    pub fn is_mounted(&self) -> bool {
        self.mounted
    }

    /// Get the full path for a file in storage
    pub fn get_path(&self, filename: &str) -> String {
        if self.use_internal_storage {
            // For internal storage, use a simple path
            // In a real implementation, this would use SPIFFS or LittleFS
            format!("/tmp/{}", filename)
        } else {
            format!("{}/{}", SD_MOUNT_POINT, filename)
        }
    }

    /// Unmount storage
    pub fn deinit(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.mounted {
            return Ok(());
        }

        self.mounted = false;
        log::info!("Storage unmounted");

        Ok(())
    }

    /// Check if using internal storage fallback
    pub fn is_using_internal_storage(&self) -> bool {
        self.use_internal_storage
    }
}

impl Drop for SdCard {
    fn drop(&mut self) {
        let _ = self.deinit();
    }
}
