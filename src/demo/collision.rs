use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::demo::balistics::ExplosionBundle;

use super::balistics::{ExplosionAssets, Fireball};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, spawn_ground_sensor)
        .add_systems(Update, update_on_ground)
        .add_systems(
            Update,
            fireball_collisions.run_if(resource_exists::<ExplosionAssets>),
        );
}

fn fireball_collisions(
    mut commands: Commands,
    explosion_assets: ResMut<ExplosionAssets>,
    query: Query<(Entity, &CollidingEntities, &Transform), With<Fireball>>,
) {
    for (entity, colliding_entities, transform) in &query {
        info!(
            "Fireball entity: {:?} has colliding entities: {:?}",
            entity, colliding_entities
        );
        if colliding_entities.is_empty() {
            continue;
        } else {
            info!(
                "Fireball entity: {:?} has {} colliding entities",
                entity,
                colliding_entities.len()
            );
            commands.spawn(ExplosionBundle::new(transform, &explosion_assets));
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Clone, Bundle, LdtkIntCell)]
pub struct CollisionBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub colloding_entities: CollidingEntities,
}

impl Default for CollisionBundle {
    fn default() -> Self {
        Self {
            collider: Collider::rectangle(16.0, 16.0), // Default size for collision
            rigid_body: RigidBody::Dynamic,
            colloding_entities: CollidingEntities::default(),
        }
    }
}

#[derive(Clone, Bundle, Default, LdtkEntity)]
pub struct HeroCollisionBundle {
    pub collision_bundle: CollisionBundle,
    pub ground_detection: GroundDetection,
}

#[derive(Clone, Default, Component)]
pub struct GroundDetection {
    pub on_ground: bool,
}

pub fn spawn_ground_sensor(
    mut commands: Commands,
    detect_ground_for: Query<Entity, Added<GroundDetection>>,
) {
    for entity in &detect_ground_for {
        // Create a shape caster with a fixed width relative to the entity
        let entity_width = 16.0; // Use a reasonable default width or extract from transform
        let sensor_width = entity_width * 0.8; // Slightly smaller than entity

        // Create a shape caster for ground detection
        let ground_caster = ShapeCaster::new(
            Collider::rectangle(sensor_width, 2.0), // Thin rectangle for ground detection
            Vec2::new(0.0, -10.0),                  // Offset below the entity
            0.0,                                    // No rotation
            Dir2::NEG_Y,                            // Cast downward
        )
        .with_max_distance(5.0); // Detection distance

        commands.entity(entity).insert(ground_caster);
    }
}
// Then update the ground detection function to use ShapeHits:
pub fn update_on_ground(mut ground_detectors: Query<(&mut GroundDetection, &ShapeHits)>) {
    for (mut ground_detection, hits) in &mut ground_detectors {
        ground_detection.on_ground = !hits.is_empty();
    }
}
