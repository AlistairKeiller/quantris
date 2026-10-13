//! Heads-up text, the game-over screen and restarting.

use bevy::{prelude::*, sprite::Anchor};

use crate::{
    GameState, Score,
    board::{VIEW_HEIGHT, VIEW_WIDTH},
    objective::Objective,
    piece::ActivePiece,
};

const RESTART: KeyCode = KeyCode::KeyR;
const OBJECTIVE_FONT_SIZE: f32 = 80.;
const SCORE_FONT_SIZE: f32 = 48.;
const GAME_OVER_FONT_SIZE: f32 = 192.;
const GAME_OVER_HINT_FONT_SIZE: f32 = 48.;
const MARGIN: f32 = 16.;
/// Text draws above every block.
const HUD_Z: f32 = 3.;

#[derive(Component)]
pub struct ScoreLabel;

#[derive(Component)]
pub struct ObjectiveLabel;

fn font(size: f32) -> TextFont {
    TextFont {
        font_size: FontSize::Px(size),
        ..default()
    }
}

pub fn setup_hud(mut commands: Commands) {
    commands.spawn((
        Text2d::default(),
        font(SCORE_FONT_SIZE),
        TextColor(Color::BLACK),
        Anchor::TOP_LEFT,
        Transform::from_xyz(-VIEW_WIDTH / 2. + MARGIN, VIEW_HEIGHT / 2. - MARGIN, HUD_Z),
        ScoreLabel,
    ));
    commands.spawn((
        Text2d::default(),
        font(OBJECTIVE_FONT_SIZE),
        TextColor(Color::BLACK),
        Anchor::BOTTOM_CENTER,
        Transform::from_xyz(0., -VIEW_HEIGHT / 2. + MARGIN, HUD_Z),
        ObjectiveLabel,
    ));
}

pub fn update_score_label(score: Res<Score>, mut label: Single<&mut Text2d, With<ScoreLabel>>) {
    if score.is_changed() {
        label.0 = format!("Score: {}", score.0);
    }
}

pub fn update_objective_label(
    objective: Res<Objective>,
    mut label: Single<&mut Text2d, With<ObjectiveLabel>>,
) {
    if objective.is_changed() {
        label.0 = format!("Current Objective: {}", objective.name());
    }
}

pub fn show_lose_screen(mut commands: Commands) {
    commands.spawn((
        Text2d::new("Game Over"),
        font(GAME_OVER_FONT_SIZE),
        TextColor(Color::BLACK),
        TextLayout::justify(Justify::Center),
        Transform::from_xyz(0., 0., HUD_Z),
        DespawnOnExit(GameState::Lost),
        children![(
            TextSpan::new("\nPress R to Restart"),
            font(GAME_OVER_HINT_FONT_SIZE),
            TextColor(Color::BLACK),
        )],
    ));
}

/// Restarting re-enters [`GameState::Playing`] from either state, which despawns the blocks
/// and runs [`reset_game`].
pub fn restart(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if keys.just_pressed(RESTART) {
        next_state.set(GameState::Playing);
    }
}

pub fn reset_game(mut score: ResMut<Score>, mut active: ResMut<ActivePiece>) {
    *score = Score::default();
    *active = ActivePiece::default();
}
