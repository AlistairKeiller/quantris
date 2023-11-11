use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

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

fn main() {
    App::new()
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(PieceInfo {
            last_drop: 0.,
            shape: Shape::I,
            rotation: 0,
            pieces_since_measurment: 0,
            objective: Objective::Measure0,
        })
        .add_state::<GameState>()
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
            )
                .run_if(in_state(GameState::Playing)),
        )
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

pub fn setup_background(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for y in 1..Y_COUNT + 1 {
        commands.spawn(MaterialMesh2dBundle {
            mesh: meshes
                .add(
                    shape::Quad::new(Vec2::new(X_GAPS * (X_COUNT as f32 - 1.), WIRE_WIDTH as f32))
                        .into(),
                )
                .into(),
            material: materials.add(ColorMaterial::from(WIRE_COLOR)),
            transform: Transform::from_translation(Vec3::new(
                0.,
                y as f32 * Y_GAPS - REFERENCE_SCREEN_HEIGHT as f32 / 2.,
                0.,
            )),
            ..default()
        });
    }
}
