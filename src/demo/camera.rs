use super::player::Player;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_ecs_ldtk::{LdtkProjectHandle, prelude::*};

const ZOOM_FACTOR: f32 = 1.0;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, snap_camera_to_current_level);
}

#[allow(clippy::type_complexity)]
pub fn snap_camera_to_current_level(
    mut camera_query: Query<
        (&mut bevy::render::camera::Projection, &mut Transform),
        Without<Player>,
    >,
    player_query: Query<&Transform, With<Player>>,
    level_query: Query<(&Transform, &LevelIid), (Without<Projection>, Without<Player>)>,
    ldtk_projects: Query<&LdtkProjectHandle>,
    level_selection: Res<LevelSelection>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
) -> Result {
    // Bail early if the player isn't spawned.
    let Ok(Transform {
        translation: player_translation,
        ..
    }) = player_query.single()
    else {
        return Ok(());
    };

    let primary_window = primary_window_query.single()?;
    let aspect_ratio = primary_window.resolution.width() / primary_window.resolution.height();
    let player_translation = *player_translation;

    let (mut projection, mut camera_transform) = camera_query.single_mut()?;
    let Projection::Orthographic(ref mut orthographic_projection) = *projection else {
        return Err(BevyError::from("non-orthographic projection found"));
    };

    for (level_transform, level_iid) in &level_query {
        let ldtk_project = ldtk_project_assets
            .get(ldtk_projects.single()?)
            .expect("Project should be loaded if level has spawned");

        let level = ldtk_project
            .get_raw_level_by_iid(&level_iid.to_string())
            .expect("Spawned level should exist in LDtk project");

        if level_selection.is_match(&LevelIndices::default(), level) {
            let level_ratio = level.px_wid as f32 / level.px_hei as f32;
            orthographic_projection.viewport_origin = Vec2::ZERO;
            if level_ratio > aspect_ratio {
                // level is wider than the screen
                let height = (level.px_hei as f32 / 9.).round() * 9. * ZOOM_FACTOR;
                let width = height * aspect_ratio;
                orthographic_projection.scaling_mode =
                    bevy::render::camera::ScalingMode::Fixed { width, height };
                camera_transform.translation.x = (player_translation.x - width / 2.).clamp(
                    level_transform.translation.x,
                    level_transform.translation.x + level.px_wid as f32 - width,
                );
                camera_transform.translation.y = level_transform.translation.y;
            } else {
                // level is taller than the screen
                let width = (level.px_wid as f32 / 16.).round() * 16. * ZOOM_FACTOR;
                let height = width / aspect_ratio; 
                orthographic_projection.scaling_mode =
                    bevy::render::camera::ScalingMode::Fixed { width, height };
                camera_transform.translation.y = (player_translation.y - height / 2.).clamp(
                    level_transform.translation.y,
                    level_transform.translation.y + level.px_hei as f32 - height,
                );
                camera_transform.translation.x = level_transform.translation.x;
            }

            // Adjust camera translation to follow the player
            camera_transform.translation.x += level_transform.translation.x;
            camera_transform.translation.y += level_transform.translation.y;
        }
    }
    Ok(())
}
