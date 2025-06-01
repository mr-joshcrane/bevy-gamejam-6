//! Spawn the main level.

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::{asset_tracking::LoadResource, audio::music, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(LdtkPlugin);
    app.init_asset::<LdtkProject>();
    app.register_type::<LevelAssets>();
    app.load_resource::<LevelAssets>();
    app.insert_resource(LevelSelection::index(0));
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSource>,
    ldtk_level: LdtkProjectHandle,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Fluffing A Duck.ogg"),
            ldtk_level: LdtkProjectHandle {
                handle: assets.load("levels/level.ldtk"),
            },
        }
    }
}

/// A system that spawns the main level.
pub fn spawn_level(mut commands: Commands, level_assets: Res<LevelAssets>) {
    commands.spawn((
        Name::new("Level"),
        LdtkWorldBundle {
            ldtk_handle: level_assets.ldtk_level.clone(),
            ..default()
        },
        StateScoped(Screen::Gameplay),
        children![(
            Name::new("Gameplay Music"),
            music(level_assets.music.clone())
        )],
    ));
}
