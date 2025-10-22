use embedded_sdmmc::{TimeSource, Timestamp, VolumeIdx, VolumeManager};

/// Simple TimeSource implementation for SD card file timestamps
/// Returns a fixed timestamp for now (can be enhanced with RTC later)
pub struct DummyTimeSource;

impl TimeSource for DummyTimeSource {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 55,   // 2025
            zero_indexed_month: 9, // October (0-indexed)
            zero_indexed_day: 20,  // 21st (0-indexed)
            hours: 12,
            minutes: 0,
            seconds: 0,
        }
    }
}

/// List all files in the root directory of the SD card
pub fn list_sd_card_files<D, T>(
    volume_mgr: &mut VolumeManager<D, T, 4, 4, 1>,
) -> Result<(), embedded_sdmmc::Error<D::Error>>
where
    D: embedded_sdmmc::BlockDevice,
    T: TimeSource,
    D::Error: core::fmt::Debug,
{
    esp_println::println!("\n=== SD Card Directory Listing ===");

    // Open first volume (partition)
    let mut volume = volume_mgr.open_volume(VolumeIdx(0))?;
    esp_println::println!("Volume opened successfully");

    // Open root directory
    let mut root_dir = volume.open_root_dir()?;
    esp_println::println!("Root directory opened\n");

    // Iterate through directory entries
    root_dir.iterate_dir(|entry| {
        let is_dir = entry.attributes.is_directory();
        let size = entry.size;

        if is_dir {
            esp_println::println!("  [DIR]  {:?}", entry.name);
        } else {
            esp_println::println!("  [FILE] {:?} ({} bytes)", entry.name, size);
        }
    })?;

    esp_println::println!("=== End of Directory ===\n");
    Ok(())
}

/// Read and display contents of a text file from SD card
pub fn read_sd_card_file<D, T>(
    volume_mgr: &mut VolumeManager<D, T, 4, 4, 1>,
    filename: &str,
) -> Result<(), embedded_sdmmc::Error<D::Error>>
where
    D: embedded_sdmmc::BlockDevice,
    T: TimeSource,
    D::Error: core::fmt::Debug,
{
    esp_println::println!("\n=== Reading file: {} ===", filename);

    // Open volume and root directory
    let mut volume = volume_mgr.open_volume(VolumeIdx(0))?;
    let mut root_dir = volume.open_root_dir()?;

    // Try to open the file
    let mut file = root_dir.open_file_in_dir(filename, embedded_sdmmc::Mode::ReadOnly)?;

    // Read file in chunks
    let mut buffer = [0u8; 512];
    let mut total_read = 0;

    loop {
        match file.read(&mut buffer) {
            Ok(bytes_read) if bytes_read > 0 => {
                total_read += bytes_read;

                // Count printable vs non-printable characters
                let text_slice = &buffer[..bytes_read];
                let printable_count = text_slice
                    .iter()
                    .filter(|&&b| b >= 0x20 && b <= 0x7E || b == b'\n' || b == b'\r')
                    .count();

                esp_println::println!(
                    "Read {} bytes ({} printable chars)",
                    bytes_read, printable_count
                );

                if bytes_read < buffer.len() {
                    break; // End of file
                }
            }
            Ok(_) => break, // End of file (0 bytes read)
            Err(e) => {
                esp_println::println!("\nError reading file: {:?}", e);
                break;
            }
        }
    }

    esp_println::println!("\n\nTotal bytes read: {}", total_read);
    esp_println::println!("=== End of file ===\n");

    Ok(())
}
