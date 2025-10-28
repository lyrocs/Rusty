// Render command system

use bevy_ecs::prelude::*;
use crossbeam_channel::Sender;
use crate::types::{RenderCommand, Color};

/// Resource to send render commands to render thread
#[derive(Resource)]
pub struct RenderCommandSender(pub Sender<RenderCommand>);

/// Send render commands for the current frame
pub fn send_render_commands_system(
    render_tx: Res<RenderCommandSender>,
) {
    // Clear screen
    render_tx.0.send(RenderCommand::Clear).ok();

    // TODO: Send actual render commands based on game state

    // Example: Draw a test rectangle
    render_tx.0.send(RenderCommand::DrawRect {
        x: 50,
        y: 50,
        width: 100,
        height: 100,
        color: Color::GREEN,
    }).ok();

    // Present the frame
    render_tx.0.send(RenderCommand::Present).ok();
}
