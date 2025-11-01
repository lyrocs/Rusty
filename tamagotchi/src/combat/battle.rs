/// Battle state management
///
/// Contains state machines for different battle types (Whac-A-Mole, JRPG, Zelda, etc.)

/// Battle state for Whac-A-Mole style combat
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleState {
    Idle,    // Waiting to start
    Playing, // Active gameplay
    Victory, // Won the game
    Defeat,  // Lost the game
}

/// Battle state for Zelda-style timing combat
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZeldaBattleState {
    Idle,    // Waiting to start
    Playing, // Active gameplay
    Victory, // Won the game
    Defeat,  // Lost the game
}

/// Battle animation phase for manual fighting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleAnimationPhase {
    BothIdle,         // Both hero and monster idle
    MonsterAttacking, // Monster attacks (16.gif), hero gets hit (52.gif)
    HeroAttacking,    // Hero attacks (84.gif), monster gets hit (24.gif)
}

/// Circle types for Whac-A-Mole battle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircleType {
    GoodTarget,   // Green - hit for damage
    BadTarget,    // Red - avoid (penalty)
}

/// Circle entity for Whac-A-Mole battles
#[derive(Debug, Clone, Copy)]
pub struct Circle {
    pub x: i32,
    pub y: i32,
    pub radius: u16,
    pub circle_type: CircleType,
    pub spawn_time: u32, // When this circle spawned (milliseconds)
    pub lifetime: u32,   // How long circle lasts (milliseconds)
}

impl Circle {
    pub fn new(x: i32, y: i32, radius: u16, circle_type: CircleType, spawn_time: u32, lifetime: u32) -> Self {
        Self {
            x,
            y,
            radius,
            circle_type,
            spawn_time,
            lifetime,
        }
    }

    /// Check if a point (touch) is inside this circle
    pub fn contains_point(&self, px: i32, py: i32) -> bool {
        let dx = px - self.x;
        let dy = py - self.y;
        (dx * dx + dy * dy) <= (self.radius as i32 * self.radius as i32)
    }

    /// Check if this circle has expired based on current time
    pub fn is_expired(&self, current_time: u32) -> bool {
        current_time >= self.spawn_time + self.lifetime
    }
}

/// Enemy entity for Zelda-style battles
/// Enemies spawn from the right and walk towards the hero (center)
#[derive(Debug, Clone, Copy)]
pub struct ZeldaEnemy {
    pub x: i32,         // Current X position (starts at right edge ~536)
    pub y: i32,         // Y position (fixed, center of screen ~240)
    pub hp: u16,        // Remaining HP
    pub max_hp: u16,    // Maximum HP
    pub speed: i32,     // Movement speed (pixels per second)
    pub spawn_time: u32, // When this enemy spawned
    pub is_in_hit_zone: bool, // Whether enemy is in the timing hit zone
    pub is_hit: bool,   // Whether player successfully hit this enemy
}

impl ZeldaEnemy {
    pub fn new(x: i32, y: i32, hp: u16, speed: i32, spawn_time: u32) -> Self {
        Self {
            x,
            y,
            hp,
            max_hp: hp,
            speed,
            spawn_time,
            is_in_hit_zone: false,
            is_hit: false,
        }
    }

    /// Update enemy position based on elapsed time
    pub fn update_position(&mut self, delta_ms: u32) {
        // Move left towards hero (center is at x=184)
        let movement = (self.speed * delta_ms as i32) / 1000;
        self.x -= movement;
    }

    /// Check if enemy has reached the hero (failed to hit)
    pub fn has_reached_hero(&self) -> bool {
        self.x < 150 // If enemy gets past the hero position
    }

    /// Check if enemy is in the hit zone (timing window for player to tap)
    pub fn check_hit_zone(&mut self, hero_x: i32, hit_zone_width: i32) {
        let distance_from_hero = (self.x - hero_x).abs();
        self.is_in_hit_zone = distance_from_hero <= hit_zone_width;
    }
}
