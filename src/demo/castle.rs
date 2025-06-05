use avian2d::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};
use bevy_ecs_ldtk::prelude::*;

use crate::demo::{balistics::ExplosionAssets, collision::ShockwaveHit, level::LdtkReady};

use super::collision::CollisionBundle;

pub(super) fn plugin(app: &mut App) {
    app.register_ldtk_entity::<CastleBundle>("Castle")
        .add_systems(Update, create_mortar_joints)
        .add_systems(
            Update,
            update_castle_mass.run_if(resource_exists::<LdtkReady>),
        )
        .add_systems(
            Update,
            (handle_castle_impulses).run_if(resource_exists::<ExplosionAssets>),
        );
}

#[derive(Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct CastleBlock {
    joints: Vec<Entity>,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct BlockSize(pub Vec2);

impl Default for BlockSize {
    fn default() -> Self {
        // Default block size, can be overridden by LDTK entity fields
        BlockSize(Vec2::new(16.0, 16.0))
    }
}
impl From<&EntityInstance> for BlockSize {
    fn from(entity_instance: &EntityInstance) -> Self {
        // LDtk puts width/height at the top level of the entity instance
        let width = entity_instance.width as f32;
        let height = entity_instance.height as f32;
        info!("Block size from LDtk entity: {} x {}", width, height);
        BlockSize(Vec2::new(width, height))
    }
}

#[derive(Component, Default, Clone, Debug)]
pub struct CastleSection(String);

// Add this implementation
impl From<&EntityInstance> for CastleSection {
    fn from(entity_instance: &EntityInstance) -> Self {
        // Extract the section name from LDTK entity fields
        for field in &entity_instance.field_instances {
            info!(
                "Found field: {} of type: {:?}",
                field.identifier, field.value
            );
        }
        let section_name = entity_instance
            .field_instances
            .iter()
            .find(|f| f.identifier == "SectionName")
            .and_then(|f| match &f.value {
                FieldValue::Strings(strings) if !strings.is_empty() => strings[0].clone(),
                _ => None,
            });

        CastleSection(section_name.unwrap_or_else(|| "default".to_string()))
    }
}

#[derive(Bundle, Default, LdtkEntity)]
pub struct CastleBundle {
    a: CastleBlock,
    #[sprite("images/stone.png")]
    pub sprite: Sprite,
    #[from_entity_instance]
    pub collision_bundle: CollisionBundle,
    #[grid_coords]
    pub grid_coords: GridCoords,
    pub mass: Mass,
    #[from_entity_instance]
    pub section: CastleSection,
    #[from_entity_instance]
    pub block_size: BlockSize,
}

fn update_castle_mass(
    mut ran_update_mass: Local<bool>,
    mut commands: Commands,
    query: Query<Entity, Added<CastleBlock>>,
) {
    if *ran_update_mass {
        return; // Prevent running this system multiple times
    }
    if query.is_empty() {
        info!("No castle blocks found to update mass.");
        return;
    }
    for entity in &query {
        info!("Setting mass for castle entity: {:?}", entity);
        commands.entity(entity).insert(Mass(10000.0)); // Set a default mass for the castle
    }
    *ran_update_mass = true; // Mark that we've run this system
}

fn visualize_castle_sections(mut query: Query<(&CastleSection, &mut Sprite), With<CastleBlock>>) {
    // Define colors for different sections
    let section_colors = [
        ("Section1", Color::srgb(0.8, 0.2, 0.2)),
        ("Section2", Color::srgb(0.2, 0.8, 0.2)),
        ("Section3", Color::srgb(0.2, 0.2, 0.8)),
    ];

    // Create a HashMap for quick lookups
    let color_map: HashMap<&str, Color> = section_colors.iter().cloned().collect();

    // Apply colors based on section name
    for (section, mut sprite) in &mut query {
        // Get color for this section (or use white if not found)
        let color = color_map
            .get(section.0.as_str())
            .copied()
            .unwrap_or(Color::BLACK);

        // Apply the color tint
        sprite.color = color;
    }
}

fn create_mortar_joints(
    mut ran_mortar_joints: Local<bool>,
    mut commands: Commands,
    mut castle_query: Query<(Entity, &GridCoords, &CastleSection, &BlockSize), Added<CastleBlock>>,
) {
    if *ran_mortar_joints {
        return; // Prevent running this system multiple times
    }
    if castle_query.is_empty() {
        info!("No castle blocks found to create mortar joints.");
        return;
    }
    info!("Creating mortar joints for castle blocks...");
    let mut section_blocks: HashMap<String, Vec<(Entity, IVec2)>> = HashMap::new();

    // First pass: collect all blocks by section
    for (castle_entity, coords, section, block_size) in &mut castle_query {
        let pos = IVec2::new(coords.x, coords.y);
        section_blocks
            .entry(section.0.clone())
            .or_default()
            .push((castle_entity, pos));
    }

    // Second pass: for each section, build grid map and create joints
    for (section_name, blocks) in &section_blocks {
        // Build a grid map for quick neighbor lookup
        let mut grid_map: HashMap<IVec2, Entity> = HashMap::new();
        for &(entity, pos) in blocks {
            grid_map.insert(pos, entity);
        }

        // Create joints between adjacent blocks
        for &(entity, pos) in blocks {
            let directions = [
                IVec2::new(1, 0),  // Right
                IVec2::new(-1, 0), // Left
                IVec2::new(0, 1),  // Up
                IVec2::new(0, -1), // Down
            ];

            for dir in directions {
                let neighbor_pos = pos + dir;
                if let Some(&neighbor) = grid_map.get(&neighbor_pos) {
                    // Avoid duplicate joints by only creating if entity < neighbor
                    if entity < neighbor {
                        let joint_entity = commands
                            .spawn((
                                create_joint(entity, neighbor, dir.as_vec2()),
                                Name::new(format!("Mortar Joint: {}", section_name)),
                            ))
                            .id();

                        commands.entity(entity).add_child(joint_entity);
                        info!(
                            "Created mortar joint between blocks in section: {}",
                            section_name
                        );
                    }
                }
            }
        }
    }

    *ran_mortar_joints = true; // Mark that we've run this system
}

fn create_joint(entity1: Entity, entity2: Entity, connection: Vec2) -> FixedJoint {
    FixedJoint::new(entity1, entity2)
        .with_compliance(0.000005)
        .with_linear_velocity_damping(0.1) // Some vibration damping
        .with_angular_velocity_damping(0.1)
        .with_local_anchor_1(connection * 8.0)
        .with_local_anchor_2(-connection * 8.0)
}

fn handle_castle_impulses(
    mut commands: Commands,
    mut castle_query: Query<
        (Entity, &ShockwaveHit, &Children),
        (With<CastleBlock>, Added<ShockwaveHit>),
    >,
) {
    const BREAKING_IMPULSE_THRESHOLD: f32 = 30000.0; // Adjust this value

    for (castle_entity, shockwave_hit, child_joints) in &mut castle_query {
        let impulse_magnitude = shockwave_hit.impulse.length();

        info!(
            "Castle {:?} received impulse, magnitude: {}",
            castle_entity, impulse_magnitude
        );

        // Check if the impulse exceeds the breaking threshold
        if impulse_magnitude > BREAKING_IMPULSE_THRESHOLD {
            info!(
                "Castle {:?} received a breaking impulse of {}",
                castle_entity, impulse_magnitude
            );
            // Clone the joints to avoid borrowing issues

            // Find all joints connected to this castle entity
            for joint_entity in child_joints {
                commands.entity(*joint_entity).despawn();
            }
        }

        // Remove the ShockwaveHit component after processing
        commands.entity(castle_entity).remove::<ShockwaveHit>();
    }
}
