//! Handle player input and translate it into movement through a character
//! controller. A character controller is the collection of systems that govern
//! the movement of characters.
//!
//! In our case, the character controller has the following logic:
//! - Set [`MovementController`] intent based on directional keyboard input.
//!   This is done in the `player` module, as it is specific to the player
//!   character.
//! - Apply movement based on [`MovementController`] intent and maximum speed.
//! - Wrap the character within the window.
//!
//! Note that the implementation used here is limited for demonstration
//! purposes. If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/main/examples/movement/physics_in_fixed_timestep.rs).

use avian2d::{math::AdjustPrecision, prelude::*};
use bevy::prelude::*;

use crate::{AppSystems, PausableSystems};

use super::balistics::Fireball;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<MovementController>();
    app.register_type::<ScreenWrap>();
    app.add_plugins(PhysicsPlugins::default());
    app.add_plugins(PhysicsDebugPlugin::default());
    app.add_systems(
        Update,
        (movement_to_physics, apply_gravity, apply_movement_damping)
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

// Add this new component for movement-only entities
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct MovementController {
    /// The direction and intensity of movement
    pub direction: Vec2,

    /// Maximum speed in world units per second
    pub speed: f32,
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            direction: Vec2::ZERO,
            speed: 4.0,
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ScreenWrap;

fn movement_to_physics(mut query: Query<(&mut MovementController, Option<&mut LinearVelocity>)>) {
    for (mut movement_controller, maybe_velocity) in &mut query {
        // If the entity has a LinearVelocity component, use it
        if let Some(mut velocity) = maybe_velocity {
            // Convert movement intent to velocity
            velocity.0 += movement_controller.direction * movement_controller.speed;
            movement_controller.direction = Vec2::ZERO;
        }
    }
}

fn apply_gravity(
    time: Res<Time>,
    mut controllers: Query<(&MovementController, &mut LinearVelocity)>,
) {
    // Precision is adjusted so that the example works with
    // both the `f32` and `f64` features. Otherwise you don't need this.
    let delta_time = time.delta_secs_f64().adjust_precision();

    for (_gravity, mut linear_velocity) in &mut controllers {
        linear_velocity.0 += -9.8 * delta_time;
    }
}

/// Slows down movement in the X direction.
fn apply_movement_damping(
    mut query: Query<(&MovementController, &mut LinearVelocity), Without<Fireball>>,
) {
    for (_damping_factor, mut linear_velocity) in &mut query {
        // We could use `LinearDamping`, but we don't want to dampen movement along the Y axis
        linear_velocity.x *= 0.9;
    }
}
