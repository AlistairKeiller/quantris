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

#[derive(Clone, Copy)]
pub enum Shape {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
}
impl Shape {
    fn center(&self) -> (f32, f32) {
        match self {
            Shape::I => (1.5, 1.5),
            Shape::J | Shape::L | Shape::S | Shape::T | Shape::Z => (1., 1.),
            Shape::O => (0.5, 0.5),
        }
    }
}
pub const SHAPES: [(Shape, [[i32; 2]; 4]); 7] = [
    (Shape::I, [[0, 2], [1, 2], [2, 2], [3, 2]]),
    (Shape::J, [[0, 2], [0, 1], [1, 1], [2, 1]]),
    (Shape::L, [[0, 1], [1, 1], [2, 1], [2, 2]]),
    (Shape::O, [[0, 1], [0, 0], [1, 1], [1, 0]]),
    (Shape::S, [[0, 1], [1, 1], [1, 2], [2, 2]]),
    (Shape::T, [[0, 1], [1, 1], [1, 2], [2, 1]]),
    (Shape::Z, [[0, 2], [1, 2], [1, 1], [2, 1]]),
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
