/// Location types and classification
///
/// Defines different types of map locations in the game world.

/// Location type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationType {
    City,  // Cities with NPCs (Prontera, etc)
    Field, // Monster fields for hunting
}
