//! Heads-up text, the game-over screen and restarting.

use bevy::{prelude::*, sprite::Anchor};

use crate::{
    GameState, Score,
    board::{VIEW_HEIGHT, VIEW_WIDTH},
    objective::{MeasureCooldown, Objective},
    piece::{ActivePiece, Hold},
    quantum::Gate,
};

const RESTART: KeyCode = KeyCode::KeyR;
const OBJECTIVE_FONT_SIZE: f32 = 80.;
const HUD_FONT_SIZE: f32 = 48.;
const GAME_OVER_FONT_SIZE: f32 = 192.;
const GAME_OVER_HINT_FONT_SIZE: f32 = 48.;
const MARGIN: f32 = 16.;
/// Text draws above every block.
const HUD_Z: f32 = 3.;

#[derive(Component)]
pub struct ScoreLabel;

#[derive(Component)]
pub struct HoldLabel;

#[derive(Component)]
pub struct ObjectiveLabel;

pub fn setup_hud(mut commands: Commands) {
    let top = VIEW_HEIGHT / 2. - MARGIN;
    commands.spawn((
        Text2d::default(),
        TextFont::from_font_size(HUD_FONT_SIZE),
        TextColor::BLACK,
        Anchor::TOP_LEFT,
        Transform::from_xyz(-VIEW_WIDTH / 2. + MARGIN, top, HUD_Z),
        ScoreLabel,
    ));
    commands.spawn((
        Text2d::default(),
        TextFont::from_font_size(HUD_FONT_SIZE),
        TextColor::BLACK,
        Anchor::TOP_RIGHT,
        Transform::from_xyz(VIEW_WIDTH / 2. - MARGIN, top, HUD_Z),
        HoldLabel,
    ));
    commands.spawn((
        Text2d::default(),
        TextFont::from_font_size(OBJECTIVE_FONT_SIZE),
        TextColor::BLACK,
        Anchor::BOTTOM_CENTER,
        Transform::from_xyz(0., -VIEW_HEIGHT / 2. + MARGIN, HUD_Z),
        ObjectiveLabel,
    ));
}

pub fn update_score_label(score: Res<Score>, mut label: Single<&mut Text2d, With<ScoreLabel>>) {
    label.0 = format!("Score: {}", score.0);
}

pub fn update_hold_label(hold: Res<Hold>, mut label: Single<&mut Text2d, With<HoldLabel>>) {
    label.0 = match hold.piece {
        Some(piece) => format!("Hold: {:?} {}", piece.shape, piece.gates.map(Gate::label).join(" ")),
        None => "Hold: -".into(),
    };
}

pub fn update_objective_label(
    objective: Res<Objective>,
    mut label: Single<&mut Text2d, With<ObjectiveLabel>>,
) {
    label.0 = format!("Objective: {}", objective.name());
}

pub fn show_lose_screen(mut commands: Commands) {
    commands.spawn((
        Text2d::new("Game Over"),
        TextFont::from_font_size(GAME_OVER_FONT_SIZE),
        TextColor::BLACK,
        TextLayout::justify(Justify::Center),
        Transform::from_xyz(0., 0., HUD_Z),
        DespawnOnExit(GameState::Lost),
        children![(
            TextSpan::new("\nPress R to Restart"),
            TextFont::from_font_size(GAME_OVER_HINT_FONT_SIZE),
            TextColor::BLACK,
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

pub fn reset_game(
    mut score: ResMut<Score>,
    mut active: ResMut<ActivePiece>,
    mut hold: ResMut<Hold>,
    mut cooldown: ResMut<MeasureCooldown>,
) {
    *score = Score::default();
    *active = ActivePiece::default();
    *hold = Hold::default();
    *cooldown = MeasureCooldown::default();
}
