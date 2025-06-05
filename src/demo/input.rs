use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use super::{
    balistics::FireballCooldown, movement::MovementController, player::CharacterController,
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EnhancedInputPlugin);
        app.add_input_context::<PlatformerContext>();
        app.add_observer(binding);
        app.add_observer(record_player_fire_input);
        app.add_observer(record_player_directional_input);
    }
}

#[derive(InputContext, Clone)]
pub struct PlatformerContext;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
pub struct LateralMovement;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct FireAction;

#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub enum ActionType {
    FireballAttack { direction: Vec2 },
}

fn binding(
    trigger: Trigger<Binding<PlatformerContext>>,
    mut actions: Query<&mut Actions<PlatformerContext>>,
) {
    let mut action = actions.get_mut(trigger.target()).unwrap();
    action.bind::<LateralMovement>().to((Cardinal {
        north: KeyCode::ArrowUp,
        south: KeyCode::ArrowDown,
        east: KeyCode::ArrowRight,
        west: KeyCode::ArrowLeft,
    },));
    action.bind::<FireAction>().to(KeyCode::Space);
}

fn record_player_directional_input(
    trigger: Trigger<Fired<LateralMovement>>,
    mut controller_query: Query<&mut MovementController>,
) {
    // Collect directional input.
    let mut move_controller = controller_query.get_mut(trigger.target()).unwrap();
    let intent = trigger.value;
    move_controller.direction = intent.normalize_or_zero();
}

fn record_player_fire_input(
    trigger: Trigger<Started<FireAction>>,
    cooldown: Res<FireballCooldown>,
    mut controller_query: Query<(&mut CharacterController, &MovementController)>,
) {
    if !cooldown.timer.finished() {
        // If the timer is not finished, the ability is on cooldown
        return;
    }
    let (mut character_controller, movement_controller) =
        controller_query.get_mut(trigger.target()).unwrap();

    // Determine direction based on movement controller
    let direction = if movement_controller.direction.length_squared() > 0.0 {
        // Use the current movement direction if moving
        movement_controller.direction.normalize_or_zero()
    } else {
        // Default to last non-zero x direction or right if none
        Vec2::new(1.0, 0.0)
    };

    // Queue the action with directional information
    character_controller.queue_action(ActionType::FireballAttack { direction });
}
