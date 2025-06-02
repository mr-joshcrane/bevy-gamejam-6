use avian2d::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};
use bevy_ecs_ldtk::prelude::*;

use super::collision::CollisionBundle;

pub(super) fn plugin(app: &mut App) {
    app.register_ldtk_entity::<CastleBundle>("Castle")
        .add_systems(Update, update_castle_mass)
        .add_systems(Update, (visualize_castle_sections, create_mortar_joints));
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Castle;

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
    a: Castle,
    #[sprite("images/stone.png")]
    pub sprite: Sprite,
    pub collision_bundle: CollisionBundle,
    #[grid_coords]
    pub grid_coords: GridCoords,
    pub mass: Mass,
    #[from_entity_instance]
    pub section: CastleSection,
}

fn update_castle_mass(mut commands: Commands, query: Query<Entity, Added<Castle>>) {
    for entity in &query {
        info!("Setting mass for castle entity: {:?}", entity);
        commands.entity(entity).insert(Mass(10000.0)); // Set a default mass for the castle
    }
}

fn visualize_castle_sections(mut query: Query<(&CastleSection, &mut Sprite), With<Castle>>) {
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
    mut commands: Commands,
    castle_query: Query<(Entity, &GridCoords, &CastleSection), Added<Castle>>,
) {
    // Group blocks by section
    let mut section_blocks: HashMap<String, Vec<(Entity, IVec2)>> = HashMap::new();

    // Collect all castle blocks by section
    for (entity, coords, section) in &castle_query {
        let pos = IVec2::new(coords.x, coords.y);
        section_blocks
            .entry(section.0.clone())
            .or_default()
            .push((entity, pos));
    }

    // Create joints between adjacent blocks in same section
    for (section_name, blocks) in section_blocks.iter() {
        // Build a grid map for quick lookups
        let mut grid_map: HashMap<IVec2, Entity> = HashMap::new();
        for &(entity, pos) in blocks {
            grid_map.insert(pos, entity);
        }

        // Create joints between adjacent blocks
        for &(entity, pos) in blocks {
            // Check neighbors in cardinal directions
            let directions = [
                IVec2::new(1, 0),  // Right
                IVec2::new(-1, 0), // Left
                IVec2::new(0, 1),  // Up
                IVec2::new(0, -1), // Down
            ];

            for dir in directions {
                let neighbor_pos = pos + dir;
                if let Some(&neighbor) = grid_map.get(&neighbor_pos) {
                    // Create a fixed joint directly on the entity
                    commands.spawn((
                        create_joint(entity, neighbor, dir.as_vec2()),
                        Name::new(format!("Mortar Joint: {}", section_name)),
                    ));

                    info!(
                        "Created mortar joint between blocks in section: {}",
                        section_name
                    );
                }
            }
        }
    }
}

fn create_joint(entity1: Entity, entity2: Entity, connection: Vec2) -> FixedJoint {
    FixedJoint::new(entity1, entity2)
        .with_compliance(0.000005)
        .with_linear_velocity_damping(0.1) // Some vibration damping
        .with_angular_velocity_damping(0.1)
        .with_local_anchor_1(connection * 8.0)
        .with_local_anchor_2(-connection * 8.0)
}
