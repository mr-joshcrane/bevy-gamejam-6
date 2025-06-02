use avian2d::prelude::*;
use bevy::prelude::*;

use super::{input::ActionType, movement::MovementController, player::CharacterController};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, (update_fireballs, update_cooldowns))
        .add_systems(Update, process_fireball_actions);
}

// Add this system
fn update_cooldowns(time: Res<Time>, mut cooldowns: Query<&mut AbilityCooldown>) {
    for mut cooldown in &mut cooldowns {
        cooldown.fireball.tick(time.delta());
    }
}

#[derive(Component)]
pub struct Fireball {
    pub lifetime: Timer,
}

#[derive(Component)]
pub struct AbilityCooldown {
    pub fireball: Timer,
}

impl Default for AbilityCooldown {
    fn default() -> Self {
        Self {
            fireball: Timer::from_seconds(1.0, TimerMode::Once), // Cooldown duration for fireball
        }
    }
}

fn process_fireball_actions(
    mut commands: Commands,
    mut controllers: Query<(
        Entity,
        &Transform,
        &mut CharacterController,
        Option<&AbilityCooldown>,
    )>,
    asset_server: Res<AssetServer>,
) {
    for (entity, transform, mut controller, maybe_cooldown) in &mut controllers {
        if let Some(cooldown) = maybe_cooldown {
            if !cooldown.fireball.finished() {
                continue; // Skip this entity, still on cooldown
            }
        }
        if let Some(ActionType::FireballAttack { direction }) = controller.pop_action() {
            let offset_distance = 24.0; // Adjust based on your sprite sizes
            let spawn_position = transform.translation
                + Vec3::new(
                    direction.x * offset_distance,
                    direction.y * offset_distance,
                    1.0,
                );
            commands.spawn((
                Fireball {
                    lifetime: Timer::from_seconds(2.0, TimerMode::Once),
                },
                Sprite {
                    image: asset_server.load("images/fireball.png"),
                    flip_x: direction.x < 0.0, // Flip sprite based on direction
                    flip_y: false,
                    ..default()
                },
                Transform::from_translation(spawn_position),
                GlobalTransform::default(),
                RigidBody::Dynamic,
                MovementController {
                    direction,
                    speed: 900.0, // Speed of the fireball
                },
                Collider::circle(8.0),
                Name::new("Fireball"),
                Visibility::Visible,
                InheritedVisibility::default(),
                Mass(3000.0), // Adjust mass as needed
            ));
            commands.entity(entity).insert(AbilityCooldown::default());
        }
    }
}

fn update_fireballs(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Fireball)>,
) {
    for (entity, mut fireball) in &mut query {
        fireball.lifetime.tick(time.delta());
        if fireball.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}
