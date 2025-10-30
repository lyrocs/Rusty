/// Save system - handles SD card persistence for hero data and inventory

use bevy_ecs::prelude::*;

use crate::ecs::resources::SdCardResource;
use crate::core::GameState;

/// System to handle save requests with SD card persistence
pub fn tamagotchi_save_system(
    mut sd_card_res: NonSendMut<SdCardResource>,
    mut game_state: ResMut<GameState>,
) {
    if game_state.save_requested {
        game_state.save_requested = false;

        // Generate save data
        let save_data = game_state.hero.to_save_string();

        esp_println::println!(
            "[SAVE] Saving hero: Level {} {} with {} EXP and {} Zeny, {} items",
            game_state.hero.level,
            game_state.hero.job,
            game_state.hero.exp,
            game_state.hero.zeny,
            game_state.hero.inventory.len()
        );

        // Try to write hero data to SD card
        let hero_result = save_hero_to_sd(&mut sd_card_res, save_data.as_str());

        // Try to write inventory to SD card
        let inventory_data = game_state.hero.inventory_to_save_string();
        let inventory_result = save_inventory_to_sd(&mut sd_card_res, inventory_data.as_str());

        // Try to write equipment data to SD card
        let equipment_data = game_state.hero.equipment_to_save_string();
        let equipment_result = save_equipment_to_sd(&mut sd_card_res, equipment_data.as_str());

        // Try to write quest data to SD card
        let quest_data = game_state.quests_to_save_string();
        let quest_result = save_quests_to_sd(&mut sd_card_res, quest_data.as_str());

        // Check results
        match (hero_result, inventory_result, equipment_result, quest_result) {
            (Ok(_), Ok(_), Ok(_), Ok(_)) => {
                esp_println::println!("[SAVE] Successfully saved hero, inventory, equipment, and quests to SD card");
                game_state.save_status_msg = Some("Saved to SD!");
            }
            (Ok(_), Ok(_), Ok(_), Err(e)) => {
                esp_println::println!("[SAVE] Hero, inventory, and equipment saved but quests failed: {:?}", e);
                game_state.save_status_msg = Some("Save partial!");
            }
            (Ok(_), Ok(_), Err(e), _) => {
                esp_println::println!("[SAVE] Hero and inventory saved but equipment/quests failed: {:?}", e);
                game_state.save_status_msg = Some("Save partial!");
            }
            (Ok(_), Err(e), _, _) => {
                esp_println::println!("[SAVE] Hero saved but inventory/equipment/quests failed: {:?}", e);
                game_state.save_status_msg = Some("Save partial!");
            }
            (Err(e), _, _, _) => {
                esp_println::println!("[SAVE] Error saving hero to SD: {:?}", e);
                game_state.save_status_msg = Some("Save failed!");
            }
        }

        // Show success message for 3 seconds
        game_state.save_status_timeout = game_state.last_update_ms + 3000;
        game_state.needs_redraw = true; // Redraw to show save message
    }

    // Clear save message after timeout
    if game_state.save_status_timeout > 0
        && game_state.last_update_ms >= game_state.save_status_timeout
    {
        game_state.save_status_msg = None;
        game_state.save_status_timeout = 0;
        game_state.needs_redraw = true; // Redraw to clear message
    }
}

/// Helper function to save hero data to SD card
fn save_hero_to_sd(
    sd_card_res: &mut SdCardResource,
    save_data: &str,
) -> Result<(), embedded_sdmmc::Error<embedded_sdmmc::SdCardError>> {
    use embedded_sdmmc::{Mode, VolumeIdx};

    // Open volume
    let mut volume = sd_card_res.volume_mgr.open_volume(VolumeIdx(0))?;

    // Open root directory
    let mut root_dir = volume.open_root_dir()?;

    // Create or truncate save file
    let mut file = root_dir.open_file_in_dir("HERO.SAV", Mode::ReadWriteCreateOrTruncate)?;

    // Write save data
    file.write(save_data.as_bytes())?;

    Ok(())
}

/// Helper function to save inventory data to SD card
fn save_inventory_to_sd(
    sd_card_res: &mut SdCardResource,
    inventory_data: &str,
) -> Result<(), embedded_sdmmc::Error<embedded_sdmmc::SdCardError>> {
    use embedded_sdmmc::{Mode, VolumeIdx};

    // Open volume
    let mut volume = sd_card_res.volume_mgr.open_volume(VolumeIdx(0))?;

    // Open root directory
    let mut root_dir = volume.open_root_dir()?;

    // Create or truncate inventory file
    let mut file = root_dir.open_file_in_dir("ITEMS.SAV", Mode::ReadWriteCreateOrTruncate)?;

    // Write inventory data
    file.write(inventory_data.as_bytes())?;

    Ok(())
}

/// Helper function to save equipment data to SD card
fn save_equipment_to_sd(
    sd_card_res: &mut SdCardResource,
    equipment_data: &str,
) -> Result<(), embedded_sdmmc::Error<embedded_sdmmc::SdCardError>> {
    use embedded_sdmmc::{Mode, VolumeIdx};

    // Open volume
    let mut volume = sd_card_res.volume_mgr.open_volume(VolumeIdx(0))?;

    // Open root directory
    let mut root_dir = volume.open_root_dir()?;

    // Create or truncate equipment file
    let mut file = root_dir.open_file_in_dir("EQUIP.SAV", Mode::ReadWriteCreateOrTruncate)?;

    // Write equipment data
    file.write(equipment_data.as_bytes())?;

    Ok(())
}

/// Helper function to save quest data to SD card
fn save_quests_to_sd(
    sd_card_res: &mut SdCardResource,
    quest_data: &str,
) -> Result<(), embedded_sdmmc::Error<embedded_sdmmc::SdCardError>> {
    use embedded_sdmmc::{Mode, VolumeIdx};

    // Open volume
    let mut volume = sd_card_res.volume_mgr.open_volume(VolumeIdx(0))?;

    // Open root directory
    let mut root_dir = volume.open_root_dir()?;

    // Create or truncate quest file
    let mut file = root_dir.open_file_in_dir("QUESTS.SAV", Mode::ReadWriteCreateOrTruncate)?;

    // Write quest data
    file.write(quest_data.as_bytes())?;

    Ok(())
}
