//! Demo gameplay. All of these modules are only intended for demonstration
//! purposes and should be replaced with your own game logic.
//! Feel free to change the logic found here if you feel like tinkering around
//! to get a feeling for the template.

use bevy::prelude::*;

mod animation;
mod camera;
mod castle;
mod collision;
mod input;
pub mod level;
mod movement;
pub mod player;
mod walls;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        input::InputPlugin,
        animation::plugin,
        level::plugin,
        movement::plugin,
        player::plugin,
        camera::plugin,
        collision::plugin,
        walls::plugin,
        castle::plugin,
    ));
}
