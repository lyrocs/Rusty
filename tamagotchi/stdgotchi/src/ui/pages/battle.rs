//! Battle Page
//!
//! Displays a map background with animated characters in battle.

use crate::display::Sh8601Driver;
use crate::ui::page::Page;
use crate::ui::sprite::{AnimatedSprite, Background};
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use std::error::Error;
use std::time::{Duration, Instant};

/// Animation types for battle entities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationType {
    Idle,
    Attack,
    Attacked,
    Death,
}

/// Entity role in battle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityRole {
    Hero,
    Enemy,
}

/// Battle entity with animation state
pub struct BattleEntity {
    idle_sprite: AnimatedSprite,
    attack_sprite: AnimatedSprite,
    attacked_sprite: AnimatedSprite,
    death_sprite: Option<AnimatedSprite>,
    current_animation: AnimationType,
    role: EntityRole,
    last_attack_time: Instant,
    attack_interval: Duration,
    is_dead: bool,
    death_time: Option<Instant>,
}

impl BattleEntity {
    /// Create a new battle entity
    pub fn new(
        role: EntityRole,
        idle_data: &[u8],
        attack_data: &[u8],
        attacked_data: &[u8],
        death_data: Option<&[u8]>,
        position: (i32, i32),
        attack_interval: Duration,
    ) -> Result<Self, Box<dyn Error>> {
        let frame_delay = Duration::from_millis(100);

        let idle_sprite = AnimatedSprite::new(idle_data, position, frame_delay, None)?;
        let attack_sprite = AnimatedSprite::new(attack_data, position, frame_delay, Some(1))?;
        let attacked_sprite = AnimatedSprite::new(attacked_data, position, frame_delay, Some(1))?;
        let death_sprite = death_data
            .map(|data| AnimatedSprite::new(data, position, frame_delay, Some(1)))
            .transpose()?;

        Ok(Self {
            idle_sprite,
            attack_sprite,
            attacked_sprite,
            death_sprite,
            current_animation: AnimationType::Idle,
            role,
            last_attack_time: Instant::now(),
            attack_interval,
            is_dead: false,
            death_time: None,
        })
    }

    /// Get current sprite based on animation state
    fn current_sprite(&self) -> &AnimatedSprite {
        match self.current_animation {
            AnimationType::Idle => &self.idle_sprite,
            AnimationType::Attack => &self.attack_sprite,
            AnimationType::Attacked => &self.attacked_sprite,
            AnimationType::Death => self.death_sprite.as_ref().unwrap_or(&self.idle_sprite),
        }
    }

    /// Get mutable current sprite
    fn current_sprite_mut(&mut self) -> &mut AnimatedSprite {
        match self.current_animation {
            AnimationType::Idle => &mut self.idle_sprite,
            AnimationType::Attack => &mut self.attack_sprite,
            AnimationType::Attacked => &mut self.attacked_sprite,
            AnimationType::Death => self.death_sprite.as_mut().unwrap_or(&mut self.idle_sprite),
        }
    }

    /// Set animation type
    fn set_animation(&mut self, animation: AnimationType) {
        if self.current_animation != animation {
            self.current_animation = animation;
            // Reset the new animation's sprite
            self.current_sprite_mut().reset_animation();
        }
    }

    /// Check if current animation is complete
    fn is_animation_complete(&self) -> bool {
        self.current_sprite().is_animation_complete()
    }

    /// Update entity state
    fn update(&mut self) {
        self.current_sprite_mut().update();
    }

    /// Draw entity
    fn draw(&self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        self.current_sprite().draw(display)
    }

    /// Get bounding box
    fn bounds(&self) -> (i32, i32, u32, u32) {
        self.current_sprite().bounds()
    }

    /// Check if it's time to attack
    fn should_attack(&self) -> bool {
        !self.is_dead && self.last_attack_time.elapsed() >= self.attack_interval
    }

    /// Trigger attack
    fn start_attack(&mut self) {
        self.set_animation(AnimationType::Attack);
        self.last_attack_time = Instant::now();
    }

    /// Trigger being attacked
    fn start_attacked(&mut self) {
        if !self.is_dead {
            self.set_animation(AnimationType::Attacked);
        }
    }

    /// Trigger death
    fn start_death(&mut self) {
        self.is_dead = true;
        self.death_time = Some(Instant::now());
        self.set_animation(AnimationType::Death);
    }

    /// Check if death animation is complete and waiting period is over
    fn is_death_complete(&self) -> bool {
        if let Some(death_time) = self.death_time {
            death_time.elapsed() >= Duration::from_secs(2) && self.is_animation_complete()
        } else {
            false
        }
    }
}

/// Enemy type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyType {
    Hornet,
    Poring,
    Fabre,
}

/// Battle page showing map and battle entities
pub struct BattlePage {
    background: Option<Background>,
    background_color: Rgb888,
    hero: Option<BattleEntity>,
    enemy: Option<BattleEntity>,
    fps: f32,
    first_draw: bool,
    enemy_types: Vec<EnemyType>,
    current_enemy_index: usize,
}

impl BattlePage {
    /// Create a new battle page with a solid color background
    ///
    /// # Arguments
    /// * `background_color` - RGB color for background
    pub fn new(background_color: Rgb888) -> Self {
        Self {
            background: None,
            background_color,
            hero: None,
            enemy: None,
            fps: 0.0,
            first_draw: true,
            enemy_types: Vec::new(),
            current_enemy_index: 0,
        }
    }

    /// Create a new battle page with a GIF background (memory intensive!)
    ///
    /// # Arguments
    /// * `map_data` - GIF data for the background map
    /// * `map_position` - Position of the map
    #[allow(dead_code)]
    pub fn new_with_background(map_data: &[u8], map_position: (i32, i32)) -> Result<Self, Box<dyn Error>> {
        let background = Background::new(map_data, map_position)?;

        Ok(Self {
            background: Some(background),
            background_color: Rgb888::BLACK,
            hero: None,
            enemy: None,
            fps: 0.0,
            first_draw: true,
            enemy_types: Vec::new(),
            current_enemy_index: 0,
        })
    }

    /// Get enemy GIF data by type
    fn get_enemy_data(enemy_type: EnemyType) -> (&'static [u8], &'static [u8], &'static [u8], Option<&'static [u8]>) {
        match enemy_type {
            EnemyType::Hornet => (
                include_bytes!("../../../assets/images/hornet/6.gif"),
                include_bytes!("../../../assets/images/hornet/22.gif"),
                include_bytes!("../../../assets/images/hornet/38.gif"),
                Some(include_bytes!("../../../assets/images/hornet/30.gif")),
            ),
            EnemyType::Poring => (
                include_bytes!("../../../assets/images/poring/6.gif"),   // idle
                include_bytes!("../../../assets/images/poring/22.gif"),  // attack
                include_bytes!("../../../assets/images/poring/6.gif"),   // attacked (reusing idle - 38.gif is 341x336, too large!)
                Some(include_bytes!("../../../assets/images/poring/30.gif")), // death
            ),
            EnemyType::Fabre => (
                include_bytes!("../../../assets/images/fabre/6.gif"),
                include_bytes!("../../../assets/images/fabre/22.gif"),
                include_bytes!("../../../assets/images/fabre/38.gif"),
                Some(include_bytes!("../../../assets/images/fabre/30.gif")),
            ),
        }
    }

    /// Add hero to the battle
    ///
    /// # Arguments
    /// * `idle_data` - GIF for idle/standing animation
    /// * `attack_data` - GIF for attack animation
    /// * `attacked_data` - GIF for being attacked animation
    /// * `position` - Position on screen
    pub fn add_hero(
        &mut self,
        idle_data: &[u8],
        attack_data: &[u8],
        attacked_data: &[u8],
        position: (i32, i32),
    ) -> Result<(), Box<dyn Error>> {
        let hero = BattleEntity::new(
            EntityRole::Hero,
            idle_data,
            attack_data,
            attacked_data,
            None, // Heroes don't die in this version
            position,
            Duration::from_secs(2), // Hero attacks every 2 seconds
        )?;

        self.hero = Some(hero);
        Ok(())
    }

    /// Add enemy to the battle by type
    ///
    /// # Arguments
    /// * `enemy_type` - Type of enemy to add
    /// * `position` - Position on screen
    pub fn add_enemy(
        &mut self,
        enemy_type: EnemyType,
        position: (i32, i32),
    ) -> Result<(), Box<dyn Error>> {
        // Get enemy data based on type
        let (idle_data, attack_data, attacked_data, death_data) = Self::get_enemy_data(enemy_type);

        let enemy = BattleEntity::new(
            EntityRole::Enemy,
            idle_data,
            attack_data,
            attacked_data,
            death_data,
            position,
            Duration::from_secs(3), // Enemy attacks every 3 seconds
        )?;

        // Store enemy type for respawning
        if self.enemy_types.is_empty() {
            self.enemy_types.push(enemy_type);
        }

        self.enemy = Some(enemy);
        Ok(())
    }

    /// Add enemy type to respawn pool (for cycling through different enemies)
    pub fn add_enemy_type_to_pool(&mut self, enemy_type: EnemyType) {
        self.enemy_types.push(enemy_type);
    }

    /// Respawn enemy with next one in pool
    fn respawn_enemy(&mut self) -> Result<(), Box<dyn Error>> {
        if self.enemy_types.is_empty() {
            return Ok(());
        }

        // Drop old enemy to free memory
        self.enemy = None;

        // Cycle to next enemy type
        self.current_enemy_index = (self.current_enemy_index + 1) % self.enemy_types.len();
        let enemy_type = self.enemy_types[self.current_enemy_index];

        log::info!("Respawning enemy: {:?}", enemy_type);

        // Calculate left-centered position for enemy
        const DISPLAY_WIDTH: i32 = 368;
        const DISPLAY_HEIGHT: i32 = 448;
        const HALF_WIDTH: i32 = DISPLAY_WIDTH / 2;

        // Position enemy on left side
        let x = HALF_WIDTH / 2;
        let y = DISPLAY_HEIGHT / 2;

        // Load enemy data on-demand
        let (idle, attack, attacked, death) = Self::get_enemy_data(enemy_type);

        let enemy = BattleEntity::new(
            EntityRole::Enemy,
            idle,
            attack,
            attacked,
            death,
            (x, y),
            Duration::from_secs(3),
        )?;

        self.enemy = Some(enemy);
        Ok(())
    }

    /// Set FPS for display
    pub fn set_fps(&mut self, fps: f32) {
        self.fps = fps;
    }

    /// Draw FPS overlay (without flushing)
    fn draw_fps_overlay(&self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        // Draw semi-transparent background box for FPS
        Rectangle::new(Point::new(5, 2), Size::new(70, 15))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0, 0, 0)))
            .draw(display)?;

        // Draw FPS text
        let mut fps_str = heapless::String::<16>::new();
        write!(fps_str, "FPS: {:.1}", self.fps).ok();

        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::YELLOW);
        Text::new(&fps_str, Point::new(10, 10), text_style).draw(display)?;

        Ok(())
    }
}

impl Page for BattlePage {
    fn update(&mut self) -> bool {
        // Check if enemy is dead and should respawn
        if let Some(enemy) = &self.enemy {
            if enemy.is_dead && enemy.is_death_complete() {
                log::info!("Enemy death complete, respawning...");
                if let Err(e) = self.respawn_enemy() {
                    log::error!("Failed to respawn enemy: {:?}", e);
                }
            }
        }

        // Check for attacks and update animations
        let hero_attacking = self.hero.as_ref().map_or(false, |h| h.should_attack());
        let enemy_attacking = self.enemy.as_ref().map_or(false, |e| e.should_attack());

        // Handle hero attack
        if hero_attacking {
            if let Some(hero) = &mut self.hero {
                hero.start_attack();
                log::info!("Hero attacks!");
            }
        }

        // Handle enemy attack
        if enemy_attacking {
            if let Some(enemy) = &mut self.enemy {
                enemy.start_attack();
                log::info!("Enemy attacks!");
            }
        }

        // Update entity animations
        if let Some(hero) = &mut self.hero {
            hero.update();

            // After attack animation completes, check if enemy should show attacked animation
            if hero.current_animation == AnimationType::Attack && hero.is_animation_complete() {
                // Return to idle after attack
                hero.set_animation(AnimationType::Idle);

                // Make enemy show attacked animation (if not attacking and not dead)
                if !enemy_attacking {
                    if let Some(enemy) = &mut self.enemy {
                        if !enemy.is_dead {
                            enemy.start_attacked();
                            // For now, enemy dies after being hit
                            // In future, add HP system
                            enemy.start_death();
                            log::info!("Enemy defeated!");
                        }
                    }
                }
            }

            // Return to idle after being attacked
            if hero.current_animation == AnimationType::Attacked && hero.is_animation_complete() {
                hero.set_animation(AnimationType::Idle);
            }
        }

        if let Some(enemy) = &mut self.enemy {
            enemy.update();

            // After attack animation completes, check if hero should show attacked animation
            if enemy.current_animation == AnimationType::Attack && enemy.is_animation_complete() {
                // Return to idle after attack
                enemy.set_animation(AnimationType::Idle);

                // Make hero show attacked animation (if not attacking)
                if !hero_attacking {
                    if let Some(hero) = &mut self.hero {
                        hero.start_attacked();
                        log::info!("Hero hit!");
                    }
                }
            }

            // Return to idle after being attacked
            if enemy.current_animation == AnimationType::Attacked && enemy.is_animation_complete() {
                if !enemy.is_dead {
                    enemy.set_animation(AnimationType::Idle);
                }
            }
        }

        // Continue running
        true
    }

    fn draw(
        &mut self,
        display: &mut Sh8601Driver,
        full_redraw: bool,
    ) -> Result<(), Box<dyn Error>> {
        use embedded_graphics::prelude::*;
        use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};

        // Draw background
        if full_redraw {
            if let Some(background) = &self.background {
                // Draw GIF background
                background.draw(display)?;
            } else {
                // Draw solid color background
                display.clear(self.background_color)?;
            }
        } else {
            // For subsequent frames, only clear and redraw entity zones
            // Add 20px padding on each side to fully clean sprite area
            const PADDING: i32 = 20;

            if let Some(hero) = &self.hero {
                let bounds = hero.bounds();
                let padded_bounds = (
                    bounds.0 - PADDING,
                    bounds.1 - PADDING,
                    bounds.2 + (PADDING * 2) as u32,
                    bounds.3 + (PADDING * 2) as u32,
                );

                if let Some(background) = &self.background {
                    background.draw_region(display, padded_bounds)?;
                } else {
                    // Clear with solid color
                    let rect = Rectangle::new(
                        Point::new(padded_bounds.0, padded_bounds.1),
                        Size::new(padded_bounds.2, padded_bounds.3),
                    );
                    rect.into_styled(
                        PrimitiveStyleBuilder::new()
                            .fill_color(self.background_color)
                            .build(),
                    )
                    .draw(display)?;
                }
            }

            if let Some(enemy) = &self.enemy {
                let bounds = enemy.bounds();
                let padded_bounds = (
                    bounds.0 - PADDING,
                    bounds.1 - PADDING,
                    bounds.2 + (PADDING * 2) as u32,
                    bounds.3 + (PADDING * 2) as u32,
                );

                if let Some(background) = &self.background {
                    background.draw_region(display, padded_bounds)?;
                } else {
                    // Clear with solid color
                    let rect = Rectangle::new(
                        Point::new(padded_bounds.0, padded_bounds.1),
                        Size::new(padded_bounds.2, padded_bounds.3),
                    );
                    rect.into_styled(
                        PrimitiveStyleBuilder::new()
                            .fill_color(self.background_color)
                            .build(),
                    )
                    .draw(display)?;
                }
            }
        }

        // Draw entities
        if let Some(hero) = &self.hero {
            hero.draw(display)?;
        }

        if let Some(enemy) = &self.enemy {
            enemy.draw(display)?;
        }

        // Draw FPS overlay (no flush)
        self.draw_fps_overlay(display)?;

        // Flush to display once at the end
        display.flush()?;

        // Mark that we've done the first draw
        if self.first_draw {
            self.first_draw = false;
        }

        Ok(())
    }

    fn on_enter(&mut self) {
        log::info!("Entering battle page");
        self.first_draw = true; // Force full redraw when entering
    }

    fn on_exit(&mut self) {
        log::info!("Exiting battle page");
    }

    fn mark_dirty(&mut self) {
        self.first_draw = true;
    }

    fn needs_full_redraw(&self) -> bool {
        self.first_draw
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
