//! Crafting Page
//!
//! Displays recipes and allows crafting items

use crate::display::Sh8601Driver;
use crate::game::{GameData, Hero, Recipe};
use crate::ui::page::Page;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use std::error::Error;

/// Crafting page actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CraftingAction {
    SelectRecipe(usize), // Index in filtered recipes list
    Craft,
    Close,
    Back, // Return from detail view to list
}

/// View mode for crafting page
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    RecipeList,   // Showing list of recipes
    RecipeDetail, // Showing detailed view of selected recipe
}

/// Touch area
#[derive(Debug, Clone)]
struct TouchArea {
    bounds: (i32, i32, u32, u32),
    action: CraftingAction,
}

impl TouchArea {
    fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.bounds.0
            && x < self.bounds.0 + self.bounds.2 as i32
            && y >= self.bounds.1
            && y < self.bounds.1 + self.bounds.3 as i32
    }
}

/// Crafting page
pub struct CraftingPage {
    background_color: Rgb888,
    touch_areas: Vec<TouchArea>,
    needs_full_redraw: bool,
    pub current_location: String,      // City name (e.g., "prontera")
    selected_recipe_index: Option<usize>, // Selected recipe in filtered list
    scroll_offset: usize,          // Scroll position
    craft_success_item: Option<(String, u32)>, // (item_name, item_id) for confirmation dialog
    view_mode: ViewMode,          // Current view mode
}

impl CraftingPage {
    /// Create new crafting page
    pub fn new() -> Self {
        Self {
            background_color: Rgb888::new(15, 20, 30),
            touch_areas: Vec::new(),
            needs_full_redraw: true,
            current_location: String::from("prontera"),
            selected_recipe_index: None,
            scroll_offset: 0,
            craft_success_item: None,
            view_mode: ViewMode::RecipeList,
        }
    }

    /// Set the current location for crafting
    pub fn set_location(&mut self, location: String) {
        self.current_location = location;
        self.selected_recipe_index = None;
        self.scroll_offset = 0;
        self.view_mode = ViewMode::RecipeList;
        self.needs_full_redraw = true;
    }

    /// Get selected recipe index
    pub fn selected_recipe_index(&self) -> Option<usize> {
        self.selected_recipe_index
    }

    /// Select a recipe by index - switches to detail view
    pub fn select_recipe(&mut self, index: usize) {
        self.selected_recipe_index = Some(index);
        self.view_mode = ViewMode::RecipeDetail;
        self.needs_full_redraw = true;
    }

    /// Return to recipe list view
    pub fn back_to_list(&mut self) {
        self.view_mode = ViewMode::RecipeList;
        self.needs_full_redraw = true;
    }

    /// Show craft success dialog
    pub fn show_craft_success(&mut self, item_name: String, item_id: u32) {
        self.craft_success_item = Some((item_name, item_id));
        self.needs_full_redraw = true;
    }

    /// Clear craft success dialog
    pub fn clear_craft_success(&mut self) {
        self.craft_success_item = None;
        self.needs_full_redraw = true;
    }

    /// Check if success dialog is open
    pub fn is_success_dialog_open(&self) -> bool {
        self.craft_success_item.is_some()
    }

    /// Handle touch input
    pub fn handle_touch(&mut self, x: i32, y: i32) -> Option<CraftingAction> {
        for area in &self.touch_areas {
            if area.contains(x, y) {
                log::info!("Crafting action: {:?}", area.action);
                return Some(area.action);
            }
        }
        None
    }

    /// Draw crafting screen
    pub fn draw_crafting(
        &mut self,
        display: &mut Sh8601Driver,
        hero: &Hero,
        game_data: &GameData,
        full_redraw: bool,
    ) -> Result<(), Box<dyn Error>> {
        if full_redraw || self.needs_full_redraw {
            display.clear(self.background_color)?;
            self.needs_full_redraw = false;
        }

        self.touch_areas.clear();

        // Get recipes for current location
        let recipes = game_data.get_recipes_for_city(&self.current_location);

        // Check view mode
        match self.view_mode {
            ViewMode::RecipeList => {
                // Header
                self.draw_header(display)?;

                if let Some(recipes) = recipes {
                    // Recipe list
                    self.draw_recipe_list(display, hero, game_data, recipes)?;
                } else {
                    // No recipes available
                    let text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(150, 150, 150));
                    Text::new("No recipes available here", Point::new(60, 200), text_style).draw(display)?;
                }

                // Close button (increased height)
                self.draw_close_button(display)?;
            }
            ViewMode::RecipeDetail => {
                // Full-screen detail view
                if let Some(recipes) = recipes {
                    if let Some(index) = self.selected_recipe_index {
                        if let Some(recipe) = recipes.get(index) {
                            self.draw_fullscreen_recipe_detail(display, hero, game_data, recipe)?;
                        }
                    }
                }
            }
        }

        // Draw success dialog overlay if crafting was successful
        if self.craft_success_item.is_some() {
            self.draw_success_dialog(display, game_data)?;
        }

        display.flush()?;
        Ok(())
    }

    /// Draw header
    fn draw_header(&self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0));

        use core::fmt::Write;
        let mut title = heapless::String::<32>::new();
        write!(title, "CRAFTING - {}", self.current_location.to_uppercase()).ok();

        Text::new(&title, Point::new(40, 20), text_style).draw(display)?;
        Ok(())
    }

    /// Draw recipe list
    fn draw_recipe_list(
        &mut self,
        display: &mut Sh8601Driver,
        hero: &Hero,
        game_data: &GameData,
        recipes: &[Recipe],
    ) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        let margin = 10;
        let start_y = 45;
        let item_height = 55i32;
        let item_spacing = 5;
        let visible_items = 6;

        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_style_disabled = MonoTextStyle::new(&FONT_10X20, Rgb888::new(100, 100, 100));

        for (index, recipe) in recipes
            .iter()
            .skip(self.scroll_offset)
            .take(visible_items)
            .enumerate()
        {
            let y = start_y + (index as i32 * (item_height + item_spacing));
            let actual_index = index + self.scroll_offset;

            // Check if can craft
            let can_craft = self.can_craft_recipe(hero, recipe);
            let is_selected = self.selected_recipe_index == Some(actual_index);

            // Background
            let bg_color = if is_selected {
                Rgb888::new(60, 80, 100)
            } else if can_craft {
                Rgb888::new(40, 60, 40)
            } else {
                Rgb888::new(40, 40, 40)
            };

            Rectangle::new(Point::new(margin, y), Size::new(348, item_height as u32))
                .into_styled(PrimitiveStyle::with_fill(bg_color))
                .draw(display)?;

            // Recipe name
            let mut name_text = heapless::String::<32>::new();
            write!(name_text, "{}", recipe.result_item_name).ok();

            let style = if can_craft { text_style } else { text_style_disabled };
            Text::new(&name_text, Point::new(margin + 8, y + 35), style).draw(display)?;

            // Level requirement
            let level_style = if hero.level >= recipe.required_level {
                text_style
            } else {
                MonoTextStyle::new(&FONT_10X20, Rgb888::RED)
            };
            let mut level_text = heapless::String::<16>::new();
            write!(level_text, "Lv{}", recipe.required_level).ok();
            Text::new(&level_text, Point::new(285, y + 35), level_style).draw(display)?;

            // Register touch area
            self.touch_areas.push(TouchArea {
                bounds: (margin, y, 348, item_height as u32),
                action: CraftingAction::SelectRecipe(actual_index),
            });
        }

        Ok(())
    }

    /// Draw recipe details panel
    fn draw_recipe_details(
        &mut self,
        display: &mut Sh8601Driver,
        hero: &Hero,
        recipe: &Recipe,
    ) -> Result<(), Box<dyn Error>> {
        let panel_y = 230;
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
        let text_style_has = MonoTextStyle::new(&FONT_6X10, Rgb888::GREEN);
        let text_style_missing = MonoTextStyle::new(&FONT_6X10, Rgb888::RED);

        // Panel background
        Rectangle::new(Point::new(5, panel_y), Size::new(358, 120))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(25, 30, 40)))
            .draw(display)?;

        use core::fmt::Write;

        // NPC name
        let mut npc_text = heapless::String::<32>::new();
        write!(npc_text, "NPC: {}", recipe.npc).ok();
        Text::new(&npc_text, Point::new(10, panel_y + 15), text_style).draw(display)?;

        // Gold cost
        let has_gold = hero.gold >= recipe.gold_cost;
        let gold_style = if has_gold { text_style_has } else { text_style_missing };
        let mut gold_text = heapless::String::<32>::new();
        write!(gold_text, "Cost: {}G (have: {}G)", recipe.gold_cost, hero.gold).ok();
        Text::new(&gold_text, Point::new(10, panel_y + 30), gold_style).draw(display)?;

        // Materials
        Text::new("Materials:", Point::new(10, panel_y + 45), text_style).draw(display)?;

        for (i, material) in recipe.materials.iter().enumerate() {
            let y = panel_y + 60 + (i as i32 * 15);
            let has_quantity = hero.inventory.get_material_quantity(material.item_id);
            let has_enough = has_quantity >= material.quantity;
            let mat_style = if has_enough { text_style_has } else { text_style_missing };

            let mut mat_text = heapless::String::<48>::new();
            write!(
                mat_text,
                "  {}: {}/{}",
                material.name,
                has_quantity,
                material.quantity
            ).ok();
            Text::new(&mat_text, Point::new(10, y), mat_style).draw(display)?;
        }

        // Craft button
        let can_craft = self.can_craft_recipe(hero, recipe);
        let button_y = 360;
        let button_color = if can_craft {
            Rgb888::new(40, 120, 40)
        } else {
            Rgb888::new(60, 60, 60)
        };

        Rectangle::new(Point::new(10, button_y), Size::new(348, 40))
            .into_styled(PrimitiveStyle::with_fill(button_color))
            .draw(display)?;

        let button_text = if can_craft { "CRAFT" } else { "Cannot Craft" };
        let button_text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        Text::new(button_text, Point::new(130, button_y + 25), button_text_style).draw(display)?;

        if can_craft {
            self.touch_areas.push(TouchArea {
                bounds: (10, button_y, 348, 40),
                action: CraftingAction::Craft,
            });
        }

        Ok(())
    }

    /// Draw close button
    fn draw_close_button(&mut self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        let button_y = 395;
        let button_height = 50u32;
        Rectangle::new(Point::new(10, button_y), Size::new(348, button_height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(80, 40, 40)))
            .draw(display)?;

        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        Text::new("CLOSE", Point::new(150, button_y + 32), text_style).draw(display)?;

        self.touch_areas.push(TouchArea {
            bounds: (10, button_y, 348, button_height),
            action: CraftingAction::Close,
        });

        Ok(())
    }

    /// Check if hero can craft a recipe
    fn can_craft_recipe(&self, hero: &Hero, recipe: &Recipe) -> bool {
        // Check level
        if hero.level < recipe.required_level {
            return false;
        }

        // Check gold
        if hero.gold < recipe.gold_cost {
            return false;
        }

        // Check materials
        for material in &recipe.materials {
            let has = hero.inventory.get_material_quantity(material.item_id);
            if has < material.quantity {
                return false;
            }
        }

        true
    }

    /// Draw success confirmation dialog
    fn draw_success_dialog(
        &self,
        display: &mut Sh8601Driver,
        game_data: &GameData,
    ) -> Result<(), Box<dyn Error>> {
        let Some((ref item_name, item_id)) = self.craft_success_item else {
            return Ok(());
        };

        // Semi-transparent background overlay
        Rectangle::new(Point::new(0, 0), Size::new(368, 448))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0, 0, 0)))
            .draw(display)?;

        // Dialog box
        let dialog_x = 40;
        let dialog_y = 100;
        let dialog_width = 288u32;
        let dialog_height = 248u32;

        Rectangle::new(Point::new(dialog_x, dialog_y), Size::new(dialog_width, dialog_height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(30, 40, 50)))
            .draw(display)?;

        let text_style_title = MonoTextStyle::new(&FONT_10X20, Rgb888::new(100, 255, 100));
        let text_style_item = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 255, 100));
        let text_style_stats = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_style_hint = MonoTextStyle::new(&FONT_10X20, Rgb888::new(150, 150, 150));

        use core::fmt::Write;

        // Success title
        Text::new("CRAFTED!", Point::new(dialog_x + 85, dialog_y + 30), text_style_title).draw(display)?;

        // Item name
        Text::new(item_name, Point::new(dialog_x + 20, dialog_y + 65), text_style_item).draw(display)?;

        // Show equipment stats if it's equipment
        if let Some(item_data) = game_data.get_item(item_id) {
            let mut y = dialog_y + 95;

            if item_data.slot.is_some() {
                // It's equipment - show stats
                if let Some(atk) = item_data.base_atk {
                    let mut text = heapless::String::<32>::new();
                    write!(text, "ATK: +{}", atk).ok();
                    Text::new(&text, Point::new(dialog_x + 20, y), text_style_stats).draw(display)?;
                    y += 25;
                }

                if let Some(def) = item_data.base_def {
                    let mut text = heapless::String::<32>::new();
                    write!(text, "DEF: +{}", def).ok();
                    Text::new(&text, Point::new(dialog_x + 20, y), text_style_stats).draw(display)?;
                    y += 25;
                }

                if let Some(flee) = item_data.base_flee {
                    let mut text = heapless::String::<32>::new();
                    write!(text, "FLEE: +{}", flee).ok();
                    Text::new(&text, Point::new(dialog_x + 20, y), text_style_stats).draw(display)?;
                    y += 25;
                }

                if let Some(hit) = item_data.base_hit {
                    let mut text = heapless::String::<32>::new();
                    write!(text, "HIT: +{}", hit).ok();
                    Text::new(&text, Point::new(dialog_x + 20, y), text_style_stats).draw(display)?;
                }
            }
        }

        // Hint to close
        Text::new("(Tap to continue)", Point::new(dialog_x + 65, dialog_y + dialog_height as i32 - 20), text_style_hint).draw(display)?;

        Ok(())
    }

    /// Draw fullscreen recipe detail view
    fn draw_fullscreen_recipe_detail(
        &mut self,
        display: &mut Sh8601Driver,
        hero: &Hero,
        game_data: &GameData,
        recipe: &Recipe,
    ) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        let margin = 15;
        let text_style_title = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0));
        let text_style_label = MonoTextStyle::new(&FONT_10X20, Rgb888::new(200, 200, 200));
        let text_style_value = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_style_has = MonoTextStyle::new(&FONT_10X20, Rgb888::GREEN);
        let text_style_missing = MonoTextStyle::new(&FONT_10X20, Rgb888::RED);

        let mut y = margin + 15;

        // Item name
        Text::new(&recipe.result_item_name, Point::new(margin + 5, y), text_style_title).draw(display)?;
        y += 25;

        // Get item data to show stats
        if let Some(item_data) = game_data.get_item(recipe.result_item_id) {
            // Description
            if !item_data.description.is_empty() {
                Text::new(&item_data.description, Point::new(margin + 5, y), text_style_label).draw(display)?;
                y += 20;
            }

            // Equipment stats
            if item_data.slot.is_some() {
                y += 10;
                Text::new("STATS:", Point::new(margin + 5, y), text_style_title).draw(display)?;
                y += 22;

                if let Some(atk) = item_data.base_atk {
                    let mut text = heapless::String::<32>::new();
                    write!(text, "  ATK: +{}", atk).ok();
                    Text::new(&text, Point::new(margin + 5, y), text_style_value).draw(display)?;
                    y += 20;
                }

                if let Some(def) = item_data.base_def {
                    let mut text = heapless::String::<32>::new();
                    write!(text, "  DEF: +{}", def).ok();
                    Text::new(&text, Point::new(margin + 5, y), text_style_value).draw(display)?;
                    y += 20;
                }

                if let Some(flee) = item_data.base_flee {
                    let mut text = heapless::String::<32>::new();
                    write!(text, "  FLEE: +{}", flee).ok();
                    Text::new(&text, Point::new(margin + 5, y), text_style_value).draw(display)?;
                    y += 20;
                }

                if let Some(hit) = item_data.base_hit {
                    let mut text = heapless::String::<32>::new();
                    write!(text, "  HIT: +{}", hit).ok();
                    Text::new(&text, Point::new(margin + 5, y), text_style_value).draw(display)?;
                    y += 20;
                }
            }
        }

        y += 10;

        // Requirements section
        Text::new("REQUIREMENTS:", Point::new(margin + 5, y), text_style_title).draw(display)?;
        y += 22;

        // Level requirement
        let has_level = hero.level >= recipe.required_level;
        let level_style = if has_level { text_style_has } else { text_style_missing };
        let mut level_text = heapless::String::<32>::new();
        write!(level_text, "  Level: {} (You: {})", recipe.required_level, hero.level).ok();
        Text::new(&level_text, Point::new(margin + 5, y), level_style).draw(display)?;
        y += 20;

        // Gold cost
        let has_gold = hero.gold >= recipe.gold_cost;
        let gold_style = if has_gold { text_style_has } else { text_style_missing };
        let mut gold_text = heapless::String::<32>::new();
        write!(gold_text, "  Gold: {} (You: {})", recipe.gold_cost, hero.gold).ok();
        Text::new(&gold_text, Point::new(margin + 5, y), gold_style).draw(display)?;
        y += 20;

        // Materials
        y += 5;
        Text::new("  Materials:", Point::new(margin + 5, y), text_style_label).draw(display)?;
        y += 20;

        for material in &recipe.materials {
            let has_quantity = hero.inventory.get_material_quantity(material.item_id);
            let has_enough = has_quantity >= material.quantity;
            let mat_style = if has_enough { text_style_has } else { text_style_missing };

            let mut mat_text = heapless::String::<48>::new();
            write!(
                mat_text,
                "    {}: {}/{}",
                material.name,
                has_quantity,
                material.quantity
            ).ok();
            Text::new(&mat_text, Point::new(margin + 5, y), mat_style).draw(display)?;
            y += 18;
        }

        // Buttons at bottom
        let button_y = 368;
        let button_height = 55u32;
        let button_spacing = 8;
        let button_width = 170u32;

        let can_craft = self.can_craft_recipe(hero, recipe);

        // Back button (left)
        Rectangle::new(Point::new(margin, button_y), Size::new(button_width, button_height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(60, 60, 60)))
            .draw(display)?;
        Text::new("BACK", Point::new(margin + 60, button_y + 35), text_style_value).draw(display)?;

        self.touch_areas.push(TouchArea {
            bounds: (margin, button_y, button_width, button_height),
            action: CraftingAction::Back,
        });

        // Craft button (right)
        let craft_x = margin + button_width as i32 + button_spacing;
        let craft_color = if can_craft {
            Rgb888::new(40, 120, 40)
        } else {
            Rgb888::new(60, 60, 60)
        };

        Rectangle::new(Point::new(craft_x, button_y), Size::new(button_width, button_height))
            .into_styled(PrimitiveStyle::with_fill(craft_color))
            .draw(display)?;

        let button_text = if can_craft { "CRAFT" } else { "Cannot Craft" };
        let text_x = if can_craft { craft_x + 55 } else { craft_x + 20 };
        Text::new(button_text, Point::new(text_x, button_y + 35), text_style_value).draw(display)?;

        if can_craft {
            self.touch_areas.push(TouchArea {
                bounds: (craft_x, button_y, button_width, button_height),
                action: CraftingAction::Craft,
            });
        }

        Ok(())
    }
}

impl Page for CraftingPage {
    fn update(&mut self) -> bool {
        true
    }

    fn draw(
        &mut self,
        _display: &mut Sh8601Driver,
        _full_redraw: bool,
    ) -> Result<(), Box<dyn Error>> {
        // Use draw_crafting instead
        Ok(())
    }

    fn mark_dirty(&mut self) {
        self.needs_full_redraw = true;
    }

    fn needs_full_redraw(&self) -> bool {
        self.needs_full_redraw
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
