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

use crate::{
    AppSystems, PausableSystems,
    demo::{
        animation::PlayerAnimation,
        balistics::Ability,
        player::{LightningState, Player, PlayerAssets},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<MovementController>();
    app.add_plugins(PhysicsPlugins::default());
    app.add_systems(
        Update,
        (
            movement_to_physics,
            apply_gravity,
            apply_movement_damping,
            revert_to_upright,
            movement_to_physics_lightning_mode,
            apply_lightning_mode,
            revert_lightning_mode,
        )
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems)
            .run_if(resource_exists::<PlayerAssets>),
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
            speed: 8.0,
        }
    }
}

fn movement_to_physics(
    mut query: Query<
        (&mut MovementController, Option<&mut LinearVelocity>),
        Without<LightningState>,
    >,
) {
    for (mut movement_controller, maybe_velocity) in &mut query {
        // If the entity has a LinearVelocity component, use it
        if let Some(mut velocity) = maybe_velocity {
            // Convert movement intent to velocity
            velocity.0 += movement_controller.direction * movement_controller.speed;
            movement_controller.direction = Vec2::ZERO;
        }
    }
}

fn movement_to_physics_lightning_mode(
    mut query: Query<(&mut MovementController, Option<&mut LinearVelocity>), With<LightningState>>,
) {
    for (mut movement_controller, maybe_velocity) in &mut query {
        // If the entity has a LinearVelocity component, use it
        if let Some(mut velocity) = maybe_velocity {
            // Set velocity directly to match the movement direction and speed
            velocity.0 = movement_controller.direction * movement_controller.speed;

            movement_controller.direction = Vec2::ZERO;
        }
    }
}

fn revert_to_upright(
    mut query: Query<(&mut AngularVelocity, &GlobalTransform, &RigidBody), With<Player>>,
) {
    const CORRECTION_SPEED: f32 = 0.1; // Adjust the speed of correction
    const ANGULAR_DAMPING_FACTOR: f32 = 0.95; // Optional damping factor

    for (mut angular_velocity, transform, rigid_body) in &mut query {
        if matches!(rigid_body, RigidBody::Dynamic) {
            // Get the current rotation angle (assuming 2D rotation around Z-axis)
            let current_rotation = transform.rotation().to_euler(EulerRot::XYZ).2;

            // Calculate the corrective angular velocity to move toward upright (0 radians)
            let corrective_angular_velocity = -current_rotation * CORRECTION_SPEED;

            // Apply the corrective angular velocity
            angular_velocity.0 = corrective_angular_velocity;

            // Optionally apply damping to smooth out the motion
            angular_velocity.0 *= ANGULAR_DAMPING_FACTOR;
        }
    }
}

fn apply_gravity(
    time: Res<Time>,
    mut controllers: Query<(&mut LinearVelocity,), Without<LightningState>>,
) {
    // Precision is adjusted so that the example works with
    // both the `f32` and `f64` features. Otherwise you don't need this.
    let delta_time = time.delta_secs_f64().adjust_precision();

    for mut linear_velocity in &mut controllers {
        linear_velocity.0.y += -9.8 * delta_time * 10.;
    }
}

/// Slows down movement in the X direction.
fn apply_movement_damping(
    mut query: Query<
        (&MovementController, &mut LinearVelocity),
        (Without<Ability>, Without<LightningState>),
    >,
) {
    for (_damping_factor, mut linear_velocity) in &mut query {
        // We could use `LinearDamping`, but we don't want to dampen movement along the Y axis
        linear_velocity.x *= 0.9;
    }
}

fn apply_lightning_mode(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut MovementController,
            Option<&mut PlayerAnimation>,
        ),
        Added<LightningState>,
    >,
    player_assets: Res<PlayerAssets>,
) {
    for (entity, mut movement_controller, maybe_animation) in &mut query {
        // Increase movement speed
        movement_controller.speed *= 100.0;

        // Replace the sprite with the lightning sprite
        commands.entity(entity).insert(Sprite {
            image: player_assets.lightning.clone(),
            texture_atlas: None, // No texture atlas for the lightning sprite
            ..default()
        });

        // Disable or remove the player's animation
        if let Some(_) = maybe_animation {
            commands.entity(entity).remove::<PlayerAnimation>();
        }
        commands.entity(entity).insert(Mass(1.0)); // Lightning shouldn't be able to knock down buildings.
    }
}

fn revert_lightning_mode(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut LightningState, Option<&mut PlayerAnimation>), With<Player>>,
    player_assets: Res<PlayerAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for (entity, mut lightning_state, maybe_animation) in &mut query {
        // Tick the timer
        lightning_state.timer.tick(time.delta());
        if !lightning_state.timer.finished() {
            return;
        }

        commands
            .entity(entity)
            .insert(MovementController::default());

        // Reset the player's sprite to the default duck sprite
        let player_animation = PlayerAnimation::new();
        commands.entity(entity).insert(Sprite {
            image: player_assets.ducky.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                    UVec2::splat(32),
                    6,
                    2,
                    Some(UVec2::splat(1)),
                    None,
                )),
                index: player_animation.get_atlas_index(),
            }),
            ..default()
        });

        // Re-enable the player's animation
        if maybe_animation.is_none() {
            commands.entity(entity).insert(player_animation);
        };
        commands.entity(entity).insert(Mass(30.)); // Remove the mass component
        // Remove the LightningState component
        commands.entity(entity).remove::<LightningState>();
    }
}
