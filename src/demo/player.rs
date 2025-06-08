//! Player-specific behavior.

use std::collections::VecDeque;

use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};
use bevy_ecs_ldtk::prelude::*;

use crate::{
    asset_tracking::LoadResource,
    demo::{animation::PlayerAnimation, movement::MovementController},
};

use bevy_enhanced_input::prelude::*;

use super::{
    collision::HeroCollisionBundle,
    input::{ActionType, PlatformerContext},
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();
    app.register_type::<PlayerAssets>();
    app.load_resource::<PlayerAssets>();
    app.register_ldtk_entity::<PlayerBundle>("Player");
    app.add_systems(
        Update,
        post_process_player_bundle.run_if(resource_exists::<PlayerAssets>),
    );
}

#[derive(Bundle, Default, LdtkEntity)]
pub struct PlayerBundle {
    a: Player,
    pub actions: Actions<PlatformerContext>,
    pub sprite: Sprite,
    pub player_animation: PlayerAnimation,
    pub movement_controller: MovementController,
    pub character_controller: CharacterController,
    pub collision_bundle: HeroCollisionBundle,
    #[grid_coords]
    pub grid_coords: GridCoords,
}

fn post_process_player_bundle(
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    query: Query<Entity, Added<Player>>,
) {
    let player_animation = PlayerAnimation::new();

    for entity in &query {
        // Modify just the components you care about
        commands.entity(entity).insert(Sprite {
            image: player_assets.ducky.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                    UVec2::splat(32),
                    6,
                    2,
                    Some(UVec2::splat(1)),
                    None,
                )),
                index: player_animation.get_atlas_index(),
            }),
            ..default()
        });
        commands.entity(entity).insert(player_animation.clone());
    }
}

#[derive(Component, Debug, Reflect)]
pub struct LightningState {
    pub timer: Timer,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    pub ducky: Handle<Image>,
    #[dependency]
    pub lightning: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            ducky: assets.load_with_settings(
                "images/ducky.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            lightning: assets.load("images/lightning.png"),
            steps: vec![
                assets.load("audio/sound_effects/step1.ogg"),
                assets.load("audio/sound_effects/step2.ogg"),
                assets.load("audio/sound_effects/step3.ogg"),
                assets.load("audio/sound_effects/step4.ogg"),
            ],
        }
    }
}

#[derive(Component, Default, Debug, Clone)]
pub struct CharacterController {
    pub action_queue: VecDeque<ActionType>,
}

impl CharacterController {
    /// Queue an action to be processed
    pub fn queue_action(&mut self, action: ActionType) {
        self.action_queue.push_back(action);
    }

    /// Pop the next action from the queue
    pub fn pop_action(&mut self) -> Option<ActionType> {
        self.action_queue.pop_front()
    }
}
