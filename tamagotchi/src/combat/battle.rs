/// Battle state management
///
/// Contains state machines for different battle types (Whac-A-Mole, JRPG, etc.)

/// Battle state for Whac-A-Mole style combat
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleState {
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
