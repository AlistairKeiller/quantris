use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use rand::seq::SliceRandom;

use crate::constants::*;

#[derive(Component)]
pub struct Block {
    x: i32,
    y: i32,
    gate: Gate,
}

#[derive(Component)]
pub struct Piece {
    x: i32,
    y: i32,
}

#[derive(Resource, Default)]
pub struct PieceInfo {
    pub last_drop: f32,
    pub center_x: i32,
    pub center_y: i32,
}

pub fn falling_piece(
    mut commands: Commands,
    mut piece_query: Query<(Entity, &mut Block), With<Piece>>,
    block_query: Query<&Block, Without<Piece>>,
    time: Res<Time>,
    mut piece_info: ResMut<PieceInfo>,
    keys: Res<Input<KeyCode>>,
) {
    if !keys.just_pressed(KeyCode::Left)
        && time.elapsed_seconds() - piece_info.last_drop
            < if keys.pressed(KeyCode::Left) {
                FAST_DROP_PERIOD
            } else {
                DROP_PERIOD
            }
    {
        return;
    }
    piece_info.last_drop = time.elapsed_seconds();
    if piece_query.iter().all(|(_, piece)| {
        !block_query
            .iter()
            .any(|block| block.x == (piece.x - 1) && block.y == piece.y)
            && (piece.x - 1) >= 0
            && piece.y >= 0
            && piece.y < Y_COUNT
    }) {
        for (_, mut block) in &mut piece_query {
            block.x -= 1;
        }
    } else {
        for (entity, _) in &mut piece_query {
            commands.entity(entity).remove::<Piece>();
        }
    }
}

pub fn move_piece(
    mut piece_query: Query<&mut Block, With<Piece>>,
    block_query: Query<&Block, Without<Piece>>,
    keys: Res<Input<KeyCode>>,
) {
    let ymove = if keys.just_pressed(KeyCode::Down) {
        -1
    } else if keys.just_pressed(KeyCode::Up) {
        1
    } else {
        return;
    };
    if piece_query.iter().all(|piece| {
        !block_query
            .iter()
            .any(|block| block.x == piece.x && block.y == (piece.y + ymove))
            && piece.x >= 0
            && (piece.y + ymove) >= 0
            && (piece.y + ymove) < Y_COUNT
    }) {
        for mut block in &mut piece_query {
            block.y += ymove;
        }
    }
}

pub fn rotate_clockwise(
    mut piece_query: Query<(&Block, &mut Piece)>,
    block_query: Query<&Block, Without<Piece>>,
    keys: Res<Input<KeyCode>>,
) {
}

pub fn hide_outside_blocks(mut query: Query<(&mut Visibility, &Block)>) {
    for (mut visibility, block) in &mut query {
        *visibility = if block.x < X_COUNT {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn update_block_transforms(mut query: Query<(&mut Transform, &Block)>) {
    for (mut transform, block) in &mut query {
        transform.translation.x =
            (block.x + 1) as f32 * X_GAPS - REFERENCE_SCREEN_WIDTH as f32 / 2.;
        transform.translation.y =
            (block.y + 1) as f32 * Y_GAPS - REFERENCE_SCREEN_HEIGHT as f32 / 2.;
    }
}

pub fn generate_new_piece(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<With<Piece>>,
) {
    if !query.is_empty() {
        return;
    }
    if let Some((piece, [center_x, center_y])) = SHAPES.choose(&mut rand::thread_rng()) {
        for [x, y] in piece {
            if let Some(gate) = GATES.choose(&mut rand::thread_rng()) {
                commands
                    .spawn((
                        Block {
                            x: X_COUNT - 1 + *x,
                            y: *y,
                            gate: *gate,
                        },
                        Piece { x: *x, y: *y },
                        MaterialMesh2dBundle {
                            mesh: meshes
                                .add(
                                    shape::Quad::new(Vec2::new(
                                        OPERATOR_SIZE as f32,
                                        OPERATOR_SIZE as f32,
                                    ))
                                    .into(),
                                )
                                .into(),
                            material: materials
                                .add(ColorMaterial::from(Color::rgb_u8(111, 164, 255))), // Placeholder, fix later
                            transform: Transform::from_translation(Vec3::new(100000., 100000., 1.)), // Prob not the best way to do this
                            ..default()
                        },
                    ))
                    .with_children(|parent| {
                        parent.spawn(Text2dBundle {
                            text: Text::from_section(
                                gate.to_string(),
                                TextStyle {
                                    font_size: OPERATOR_FONT_SIZE as f32,
                                    color: OPERATOR_FONT_COLOR,
                                    ..default()
                                },
                            )
                            .with_alignment(TextAlignment::Center),
                            transform: Transform::from_translation(Vec3::new(0., 0., 2.)),
                            ..default()
                        });
                    });
            }
        }
    }
}
