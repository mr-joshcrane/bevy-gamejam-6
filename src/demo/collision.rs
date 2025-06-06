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
    explosion_assets: Res<ExplosionAssets>, // Changed from ResMut if not mutated by this system directly
    fireball_query: Query<(Entity, &CollidingEntities, &GlobalTransform), With<Fireball>>,
    // Query for all dynamic rigid bodies that could be affected by the shockwave
    mut dynamic_bodies_query: Query<(Entity, &GlobalTransform, &RigidBody), Without<Fireball>>,
) {
    for (fireball_entity, colliding_entities, fireball_gt) in &fireball_query {
        if colliding_entities.is_empty() {
            continue;
        }

        info!(
            "Fireball entity: {:?} has {} colliding entities. Creating explosion.",
            fireball_entity,
            colliding_entities.len()
        );

        // Spawn the explosion visual
        // Ensure ExplosionBundle::new takes Vec3 and &ExplosionAssets
        commands.spawn(ExplosionBundle::new(
            &fireball_gt.compute_transform(),
            &explosion_assets,
        ));

        // Apply shockwave by calling the new function
        apply_explosion_shockwave(
            &mut commands,             // Pass commands
            fireball_gt.translation(), // Use the fireball's position as the explosion origin
            &mut dynamic_bodies_query, // Pass the query for dynamic bodies
        );

        // Despawn the fireball
        commands.entity(fireball_entity).despawn();
    }
}

#[derive(Clone, Bundle, LdtkIntCell)]
pub struct CollisionBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub colliding_entities: CollidingEntities,
    pub sleeping: Sleeping,
}

impl Default for CollisionBundle {
    fn default() -> Self {
        Self {
            collider: Collider::rectangle(16.0, 16.0), // Default size for collision
            rigid_body: RigidBody::Dynamic,
            colliding_entities: CollidingEntities::default(),
            sleeping: Sleeping
        }
    }
}

impl From<&EntityInstance> for CollisionBundle {
    fn from(entity_instance: &EntityInstance) -> Self {
        let width = entity_instance.width as f32;
        let height = entity_instance.height as f32;

        info!("Block size from LDtk entity: {} x {}", width, height);

        Self {
            collider: Collider::rectangle(width, height),
            ..Default::default()
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

#[derive(Component, Debug, Clone, Copy)]
pub struct ShockwaveHit {
    pub impulse: Vec2,
}

fn apply_explosion_shockwave(
    commands: &mut Commands,
    explosion_origin_pos: Vec3,
    dynamic_bodies_query: &mut Query<(Entity, &GlobalTransform, &RigidBody), Without<Fireball>>,
) {
    info!(
        "Starting shockwave application at position: {:?}",
        explosion_origin_pos
    );
    const SHOCKWAVE_RADIUS: f32 = 200.0;
    const SHOCKWAVE_RADIUS_SQUARED: f32 = SHOCKWAVE_RADIUS * SHOCKWAVE_RADIUS;
    const SHOCKWAVE_BASE_IMPULSE: f32 = 75000.0;
    const MIN_DISTANCE_SQUARED: f32 = 0.01;

    for (target_entity, target_gt, target_rb) in dynamic_bodies_query.iter_mut() {
        // iter_mut if you might modify components, else iter
        if !matches!(target_rb, RigidBody::Dynamic) {
            continue;
        }

        let target_world_pos = target_gt.translation();
        let vector_to_target = target_world_pos - explosion_origin_pos;
        let distance_squared = vector_to_target.length_squared();

        if distance_squared < SHOCKWAVE_RADIUS_SQUARED && distance_squared > MIN_DISTANCE_SQUARED {
            let distance = distance_squared.sqrt();
            let direction_2d = (vector_to_target.truncate() / distance).normalize_or_zero();

            if direction_2d == Vec2::ZERO {
                continue;
            }

            // let falloff_factor = 1.0 - (distance / SHOCKWAVE_RADIUS); // Linear falloff
            let falloff_factor = (1.0 - (distance / SHOCKWAVE_RADIUS)).powi(2); // Quadratic falloff
            // let falloff_factor = 1.0 / (1.0 + distance_squared / (SHOCKWAVE_RADIUS * SHOCKWAVE_RADIUS)).max(0.0); // Inverse square falloff
            let impulse_magnitude = SHOCKWAVE_BASE_IMPULSE * falloff_factor;
            if impulse_magnitude <= 0.0 {
                continue;
            }

            commands.entity(target_entity).insert((
                ExternalImpulse::new(direction_2d * impulse_magnitude),
                ShockwaveHit {
                    impulse: direction_2d * impulse_magnitude,
                }, // Add the tag component with the impulse
            ));
        }
    }
}
