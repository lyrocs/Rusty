use core::fmt::Write;
use embedded_graphics::{
    image::Image,
    mono_font::{
        MonoTextStyle,
        ascii::{FONT_9X15, FONT_9X18_BOLD, FONT_10X20},
    },
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Circle as EgCircle, Line, PrimitiveStyle, Rectangle},
    text::Text,
};
use heapless::String;
use tinygif::Gif;

use crate::core::GameState;
use crate::tamagotchi::models::{BattleState, CircleType, Enemy, FarmState, LocationType, MapHelper, RestState};
use super::super::helpers::*;

use super::super::colors::*;

/// Draw the quest page with quest list or quest details
pub fn draw_quests_page<D>(display: &mut D, game_state: &GameState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    display.clear(COLOR_BG)?;

    // Draw farming header if active
    use crate::ui::farming_header::draw_farming_header;
    let has_farming_header = draw_farming_header(display, game_state)?;

    // Check if viewing quest details or quest list
    if let Some(quest_id) = game_state.selected_quest_id {
        return draw_quest_details(display, game_state, quest_id);
    }

    // === Quest List View ===

    let title_y = if has_farming_header { 40 } else { 20 };

    // Header
    draw_text(
        display,
        "=== QUESTS ===",
        Point::new(100, title_y),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // Filter active quests (not claimed) and sort by priority
    let mut active_quests: heapless::Vec<&crate::tamagotchi::models::ActiveQuest, 16> = game_state
        .active_quests
        .iter()
        .filter(|q| !q.claimed)
        .collect();

    // Sort by priority (lower priority value = higher priority)
    active_quests.sort_by(|a, b| {
        let a_data = crate::tamagotchi::quest_system::get_quest_data(a.quest_id);
        let b_data = crate::tamagotchi::quest_system::get_quest_data(b.quest_id);

        match (a_data, b_data) {
            (Some(a_quest), Some(b_quest)) => a_quest.priority.cmp(&b_quest.priority),
            (Some(_), None) => core::cmp::Ordering::Less,
            (None, Some(_)) => core::cmp::Ordering::Greater,
            (None, None) => core::cmp::Ordering::Equal,
        }
    });

    if active_quests.is_empty() {
        draw_text(
            display,
            "No active quests!",
            Point::new(90, 200),
            &FONT_10X20,
            COLOR_TEXT_DIM,
        )?;
        draw_text(
            display,
            "Visit the Guild Master",
            Point::new(60, 230),
            &FONT_9X15,
            COLOR_TEXT_DIM,
        )?;
        draw_text(
            display,
            "in Prontera for quests",
            Point::new(60, 250),
            &FONT_9X15,
            COLOR_TEXT_DIM,
        )?;
    } else {
        // Draw quest cards (show up to 4 quests)
        let start_index = game_state.quest_page_scroll as usize;
        let visible_quests = active_quests.iter().skip(start_index).take(4);

        let mut card_y = 60;
        for active_quest in visible_quests {
            // Get quest data
            if let Some(quest_data) = crate::tamagotchi::quest_system::get_quest_data(active_quest.quest_id) {
                // Quest card background
                let card_height = 80u32;
                Rectangle::new(Point::new(10, card_y), Size::new(348, card_height))
                    .into_styled(PrimitiveStyle::with_fill(COLOR_PANEL))
                    .draw(display)?;

                Rectangle::new(Point::new(10, card_y), Size::new(348, card_height))
                    .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
                    .draw(display)?;

                // Quest type indicator
                let type_color = match quest_data.quest_type {
                    crate::tamagotchi::models::QuestType::Daily => Rgb888::new(100, 150, 255),
                    crate::tamagotchi::models::QuestType::Story => Rgb888::new(255, 200, 100),
                    crate::tamagotchi::models::QuestType::Achievement => Rgb888::new(255, 100, 255),
                };

                let type_label = match quest_data.quest_type {
                    crate::tamagotchi::models::QuestType::Daily => "DAILY",
                    crate::tamagotchi::models::QuestType::Story => "STORY",
                    crate::tamagotchi::models::QuestType::Achievement => "ACHIEVEMENT",
                };

                draw_text(
                    display,
                    type_label,
                    Point::new(20, card_y + 18),
                    &FONT_9X15,
                    type_color,
                )?;

                // Quest name
                draw_text(
                    display,
                    quest_data.name,
                    Point::new(20, card_y + 35),
                    &FONT_9X15,
                    COLOR_TEXT,
                )?;

                // Progress for first objective
                if !quest_data.objectives.is_empty() && !active_quest.progress.is_empty() {
                    let objective = &quest_data.objectives[0];
                    let progress = active_quest.progress[0];

                    let target = match objective.objective_type {
                        "KillMonster" => objective.count,
                        "CollectItem" => objective.count,
                        "ReachLevel" => objective.level,
                        "EarnZeny" => objective.amount as u16,
                        "RefineEquipment" => objective.count,
                        "CompleteBattles" => objective.count,
                        _ => 0,
                    };

                    // Progress text
                    let mut progress_str = String::<32>::new();
                    write!(progress_str, "{}/{}", progress, target).ok();
                    draw_text(
                        display,
                        &progress_str,
                        Point::new(20, card_y + 52),
                        &FONT_9X15,
                        COLOR_TEXT_DIM,
                    )?;

                    // Progress bar
                    let bar_width = 308;
                    let bar_x = 20;
                    let bar_y = card_y + 58;

                    // Background bar
                    Rectangle::new(Point::new(bar_x, bar_y), Size::new(bar_width, 8))
                        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 40, 40)))
                        .draw(display)?;

                    // Progress bar fill
                    let progress_percent = if target > 0 {
                        (progress as u32 * 100) / target as u32
                    } else {
                        0
                    };
                    let fill_width = (bar_width as u32 * progress_percent / 100) as u32;

                    let bar_color = if active_quest.completed {
                        Rgb888::GREEN
                    } else {
                        COLOR_EXP
                    };

                    Rectangle::new(Point::new(bar_x, bar_y), Size::new(fill_width, 8))
                        .into_styled(PrimitiveStyle::with_fill(bar_color))
                        .draw(display)?;
                }

                // Status/Claim button
                if active_quest.completed {
                    // Claim button
                    Rectangle::new(Point::new(250, card_y + 10), Size::new(98, 60))
                        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(50, 100, 50)))
                        .draw(display)?;

                    Rectangle::new(Point::new(250, card_y + 10), Size::new(98, 60))
                        .into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, 2))
                        .draw(display)?;

                    draw_text(
                        display,
                        "CLAIM",
                        Point::new(262, card_y + 45),
                        &FONT_9X18_BOLD,
                        Rgb888::WHITE,
                    )?;
                } else {
                    // In progress indicator
                    draw_text(
                        display,
                        "In Progress",
                        Point::new(260, card_y + 35),
                        &FONT_9X15,
                        COLOR_TEXT_DIM,
                    )?;
                }

                card_y += card_height as i32 + 8;
            }
        }

        // Scroll indicator if needed
        if active_quests.len() > 4 {
            let mut scroll_str = String::<32>::new();
            write!(
                scroll_str,
                "{}-{} of {}",
                start_index + 1,
                (start_index + 4).min(active_quests.len()),
                active_quests.len()
            )
            .ok();
            draw_text(
                display,
                &scroll_str,
                Point::new(140, 390),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;
        }
    }

    // Navigation buttons - full width, taller, positioned higher
    let button_y = 365;
    let button_height = 55;
    let button_width = 110i32;
    let button_spacing = 10i32;

    // Calculate button positions (as i32 for Point::new)
    let up_x = 15i32;
    let back_x = 15 + button_width + button_spacing;
    let down_x = 15 + (button_width + button_spacing) * 2;

    // Up arrow button (left) - always show but disabled if at start
    let up_color = if game_state.quest_page_scroll > 0 {
        COLOR_PANEL
    } else {
        Rgb888::new(40, 40, 40) // Darker when disabled
    };

    Rectangle::new(Point::new(up_x, button_y), Size::new(button_width as u32, button_height))
        .into_styled(PrimitiveStyle::with_fill(up_color))
        .draw(display)?;
    Rectangle::new(Point::new(up_x, button_y), Size::new(button_width as u32, button_height))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
        .draw(display)?;

    // Draw up arrow (^)
    draw_text(
        display,
        "^ UP",
        Point::new(up_x + 25, button_y + 30),
        &FONT_9X18_BOLD,
        if game_state.quest_page_scroll > 0 { COLOR_TEXT } else { COLOR_TEXT_DIM },
    )?;

    // Back button (center)
    Rectangle::new(Point::new(back_x, button_y), Size::new(button_width as u32, button_height))
        .into_styled(PrimitiveStyle::with_fill(COLOR_PANEL))
        .draw(display)?;
    Rectangle::new(Point::new(back_x, button_y), Size::new(button_width as u32, button_height))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
        .draw(display)?;

    draw_text(
        display,
        "BACK",
        Point::new(back_x + 25, button_y + 30),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;

    // Down arrow button (right) - always show but disabled if at end
    let can_scroll_down = !active_quests.is_empty() && (game_state.quest_page_scroll as usize + 4) < active_quests.len();
    let down_color = if can_scroll_down {
        COLOR_PANEL
    } else {
        Rgb888::new(40, 40, 40) // Darker when disabled
    };

    Rectangle::new(Point::new(down_x, button_y), Size::new(button_width as u32, button_height))
        .into_styled(PrimitiveStyle::with_fill(down_color))
        .draw(display)?;
    Rectangle::new(Point::new(down_x, button_y), Size::new(button_width as u32, button_height))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
        .draw(display)?;

    // Draw down arrow (v)
    draw_text(
        display,
        "v DOWN",
        Point::new(down_x + 20, button_y + 30),
        &FONT_9X18_BOLD,
        if can_scroll_down { COLOR_TEXT } else { COLOR_TEXT_DIM },
    )?;

    Ok(())
}

/// Draw quest details page with full information and claim button
fn draw_quest_details<D>(display: &mut D, game_state: &GameState, quest_id: u32) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Find the quest data and active quest
    let quest_data = match crate::tamagotchi::quest_system::get_quest_data(quest_id) {
        Some(data) => data,
        None => {
            // Quest not found - shouldn't happen
            draw_text(
                display,
                "Quest not found",
                Point::new(100, 200),
                &FONT_10X20,
                COLOR_TEXT,
            )?;
            return Ok(());
        }
    };

    let active_quest = game_state
        .active_quests
        .iter()
        .find(|q| q.quest_id == quest_id);

    // Header
    draw_text(
        display,
        "=== QUEST DETAILS ===",
        Point::new(60, 20),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // Quest type badge
    let type_color = match quest_data.quest_type {
        crate::tamagotchi::models::QuestType::Daily => Rgb888::new(100, 150, 255),
        crate::tamagotchi::models::QuestType::Story => Rgb888::new(255, 200, 100),
        crate::tamagotchi::models::QuestType::Achievement => Rgb888::new(255, 100, 255),
    };

    let type_label = match quest_data.quest_type {
        crate::tamagotchi::models::QuestType::Daily => "DAILY QUEST",
        crate::tamagotchi::models::QuestType::Story => "STORY QUEST",
        crate::tamagotchi::models::QuestType::Achievement => "ACHIEVEMENT",
    };

    draw_text(
        display,
        type_label,
        Point::new(110, 50),
        &FONT_9X18_BOLD,
        type_color,
    )?;

    // Quest name
    draw_text(
        display,
        quest_data.name,
        Point::new(20, 80),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;

    // Quest description
    let mut y_offset = 105;
    draw_text(
        display,
        quest_data.description,
        Point::new(20, y_offset),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    // Objectives section
    y_offset = 140;
    draw_text(
        display,
        "Objectives:",
        Point::new(20, y_offset),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;

    y_offset += 25;
    for (i, objective) in quest_data.objectives.iter().enumerate() {
        let progress = if let Some(aq) = active_quest {
            aq.progress[i]
        } else {
            0
        };

        let (target, desc) = match objective.objective_type {
            "KillMonster" => {
                let monster_name = if objective.enemy_id == 1002 {
                    "Poring"
                } else if objective.enemy_id == 1004 {
                    "Hornet"
                } else if objective.enemy_id == 0 {
                    "Monster"
                } else {
                    "Enemy"
                };
                let mut desc_str = heapless::String::<48>::new();
                write!(desc_str, "Defeat {}", monster_name).ok();
                (objective.count, desc_str)
            }
            "CollectItem" => {
                let mut desc_str = heapless::String::<48>::new();
                desc_str.push_str("Collect items").ok();
                (objective.count, desc_str)
            }
            "ReachLevel" => {
                let mut desc_str = heapless::String::<48>::new();
                write!(desc_str, "Reach Level {}", objective.level).ok();
                (objective.level, desc_str)
            }
            "EarnZeny" => {
                let mut desc_str = heapless::String::<48>::new();
                desc_str.push_str("Earn Zeny").ok();
                (objective.amount as u16, desc_str)
            }
            "RefineEquipment" => {
                let mut desc_str = heapless::String::<48>::new();
                desc_str.push_str("Refine Equipment").ok();
                (objective.count, desc_str)
            }
            "CompleteBattles" => {
                let mut desc_str = heapless::String::<48>::new();
                desc_str.push_str("Complete Battles").ok();
                (objective.count, desc_str)
            }
            _ => (0, heapless::String::<48>::new()),
        };

        let mut obj_text = String::<48>::new();
        write!(obj_text, "- {} ({}/{})", desc, progress, target).ok();

        let obj_color = if progress >= target {
            Rgb888::GREEN
        } else {
            COLOR_TEXT
        };

        draw_text(
            display,
            &obj_text,
            Point::new(30, y_offset),
            &FONT_9X15,
            obj_color,
        )?;

        y_offset += 20;
    }

    // Rewards section
    y_offset += 15;
    draw_text(
        display,
        "Rewards:",
        Point::new(20, y_offset),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;

    y_offset += 25;
    if quest_data.rewards.base_exp > 0 {
        let mut reward_text = String::<32>::new();
        write!(reward_text, "- EXP: {}", quest_data.rewards.base_exp).ok();
        draw_text(
            display,
            &reward_text,
            Point::new(30, y_offset),
            &FONT_9X15,
            COLOR_EXP,
        )?;
        y_offset += 20;
    }

    if quest_data.rewards.zeny > 0 {
        let mut reward_text = String::<32>::new();
        write!(reward_text, "- Zeny: {}", quest_data.rewards.zeny).ok();
        draw_text(
            display,
            &reward_text,
            Point::new(30, y_offset),
            &FONT_9X15,
            Rgb888::new(255, 215, 0), // Gold color
        )?;
        y_offset += 20;
    }

    for (item_id, quantity) in &quest_data.rewards.items {
        if *item_id > 0 && *quantity > 0 {
            let mut item_text = String::<32>::new();
            write!(item_text, "- Item {} x{}", item_id, quantity).ok();
            draw_text(
                display,
                &item_text,
                Point::new(30, y_offset),
                &FONT_9X15,
                Rgb888::new(200, 255, 200),
            )?;
            y_offset += 20;
        }
    }

    // Buttons at bottom - positioned higher
    let button_y = 360;

    // Back button (left)
    Rectangle::new(Point::new(20, button_y), Size::new(150, 60))
        .into_styled(PrimitiveStyle::with_fill(COLOR_PANEL))
        .draw(display)?;

    Rectangle::new(Point::new(20, button_y), Size::new(150, 60))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
        .draw(display)?;

    draw_text(
        display,
        "BACK",
        Point::new(58, button_y + 35),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;

    // Claim button (right) - only if quest is completed
    if let Some(aq) = active_quest {
        if aq.completed && !aq.claimed {
            Rectangle::new(Point::new(190, button_y), Size::new(150, 60))
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(50, 100, 50)))
                .draw(display)?;

            Rectangle::new(Point::new(190, button_y), Size::new(150, 60))
                .into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, 3))
                .draw(display)?;

            draw_text(
                display,
                "CLAIM",
                Point::new(218, button_y + 35),
                &FONT_9X18_BOLD,
                Rgb888::WHITE,
            )?;
        } else if aq.completed && aq.claimed {
            // Already claimed
            draw_text(
                display,
                "CLAIMED",
                Point::new(200, button_y + 35),
                &FONT_9X18_BOLD,
                COLOR_TEXT_DIM,
            )?;
        } else {
            // In progress
            draw_text(
                display,
                "IN PROGRESS",
                Point::new(185, button_y + 35),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;
        }
    }

    Ok(())
}

