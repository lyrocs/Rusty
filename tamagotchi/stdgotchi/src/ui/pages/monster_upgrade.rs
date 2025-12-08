//! Monster Upgrade Page
//!
//! Allows upgrading monster stats using crystals and essences.

use crate::display::Sh8601Driver;
use crate::game::core::{Element, Monster};
use crate::game::systems::progression::upgrade::{upgrade_cost_crystals, major_upgrade_cost, MAX_STAT};
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_9X15, FONT_10X20}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyle},
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
    /// Upgrade ATK (+1)
    UpgradeAtk,
    /// Upgrade DEF (+1)
    UpgradeDef,
    /// Upgrade SPD (+1)
    UpgradeSpd,
    /// Upgrade HP (+10)
    UpgradeHp,
    /// Major upgrade ATK (+10)
    MajorUpgradeAtk,
}

/// Monster upgrade page
pub struct MonsterUpgradePage {
    // Monster data
    monster_id: String,
    name: String,
    element: Element,
    atk: u16,
    def: u16,
    spd: u16,
    hp_max: u16,

    // Player resources
    crystals: u32,
    essence_count: u16,

    // Costs
    atk_cost: u32,
    def_cost: u32,
    spd_cost: u32,
    hp_cost: u32,
    major_atk_cost: (u32, u8),

    // Touch areas
    back_area: Option<Rectangle>,
    atk_button: Option<Rectangle>,
    def_button: Option<Rectangle>,
    spd_button: Option<Rectangle>,
    hp_button: Option<Rectangle>,
    major_atk_button: Option<Rectangle>,

    dirty: bool,
}

impl MonsterUpgradePage {
    pub fn new(monster: &Monster, crystals: u32, essence_count: u16) -> Self {
        Self {
            monster_id: monster.id.clone(),
            name: monster.name.clone(),
            element: monster.element,
            atk: monster.atk,
            def: monster.def,
            spd: monster.spd,
            hp_max: monster.hp_max,
            crystals,
            essence_count,
            atk_cost: upgrade_cost_crystals(monster.atk),
            def_cost: upgrade_cost_crystals(monster.def),
            spd_cost: upgrade_cost_crystals(monster.spd),
            hp_cost: upgrade_cost_crystals(monster.hp_max),
            major_atk_cost: major_upgrade_cost(monster.atk),
            back_area: None,
            atk_button: None,
            def_button: None,
            spd_button: None,
            hp_button: None,
            major_atk_button: None,
            dirty: true,
        }
    }

    /// Get monster ID for applying upgrades
    pub fn monster_id(&self) -> &str {
        &self.monster_id
    }

    /// Update page after upgrade
    pub fn refresh(&mut self, monster: &Monster, crystals: u32, essence_count: u16) {
        self.atk = monster.atk;
        self.def = monster.def;
        self.spd = monster.spd;
        self.hp_max = monster.hp_max;
        self.crystals = crystals;
        self.essence_count = essence_count;
        self.atk_cost = upgrade_cost_crystals(monster.atk);
        self.def_cost = upgrade_cost_crystals(monster.def);
        self.spd_cost = upgrade_cost_crystals(monster.spd);
        self.hp_cost = upgrade_cost_crystals(monster.hp_max);
        self.major_atk_cost = major_upgrade_cost(monster.atk);
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
            if rect.contains(point) && self.crystals >= self.atk_cost && self.atk < MAX_STAT {
                return MonsterUpgradeAction::UpgradeAtk;
            }
        }

        if let Some(rect) = self.def_button {
            if rect.contains(point) && self.crystals >= self.def_cost && self.def < MAX_STAT {
                return MonsterUpgradeAction::UpgradeDef;
            }
        }

        if let Some(rect) = self.spd_button {
            if rect.contains(point) && self.crystals >= self.spd_cost && self.spd < MAX_STAT {
                return MonsterUpgradeAction::UpgradeSpd;
            }
        }

        if let Some(rect) = self.hp_button {
            if rect.contains(point) && self.crystals >= self.hp_cost && self.hp_max < MAX_STAT {
                return MonsterUpgradeAction::UpgradeHp;
            }
        }

        if let Some(rect) = self.major_atk_button {
            if rect.contains(point)
                && self.crystals >= self.major_atk_cost.0
                && self.essence_count >= self.major_atk_cost.1 as u16
                && self.atk + 10 <= MAX_STAT
            {
                return MonsterUpgradeAction::MajorUpgradeAtk;
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
        }
    }
}

impl Page for MonsterUpgradePage {
    fn draw(&mut self, display: &mut Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0));
        let text_style = MonoTextStyle::new(&FONT_9X15, Rgb888::WHITE);
        let dim_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(150, 150, 150));
        let green_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(100, 200, 100));
        let red_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(200, 100, 100));

        if full_redraw {
            let bg = Rectangle::new(Point::new(0, 0), Size::new(368, 448));
            display.fill_solid(&bg, Rgb888::new(20, 25, 35))?;
        }

        // Header
        Text::new("UPGRADE", Point::new(140, 35), title_style).draw(display)?;

        // Monster name with element
        let elem_style = MonoTextStyle::new(&FONT_10X20, self.element_color());
        let monster_text = format!("[{}] {}", self.element_char(), self.name);
        Text::new(&monster_text, Point::new(100, 65), elem_style).draw(display)?;

        // Stat upgrade buttons
        let button_y_start = 100;
        let button_height = 45u32;
        let button_spacing = 10;

        // ATK upgrade
        let atk_y = button_y_start;
        let can_atk = self.crystals >= self.atk_cost && self.atk < MAX_STAT;
        self.atk_button = Some(self.draw_stat_button(
            display, "ATK", self.atk, 1, self.atk_cost, can_atk, atk_y
        )?);

        // DEF upgrade
        let def_y = atk_y + button_height as i32 + button_spacing;
        let can_def = self.crystals >= self.def_cost && self.def < MAX_STAT;
        self.def_button = Some(self.draw_stat_button(
            display, "DEF", self.def, 1, self.def_cost, can_def, def_y
        )?);

        // SPD upgrade
        let spd_y = def_y + button_height as i32 + button_spacing;
        let can_spd = self.crystals >= self.spd_cost && self.spd < MAX_STAT;
        self.spd_button = Some(self.draw_stat_button(
            display, "SPD", self.spd, 1, self.spd_cost, can_spd, spd_y
        )?);

        // HP upgrade
        let hp_y = spd_y + button_height as i32 + button_spacing;
        let can_hp = self.crystals >= self.hp_cost && self.hp_max < MAX_STAT;
        self.hp_button = Some(self.draw_stat_button(
            display, "HP", self.hp_max, 10, self.hp_cost, can_hp, hp_y
        )?);

        // Separator
        let sep_y = hp_y + button_height as i32 + 15;
        Text::new("--- Major Upgrade ---", Point::new(90, sep_y), dim_style).draw(display)?;

        // Major ATK upgrade
        let major_y = sep_y + 20;
        let (major_crystals, major_essence) = self.major_atk_cost;
        let can_major = self.crystals >= major_crystals
            && self.essence_count >= major_essence as u16
            && self.atk + 10 <= MAX_STAT;

        let major_color = if can_major {
            Rgb888::new(50, 80, 50)
        } else {
            Rgb888::new(40, 40, 40)
        };

        let major_rect = Rectangle::new(Point::new(30, major_y), Size::new(308, 50));
        display.fill_solid(&major_rect, major_color)?;
        Rectangle::new(Point::new(30, major_y), Size::new(308, 50))
            .into_styled(PrimitiveStyle::with_stroke(
                if can_major { Rgb888::new(100, 180, 100) } else { Rgb888::new(80, 80, 80) },
                2
            ))
            .draw(display)?;

        let major_text = format!("ATK {} -> {} [+10]", self.atk, self.atk + 10);
        let major_style = if can_major { text_style } else { dim_style };
        Text::new(&major_text, Point::new(45, major_y + 22), major_style).draw(display)?;

        let cost_text = format!("{} + {}x{}", major_crystals, self.element_char(), major_essence);
        Text::new(&cost_text, Point::new(220, major_y + 22), dim_style).draw(display)?;

        self.major_atk_button = Some(major_rect);

        // Resources section
        let res_y = major_y + 70;
        Text::new("Your Resources:", Point::new(30, res_y), text_style).draw(display)?;

        let crystal_text = format!("Crystals: {}", self.crystals);
        Text::new(&crystal_text, Point::new(45, res_y + 25), green_style).draw(display)?;

        let essence_text = format!("{} Essence: {}", self.element_char(), self.essence_count);
        Text::new(&essence_text, Point::new(200, res_y + 25),
            MonoTextStyle::new(&FONT_9X15, self.element_color())
        ).draw(display)?;

        // Back button
        let back_rect = Rectangle::new(Point::new(15, 405), Size::new(80, 35));
        display.fill_solid(&back_rect, Rgb888::new(80, 60, 60))?;
        Text::new("< BACK", Point::new(25, 428), text_style).draw(display)?;
        self.back_area = Some(back_rect);

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
    fn draw_stat_button(
        &self,
        display: &mut Sh8601Driver,
        stat_name: &str,
        current: u16,
        increase: u16,
        cost: u32,
        can_afford: bool,
        y: i32,
    ) -> Result<Rectangle, Box<dyn Error>> {
        let text_style = MonoTextStyle::new(&FONT_9X15, Rgb888::WHITE);
        let dim_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(150, 150, 150));
        let green_style = MonoTextStyle::new(&FONT_9X15, Rgb888::new(100, 200, 100));

        let button_color = if can_afford {
            Rgb888::new(50, 60, 50)
        } else {
            Rgb888::new(40, 40, 40)
        };

        let rect = Rectangle::new(Point::new(30, y), Size::new(308, 45));
        display.fill_solid(&rect, button_color)?;

        // Border
        Rectangle::new(Point::new(30, y), Size::new(308, 45))
            .into_styled(PrimitiveStyle::with_stroke(
                if can_afford { Rgb888::new(100, 150, 100) } else { Rgb888::new(80, 80, 80) },
                2
            ))
            .draw(display)?;

        // Stat name and values
        let style = if can_afford { text_style } else { dim_style };
        let stat_text = format!("{}: {} -> {} [+{}]", stat_name, current, current + increase, increase);
        Text::new(&stat_text, Point::new(45, y + 28), style).draw(display)?;

        // Cost
        let cost_text = format!("{}", cost);
        let cost_style = if can_afford { green_style } else { dim_style };
        Text::new(&cost_text, Point::new(280, y + 28), cost_style).draw(display)?;

        Ok(rect)
    }
}
