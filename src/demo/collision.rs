use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use rand::Rng;

use crate::demo::{
    balistics::{Ability, ExplosionBundle, FrostAssets, FrostBundle, Frostbolt},
    castle::CastleBlock,
    player::{LightningState, Player},
};

use super::balistics::{ExplosionAssets, Fireball};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, spawn_ground_sensor)
        .add_systems(Update, update_on_ground)
        .add_systems(
            Update,
            (fireball_collisions, frostbolt_collisions, apply_frostbite)
                .run_if(resource_exists::<ExplosionAssets>),
        );
}

fn fireball_collisions(
    mut commands: Commands,
    explosion_assets: Res<ExplosionAssets>, // Changed from ResMut if not mutated by this system directly
    fireball_query: Query<(Entity, &CollidingEntities, &GlobalTransform), With<Fireball>>,
    // Query for all dynamic rigid bodies that could be affected by the shockwave
    mut dynamic_bodies_query: Query<
        (Entity, &GlobalTransform, &RigidBody),
        (Without<Fireball>, Without<LightningState>),
    >,
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

fn frostbolt_collisions(
    mut commands: Commands,
    frost_assets: Res<FrostAssets>,
    frostbolt_query: Query<(Entity, &CollidingEntities, &GlobalTransform), With<Frostbolt>>,
    mut dynamic_bodies_query: Query<(Entity, &GlobalTransform, &RigidBody), Without<Ability>>,
) {
    for (frostbolt_entity, colliding_entities, frostbolt_gt) in &frostbolt_query {
        if colliding_entities.is_empty() {
            continue;
        }
        commands.spawn(FrostBundle::new(
            &frostbolt_gt.compute_transform(),
            &frost_assets,
        ));
        const CONE_RADIUS: f32 = 128.0;
        const CONE_RADIUS_SQUARED: f32 = CONE_RADIUS * CONE_RADIUS;
        const FROST_EFFECT_BASE: f32 = 200.0;

        let frostbolt_position = frostbolt_gt.translation();
        let frostbolt_direction = Vec2::new(1.0, 0.0); // Fixed direction (rightward)

        for (target_entity, target_transform, target_rb) in &mut dynamic_bodies_query {
            if !matches!(target_rb, RigidBody::Dynamic) {
                continue;
            }

            let target_position = target_transform.translation();
            let vector_to_target = target_position - frostbolt_position;
            let distance_squared = vector_to_target.length_squared();

            if distance_squared > CONE_RADIUS_SQUARED {
                continue;
            }
            info!("Inserting frost effect");
            commands
                .entity(target_entity)
                .insert(FrostEffect { magnitude: 1.0 });
            let direction_to_target = vector_to_target.truncate().normalize_or_zero();
            assert!(
                direction_to_target != Vec2::ZERO,
                "Direction to target is zero. vector_to_target: {:?}",
                vector_to_target
            );

            let angle_to_target = frostbolt_direction.angle_to(direction_to_target);
            assert!(
                !angle_to_target.is_nan(),
                "Angle to target is NaN. frostbolt_direction: {:?}, direction_to_target: {:?}",
                frostbolt_direction,
                direction_to_target
            );

            if angle_to_target.abs() <= std::f32::consts::PI / 4.0 {
                let distance = distance_squared.sqrt();
                let falloff_factor = (1.0 - (distance / CONE_RADIUS)).powi(2);
                let frost_effect_magnitude = FROST_EFFECT_BASE * falloff_factor;

                info!(
                    "Applying frost effect to entity {:?}. Distance: {}, Falloff factor: {}, Magnitude: {}",
                    target_entity, distance, falloff_factor, frost_effect_magnitude
                );

                commands.entity(target_entity).insert(FrostEffect {
                    magnitude: frost_effect_magnitude,
                });
            } else {
                info!(
                    "Entity {:?} is outside the cone angle. Angle to target: {}",
                    target_entity, angle_to_target
                );
            }
        }

        commands.entity(frostbolt_entity).despawn();
    }
}

#[derive(Clone, Bundle, LdtkIntCell)]
pub struct CollisionBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub colliding_entities: CollidingEntities,
}

impl Default for CollisionBundle {
    fn default() -> Self {
        Self {
            collider: Collider::rectangle(16.0, 16.0), // Default size for collision
            rigid_body: RigidBody::Dynamic,
            colliding_entities: CollidingEntities::default(),
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
#[derive(Component)]
pub struct FrostEffect {
    pub magnitude: f32,
}

fn apply_frostbite(
    mut commands: Commands,
    time: Res<Time>,
    mut frostbite_timer: Local<Timer>,
    mut frost_query: Query<
        (Entity, &Transform, &mut FrostEffect, &mut Sprite),
        (With<CastleBlock>, Without<Player>),
    >,
    mut adjacent_query: Query<
        (Entity, &Transform, &mut Sprite),
        (Without<FrostEffect>, With<CastleBlock>),
    >,
) {
    // Initialize the timer if it hasn't been set yet
    if frostbite_timer.elapsed_secs() == 0.0 {
        *frostbite_timer = Timer::from_seconds(2.0, TimerMode::Repeating);
    }

    frostbite_timer.tick(time.delta());

    // Only proceed if the timer has finished
    if !frostbite_timer.finished() {
        return;
    }

    const MAX_FROST_STACKS: u32 = 4;
    const SPREAD_RADIUS: f32 = 32.0; // Distance to check for adjacent entities
    const PROPAGATION_CHANCE: f32 = 0.1;

    // Collect entities to despawn after processing
    let mut entities_to_despawn = Vec::new();

    // Iterate over all frostbitten entities
    for (frostbitten_entity, frostbitten_transform, mut frost_effect, mut sprite) in
        frost_query.iter_mut()
    {
        info!(
            "Spreading frostbite from entity {:?} with magnitude: {}",
            frostbitten_entity, frost_effect.magnitude
        );

        // Adjust the sprite color progressively more blue based on frost magnitude
        let blue_intensity = (frost_effect.magnitude / MAX_FROST_STACKS as f32).clamp(0.0, 1.0);
        sprite.color = Color::srgb(1.0 - blue_intensity, 1.0 - blue_intensity, 1.0);

        // Spread frost to adjacent entities without FrostEffect
        for (adjacent_entity, adjacent_transform, mut adjacent_sprite) in adjacent_query.iter_mut()
        {
            let distance = frostbitten_transform
                .translation
                .distance(adjacent_transform.translation);

            if distance <= SPREAD_RADIUS {
                // Generate a random number and check against the propagation chance
                let mut rng = rand::thread_rng();
                let random_value: f32 = rng.r#gen();

                if random_value <= PROPAGATION_CHANCE {
                    info!(
                        "Applying frostbite to adjacent entity {:?} at distance: {}",
                        adjacent_entity, distance
                    );

                    commands.entity(adjacent_entity).insert(FrostEffect {
                        magnitude: frost_effect.magnitude + 1.0, // Increment magnitude
                    });

                    // Adjust the adjacent sprite color progressively more blue
                    let adjacent_blue_intensity =
                        ((frost_effect.magnitude + 1.0) / MAX_FROST_STACKS as f32).clamp(0.0, 1.0);
                    adjacent_sprite.color = Color::srgb(
                        1.0 - adjacent_blue_intensity,
                        1.0 - adjacent_blue_intensity,
                        1.0,
                    );
                } else {
                    info!(
                        "Frostbite propagation to entity {:?} failed (random value: {}).",
                        adjacent_entity, random_value
                    );
                }
            }
        }

        // Increment frost magnitude for the current entity
        frost_effect.magnitude += 1.0;

        // Add the entity to the despawn list if the magnitude reaches the maximum
        if frost_effect.magnitude >= MAX_FROST_STACKS as f32 {
            info!(
                "Marking frostbitten entity {:?} for despawn as it reached max frost stacks.",
                frostbitten_entity
            );
            entities_to_despawn.push(frostbitten_entity);
        }
    }

    // Despawn entities after processing
    for entity in entities_to_despawn {
        commands.entity(entity).despawn();
    }
}

fn apply_explosion_shockwave(
    commands: &mut Commands,
    explosion_origin_pos: Vec3,
    dynamic_bodies_query: &mut Query<
        (Entity, &GlobalTransform, &RigidBody),
        (Without<Fireball>, Without<LightningState>),
    >,
) {
    info!(
        "Starting shockwave application at position: {:?}",
        explosion_origin_pos
    );
    const SHOCKWAVE_RADIUS: f32 = 200.0;
    const SHOCKWAVE_RADIUS_SQUARED: f32 = SHOCKWAVE_RADIUS * SHOCKWAVE_RADIUS;
    const SHOCKWAVE_BASE_IMPULSE: f32 = 7500.0 * 5.;
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
