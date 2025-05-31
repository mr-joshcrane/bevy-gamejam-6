use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use super::{movement::MovementController};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EnhancedInputPlugin);
        app.add_input_context::<PlatformerContext>();
        app.add_observer(binding);
        app.add_observer(record_player_directional_input);
    }
}

#[derive(InputContext)]
pub struct PlatformerContext;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
pub struct LateralMovement;

fn binding(
    trigger: Trigger<Binding<PlatformerContext>>,
    mut actions: Query<&mut Actions<PlatformerContext>>,
) {
    let mut action = actions.get_mut(trigger.target()).unwrap();
    action.bind::<LateralMovement>().to(Cardinal {
        north: KeyCode::ArrowUp,
        south: KeyCode::ArrowDown,
        east: KeyCode::ArrowRight,
        west: KeyCode::ArrowLeft,
    });
}

fn record_player_directional_input(
    trigger: Trigger<Fired<LateralMovement>>,
    mut controller_query: Query<&mut MovementController>,
) {
    // Collect directional input.
    let mut move_controller = controller_query.get_mut(trigger.target()).unwrap();
    let intent = trigger.value;
    move_controller.intent = intent.normalize_or_zero();
}
