use crate::{screens::Screen, theme::widget};
use bevy::prelude::*;
use bevy::time::Stopwatch;
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GameTimer>() // Initialize the timer resource
        .add_systems(OnEnter(Screen::Gameplay), spawn_game_timer_ui) // Setup the timer UI
        .add_systems(
            Update,
            update_game_timer_ui.run_if(in_state(Screen::Gameplay)),
        ); // Update the timer
}

#[derive(Resource)]
pub struct GameTimer {
    pub timer: Stopwatch,
}

impl Default for GameTimer {
    fn default() -> Self {
        Self {
            timer: Stopwatch::new(),
        }
    }
}

fn spawn_game_timer_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        widget::ui_root("Game Timer"),
        GlobalZIndex(2),
        StateScoped(Screen::Gameplay),
        Text2d::new("Time: 0.00 seconds"),
        TextLayout {
            justify: JustifyText::Left, // Align text to the left
            ..default()
        },
    ));
}

fn update_game_timer_ui(
    time: Res<Time>,
    mut timer: ResMut<GameTimer>,
    mut query: Query<&mut Text>,
) {
    // Tick the timer
    timer.timer.tick(time.delta());

    // Update the text with the remaining time
    for mut text in &mut query {
        let remaining_time = timer.timer.elapsed_secs();
        text.0 = format!("Time: {:.2} seconds", remaining_time);
    }
}
