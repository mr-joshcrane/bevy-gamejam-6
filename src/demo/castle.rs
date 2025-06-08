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
pub struct CastleSection();

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
        let _section_name = entity_instance
            .field_instances
            .iter()
            .find(|f| f.identifier == "SectionName")
            .and_then(|f| match &f.value {
                FieldValue::Strings(strings) if !strings.is_empty() => strings[0].clone(),
                _ => None,
            });

        // CastleSection(section_name.unwrap_or_else(|| "default".to_string()))
        CastleSection()
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
    query: Query<(Entity, &BlockSize, &Sprite), Added<CastleBlock>>,
) {
    if *ran_update_mass {
        return; // Prevent running this system multiple times
    }
    if query.is_empty() {
        info!("No castle blocks found to update mass.");
        return;
    }
    for (entity, block_size, sprite) in query {
        let base_mass = 100.0; // Base mass for a 16x16 block
        let area = block_size.0.x * block_size.0.y;
        let mass = base_mass * (area / (16.0 * 16.0)).sqrt();

        // let mass = 100.;
        info!("Setting mass for castle entity: {:?}", entity);
        commands.entity(entity).insert(Mass(mass)); // Set a default mass for the castle
        let desired_tile_size = 16.; // Tile size in pixels
        let stretch_value_x = desired_tile_size / block_size.0.x;
        let stretch_value_y = desired_tile_size / block_size.0.y;
        let updated_sprite = Sprite {
            image_mode: SpriteImageMode::Tiled {
                tile_x: true,
                tile_y: true,
                stretch_value: stretch_value_y.min(stretch_value_x), // Use the smaller value for consistent tiling
            },
            ..sprite.clone() // Preserve other fields
        };
        commands.entity(entity).insert(updated_sprite);
    }

    *ran_update_mass = true; // Mark that we've run this system
}

// fn visualize_castle_sections(mut query: Query<(&CastleSection, &mut Sprite), With<CastleBlock>>) {
//     // Define colors for different sections
//     let section_colors = [
//         ("Section1", Color::srgb(0.8, 0.2, 0.2)),
//         ("Section2", Color::srgb(0.2, 0.8, 0.2)),
//         ("Section3", Color::srgb(0.2, 0.2, 0.8)),
//     ];

//     // Create a HashMap for quick lookups
//     let color_map: HashMap<&str, Color> = section_colors.iter().cloned().collect();

//     // Apply colors based on section name
//     for (section, mut sprite) in &mut query {
//         // Get color for this section (or use white if not found)
//         let color = color_map
//             .get(section.0.as_str())
//             .copied()
//             .unwrap_or(Color::BLACK);

//         // Apply the color tint
//         sprite.color = color;
//     }
// }

#[derive(Debug, Copy, Clone)]
struct BlockComposite {
    entity: Entity,
    block_size: BlockSize,
    center_point: Vec2,
}

static GRID_SIZE: i32 = 16;

fn register_all_blocks_for_castle_section(
    global_grid: &mut HashMap<GridCoords, BlockComposite>,
    top_left: &GridCoords,
    entity: Entity,
    block_size: &BlockSize,
) {
    let width_normalised = block_size.0.x as i32 / GRID_SIZE;
    let depth_normalised = block_size.0.y as i32 / GRID_SIZE;
    let shape_end_x = top_left.x + width_normalised;
    let shape_end_y = top_left.y - depth_normalised;

    let center_point = Vec2::new(
        top_left.x as f32 + (width_normalised as f32 / 2.),
        top_left.y as f32 - (depth_normalised as f32 / 2.),
    );

    for x in top_left.x..shape_end_x {
        for y in shape_end_y + 1..=top_left.y {
            let bk = BlockComposite {
                entity: entity,
                block_size: block_size.clone(),
                center_point: center_point,
            };
            info!("{:?} inserted at {:?}", bk, (x, y));
            global_grid.insert(GridCoords { x, y }, bk);
        }
    }
}

fn calculate_anchor(bk1: BlockComposite, bk2: BlockComposite) -> Vec2 {
    // Translate everything relative to grid size
    let translated_other = (bk2.center_point - bk1.center_point) * GRID_SIZE as f32;
    let x_max = bk1.block_size.0.x / 2.;
    let y_max = bk1.block_size.0.y / 2.;

    if bk1.block_size.0.x > bk2.block_size.0.x {
        let mut x = translated_other.x.clamp(-x_max, y_max);
        let mut y = translated_other.y.clamp(-y_max, y_max);
        if x.abs() == x_max && y.abs() == y_max {
            if x > y {
                y = 0.0;
            } else {
                x = 0.0;
            }
        }

        Vec2::new(x, y)
    } else {
        let mut x = translated_other.x;
        let mut y = translated_other.y;
        if x.abs() > y.abs() {
            y = 0.0;
        } else {
            x = 0.0;
        }
        x = x.clamp(-x_max, x_max);
        y = y.clamp(-y_max, y_max);
        Vec2::new(x, y)
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
    let mut global_grid = HashMap::<GridCoords, BlockComposite>::new();
    // First pass: collect all blocks by section
    for (castle_entity, coords, _section, block_size) in &mut castle_query {
        register_all_blocks_for_castle_section(&mut global_grid, coords, castle_entity, block_size);
    }

    let directions = [
        GridCoords::new(1, 0),  // Right
        GridCoords::new(0, -1), // Down
    ];
    // Second pass: for each section, build grid map and create joints
    for (coordinate, block_composite) in &global_grid {
        // Detect neighbours
        // If neighbours, detect if same entity, if so pass

        for dir in directions {
            let potential_neighbor_coords = GridCoords {
                x: coordinate.x + dir.x,
                y: coordinate.y + dir.y,
            };

            let candidate = global_grid.get(&potential_neighbor_coords);

            if candidate.is_none() {
                continue;
            }
            let candidate = candidate.unwrap();
            if candidate.entity == block_composite.entity {
                continue;
            }

            let joint_id = commands
                .spawn(create_joint(*block_composite, *candidate))
                .id();
            commands.entity(block_composite.entity).add_child(joint_id);
            commands.entity(candidate.entity).add_child(joint_id);
        }
    }
    *ran_mortar_joints = true; // Mark that we've run this system
}

fn create_joint(bk1: BlockComposite, bk2: BlockComposite) -> FixedJoint {
    let anchor1 = calculate_anchor(bk1, bk2);
    let anchor2 = calculate_anchor(bk2, bk1);
    info!("Anchor point 1 {:?}", anchor1);
    info!("Anchor point 2 {:?}", anchor2);
    let mut joint = FixedJoint::new(bk1.entity, bk2.entity)
        .with_compliance(0.00001)
        .with_linear_velocity_damping(0.1) // Some vibration damping
        .with_angular_velocity_damping(0.1)
        .with_local_anchor_1(anchor1)
        .with_local_anchor_2(anchor2);

    // joint.force = Vec2::new(100000000., 100000000.);
    joint.align_torque = 10000000.0;
    joint
}

fn handle_castle_impulses(
    mut commands: Commands,
    mut castle_query: Query<
        (Entity, &ShockwaveHit, &Children),
        (With<CastleBlock>, Added<ShockwaveHit>),
    >,
) {
    const BREAKING_IMPULSE_THRESHOLD: f32 = 5000.0; // Adjust this value

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
            info!("length of child joints {:?}", child_joints.len());
            for joint_entity in child_joints {
                commands.entity(*joint_entity).despawn();
            }
        }

        // Remove the ShockwaveHit component after processing
        commands.entity(castle_entity).remove::<ShockwaveHit>();
    }
}
