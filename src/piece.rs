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
    center_x: f32,
    center_y: f32,
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
    query: Query<&Piece>,
) {
    if query.is_empty() {
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
                            Piece {
                                x: *x,
                                y: *y,
                                center_x: *center_x,
                                center_y: *center_y,
                            },
                            MaterialMesh2dBundle {
                                mesh: meshes
                                    .add(
                                        shape::Quad::new(Vec2::new(
                                            OPERATOR_SIZE as f32,
                                            OPERATOR_SIZE as f32,
                                            // 0 as f32, 0 as f32,
                                        ))
                                        .into(),
                                    )
                                    .into(),
                                material: materials
                                    .add(ColorMaterial::from(Color::rgb_u8(111, 164, 255))), // Placeholder, fix later
                                transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
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
        // let x = SHAPES.choose(&mut rand::thread_rng());
        // for x, y in piece {
        // commands.spawn(Block {x: 5, y: 5});
        // }
        // let color = piece_config.color;
        // let visibility = Visibility::Hidden;
        // let piece_type = piece_config.piece_type.clone();

        // for block in piece_config.blocks.iter() {
        //     commands
        //         .spawn(piece_type)
        //         .insert(new_block_sprite(&block, color, visibility))
        //         .insert(*block)
        //         .insert(Movable {
        //             can_down: true,
        //             can_left: true,
        //             can_right: true,
        //         });
        // }
    }
}
