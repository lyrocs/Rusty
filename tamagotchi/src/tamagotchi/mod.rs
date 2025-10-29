pub mod models;

// Re-export systems from the new location for backward compatibility
pub use crate::systems as systems;

// Re-export quest system from the new location for backward compatibility
pub use crate::quest::system as quest_system;

// Re-export ui from the new location for backward compatibility
pub use crate::ui as ui;

// Re-export data from the new location for backward compatibility
pub use crate::data as game_data;

pub use models::*;
pub use quest_system::*;
pub use systems::*;
pub use ui::*;
pub use game_data::*;
