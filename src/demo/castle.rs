use std::{fs::File, io::Write};

use avian2d::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};
use bevy_ecs_ldtk::prelude::*;

use avian2d::math::Vector2 as Vec2;

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

fn insert_rectangle_into_map(
    map: &mut HashMap<GridCoords, (Entity, BlockSize)>,
    top_left: &GridCoords,
    entity: Entity,
    block_size: &BlockSize,
) {
    let grid_cell_size = 16;
    let cells_wide = (block_size.0.x / grid_cell_size as f32).round() as i32;
    let cells_high = (block_size.0.y / grid_cell_size as f32).round() as i32;
    for x in top_left.x..(top_left.x + cells_wide) {
        for y in (top_left.y - cells_high + 1)..=top_left.y {
            map.insert(GridCoords { x, y }, (entity, block_size.clone()));
        }
    }
}

fn grid_coords_to_vec2(coords: &GridCoords) -> Vec2 {
    Vec2::new(coords.x as f32, coords.y as f32)
}

fn create_mortar_joints(
    mut ran_mortar_joints: Local<bool>,
    mut commands: Commands,
    mut castle_query: Query<(Entity, &GridCoords, &CastleSection, &BlockSize), Added<CastleBlock>>,
) {
    if *ran_mortar_joints {
        return; // Prevent running this system multiple times
    }
    let grid_size = 1; // Size of the grid cells
    if castle_query.is_empty() {
        info!("No castle blocks found to create mortar joints.");
        return;
    }
    info!("Creating mortar joints for castle blocks...");
    let mut global_grid = HashMap::<GridCoords, (Entity, BlockSize)>::new();
    let mut block_coords = Vec::new();

    // First pass: collect all blocks by section
    for (castle_entity, coords, _section, block_size) in &mut castle_query {
        // The coords represent the center block, so we need to add every blocks position
        // using the half block size to calculate the edges, and add every block in the area
        insert_rectangle_into_map(&mut global_grid, coords, castle_entity, block_size);
        block_coords.push((coords, castle_entity, block_size.clone()));
    }
    if let Ok(mut file) = File::create("block_coords_dump.txt") {
        for (coords, entity, e1_block_size) in &block_coords {
            let line = format!("Entity: {:?}, Coords: {:?}\n", entity, coords);
            let _ = file.write_all(line.as_bytes());
        }
    }
    if let Ok(mut file) = File::create("global_grid_dump.txt") {
        for (cell, entity) in &global_grid {
            let line = format!("Cell: {:?}, Entity: {:?}\n", cell, entity);
            let _ = file.write_all(line.as_bytes());
        }
    }

    // Second pass: for each section, build grid map and create joints
    for (coordinate, castle_entity, e1_block_size) in block_coords {
        info!(
            "Processing castle entity: {:?} at coordinate: {:?}",
            castle_entity, coordinate
        );
        // Create joints between adjacent blocks

        let directions = [
            GridCoords::new(1, 0), // Right
            // GridCoords::new(-1, 0), // Left
            // GridCoords::new(0, 1),  // Up
            GridCoords::new(0, -1), // Down
        ];

        for dir in directions {
            let potential_neighbor = GridCoords {
                x: coordinate.x + dir.x,
                y: coordinate.y + dir.y,
            };
            info!(
                "Checking potential neighbor at: {:?} for entity: {:?}",
                potential_neighbor, castle_entity
            );
            let candidate = global_grid.get(&potential_neighbor);
            info!(
                "Candidate for neighbor at {:?} is: {:?}",
                potential_neighbor, candidate
            );
            if global_grid.get(&potential_neighbor).is_some() {
                let (neighbor, block_size) = global_grid[&potential_neighbor];
                if neighbor == castle_entity {
                    info!(
                        "Skipping self-reference for entity: {:?} at coordinate: {:?}",
                        neighbor, potential_neighbor
                    );
                    continue; // Skip self-reference
                }
                info!(
                    "Found neighbor entity: {:?} at coordinate: {:?} for entity: {:?}",
                    neighbor, potential_neighbor, castle_entity,
                );
                commands.spawn((create_joint(
                    castle_entity,
                    neighbor,
                    grid_coords_to_vec2(coordinate),
                    grid_coords_to_vec2(&dir),
                    e1_block_size.clone(),
                    block_size,
                ),));
            }
        }
    }

    *ran_mortar_joints = true; // Mark that we've run this system
}

fn calculate_anchor(direction: Vec2, block_size: BlockSize) -> Vec2 {
    // Calculate the second anchor point based on the connection vector and block size
    // Normalize direction to unit vector
    let norm = direction.normalize_or_zero();

    // Scale by half block size (since your block goes from -blocksize to +blocksize)
    let result = Vec2::new(
        (norm.x * block_size.0.x) / 2.,
        (norm.y * block_size.0.y) / 2.,
    );

    if (result.x > block_size.0.x / 2.0)
        || (result.x < -block_size.0.x / 2.0)
        || (result.y > block_size.0.y / 2.0)
        || (result.y < -block_size.0.y / 2.0)
    {
        warn!(
            "Calculated anchor point {:?} is outside the expected range for block size {:?}",
            result, block_size
        );
    }
    result
}

fn create_joint(
    entity1: Entity,
    entity2: Entity,
    anchor_point: Vec2,
    connection: Vec2,
    block_size_one: BlockSize,
    block_size_two: BlockSize,
) -> FixedJoint {
    let anchor_point_one = calculate_anchor(connection, block_size_one);
    let anchor_point_two = -calculate_anchor(connection, block_size_two);

    info!(
        "Creating joint between {:?} and {:?} at anchor {:?} with connection {:?}",
        entity1, entity2, anchor_point, anchor_point_two,
    );
    FixedJoint::new(entity1, entity2)
        .with_compliance(1.0)
        .with_linear_velocity_damping(0.1) // Some vibration damping
        .with_angular_velocity_damping(0.1)
        .with_local_anchor_1(anchor_point_one)
        .with_local_anchor_2(anchor_point_two)
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
