use std::time::Duration;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    asset_tracking::LoadResource,
    demo::player::{LightningState, Player},
};

use super::{
    animation::ExplosionAnimation, input::ActionType, movement::MovementController,
    player::CharacterController,
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<FireballCooldown>()
        .init_resource::<FrostCooldown>()
        .init_resource::<LightningCooldown>()
        .insert_resource(FrostCooldown::new(1.0))
        .insert_resource(FireballCooldown::new(0.5))
        .insert_resource(LightningCooldown::new(5.0))
        .load_resource::<ExplosionAssets>()
        .load_resource::<FrostAssets>()
        .add_systems(Update, (update_abilities, update_cooldowns))
        .add_systems(Update, process_ability_actions);
}

fn update_cooldowns(
    time: Res<Time>,
    mut fire_cooldown: ResMut<FireballCooldown>,
    mut frost_cooldown: ResMut<FrostCooldown>,
    mut lightning_cooldown: ResMut<LightningCooldown>,
) {
    fire_cooldown.timer.tick(time.delta());
    frost_cooldown.timer.tick(time.delta());
    lightning_cooldown.timer.tick(time.delta());
}

#[derive(Component)]
pub struct Ability;

#[derive(Component)]
pub struct Fireball;

#[derive(Component)]
pub struct Frostbolt;

#[derive(Component)]
pub struct LightningBolt;

#[derive(Component)]
pub struct Lifetime {
    pub lifetime: Timer,
}

#[derive(Bundle)]
pub struct FireballBundle {
    pub fireball: Fireball,
    pub ability: Ability,
    pub lifetime: Lifetime,
    pub sprite: Sprite,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub rigid_body: RigidBody,
    pub movement_controller: MovementController,
    pub collider: Collider,
    pub colliding_entities: CollidingEntities,
    pub name: Name,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub mass: Mass,
}

fn create_fireball_bundle(
    spawn_position: Vec3,
    direction: Vec2,
    asset_server: &Res<AssetServer>,
) -> FireballBundle {
    FireballBundle {
        fireball: Fireball,
        ability: Ability,
        lifetime: Lifetime {
            lifetime: Timer::from_seconds(2.0, TimerMode::Once),
        },
        sprite: Sprite {
            image: asset_server.load("images/fireball.png"),
            flip_x: direction.x < 0.0,
            flip_y: false,
            ..default()
        },
        transform: Transform::from_translation(spawn_position),
        global_transform: GlobalTransform::default(),
        rigid_body: RigidBody::Dynamic,
        movement_controller: MovementController {
            direction,
            speed: 900.0,
        },
        collider: Collider::circle(8.0),
        colliding_entities: CollidingEntities::default(),
        name: Name::new("Fireball"),
        visibility: Visibility::Visible,
        inherited_visibility: InheritedVisibility::default(),
        mass: Mass(100.),
    }
}

#[derive(Bundle)]
pub struct FrostballBundle {
    pub ability: Ability,
    pub frostbolt: Frostbolt,
    pub lifetime: Lifetime,
    pub sprite: Sprite,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub rigid_body: RigidBody,
    pub movement_controller: MovementController,
    pub collider: Collider,
    pub colliding_entities: CollidingEntities,
    pub name: Name,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub mass: Mass,
}

fn create_frostball_bundle(
    spawn_position: Vec3,
    direction: Vec2,
    asset_server: &Res<AssetServer>,
) -> FrostballBundle {
    FrostballBundle {
        ability: Ability,
        frostbolt: Frostbolt,
        lifetime: Lifetime {
            lifetime: Timer::from_seconds(4.0, TimerMode::Once),
        },
        sprite: Sprite {
            image: asset_server.load("images/fireball.png"),
            flip_x: direction.x < 0.0,
            flip_y: false,
            color: Color::srgb(0.0, 0.0, 1.0), // Blue for Frostball
            ..default()
        },
        transform: Transform::from_translation(spawn_position),
        global_transform: GlobalTransform::default(),
        rigid_body: RigidBody::Dynamic,
        movement_controller: MovementController {
            direction,
            speed: 200.0,
        },
        collider: Collider::circle(8.0),
        colliding_entities: CollidingEntities::default(),
        name: Name::new("Frostball"),
        visibility: Visibility::Visible,
        inherited_visibility: InheritedVisibility::default(),
        mass: Mass(400.0),
    }
}

fn spawn_ability(
    ability_type: ActionType,
    commands: &mut Commands,
    position: Vec3,
    direction: Vec2,
    asset_server: &Res<AssetServer>,
    player_query: Query<Entity, With<Player>>,
) {
    let offset_distance = 24.0; // Adjust based on your sprite sizes
    let spawn_position = position
        + Vec3::new(
            direction.x * offset_distance,
            direction.y * offset_distance,
            1.0,
        );
    match ability_type {
        ActionType::FireballAttack { direction } => {
            let fireball_bundle = create_fireball_bundle(spawn_position, direction, asset_server);
            commands.spawn(fireball_bundle);
        }
        ActionType::FrostAttack { direction } => {
            let frostball_bundle = create_frostball_bundle(spawn_position, direction, asset_server);
            commands.spawn(frostball_bundle);
        }
        ActionType::LightningAttack { .. } => {
            for (entity) in player_query {
                commands.entity(entity).insert(LightningState {
                    timer: Timer::new(Duration::from_millis(1500), TimerMode::Once),
                });
                return;
            }
        }
    }
}

fn process_ability_actions(
    mut commands: Commands,
    mut fireball_cooldown: ResMut<FireballCooldown>,
    mut frost_cooldown: ResMut<FrostCooldown>,
    mut lightning_cooldown: ResMut<LightningCooldown>,
    mut controllers: Query<(&Transform, &mut CharacterController)>,
    asset_server: Res<AssetServer>,
    player_query: Query<Entity, With<Player>>,
) {
    for (transform, mut controller) in &mut controllers {
        if let Some(action) = controller.pop_action() {
            match action {
                ActionType::FireballAttack { direction } => {
                    spawn_ability(
                        action,
                        &mut commands,
                        transform.translation,
                        direction,
                        &asset_server,
                        player_query,
                    );
                    fireball_cooldown.timer.reset();
                }
                ActionType::FrostAttack { direction } => {
                    spawn_ability(
                        action,
                        &mut commands,
                        transform.translation,
                        direction,
                        &asset_server,
                        player_query,
                    );
                    frost_cooldown.timer.reset();
                }
                ActionType::LightningAttack { direction } => {
                    spawn_ability(
                        action,
                        &mut commands,
                        transform.translation,
                        direction,
                        &asset_server,
                        player_query,
                    );
                    lightning_cooldown.timer.reset();
                }
            }
        }
    }
}

fn update_abilities(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime), With<Ability>>,
) {
    for (entity, mut lifetime) in &mut query {
        lifetime.lifetime.tick(time.delta());
        if lifetime.lifetime.finished() {
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

#[derive(Resource, Default)]
pub struct FrostCooldown {
    pub timer: Timer,
}

impl FrostCooldown {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

#[derive(Resource, Default)]
pub struct LightningCooldown {
    pub timer: Timer,
}

impl LightningCooldown {
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
    pub fn new(transform: &Transform, assets: &Res<ExplosionAssets>) -> Self {
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

#[derive(Resource, Asset, Clone, Reflect)]
pub struct FrostAssets {
    pub image_handle: Handle<Image>,
    pub layout_handle: Handle<TextureAtlasLayout>,
}

impl FromWorld for FrostAssets {
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
            image_handle: assets.load("images/ice_explosion.png"),
            layout_handle,
        }
    }
}
#[derive(Bundle, Default)]
pub struct FrostBundle {
    pub animation: ExplosionAnimation,
    pub sprite: Sprite,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub name: Name,
}

impl FrostBundle {
    pub fn new(transform: &Transform, assets: &Res<FrostAssets>) -> Self {
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
            name: Name::new("FrostExplosion"),
        }
    }
}
