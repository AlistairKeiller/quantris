use bevy::prelude::*;

use crate::*;

#[derive(Resource)]
pub struct Score {
    pub score: i32,
}

#[derive(Component)]
pub struct Scoreboard;

#[derive(Component)]
pub struct LoseScreen;

pub fn edit_objective_label(
    mut objective_label_query: Query<&mut Text, With<ObjectiveLabel>>,
    objective: Res<Objective>,
) {
    for mut text in &mut objective_label_query {
        text.sections[0].value = format!("Current Objective: {}", objective.get_name());
    }
}

pub fn edit_scoreboard(
    mut scoreboard_query: Query<&mut Text, With<Scoreboard>>,
    score: Res<Score>,
) {
    for mut text in &mut scoreboard_query {
        text.sections[0].value = format!("Score: {}", score.score);
    }
}

pub fn show_lose_screen(mut commands: Commands) {
    commands.spawn((
        Text2dBundle {
            text: Text::from_sections([
                TextSection::new(
                    "Game Over",
                    TextStyle {
                        font_size: GAME_OVER_LARGE_FONT_SIZE,
                        color: Color::BLACK,
                        ..default()
                    },
                ),
                TextSection::new(
                    "\nPress R to Restart",
                    TextStyle {
                        font_size: GAME_OVER_SMALL_FONT_SIZE / 2.,
                        color: Color::BLACK,
                        ..default()
                    },
                ),
            ]),
            transform: Transform::from_xyz(0., 0., 1.),
            ..default()
        },
        LoseScreen,
    ));
}

pub fn check_game_restart(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    blocks: Query<Entity, With<Block>>,
    lose_screen: Query<Entity, With<LoseScreen>>,
    mut score: ResMut<Score>,
    mut piece_info: ResMut<PieceInfo>,
) {
    if keys.just_pressed(KeyCode::R) {
        for entity in &blocks {
            commands.entity(entity).despawn_recursive();
        }
        for entity in &lose_screen {
            commands.entity(entity).despawn_recursive();
        }
        score.score = 0;
        piece_info.pieces_since_objective = 0;
        next_state.set(GameState::Playing);
    }
}
