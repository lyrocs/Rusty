use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

/// Input events from touch screen and buttons
#[derive(Debug, Clone, Copy)]
pub enum InputEvent {
    Touch { x: u16, y: u16 },
    TouchRelease,
    ButtonPress(Button),
    ButtonRelease(Button),
    Gesture(GestureType),
}

#[derive(Debug, Clone, Copy)]
pub enum Button {
    Boot,   // GPIO0 boot button
    Power,  // EXIO4 power button
}

#[derive(Debug, Clone, Copy)]
pub enum GestureType {
    SwipeUp,
    SwipeDown,
    SwipeLeft,
    SwipeRight,
    DoubleTap,
}

/// Commands for the render task
#[derive(Debug)]
pub enum RenderCommand {
    /// Request a full redraw of the current frame
    Redraw,
    /// Clear the display
    Clear,
    /// Set display brightness (0-255)
    SetBrightness(u8),
}

/// Commands for the storage/SD card task
#[derive(Debug, Clone, Copy)]
pub enum SaveCommand {
    /// Save the current game state
    SaveGame,
    /// Load game state (response will be sent back via LOAD_RESPONSE_CHANNEL)
    LoadGame,
    /// Save settings
    SaveSettings,
}

/// Response from storage operations
#[derive(Debug, Clone, Copy)]
pub enum SaveResponse {
    /// Game loaded successfully
    GameLoaded,
    /// Save/load operation failed
    Error,
    /// Operation completed successfully
    Success,
}

// Global channels for inter-task communication
// Using CriticalSectionRawMutex for no_std environments

/// Input events channel (32 events buffer - enough for burst input)
pub static INPUT_CHANNEL: Channel<CriticalSectionRawMutex, InputEvent, 32> = Channel::new();

/// Render commands channel (32 commands buffer - increased to prevent queue saturation during animations)
pub static RENDER_CHANNEL: Channel<CriticalSectionRawMutex, RenderCommand, 32> = Channel::new();

/// Storage commands channel (4 commands buffer - saves are infrequent)
pub static SAVE_CHANNEL: Channel<CriticalSectionRawMutex, SaveCommand, 4> = Channel::new();

/// Storage response channel (for load operations)
pub static LOAD_RESPONSE_CHANNEL: Channel<CriticalSectionRawMutex, SaveResponse, 2> = Channel::new();
