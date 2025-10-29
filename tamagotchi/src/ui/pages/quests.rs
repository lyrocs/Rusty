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

/// Draw the quest page with quest list
pub fn draw_quests_page<D>(display: &mut D, game_state: &GameState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    display.clear(COLOR_BG)?;

    // Header
    draw_text(
        display,
        "=== QUESTS ===",
        Point::new(100, 20),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // Filter active quests (not claimed)
    let active_quests: heapless::Vec<&crate::tamagotchi::models::ActiveQuest, 16> = game_state
        .active_quests
        .iter()
        .filter(|q| !q.claimed)
        .collect();

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

    // Navigation buttons at bottom
    // Up arrow button (left) - only show if not at start
    if game_state.quest_page_scroll > 0 {
        Rectangle::new(Point::new(10, 400), Size::new(70, 40))
            .into_styled(PrimitiveStyle::with_fill(COLOR_PANEL))
            .draw(display)?;

        Rectangle::new(Point::new(10, 400), Size::new(70, 40))
            .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
            .draw(display)?;

        // Draw up arrow (^)
        draw_text(
            display,
            "^",
            Point::new(35, 422),
            &FONT_10X20,
            COLOR_TEXT,
        )?;
        draw_text(
            display,
            "UP",
            Point::new(28, 434),
            &FONT_9X15,
            COLOR_TEXT_DIM,
        )?;
    }

    // Back button (center)
    Rectangle::new(Point::new(134, 400), Size::new(100, 40))
        .into_styled(PrimitiveStyle::with_fill(COLOR_PANEL))
        .draw(display)?;

    Rectangle::new(Point::new(134, 400), Size::new(100, 40))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
        .draw(display)?;

    draw_text(
        display,
        "BACK",
        Point::new(153, 422),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;

    // Down arrow button (right) - only show if more quests below
    if !active_quests.is_empty() && (game_state.quest_page_scroll as usize + 4) < active_quests.len() {
        Rectangle::new(Point::new(288, 400), Size::new(70, 40))
            .into_styled(PrimitiveStyle::with_fill(COLOR_PANEL))
            .draw(display)?;

        Rectangle::new(Point::new(288, 400), Size::new(70, 40))
            .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
            .draw(display)?;

        // Draw down arrow (v)
        draw_text(
            display,
            "v",
            Point::new(313, 422),
            &FONT_10X20,
            COLOR_TEXT,
        )?;
        draw_text(
            display,
            "DOWN",
            Point::new(297, 434),
            &FONT_9X15,
            COLOR_TEXT_DIM,
        )?;
    }

    Ok(())
}

