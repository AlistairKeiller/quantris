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
    number: i32,
}

#[derive(Resource)]
pub struct PieceInfo {
    pub last_drop: f32,
    pub shape: Shape,
    pub rotation: i32,
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
    if piece_query.iter().all(|(_, piece_location)| {
        !block_query.iter().any(|block_location| {
            block_location.x == (piece_location.x - 1) && block_location.y == piece_location.y
        }) && (piece_location.x - 1) >= 0
            && piece_location.y >= 0
            && piece_location.y < Y_COUNT
    }) {
        for (_, mut piece_location) in &mut piece_query {
            piece_location.x -= 1;
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
    if !keys.just_pressed(KeyCode::Down) && !keys.just_pressed(KeyCode::Up) {
        return;
    }
    if keys.just_pressed(KeyCode::Down) && keys.just_pressed(KeyCode::Up) {
        return;
    }
    let ymove = if keys.just_pressed(KeyCode::Down) {
        -1
    } else {
        1
    };
    if piece_query.iter().all(|piece_location| {
        !block_query.iter().any(|block_location| {
            block_location.x == piece_location.x && block_location.y == (piece_location.y + ymove)
        }) && piece_location.x >= 0
            && (piece_location.y + ymove) >= 0
            && (piece_location.y + ymove) < Y_COUNT
    }) {
        for mut piece_location in &mut piece_query {
            piece_location.y += ymove;
        }
    }
}

pub fn rotate_piece(
    mut piece_query: Query<(&mut Block, &Piece)>,
    block_query: Query<&Block, Without<Piece>>,
    keys: Res<Input<KeyCode>>,
    mut piece_info: ResMut<PieceInfo>,
) {
    if !keys.just_pressed(KeyCode::X) && !keys.just_pressed(KeyCode::Z) {
        return;
    }
    if keys.just_pressed(KeyCode::X) && keys.just_pressed(KeyCode::Z) {
        return;
    }
    let clockwise = keys.just_pressed(KeyCode::X);
    let next_rotation = if clockwise {
        (piece_info.rotation + 1) % 4
    } else {
        (piece_info.rotation + 3) % 4 // This will effectively rotate it counterclockwise
    };
    if let Some(&(wall_kicks_dx, wall_kicks_dy)) = piece_info
        .shape
        .wall_kicks(piece_info.rotation, clockwise)
        .iter()
        .find(|&&(wall_kicks_dx, wall_kicks_dy)| {
            piece_query.iter().all(|(piece_location, piece)| {
                let (rotation_dx, rotation_dy) = piece_info.shape.rotation_location_change(
                    piece.number,
                    piece_info.rotation,
                    next_rotation,
                );
                !block_query.iter().any(|block_location| {
                    block_location.x == piece_location.x + wall_kicks_dx + rotation_dx
                        && block_location.y == piece_location.y + wall_kicks_dy + rotation_dy
                }) && piece_location.x + wall_kicks_dx + rotation_dx >= 0
                    && piece_location.y + wall_kicks_dy + rotation_dy >= 0
                    && piece_location.y + wall_kicks_dy + rotation_dy < Y_COUNT
            })
        })
    {
        for (mut piece_location, piece) in &mut piece_query {
            let (rotation_dx, rotation_dy) = piece_info.shape.rotation_location_change(
                piece.number,
                piece_info.rotation,
                next_rotation,
            );
            piece_location.x += wall_kicks_dx + rotation_dx;
            piece_location.y += wall_kicks_dy + rotation_dy;
        }
        piece_info.rotation = next_rotation;
    }
}

// pub fn clear_rows(mut commands: Commands, block_query: Query<(, &Block), Without<Piece>>) {}

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
    mut piece_info: ResMut<PieceInfo>,
) {
    if !query.is_empty() {
        return;
    }
    if let Some(shape) = SHAPES.choose(&mut rand::thread_rng()) {
        piece_info.shape = *shape;
        piece_info.rotation = 0;
        for number in 0..4 {
            let (x, y) = shape.rotation_location(number, 0);
            if let Some(gate) = GATES.choose(&mut rand::thread_rng()) {
                commands
                    .spawn((
                        Block {
                            x: X_COUNT - 1 + x,
                            y,
                            gate: *gate,
                        },
                        Piece { number },
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
