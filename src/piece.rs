use bevy::{prelude::*, sprite::MaterialMesh2dBundle, transform::components::Transform};
use rand::prelude::*;
use rand::seq::SliceRandom;

use crate::constants::*;
use crate::quant::*;
use crate::*;

#[derive(Component)]
pub struct Block {
    pub x: i32,
    pub y: i32,
    pub gate: Gate,
}

#[derive(Component)]
pub struct Piece {
    pub number: i32,
}

#[derive(Component)]
pub struct Control {
    pub on_top: bool,
}

#[derive(Component)]
pub struct ControlWire;

#[derive(Resource)]
pub struct PieceInfo {
    pub last_drop: f32,
    pub shape: Shape,
    pub rotation: i32,
    pub pieces_since_objective: i32,
}

pub fn check_over(
    block_query: Query<&Block, Without<Piece>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if block_query.iter().any(|block| block.x >= X_COUNT) {
        next_state.set(GameState::Lost);
    }
}

pub fn check_measurment(
    mut commands: Commands,
    block_entity_query: Query<(Entity, &Block), Without<Piece>>,
    block_query: Query<&Block, Without<Piece>>,
    control_block_query: Query<(&Block, &Control), Without<Piece>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<Score>,
    mut objective: ResMut<Objective>,
) {
    let mut measure_block_locations = vec![];
    for (_, measure_block) in &block_entity_query {
        if measure_block.gate == Gate::M {
            measure_block_locations.push((measure_block.x, measure_block.y));
        }
    }
    if measure_block_locations.len() > 0 {
        if satisfies_objective(
            objective.get_desired_state(),
            &block_query,
            &control_block_query,
            measure_block_locations.clone(),
        ) {
            for (entity, block) in &block_entity_query {
                if block.x < measure_block_locations[0].0 {
                    score.score += 10;
                    commands.entity(entity).despawn_recursive();
                }
            }
            for (measure_entity, measure_block) in &block_entity_query {
                if measure_block.gate == Gate::M {
                    commands.entity(measure_entity).despawn_recursive();
                }
            }
            if let Some(&new_objective) = OBJECTIVES.choose(&mut rand::thread_rng()) {
                *objective = new_objective;
            };
        } else {
            next_state.set(GameState::Lost);
        }
    }
}

pub fn move_empty_lines(mut block_query: Query<&mut Block, Without<Piece>>) {
    for x in (0..X_COUNT).rev() {
        if !(0..Y_COUNT).any(|y| block_query.iter().any(|block| block.x == x && block.y == y)) {
            for mut block in &mut block_query {
                if block.x > x {
                    block.x -= 1;
                }
            }
        }
    }
}

pub fn falling_piece(
    mut commands: Commands,
    mut piece_query: Query<(Entity, &mut Block), With<Piece>>,
    block_query: Query<&Block, Without<Piece>>,
    time: Res<Time>,
    mut piece_info: ResMut<PieceInfo>,
    keys: Res<Input<KeyCode>>,
) {
    if !keys.just_pressed(FASTER_FALL_KEYCODE)
        && time.elapsed_seconds() - piece_info.last_drop
            < if keys.pressed(FASTER_FALL_KEYCODE) {
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
    if !keys.just_pressed(PIECE_DOWN_KEYCODE) && !keys.just_pressed(PIECE_UP_KEYCODE) {
        return;
    }
    if keys.just_pressed(PIECE_DOWN_KEYCODE) && keys.just_pressed(PIECE_UP_KEYCODE) {
        return;
    }
    let ymove = if keys.just_pressed(PIECE_DOWN_KEYCODE) {
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
    mut control_piece_query: Query<&mut Control, With<Piece>>,
) {
    if !keys.just_pressed(ROTATE_PIECE_CLOCKWISE)
        && !keys.just_pressed(ROTATE_PIECE_COUNTERCLOCKWISE)
    {
        return;
    }
    if keys.just_pressed(ROTATE_PIECE_CLOCKWISE) && keys.just_pressed(ROTATE_PIECE_COUNTERCLOCKWISE)
    {
        return;
    }
    let clockwise = keys.just_pressed(ROTATE_PIECE_CLOCKWISE);
    let next_rotation = if clockwise {
        (piece_info.rotation + 1) % 4
    } else {
        (piece_info.rotation + 3) % 4
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
        for mut control in &mut control_piece_query {
            control.on_top = piece_info.shape.control_on_top(next_rotation);
        }
        piece_info.rotation = next_rotation;
    }
}

pub fn drop_piece(
    mut commands: Commands,
    mut piece_query: Query<(Entity, &mut Block), With<Piece>>,
    block_query: Query<&Block, Without<Piece>>,
    keys: Res<Input<KeyCode>>,
    drop_sound: Res<DropSound>,
) {
    if !keys.just_pressed(DROP_PIECE_KEYCODE) {
        return;
    }
    let mut xmove = 0;
    while piece_query.iter().all(|(_, piece_location)| {
        !block_query.iter().any(|block_location| {
            block_location.x == (piece_location.x + xmove) && block_location.y == piece_location.y
        }) && (piece_location.x + xmove) >= 0
            && piece_location.y >= 0
            && piece_location.y < Y_COUNT
    }) {
        xmove -= 1;
    }
    for (entity, mut piece_location) in &mut piece_query {
        piece_location.x += xmove + 1;
        commands.entity(entity).remove::<Piece>();
    }
    commands.spawn(AudioBundle {
        source: drop_sound.0.clone(),
        settings: PlaybackSettings::DESPAWN,
    });
}

pub fn clear_columns(
    mut commands: Commands,
    block_query: Query<(Entity, &Block), Without<Piece>>,
    mut score: ResMut<Score>,
    clear_sound: Res<ClearSound>,
    quadclear_sound: Res<QuadrupleClearSound>,
) {
    let mut columns_cleared = 0;
    for x in 0..X_COUNT {
        if (0..Y_COUNT).all(|y| {
            block_query
                .iter()
                .any(|(_, block)| block.x == x && block.y == y && block.gate != Gate::M)
        }) {
            columns_cleared += 1;
            for (entity, block_location) in &block_query {
                if block_location.x == x {
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
    score.score += match columns_cleared {
        1 => 100,
        2 => 300,
        3 => 500,
        4 => 800,
        _ => 0,
    };
    if columns_cleared == 4 {
        commands.spawn(AudioBundle {
            source: quadclear_sound.0.clone(),
            settings: PlaybackSettings::DESPAWN,
        });
    } else if columns_cleared >= 1 {
        commands.spawn(AudioBundle {
            source: clear_sound.0.clone(),
            settings: PlaybackSettings::DESPAWN,
        });
    }
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
        transform.translation.x = (block.x + 1) as f32 * X_GAPS - REFERENCE_SCREEN_WIDTH / 2.;
        transform.translation.y = (block.y + 1) as f32 * Y_GAPS - REFERENCE_SCREEN_HEIGHT / 2.;
    }
}

pub fn move_control_wires(
    control_piece_query: Query<(&Children, &Control), With<Piece>>,
    mut control_piece_wire_query: Query<&mut Transform, With<ControlWire>>,
) {
    for (children, control) in &control_piece_query {
        for &child in children.iter() {
            if let Ok(mut transform) = control_piece_wire_query.get_mut(child) {
                transform.translation.y = if control.on_top { -1. } else { 1. } * Y_GAPS / 2.;
            }
        }
    }
}

pub fn generate_new_piece(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    piece_query: Query<With<Piece>>,
    mut piece_info: ResMut<PieceInfo>,
    measurment_image: Res<MeasureImage>,
    objective: Res<Objective>,
) {
    if !piece_query.is_empty() {
        return;
    }
    if piece_info.pieces_since_objective >= OBJECTIVE_PERIOD {
        piece_info.shape = Shape::M;
        piece_info.pieces_since_objective = 0;
        for y in 0..objective.measure_count() {
            commands.spawn((
                Block {
                    x: X_COUNT - 1,
                    y,
                    gate: Gate::M,
                },
                Piece { number: 0 },
                SpriteBundle {
                    texture: measurment_image.0.clone(),
                    transform: Transform::from_xyz(0., 0., 1.),
                    ..default()
                },
            ));
        }
    } else if let Some(shape) = SHAPES.choose(&mut rand::thread_rng()) {
        piece_info.shape = *shape;
        piece_info.rotation = 0;
        piece_info.pieces_since_objective += 1;
        for number in 0..4 {
            let (x, y) = shape.rotation_location(number, 0);
            if let Some(&gate) = if shape.can_control_spawn(number)
                && rand::thread_rng().gen::<f32>() < CONTROL_GATE_CHANCE
            {
                CONTROL_GATES.choose(&mut rand::thread_rng())
            } else {
                GATES_WITHOUT_CONTROL.choose(&mut rand::thread_rng())
            } {
                let mut x = commands.spawn((
                    Block {
                        x: X_COUNT - 1 + x,
                        y,
                        gate,
                    },
                    Piece { number },
                ));
                if CONTROL_GATES.contains(&gate) {
                    x.insert((
                        Control {
                            on_top: shape.control_on_top(0),
                        },
                        MaterialMesh2dBundle {
                            mesh: meshes
                                .add(shape::Circle::new(CONTROL_OUTER_RADIUS).into())
                                .into(),
                            material: materials.add(ColorMaterial::from(shape.color())),
                            transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
                            ..default()
                        },
                    ));
                    x.with_children(|parent| {
                        parent.spawn((
                            SpriteBundle {
                                sprite: Sprite {
                                    color: shape.color(),
                                    custom_size: Some(Vec2 {
                                        x: WIRE_WIDTH,
                                        y: Y_GAPS,
                                    }),
                                    ..default()
                                },
                                ..default()
                            },
                            ControlWire,
                        ));
                        if gate == Gate::AC {
                            parent.spawn(MaterialMesh2dBundle {
                                mesh: meshes
                                    .add(shape::Circle::new(CONTROL_INNER_RADIUS).into())
                                    .into(),
                                material: materials.add(ColorMaterial::from(Color::WHITE)),
                                transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
                                ..default()
                            });
                        }
                    });
                } else {
                    x.insert(SpriteBundle {
                        sprite: Sprite {
                            color: shape.color(),
                            custom_size: Some(Vec2::new(OPERATOR_SIZE, OPERATOR_SIZE)),
                            ..default()
                        },
                        transform: Transform::from_xyz(0., 0., 1.),
                        ..default()
                    });
                    x.with_children(|parent| {
                        parent.spawn(Text2dBundle {
                            text: Text::from_section(
                                gate.to_string(),
                                TextStyle {
                                    font_size: OPERATOR_FONT_SIZE,
                                    color: Color::BLACK,
                                    ..default()
                                },
                            ),
                            transform: Transform::from_xyz(0., 0., 1.),
                            ..default()
                        });
                    });
                }
            }
        }
    }
}
