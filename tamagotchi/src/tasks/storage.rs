use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Timer};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use alloc::sync::Arc;
use heapless::String as HeaplessString;
use bevy_ecs::prelude::*;
use log::{info, warn};

use super::channels::{SAVE_CHANNEL, LOAD_RESPONSE_CHANNEL, SaveCommand, SaveResponse};
use crate::ecs::resources::SdCardResource;
use crate::core::GameState;

/// Storage task - handles SD card operations in the background
/// Even though SD operations are blocking, they run in this separate task
/// which prevents blocking the game loop and render tasks
#[embassy_executor::task]
pub async fn storage_task(world: Arc<Mutex<CriticalSectionRawMutex, World>>) {
    info!("[STORAGE] Storage task started - ready for save/load operations");

    // Statistics
    let mut save_count: u32 = 0;
    let mut load_count: u32 = 0;

    loop {
        // Wait for storage commands
        match SAVE_CHANNEL.receive().await {
            SaveCommand::SaveGame => {
                let start = Instant::now();
                info!("[STORAGE] Save game requested");

                // Lock world to get save data
                let (hero_data, inventory_data, equipment_data, quest_data, hero_info) = {
                    let world_guard = world.lock().await;

                    if let Some(game_state) = world_guard.get_resource::<GameState>() {
                        let info = (
                            game_state.hero.level,
                            game_state.hero.job.clone(),
                            game_state.hero.exp,
                            game_state.hero.inventory.len(),
                        );
                        (
                            game_state.hero.to_save_string(),
                            game_state.hero.inventory_to_save_string(),
                            game_state.hero.equipment_to_save_string(),
                            game_state.quests_to_save_string(),
                            Some(info),
                        )
                    } else {
                        (HeaplessString::new(), HeaplessString::new(), HeaplessString::new(), HeaplessString::new(), None)
                    }
                };

                let save_result = if let Some((level, job, exp, item_count)) = hero_info {
                    info!(
                        "[STORAGE] Saving: Level {} {} with {} EXP, {} items",
                        level, job, exp, item_count
                    );

                    // CRITICAL FIX: Remove SD card from world, release lock, perform I/O
                    // This prevents blocking the render task during slow SD card operations
                    let mut sd_card_opt = {
                        let mut world_guard = world.lock().await;
                        world_guard.remove_non_send_resource::<SdCardResource>()
                    };
                    // Lock is released here - render and update can run during SD I/O!

                    // Perform SD card I/O WITHOUT holding world lock
                    let (hero_result, inv_result, equip_result, quest_result) =
                        if let Some(ref mut sd_card_res) = sd_card_opt {
                            (
                                save_to_sd(sd_card_res, "HERO.SAV", &hero_data),
                                save_to_sd(sd_card_res, "ITEMS.SAV", &inventory_data),
                                save_to_sd(sd_card_res, "EQUIP.SAV", &equipment_data),
                                save_to_sd(sd_card_res, "QUESTS.SAV", &quest_data),
                            )
                        } else {
                            warn!("[STORAGE] Cannot save - SD card resource not available");
                            (Err("No SD"), Err("No SD"), Err("No SD"), Err("No SD"))
                        };

                    // Check results
                    let success = hero_result.is_ok()
                        && inv_result.is_ok()
                        && equip_result.is_ok()
                        && quest_result.is_ok();

                    // Lock world again to update status and restore SD card
                    {
                        let mut world_guard = world.lock().await;

                        // Restore SD card resource
                        if let Some(sd_card_res) = sd_card_opt {
                            world_guard.insert_non_send_resource(sd_card_res);
                        }

                        // Update game state with results
                        if let Some(mut game_state) = world_guard.get_resource_mut::<GameState>() {
                            if success {
                                game_state.save_status_msg = Some("Saved to SD!");
                                game_state.needs_redraw = true;
                                info!("[STORAGE] ✓ All save files written successfully");
                            } else {
                                game_state.save_status_msg = Some("Save failed!");
                                game_state.needs_redraw = true;
                                warn!("[STORAGE] ✗ Some save operations failed");
                            }
                            game_state.save_status_timeout = game_state.last_update_ms + 2000;
                        }
                    }
                    // Lock released

                    if success {
                        Ok(())
                    } else {
                        Err("Save failed")
                    }
                } else {
                    warn!("[STORAGE] Cannot save - game state not available");
                    Err("Game state unavailable")
                };

                // Release the lock before yielding
                drop(save_result);

                let save_time = start.elapsed();
                save_count = save_count.wrapping_add(1);

                info!(
                    "[STORAGE] Save #{} completed in {}ms",
                    save_count,
                    save_time.as_millis()
                );

                // Warn if save took too long
                if save_time.as_millis() > 500 {
                    warn!(
                        "[STORAGE] Slow save operation: {}ms (target: <500ms)",
                        save_time.as_millis()
                    );
                }

                // Yield to other tasks after save
                Timer::after(Duration::from_millis(10)).await;
            }
            SaveCommand::LoadGame => {
                info!("[STORAGE] Load game requested");
                load_count = load_count.wrapping_add(1);

                // Note: Load is currently done at startup in hardware init
                // This is a placeholder for future dynamic loading
                warn!("[STORAGE] Dynamic load not yet implemented - use startup load");

                let _ = LOAD_RESPONSE_CHANNEL.try_send(SaveResponse::Error);

                // Yield
                Timer::after(Duration::from_millis(10)).await;
            }
            SaveCommand::SaveSettings => {
                info!("[STORAGE] Save settings requested");

                // Placeholder for settings save
                // Could save display brightness, sound settings, etc.
                warn!("[STORAGE] Settings save not yet implemented");

                // Yield
                Timer::after(Duration::from_millis(10)).await;
            }
        }
    }
}

/// Helper function to save data to SD card
/// Note: This is a blocking operation, but it runs in the storage task
fn save_to_sd(
    sd_card_res: &mut SdCardResource,
    filename: &str,
    data: &str,
) -> Result<(), &'static str> {
    use embedded_sdmmc::{Mode, VolumeIdx};

    // Open volume
    let mut volume = sd_card_res
        .volume_mgr
        .open_volume(VolumeIdx(0))
        .map_err(|_| "Failed to open volume")?;

    // Open root directory
    let mut root_dir = volume
        .open_root_dir()
        .map_err(|_| "Failed to open root dir")?;

    // Create or truncate file
    let mut file = root_dir
        .open_file_in_dir(filename, Mode::ReadWriteCreateOrTruncate)
        .map_err(|_| "Failed to open file")?;

    // Write data
    file.write(data.as_bytes())
        .map_err(|_| "Failed to write data")?;

    Ok(())
}
