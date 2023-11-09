use bevy::prelude::*;
use std::fmt;

pub const DROP_PERIOD: f32 = 1.;
pub const FAST_DROP_PERIOD: f32 = 0.2;

pub const BACKGROUND_COLOR: Color = Color::WHITE;

pub const REFERENCE_SCREEN_WIDTH: i32 = 1920;
pub const REFERENCE_SCREEN_HEIGHT: i32 = 1080;

pub const X_COUNT: i32 = 15;
pub const Y_COUNT: i32 = 8;

pub const X_GAPS: f32 = REFERENCE_SCREEN_WIDTH as f32 / (X_COUNT as f32 + 1.);
pub const Y_GAPS: f32 = REFERENCE_SCREEN_HEIGHT as f32 / (Y_COUNT as f32 + 1.);

pub const WIRE_WIDTH: i32 = 4;
pub const WIRE_COLOR: Color = Color::BLACK;

pub const SHAPE_I: ([[i32; 2]; 4], [f32; 2]) = ([[0, 2], [1, 2], [2, 2], [3, 2]], [1.5, 1.5]);
pub const SHAPE_J: ([[i32; 2]; 4], [f32; 2]) = ([[0, 2], [0, 1], [1, 1], [2, 1]], [1., 1.]);
pub const SHAPE_L: ([[i32; 2]; 4], [f32; 2]) = ([[0, 1], [1, 1], [2, 1], [2, 2]], [1., 1.]);
pub const SHAPE_O: ([[i32; 2]; 4], [f32; 2]) = ([[0, 1], [0, 0], [1, 1], [1, 0]], [0.5, 0.5]);
pub const SHAPE_S: ([[i32; 2]; 4], [f32; 2]) = ([[0, 1], [1, 1], [1, 2], [2, 2]], [1., 1.]);
pub const SHAPE_T: ([[i32; 2]; 4], [f32; 2]) = ([[0, 1], [1, 1], [1, 2], [2, 1]], [1., 1.]);
pub const SHAPE_Z: ([[i32; 2]; 4], [f32; 2]) = ([[0, 2], [1, 2], [1, 1], [2, 1]], [1., 1.]);
pub const SHAPES: [([[i32; 2]; 4], [f32; 2]); 7] = [
    SHAPE_I, SHAPE_J, SHAPE_L, SHAPE_O, SHAPE_S, SHAPE_T, SHAPE_Z,
];

#[derive(Clone, Copy, Debug)]
pub enum Gate {
    X,
    Y,
    Z,
    H,
    S,
    T,
}
impl fmt::Display for Gate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}
pub const GATES: [Gate; 6] = [Gate::X, Gate::Y, Gate::Z, Gate::H, Gate::S, Gate::T];

pub const OPERATOR_SIZE: i32 = 96;
pub const OPERATOR_FONT_SIZE: i32 = 96;
pub const OPERATOR_FONT_COLOR: Color = Color::BLACK;
