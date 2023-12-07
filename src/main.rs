use bevy::asset::AssetMetaCheck;
use bevy::{prelude::*, sprite::Anchor};

use constants::*;
use piece::*;
use stats::*;

mod constants;
mod piece;
mod quant;
mod stats;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Playing,
    Lost,
}

#[derive(Resource, PartialEq, Eq, Clone, Copy)]
pub enum Objective {
    Measure0,
    Measure1,
}

#[derive(Component)]
pub struct ObjectiveLabel;

#[derive(Resource)]
pub struct DropSound(Handle<AudioSource>);

#[derive(Resource)]
pub struct ClearSound(Handle<AudioSource>);

#[derive(Resource)]
pub struct QuadrupleClearSound(Handle<AudioSource>);

#[derive(Resource)]
pub struct MeasureImage(Handle<Image>);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::WHITE))
        .insert_resource(PieceInfo {
            last_drop: 0.,
            shape: Shape::I,
            rotation: 0,
            pieces_since_objective: 0,
        })
        .insert_resource(Score { score: 0 })
        .insert_resource(Objective::Measure0)
        .insert_resource(AssetMetaCheck::Never)
        .add_state::<GameState>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, (setup_camera, setup_background))
        .add_systems(
            Update,
            (
                check_over,
                generate_new_piece.after(check_over),
                check_measurment,
                falling_piece,
                move_piece,
                rotate_piece,
                clear_columns,
                drop_piece,
                move_empty_lines,
                edit_objective_label,
                edit_scoreboard,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, check_game_restart.run_if(in_state(GameState::Lost)))
        .add_systems(OnEnter(GameState::Lost), show_lose_screen)
        .add_systems(
            PostUpdate,
            (
                update_block_transforms,
                hide_outside_blocks,
                move_control_wires,
            ),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = bevy::render::camera::ScalingMode::AutoMin {
        min_width: REFERENCE_SCREEN_WIDTH,
        min_height: REFERENCE_SCREEN_HEIGHT,
    };
    commands.spawn(camera);
}

pub fn setup_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    for y in 1..Y_COUNT + 1 {
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(X_GAPS * (X_COUNT as f32 - 1.), WIRE_WIDTH)),
                ..default()
            },
            transform: Transform::from_xyz(
                0.,
                y as f32 * Y_GAPS - REFERENCE_SCREEN_HEIGHT / 2.,
                0.,
            ),
            ..default()
        });
        commands.spawn(SpriteBundle {
            texture: asset_server.load("0.png"),
            transform: Transform::from_xyz(
                -REFERENCE_SCREEN_WIDTH / 2. + INITIAL_STATE_DISTANCE_FROM_RIGHT,
                y as f32 * Y_GAPS - REFERENCE_SCREEN_HEIGHT / 2.,
                1.,
            ),
            ..default()
        });
    }
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "",
                TextStyle {
                    font_size: OBJECTIVE_FONT_SIZE,
                    color: Color::BLACK,
                    ..default()
                },
            ),
            transform: Transform::from_xyz(0., -REFERENCE_SCREEN_HEIGHT / 2. + OBJECTIVE_GAP, 3.),
            text_anchor: Anchor::BottomCenter,
            ..default()
        },
        ObjectiveLabel,
    ));
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "",
                TextStyle {
                    font_size: SCORE_FONT_SIZE,
                    color: Color::BLACK,
                    ..default()
                },
            ),
            transform: Transform::from_xyz(
                -REFERENCE_SCREEN_WIDTH / 2. + SCORE_GAP,
                REFERENCE_SCREEN_HEIGHT / 2. - SCORE_GAP,
                1.,
            ),
            text_anchor: Anchor::TopLeft,
            ..default()
        },
        Scoreboard,
    ));
    commands.spawn(AudioBundle {
        source: asset_server.load("music.ogg"),
        settings: PlaybackSettings::LOOP,
    });
    commands.insert_resource(DropSound(asset_server.load("drop.ogg")));
    commands.insert_resource(ClearSound(asset_server.load("clear.ogg")));
    commands.insert_resource(QuadrupleClearSound(asset_server.load("quadclear.ogg")));
    commands.insert_resource(MeasureImage(asset_server.load("measure.png")));
}
