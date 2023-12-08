use bevy::prelude::*;
use nalgebra::*;

use crate::constants::*;
use crate::*;

pub fn get_operator_of_column(
    block_query: &Query<&Block, Without<Piece>>,
    control_block_query: &Query<(&Block, &Control), Without<Piece>>,
    x: i32,
) -> DMatrix<Complex<f64>> {
    let mut result: DMatrix<Complex<f64>> = dmatrix![Complex::new(1., 0.)];
    for y in 0..Y_COUNT {
        if let Some(block) = block_query
            .iter()
            .find(|block_location| block_location.x == x && block_location.y == y)
        {
            if let Some(operator) = block.gate.operator() {
                let mut kroneckered = false;
                for (control_block, control) in control_block_query {
                    if control_block.x == x && control_block.y == y + 1 && control.on_top {
                        result = result.kronecker(&if control_block.gate == Gate::C {
                            Matrix4::new(
                                Complex::new(1., 0.),
                                Complex::new(0., 0.),
                                Complex::new(0., 0.),
                                Complex::new(0., 0.),
                                Complex::new(0., 0.),
                                operator[(0, 0)],
                                Complex::new(0., 0.),
                                operator[(0, 1)],
                                Complex::new(0., 0.),
                                Complex::new(0., 0.),
                                Complex::new(1., 0.),
                                Complex::new(0., 0.),
                                Complex::new(0., 0.),
                                operator[(0, 0)],
                                Complex::new(0., 0.),
                                operator[(0, 0)],
                            )
                        } else {
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
                        });
                        kroneckered = true;
                    }
                    if control_block.x == x && control_block.y == y - 1 && !control.on_top {
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

pub fn get_state_of_column(
    block_query: &Query<&Block, Without<Piece>>,
    control_query: &Query<(&Block, &Control), Without<Piece>>,
    x: i32,
) -> DVector<Complex<f64>> {
    let mut state: DVector<Complex<f64>> = DVector::zeros(2_usize.pow(Y_COUNT as u32));
    state[0] = Complex::new(1., 0.);
    for x in 0..x + 1 {
        state = get_operator_of_column(block_query, control_query, x) * state;
    }
    return state;
}

pub fn get_partial_state_of_column(
    block_query: &Query<&Block, Without<Piece>>,
    control_query: &Query<(&Block, &Control), Without<Piece>>,
    measure_block_locations: Vec<(i32, i32)>,
) -> DVector<f64> {
    let state = get_state_of_column(block_query, control_query, measure_block_locations[0].0);
    let mut partial_state: DVector<f64> =
        DVector::zeros(2_usize.pow(measure_block_locations.len() as u32));

    for (idx, amp) in state.iter().enumerate() {
        let mut partial_index = 0;
        for (i, &(_, y)) in measure_block_locations.iter().enumerate() {
            if (idx >> (Y_COUNT - 1 - y)) & 1 == 1 {
                partial_index |= 1 << i;
            }
        }
        partial_state[partial_index] += amp.norm_sqr();
    }

    partial_state
}

pub fn satisfies_objective(
    desired_state: DVector<f64>,
    block_query: &Query<&Block, Without<Piece>>,
    control_query: &Query<(&Block, &Control), Without<Piece>>,
    measure_block_locations: Vec<(i32, i32)>,
) -> bool {
    println!("{}", desired_state);
    println!(
        "{}",
        get_partial_state_of_column(block_query, control_query, measure_block_locations.clone())
    );
    println!(
        "{}",
        get_state_of_column(block_query, control_query, measure_block_locations[0].0)
    );
    get_partial_state_of_column(block_query, control_query, measure_block_locations)
        .iter()
        .zip(desired_state.iter())
        .all(|(a, b)| (a - b).abs() < TOLERANCE)
}
