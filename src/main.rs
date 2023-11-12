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

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum Objective {
    #[default]
    Measure0,
    Measure1,
}

#[derive(Component)]
pub struct ObjectiveLabel;

fn main() {
    App::new()
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(PieceInfo {
            last_drop: 0.,
            shape: Shape::I,
            rotation: 0,
            pieces_since_measurment: 0,
        })
        .add_state::<GameState>()
        .add_state::<Objective>()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_camera, setup_background))
        .add_systems(
            Update,
            (
                generate_new_piece,
                falling_piece,
                move_piece,
                rotate_piece,
                clear_columns,
                drop_piece,
                check_measurment,
                clear_lines_after_measurment,
                edit_objective_label,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, check_game_restart.run_if(in_state(GameState::Lost)))
        .add_systems(OnEnter(GameState::Lost), show_lose_screen)
        // .add_systems(OnEnter(GameState::Playing), clean_game)
        .add_systems(PostUpdate, (update_block_transforms, hide_outside_blocks))
        .run();
}

fn setup_camera(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = bevy::render::camera::ScalingMode::AutoMin {
        min_width: REFERENCE_SCREEN_WIDTH as f32,
        min_height: REFERENCE_SCREEN_HEIGHT as f32,
    };
    commands.spawn(camera);
}

pub fn setup_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    for y in 1..Y_COUNT + 1 {
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: WIRE_COLOR,
                custom_size: Some(Vec2::new(X_GAPS * (X_COUNT as f32 - 1.), WIRE_WIDTH as f32)),
                ..default()
            },
            transform: Transform::from_xyz(
                0.,
                y as f32 * Y_GAPS - REFERENCE_SCREEN_HEIGHT as f32 / 2.,
                0.,
            ),
            ..default()
        });
        commands.spawn(SpriteBundle {
            texture: asset_server.load("|0>.png"),
            transform: Transform::from_xyz(
                -REFERENCE_SCREEN_WIDTH as f32 / 2. + INITIAL_STATE_DISTANCE_FROM_RIGHT,
                y as f32 * Y_GAPS - REFERENCE_SCREEN_HEIGHT as f32 / 2.,
                1.,
            ),
            ..default()
        });
    }
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "Next Objective: Measure 0",
                TextStyle {
                    font_size: OBJECTIVE_FONT_SIZE,
                    color: Color::BLACK,
                    ..default()
                },
            ),
            transform: Transform::from_xyz(
                0.,
                REFERENCE_SCREEN_HEIGHT as f32 / 2. - OBJECTIVE_GAP_FROM_TOP,
                1.,
            ),
            text_anchor: Anchor::TopCenter,
            ..default()
        },
        ObjectiveLabel,
    ));
}
