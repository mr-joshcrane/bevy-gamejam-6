use avian2d::prelude::*;
use bevy::prelude::*;

use crate::asset_tracking::LoadResource;

use super::{
    animation::ExplosionAnimation, input::ActionType, movement::MovementController,
    player::CharacterController,
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<FireballCooldown>()
        .insert_resource(FireballCooldown::new(1.0))
        .load_resource::<ExplosionAssets>()
        .add_systems(Update, (update_fireballs, update_cooldowns))
        .add_systems(Update, process_fireball_actions);
}

fn update_cooldowns(time: Res<Time>, mut cooldown: ResMut<FireballCooldown>) {
    cooldown.timer.tick(time.delta());
}

#[derive(Component)]
pub struct Fireball {
    pub lifetime: Timer,
}

fn process_fireball_actions(
    mut commands: Commands,
    mut fireball_cooldown: ResMut<FireballCooldown>,
    mut controllers: Query<(&Transform, &mut CharacterController)>,
    asset_server: Res<AssetServer>,
) {
    for (transform, mut controller) in &mut controllers {
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
                CollidingEntities::default(), // Add this line!
                Name::new("Fireball"),
                Visibility::Visible,
                InheritedVisibility::default(),
                Mass(3000.0), // Adjust mass as needed
            ));
            fireball_cooldown.timer.reset();
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

#[derive(Resource, Default)]
pub struct FireballCooldown {
    pub timer: Timer,
}

impl FireballCooldown {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

#[derive(Bundle, Default)]
pub struct ExplosionBundle {
    pub animation: ExplosionAnimation,
    pub sprite: Sprite,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub name: Name,
}

impl ExplosionBundle {
    pub fn new(transform: &Transform, assets: &ResMut<ExplosionAssets>) -> Self {
        let image = assets.image_handle.clone();
        let layout = assets.layout_handle.clone();
        Self {
            animation: ExplosionAnimation::new(),
            sprite: Sprite {
                image: image,
                texture_atlas: Some(TextureAtlas {
                    layout: layout,
                    index: 0, // Start with the first frame
                }),
                ..default()
            },
            transform: transform.clone(),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::Visible,
            inherited_visibility: InheritedVisibility::default(),
            name: Name::new("Explosion"),
        }
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
pub struct ExplosionAssets {
    pub image_handle: Handle<Image>,
    pub layout_handle: Handle<TextureAtlasLayout>,
}

impl FromWorld for ExplosionAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>().clone();
        let mut layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout = TextureAtlasLayout::from_grid(
            UVec2::new(96, 96), // Frame size
            12,
            1, // 12 frames in 1 row
            None,
            None,
        );
        let layout_handle = layouts.add(layout);

        Self {
            image_handle: assets.load("images/explosion.png"),
            layout_handle,
        }
    }
}
