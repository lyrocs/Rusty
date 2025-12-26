//! Monster Upgrade Page
//!
//! Allows upgrading monster stat bonuses using crystals.
//! Uses Pokemon EV-style bonuses: 0-50 points per stat.

use crate::display::St7789pDriver;
use crate::game::core::{Element, Monster, MAX_STAT_BONUS};
use crate::game::systems::progression::upgrade::upgrade_cost_crystals;
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_6X10, FONT_7X13}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Rectangle, RoundedRectangle, PrimitiveStyleBuilder, CornerRadii},
    text::Text,
};
use std::error::Error;

/// Action from upgrade page
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonsterUpgradeAction {
    /// No action
    None,
    /// Go back
    Back,
    /// Upgrade ATK bonus (+1)
    UpgradeAtk,
    /// Upgrade DEF bonus (+1)
    UpgradeDef,
    /// Upgrade SPD bonus (+1)
    UpgradeSpd,
    /// Upgrade HP bonus (+1, gives +10 HP)
    UpgradeHp,
}

/// Monster upgrade page
pub struct MonsterUpgradePage {
    // Monster data
    monster_id: String,
    name: String,
    element: Element,
    // Base stats (without bonuses)
    atk_base: u16,
    def_base: u16,
    spd_base: u16,
    hp_base: u16,
    // Bonus stats (0-50 each)
    atk_bonus: u8,
    def_bonus: u8,
    spd_bonus: u8,
    hp_bonus: u8,

    // Player resources
    crystals: u32,

    // Costs (based on current bonus value)
    atk_cost: u32,
    def_cost: u32,
    spd_cost: u32,
    hp_cost: u32,

    // Touch areas
    back_area: Option<Rectangle>,
    atk_button: Option<Rectangle>,
    def_button: Option<Rectangle>,
    spd_button: Option<Rectangle>,
    hp_button: Option<Rectangle>,

    dirty: bool,
}

impl MonsterUpgradePage {
    pub fn new(monster: &Monster, crystals: u32, _essence_count: u16) -> Self {
        Self {
            monster_id: monster.id.clone(),
            name: monster.name.clone(),
            element: monster.element,
            atk_base: monster.atk,
            def_base: monster.def,
            spd_base: monster.spd,
            hp_base: monster.hp_max,
            atk_bonus: monster.atk_bonus,
            def_bonus: monster.def_bonus,
            spd_bonus: monster.spd_bonus,
            hp_bonus: monster.hp_bonus,
            crystals,
            atk_cost: upgrade_cost_crystals(monster.atk_bonus),
            def_cost: upgrade_cost_crystals(monster.def_bonus),
            spd_cost: upgrade_cost_crystals(monster.spd_bonus),
            hp_cost: upgrade_cost_crystals(monster.hp_bonus),
            back_area: None,
            atk_button: None,
            def_button: None,
            spd_button: None,
            hp_button: None,
            dirty: true,
        }
    }

    /// Get monster ID for applying upgrades
    pub fn monster_id(&self) -> &str {
        &self.monster_id
    }

    /// Update page after upgrade
    pub fn refresh(&mut self, monster: &Monster, crystals: u32, _essence_count: u16) {
        self.atk_base = monster.atk;
        self.def_base = monster.def;
        self.spd_base = monster.spd;
        self.hp_base = monster.hp_max;
        self.atk_bonus = monster.atk_bonus;
        self.def_bonus = monster.def_bonus;
        self.spd_bonus = monster.spd_bonus;
        self.hp_bonus = monster.hp_bonus;
        self.crystals = crystals;
        self.atk_cost = upgrade_cost_crystals(monster.atk_bonus);
        self.def_cost = upgrade_cost_crystals(monster.def_bonus);
        self.spd_cost = upgrade_cost_crystals(monster.spd_bonus);
        self.hp_cost = upgrade_cost_crystals(monster.hp_bonus);
        self.dirty = true;
    }

    /// Handle touch input
    pub fn handle_touch(&self, x: i32, y: i32) -> MonsterUpgradeAction {
        let point = Point::new(x, y);

        if let Some(rect) = self.back_area {
            if rect.contains(point) {
                return MonsterUpgradeAction::Back;
            }
        }

        if let Some(rect) = self.atk_button {
            if rect.contains(point) && self.crystals >= self.atk_cost && self.atk_bonus < MAX_STAT_BONUS {
                return MonsterUpgradeAction::UpgradeAtk;
            }
        }

        if let Some(rect) = self.def_button {
            if rect.contains(point) && self.crystals >= self.def_cost && self.def_bonus < MAX_STAT_BONUS {
                return MonsterUpgradeAction::UpgradeDef;
            }
        }

        if let Some(rect) = self.spd_button {
            if rect.contains(point) && self.crystals >= self.spd_cost && self.spd_bonus < MAX_STAT_BONUS {
                return MonsterUpgradeAction::UpgradeSpd;
            }
        }

        if let Some(rect) = self.hp_button {
            if rect.contains(point) && self.crystals >= self.hp_cost && self.hp_bonus < MAX_STAT_BONUS {
                return MonsterUpgradeAction::UpgradeHp;
            }
        }

        MonsterUpgradeAction::None
    }

    fn element_color(&self) -> Rgb888 {
        match self.element {
            Element::Fire => Rgb888::new(255, 100, 50),
            Element::Water => Rgb888::new(50, 150, 255),
            Element::Earth => Rgb888::new(150, 120, 50),
            Element::Wind => Rgb888::new(100, 220, 150),
            Element::Thunder => Rgb888::new(255, 255, 100),
            Element::Shadow => Rgb888::new(150, 50, 200),
            Element::Holy => Rgb888::new(255, 255, 200),
            Element::Ghost => Rgb888::new(180, 180, 220),
            Element::Neutral => Rgb888::new(180, 180, 180),
        }
    }

    fn element_char(&self) -> char {
        match self.element {
            Element::Fire => 'F',
            Element::Water => 'W',
            Element::Earth => 'E',
            Element::Wind => 'N',
            Element::Thunder => 'T',
            Element::Shadow => 'S',
            Element::Holy => 'H',
            Element::Ghost => 'G',
            Element::Neutral => 'N',
        }
    }
}

impl Page for MonsterUpgradePage {
    fn draw(&mut self, display: &mut St7789pDriver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        let title_style = MonoTextStyle::new(&FONT_7X13, Rgb888::BLACK);
        let dim_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(100, 100, 100));
        let green_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(50, 150, 50));

        if full_redraw {
            // Light theme background
            let bg = Rectangle::new(Point::new(0, 0), Size::new(240, 284));
            display.fill_solid(&bg, Rgb888::new(240, 240, 245))?;
        }

        // Header with rounded background
        let header_rect = Rectangle::new(Point::new(10, 4), Size::new(220, 24));
        let header_rounded = RoundedRectangle::new(header_rect, CornerRadii::new(Size::new(6, 6)));
        header_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(180, 150, 100))
            .build())
            .draw(display)?;

        Text::new("UPGRADE", Point::new(90, 20), title_style).draw(display)?;

        // Monster name with element
        let elem_style = MonoTextStyle::new(&FONT_6X10, self.element_color());
        let name = if self.name.len() > 14 { &self.name[..14] } else { &self.name };
        let monster_text = format!("[{}] {}", self.element_char(), name);
        Text::new(&monster_text, Point::new(70, 20), elem_style).draw(display)?;

        // Stat upgrade buttons - compact layout
        let button_y_start = 32;
        let button_height = 32u32;
        let button_spacing = 4;

        // ATK upgrade - show "base + bonus" format
        let atk_y = button_y_start;
        let can_atk = self.crystals >= self.atk_cost && self.atk_bonus < MAX_STAT_BONUS;
        self.atk_button = Some(self.draw_stat_button(
            display, "ATK", self.atk_base, self.atk_bonus, self.atk_cost, can_atk, atk_y
        )?);

        // DEF upgrade
        let def_y = atk_y + button_height as i32 + button_spacing;
        let can_def = self.crystals >= self.def_cost && self.def_bonus < MAX_STAT_BONUS;
        self.def_button = Some(self.draw_stat_button(
            display, "DEF", self.def_base, self.def_bonus, self.def_cost, can_def, def_y
        )?);

        // SPD upgrade
        let spd_y = def_y + button_height as i32 + button_spacing;
        let can_spd = self.crystals >= self.spd_cost && self.spd_bonus < MAX_STAT_BONUS;
        self.spd_button = Some(self.draw_stat_button(
            display, "SPD", self.spd_base, self.spd_bonus, self.spd_cost, can_spd, spd_y
        )?);

        // HP upgrade (bonus gives +10 HP per point)
        let hp_y = spd_y + button_height as i32 + button_spacing;
        let can_hp = self.crystals >= self.hp_cost && self.hp_bonus < MAX_STAT_BONUS;
        self.hp_button = Some(self.draw_hp_button(
            display, self.hp_base, self.hp_bonus, self.hp_cost, can_hp, hp_y
        )?);

        // Resources card
        let res_y = hp_y + button_height as i32 + 8;
        let res_rect = Rectangle::new(Point::new(10, res_y), Size::new(220, 28));
        let res_rounded = RoundedRectangle::new(res_rect, CornerRadii::new(Size::new(8, 8)));
        res_rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(250, 250, 255))
            .build())
            .draw(display)?;
        res_rounded.into_styled(PrimitiveStyleBuilder::new()
            .stroke_color(Rgb888::new(180, 185, 195))
            .stroke_width(1)
            .build())
            .draw(display)?;

        Text::new("Resources:", Point::new(18, res_y + 18), dim_style).draw(display)?;

        let crystal_text = format!("Crystals: {}", self.crystals);
        Text::new(&crystal_text, Point::new(100, res_y + 18), green_style).draw(display)?;

        // No back button (use BOOT)
        self.back_area = None;

        display.flush()?;
        self.dirty = false;
        Ok(())
    }

    fn update(&mut self) -> bool {
        true
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn needs_full_redraw(&self) -> bool {
        self.dirty
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl MonsterUpgradePage {
    /// Draw stat button showing "base + bonus" format
    fn draw_stat_button(
        &self,
        display: &mut St7789pDriver,
        stat_name: &str,
        base: u16,
        bonus: u8,
        cost: u32,
        can_afford: bool,
        y: i32,
    ) -> Result<Rectangle, Box<dyn Error>> {
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::BLACK);
        let dim_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(120, 120, 120));
        let green_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(50, 150, 50));
        let bonus_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(80, 180, 80));

        let (bg_color, border_color) = if can_afford {
            (Rgb888::new(200, 230, 200), Rgb888::new(100, 180, 100))
        } else {
            (Rgb888::new(220, 220, 225), Rgb888::new(180, 180, 185))
        };

        let rect = Rectangle::new(Point::new(10, y), Size::new(220, 32));
        let rounded = RoundedRectangle::new(rect, CornerRadii::new(Size::new(6, 6)));

        // Fill
        rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(bg_color)
            .build())
            .draw(display)?;

        // Border
        rounded.into_styled(PrimitiveStyleBuilder::new()
            .stroke_color(border_color)
            .stroke_width(1)
            .build())
            .draw(display)?;

        // Stat name and base value
        let style = if can_afford { text_style } else { dim_style };
        let stat_text = format!("{}: {}", stat_name, base);
        Text::new(&stat_text, Point::new(18, y + 14), style).draw(display)?;

        // Bonus value in green "+X"
        let bonus_text = format!("+{}", bonus);
        let b_style = if can_afford { bonus_style } else { dim_style };
        Text::new(&bonus_text, Point::new(70, y + 14), b_style).draw(display)?;

        // Show new bonus value after upgrade
        let new_bonus_text = format!("-> +{}", bonus + 1);
        Text::new(&new_bonus_text, Point::new(100, y + 14), dim_style).draw(display)?;

        // Progress bar for bonus (0-50)
        let bar_width = 60u32;
        let bar_height = 6u32;
        let bar_x = 18;
        let bar_y = y + 22;
        let fill_width = (bonus as u32 * bar_width / MAX_STAT_BONUS as u32) as u32;

        // Bar background
        let bar_bg = Rectangle::new(Point::new(bar_x, bar_y), Size::new(bar_width, bar_height));
        display.fill_solid(&bar_bg, Rgb888::new(180, 180, 185))?;

        // Bar fill
        if fill_width > 0 {
            let bar_fill = Rectangle::new(Point::new(bar_x, bar_y), Size::new(fill_width, bar_height));
            display.fill_solid(&bar_fill, Rgb888::new(80, 180, 80))?;
        }

        // Bonus count text
        let count_text = format!("{}/50", bonus);
        Text::new(&count_text, Point::new(82, y + 26), dim_style).draw(display)?;

        // Cost
        let cost_text = format!("{}", cost);
        let cost_style = if can_afford { green_style } else { dim_style };
        Text::new(&cost_text, Point::new(190, y + 20), cost_style).draw(display)?;

        Ok(rect)
    }

    /// Draw HP button (bonus gives +10 HP per point)
    fn draw_hp_button(
        &self,
        display: &mut St7789pDriver,
        base: u16,
        bonus: u8,
        cost: u32,
        can_afford: bool,
        y: i32,
    ) -> Result<Rectangle, Box<dyn Error>> {
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::BLACK);
        let dim_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(120, 120, 120));
        let green_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(50, 150, 50));
        let bonus_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(80, 180, 80));

        let (bg_color, border_color) = if can_afford {
            (Rgb888::new(200, 230, 200), Rgb888::new(100, 180, 100))
        } else {
            (Rgb888::new(220, 220, 225), Rgb888::new(180, 180, 185))
        };

        let rect = Rectangle::new(Point::new(10, y), Size::new(220, 32));
        let rounded = RoundedRectangle::new(rect, CornerRadii::new(Size::new(6, 6)));

        // Fill
        rounded.into_styled(PrimitiveStyleBuilder::new()
            .fill_color(bg_color)
            .build())
            .draw(display)?;

        // Border
        rounded.into_styled(PrimitiveStyleBuilder::new()
            .stroke_color(border_color)
            .stroke_width(1)
            .build())
            .draw(display)?;

        // HP base value
        let style = if can_afford { text_style } else { dim_style };
        let stat_text = format!("HP: {}", base);
        Text::new(&stat_text, Point::new(18, y + 14), style).draw(display)?;

        // HP bonus in green "+X0" (each point gives +10 HP)
        let bonus_hp = bonus as u16 * 10;
        let bonus_text = format!("+{}", bonus_hp);
        let b_style = if can_afford { bonus_style } else { dim_style };
        Text::new(&bonus_text, Point::new(70, y + 14), b_style).draw(display)?;

        // Show new bonus value after upgrade
        let new_bonus_text = format!("-> +{}", bonus_hp + 10);
        Text::new(&new_bonus_text, Point::new(110, y + 14), dim_style).draw(display)?;

        // Progress bar for bonus (0-50)
        let bar_width = 60u32;
        let bar_height = 6u32;
        let bar_x = 18;
        let bar_y = y + 22;
        let fill_width = (bonus as u32 * bar_width / MAX_STAT_BONUS as u32) as u32;

        // Bar background
        let bar_bg = Rectangle::new(Point::new(bar_x, bar_y), Size::new(bar_width, bar_height));
        display.fill_solid(&bar_bg, Rgb888::new(180, 180, 185))?;

        // Bar fill
        if fill_width > 0 {
            let bar_fill = Rectangle::new(Point::new(bar_x, bar_y), Size::new(fill_width, bar_height));
            display.fill_solid(&bar_fill, Rgb888::new(80, 180, 80))?;
        }

        // Bonus count text
        let count_text = format!("{}/50", bonus);
        Text::new(&count_text, Point::new(82, y + 26), dim_style).draw(display)?;

        // Cost
        let cost_text = format!("{}", cost);
        let cost_style = if can_afford { green_style } else { dim_style };
        Text::new(&cost_text, Point::new(190, y + 20), cost_style).draw(display)?;

        Ok(rect)
    }
}
