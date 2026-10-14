//! Quantris: Tetris where every block is a quantum gate and the circuit must pass measurements.

use bevy::{camera::ScalingMode, prelude::*};

mod board;
mod objective;
mod piece;
mod quantum;
mod ui;

use board::*;
use objective::*;
use piece::*;
use ui::*;

/// Distance of the |0⟩ input markers from the left edge of the view.
const KET_INSET: f32 = 48.;

#[derive(States, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Playing,
    Lost,
}

#[derive(Resource, Default)]
pub struct Score(pub u32);

/// Sound effects, loaded once.
#[derive(Resource)]
pub struct Sounds {
    pub drop: Handle<AudioSource>,
    pub clear: Handle<AudioSource>,
    pub quad_clear: Handle<AudioSource>,
}

impl FromWorld for Sounds {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            drop: assets.load("drop.ogg"),
            clear: assets.load("clear.ogg"),
            quad_clear: assets.load("quadclear.ogg"),
        }
    }
}

/// A uniformly random element of a non-empty slice.
pub fn pick<T: Copy>(options: &[T]) -> T {
    options[rand::random_range(0..options.len())]
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Quantris".into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::WHITE))
        .init_state::<GameState>()
        .init_resource::<Score>()
        .init_resource::<Objective>()
        .init_resource::<ActivePiece>()
        .init_resource::<Hold>()
        .init_resource::<MeasureCooldown>()
        .init_resource::<Sounds>()
        .init_resource::<BlockAssets>()
        .add_systems(Startup, (setup_scene, setup_hud, setup_measure_icons))
        .add_systems(OnEnter(GameState::Playing), reset_game)
        .add_systems(OnEnter(GameState::Lost), show_lose_screen)
        .add_systems(
            Update,
            (
                restart,
                (
                    spawn_piece,
                    hold_piece,
                    shift_piece,
                    rotate_piece,
                    fall_piece,
                    hard_drop,
                    measure,
                    clear_full_columns,
                    collapse_columns,
                    check_game_over,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
                (
                    update_score_label.run_if(resource_changed::<Score>),
                    update_hold_label.run_if(resource_changed::<Hold>),
                    update_objective_label.run_if(resource_changed::<Objective>),
                    sync_measure_icons,
                ),
                sync_block_transforms,
            )
                .chain(),
        )
        .run();
}

/// The camera, the background wires with their |0⟩ inputs, and the music.
fn setup_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: VIEW_WIDTH,
                min_height: VIEW_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
    let zero_ket = asset_server.load("0.png");
    for wire in 0..WIRES {
        let y = cell_to_world(IVec2::new(0, wire)).y;
        commands.spawn((
            Sprite::from_color(
                Color::BLACK,
                Vec2::new(COLUMN_GAP * (COLUMNS - 1) as f32, WIRE_WIDTH),
            ),
            Transform::from_xyz(0., y, 0.),
        ));
        commands.spawn((
            Sprite::from_image(zero_ket.clone()),
            Transform::from_xyz(-VIEW_WIDTH / 2. + KET_INSET, y, 1.),
        ));
    }
    commands.spawn((
        AudioPlayer::new(asset_server.load("music.ogg")),
        PlaybackSettings::LOOP,
    ));
}
