use bevy::prelude::*;
use nalgebra::*;
use std::fmt;

pub const DROP_PERIOD: f32 = 1.;
pub const FAST_DROP_PERIOD: f32 = 0.2;

pub const BACKGROUND_COLOR: Color = Color::WHITE;

pub const REFERENCE_SCREEN_WIDTH: i32 = 1920;
pub const REFERENCE_SCREEN_HEIGHT: i32 = 1080;

pub const X_COUNT: i32 = 15;
pub const Y_COUNT: i32 = 4;

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
    M,
}
impl Shape {
    pub fn control_on_top(&self, rotation: i32) -> bool {
        match self {
            Shape::J => [false, true, true, false][rotation as usize],
            Shape::L => [false, false, true, true][rotation as usize],
            Shape::O => [true, true, false, false][rotation as usize],
            Shape::S => [false, false, true, true][rotation as usize],
            Shape::T => [false, true, true, true][rotation as usize],
            Shape::Z => [true, false, false, true][rotation as usize],
            Shape::I | Shape::M => true,
        }
    }
    // for O, S, and Z there are 4, 2, and 2 possilbe spawn locations, but we just give one option
    pub fn can_control_spawn(&self, number: i32) -> bool {
        match self {
            Shape::J => [false, true, false, false][number as usize],
            Shape::L => [false, false, true, false][number as usize],
            Shape::O => [true, false, false, false][number as usize],
            Shape::S => [false, true, false, false][number as usize],
            Shape::T => [false, true, false, false][number as usize],
            Shape::Z => [false, true, false, false][number as usize],
            Shape::I | Shape::M => false,
        }
    }
    // from https://tetris.fandom.com/wiki/SRS#Pro
    pub fn rotation_location(&self, number: i32, rotation: i32) -> (i32, i32) {
        match self {
            Shape::I => [
                [(0, 2), (1, 2), (2, 2), (3, 2)],
                [(2, 3), (2, 2), (2, 1), (2, 0)],
                [(3, 1), (2, 1), (1, 1), (0, 1)],
                [(1, 0), (1, 1), (1, 2), (1, 3)],
            ][rotation as usize][number as usize],
            Shape::J => [
                [(0, 2), (0, 1), (1, 1), (2, 1)],
                [(2, 2), (1, 2), (1, 1), (1, 0)],
                [(2, 0), (2, 1), (1, 1), (0, 1)],
                [(0, 0), (1, 0), (1, 1), (1, 2)],
            ][rotation as usize][number as usize],
            Shape::L => [
                [(0, 1), (1, 1), (2, 1), (2, 2)],
                [(1, 2), (1, 1), (1, 0), (2, 0)],
                [(2, 1), (1, 1), (0, 1), (0, 0)],
                [(1, 0), (1, 1), (1, 2), (0, 2)],
            ][rotation as usize][number as usize],
            Shape::O => [
                [(0, 1), (0, 0), (1, 1), (1, 0)],
                [(1, 1), (0, 1), (1, 0), (0, 0)],
                [(1, 0), (1, 1), (0, 0), (0, 1)],
                [(0, 0), (1, 0), (0, 1), (1, 1)],
            ][rotation as usize][number as usize],
            Shape::S => [
                [(0, 1), (1, 1), (1, 2), (2, 2)],
                [(1, 2), (1, 1), (2, 1), (2, 0)],
                [(2, 1), (1, 1), (1, 0), (0, 0)],
                [(1, 0), (1, 1), (0, 1), (0, 2)],
            ][rotation as usize][number as usize],
            Shape::T => [
                [(0, 1), (1, 1), (1, 2), (2, 1)],
                [(1, 2), (1, 1), (2, 1), (1, 0)],
                [(2, 1), (1, 1), (1, 0), (0, 1)],
                [(1, 0), (1, 1), (0, 1), (1, 2)],
            ][rotation as usize][number as usize],
            Shape::Z => [
                [(0, 2), (1, 2), (1, 1), (2, 1)],
                [(2, 2), (2, 1), (1, 1), (1, 0)],
                [(2, 0), (1, 0), (1, 1), (0, 1)],
                [(0, 0), (0, 1), (1, 1), (1, 2)],
            ][rotation as usize][number as usize],
            Shape::M => (0, 0),
        }
    }
    pub fn rotation_location_change(
        &self,
        number: i32,
        initial_rotation: i32,
        final_rotation: i32,
    ) -> (i32, i32) {
        (
            self.rotation_location(number, final_rotation).0
                - self.rotation_location(number, initial_rotation).0,
            self.rotation_location(number, final_rotation).1
                - self.rotation_location(number, initial_rotation).1,
        )
    }
    // from https://tetris.fandom.com/wiki/SRS#Pro rotated 90 degrees clockwisee, then swapped the order by putting by putting the last element first
    pub fn wall_kicks(&self, rotation: i32, clockwise: bool) -> [(i32, i32); 5] {
        match self {
            Shape::J | Shape::L | Shape::O | Shape::S | Shape::T | Shape::Z | Shape::M => [
                [
                    [(0, 0), (0, 1), (-1, 1), (2, 0), (2, 1)],
                    [(0, 0), (0, -1), (1, -1), (-2, 0), (-2, -1)],
                    [(0, 0), (0, -1), (-1, -1), (2, 0), (2, -1)],
                    [(0, 0), (0, 1), (1, 1), (-2, 0), (-2, 1)],
                ],
                [
                    [(0, 0), (0, 1), (-1, 1), (2, 0), (2, 1)],
                    [(0, 0), (0, 1), (1, 1), (-2, 0), (-2, 1)],
                    [(0, 0), (0, -1), (-1, -1), (2, 0), (2, -1)],
                    [(0, 0), (0, -1), (1, -1), (-2, 0), (-2, -1)],
                ],
            ][clockwise as usize][rotation as usize],
            Shape::I => [
                [
                    [(0, 0), (0, 2), (0, -1), (-1, 2), (2, -1)],
                    [(0, 0), (0, 1), (0, -2), (2, 1), (-1, -2)],
                    [(0, 0), (0, -2), (0, 1), (1, -2), (-2, 1)],
                    [(0, 0), (0, -1), (0, 2), (-2, -1), (1, 2)],
                ],
                [
                    [(0, 0), (0, -1), (0, 2), (-2, -1), (1, 2)],
                    [(0, 0), (0, 2), (0, -1), (-1, 2), (2, -1)],
                    [(0, 0), (0, 1), (0, -2), (2, 1), (-1, -2)],
                    [(0, 0), (0, -2), (0, 1), (1, -2), (-2, 1)],
                ],
            ][clockwise as usize][rotation as usize],
        }
    }
}

pub const SHAPES: [Shape; 7] = [
    Shape::I,
    Shape::J,
    Shape::L,
    Shape::O,
    Shape::S,
    Shape::T,
    Shape::Z,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gate {
    X,
    Y,
    Z,
    H,
    S,
    T,
    C,
    AC,
    M,
}
impl fmt::Display for Gate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Gate {
    pub fn operator(&self) -> Option<Matrix2<Complex<f32>>> {
        match self {
            Gate::X => Some(Matrix2::new(
                Complex::new(0., 0.),
                Complex::new(1., 0.),
                Complex::new(1., 0.),
                Complex::new(0., 0.),
            )),
            Gate::Y => Some(Matrix2::new(
                Complex::new(0., 0.),
                Complex::new(0., -1.),
                Complex::new(0., 1.),
                Complex::new(0., 0.),
            )),
            Gate::Z => Some(Matrix2::new(
                Complex::new(1., 0.),
                Complex::new(0., 0.),
                Complex::new(0., 0.),
                Complex::new(-1., 0.),
            )),
            Gate::H => Some(Matrix2::new(
                Complex::new(1. / (2.).sqrt(), 0.),
                Complex::new(1. / (2.).sqrt(), 0.),
                Complex::new(1. / (2.).sqrt(), 0.),
                Complex::new(-1. / (2.).sqrt(), 0.),
            )),
            Gate::S => Some(Matrix2::new(
                Complex::new(1., 0.),
                Complex::new(0., 0.),
                Complex::new(0., 0.),
                Complex::new(0., 1.),
            )),
            Gate::T => Some(Matrix2::new(
                Complex::new(1., 0.),
                Complex::new(0., 0.),
                Complex::new(0., 0.),
                Complex::new(1. / (2.).sqrt(), 1. / (2.).sqrt()),
            )),
            Gate::M => Some(Matrix2::new(
                Complex::new(1., 0.),
                Complex::new(0., 0.),
                Complex::new(0., 0.),
                Complex::new(1., 0.),
            )),
            _ => None,
        }
    }
}
pub const GATES_WITHOUT_CONTROL: [Gate; 6] = [Gate::X, Gate::Y, Gate::Z, Gate::H, Gate::S, Gate::T];
pub const CONTROL_GATES: [Gate; 2] = [Gate::C, Gate::AC];
pub const GATES: [Gate; 8] = [
    Gate::X,
    Gate::Y,
    Gate::Z,
    Gate::H,
    Gate::S,
    Gate::T,
    Gate::C,
    Gate::AC,
];

pub const FASTER_FALL_KEYCODE: KeyCode = KeyCode::Left;
pub const PIECE_UP_KEYCODE: KeyCode = KeyCode::Up;
pub const PIECE_DOWN_KEYCODE: KeyCode = KeyCode::Down;
pub const DROP_PIECE_KEYCODE: KeyCode = KeyCode::Right;
pub const ROTATE_PIECE_CLOCKWISE: KeyCode = KeyCode::X;
pub const ROTATE_PIECE_COUNTERCLOCKWISE: KeyCode = KeyCode::Z;

pub const CONTROL_GATE_CHANCE: f32 = 0.5;
pub const MEASURMENT_GATE_PERIOD: i32 = 5;
