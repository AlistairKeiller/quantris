use bevy::prelude::*;
use nalgebra::*;

use crate::constants::*;
use crate::piece::*;

pub fn get_operator_of_column(
    block_query: &Query<(&Block, &Control), Without<Piece>>,
    x: i32,
) -> DMatrix<Complex<f32>> {
    let mut result: DMatrix<Complex<f32>> = DMatrix::zeros(1, 1);
    for y in 0..Y_COUNT {
        if let Some((block, _)) = block_query
            .iter()
            .find(|(block_location, _)| block_location.x == x && block_location.y == y)
        {
            if let Some(operator) = block.gate.operator() {
                let mut kroneckered = false;
                if let Some((up_block, up_control)) = block_query
                    .iter()
                    .find(|(block_location, _)| block_location.x == x && block_location.y == y + 1)
                {
                    if CONTROL_GATES.contains(&up_block.gate) && up_control.on_top {
                        // is being controlled from on top
                        result = result.kronecker(&if block.gate == Gate::C {
                            Matrix4::new(
                                operator[(0, 0)],
                                Complex::new(0., 0.),
                                operator[(0, 1)],
                                Complex::new(0., 0.),
                                Complex::new(0., 0.),
                                Complex::new(1., 0.),
                                Complex::new(0., 0.),
                                Complex::new(0., 0.),
                                operator[(1, 0)],
                                Complex::new(0., 0.),
                                operator[(1, 1)],
                                Complex::new(0., 0.),
                                Complex::new(0., 0.),
                                Complex::new(0., 0.),
                                Complex::new(0., 0.),
                                Complex::new(1., 0.),
                            )
                        } else {
                            Matrix4::new(
                                operator[(0, 0)],
                                operator[(0, 1)],
                                Complex::new(0., 0.),
                                Complex::new(0., 0.),
                                operator[(1, 0)],
                                operator[(1, 1)],
                                Complex::new(0., 0.),
                                Complex::new(0., 0.),
                                Complex::new(0., 0.),
                                Complex::new(0., 0.),
                                Complex::new(1., 0.),
                                Complex::new(0., 0.),
                                Complex::new(0., 0.),
                                Complex::new(0., 0.),
                                Complex::new(0., 0.),
                                Complex::new(1., 0.),
                            )
                        });
                        kroneckered = true;
                    }
                }
                if !kroneckered {
                    if let Some((up_block, up_control)) =
                        block_query.iter().find(|(block_location, _)| {
                            block_location.x == x && block_location.y == y - 1
                        })
                    {
                        if CONTROL_GATES.contains(&up_block.gate) && !up_control.on_top {
                            // is being controlled from below
                            result = result.kronecker(&if block.gate == Gate::C {
                                Matrix4::new(
                                    Complex::new(1., 0.),
                                    Complex::new(0., 0.),
                                    Complex::new(0., 0.),
                                    Complex::new(0., 0.),
                                    Complex::new(0., 0.),
                                    Complex::new(1., 0.),
                                    Complex::new(0., 0.),
                                    Complex::new(0., 0.),
                                    Complex::new(0., 0.),
                                    Complex::new(0., 0.),
                                    operator[(0, 0)],
                                    operator[(0, 1)],
                                    Complex::new(0., 0.),
                                    Complex::new(0., 0.),
                                    operator[(1, 0)],
                                    operator[(1, 1)],
                                )
                            } else {
                                Matrix4::new(
                                    operator[(0, 0)],
                                    operator[(0, 1)],
                                    Complex::new(0., 0.),
                                    Complex::new(0., 0.),
                                    operator[(1, 0)],
                                    operator[(1, 1)],
                                    Complex::new(0., 0.),
                                    Complex::new(0., 0.),
                                    Complex::new(0., 0.),
                                    Complex::new(0., 0.),
                                    Complex::new(1., 0.),
                                    Complex::new(0., 0.),
                                    Complex::new(0., 0.),
                                    Complex::new(0., 0.),
                                    Complex::new(0., 0.),
                                    Complex::new(1., 0.),
                                )
                            });
                            kroneckered = true;
                        }
                    }
                }
                if !kroneckered {
                    result = result.kronecker(&operator);
                }
            }
        } else {
            result = result.kronecker(&Matrix2::new(
                Complex::new(1., 0.),
                Complex::new(0., 0.),
                Complex::new(0., 0.),
                Complex::new(1., 0.),
            ));
        }
    }
    result
}
