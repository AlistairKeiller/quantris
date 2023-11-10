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
    // from https://tetris.fandom.com/wiki/SRS#Pro rotated 90 degrees counterclockwise, and then with their order shifted 90 degrees counterclockwise
    // add counterclockwise
    pub fn wall_kicks(&self, rotation: i32, clockwise: bool) -> [(i32, i32); 5] {
        match self {
            Shape::J | Shape::L | Shape::O | Shape::S | Shape::T | Shape::Z => [
                [
                    [(0, 0), (0, -1), (1, -1), (-2, 0), (-2, -1)],
                    [(0, 0), (0, -1), (-1, -1), (2, 0), (2, -1)],
                    [(0, 0), (0, 1), (1, 1), (-2, 0), (-2, 1)],
                    [(0, 0), (0, 1), (-1, 1), (2, 0), (2, 1)],
                ],
                [
                    [(0, 0), (0, 1), (1, 1), (-2, 0), (-2, 1)],
                    [(0, 0), (0, -1), (-1, -1), (2, 0), (2, -1)],
                    [(0, 0), (0, -1), (1, -1), (-2, 0), (-2, -1)],
                    [(0, 0), (0, 1), (-1, 1), (2, 0), (2, 1)],
                ],
            ][clockwise as usize][rotation as usize],
            Shape::I => [
                [
                    [(0, 0), (0, -1), (0, 2), (-2, -1), (1, 2)],
                    [(0, 0), (0, 2), (0, -1), (-1, 2), (2, -1)],
                    [(0, 0), (0, 1), (0, -2), (2, 1), (-1, -2)],
                    [(0, 0), (0, -2), (0, 1), (1, -2), (-2, 1)],
                ],
                [
                    [(0, 0), (0, -2), (0, 1), (1, -2), (-2, 1)],
                    [(0, 0), (0, -1), (0, 2), (-2, -1), (1, 2)],
                    [(0, 0), (0, 2), (0, -1), (-1, 2), (2, -1)],
                    [(0, 0), (0, 1), (0, -2), (2, 1), (-1, -2)],
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
    }
}
pub const GATES: [Gate; 6] = [Gate::X, Gate::Y, Gate::Z, Gate::H, Gate::S, Gate::T];

pub const OPERATOR_SIZE: i32 = 64;
pub const OPERATOR_FONT_SIZE: i32 = 48;
pub const OPERATOR_FONT_COLOR: Color = Color::BLACK;
