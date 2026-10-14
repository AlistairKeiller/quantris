//! Tetromino pieces: SRS shape tables, spawning, holding, steering, gravity and locking.

use std::time::Duration;

use bevy::prelude::*;

use crate::{
    Sounds,
    board::{Block, BlockAssets, COLUMNS, Grid, spawn_block},
    pick,
    quantum::Gate,
};

const FALL_PERIOD: Duration = Duration::from_millis(1000);
const SOFT_DROP_PERIOD: Duration = Duration::from_millis(200);
const SOFT_DROP: KeyCode = KeyCode::ArrowLeft;
const HARD_DROP: KeyCode = KeyCode::ArrowRight;
const SHIFT_UP: KeyCode = KeyCode::ArrowUp;
const SHIFT_DOWN: KeyCode = KeyCode::ArrowDown;
const ROTATE_CLOCKWISE: KeyCode = KeyCode::KeyX;
const ROTATE_COUNTERCLOCKWISE: KeyCode = KeyCode::KeyZ;
const HOLD: KeyCode = KeyCode::KeyC;
/// Gates a tetromino cell may hold when it is not the control cell.
const OPERATOR_GATES: [Gate; 4] = [Gate::X, Gate::Y, Gate::Z, Gate::H];
/// Where pieces appear: the last column, so most of a piece starts past the right edge.
const SPAWN: IVec2 = IVec2::new(COLUMNS - 1, 0);

/// Marks a block as part of the piece the player is steering; `index` is its slot in the
/// shape's SRS tables.
#[derive(Component)]
pub struct Falling {
    pub index: usize,
}

/// A piece: its shape and the gates on its four cells in the base rotation.
#[derive(Clone, Copy)]
pub struct Tetromino {
    pub shape: Shape,
    pub gates: [Gate; 4],
}

impl Tetromino {
    /// A random shape carrying random operators, with a control or anti-control on the
    /// shape's control cell.
    pub fn random() -> Self {
        let shape = pick(&Shape::ALL);
        let gates = std::array::from_fn(|index| {
            if shape.control_index() == Some(index) {
                Gate::Control {
                    negated: rand::random_bool(0.5),
                    target_below: shape.control_target_below(0),
                }
            } else {
                pick(&OPERATOR_GATES)
            }
        });
        Self { shape, gates }
    }

    /// Spawns the piece in the spawn area and makes it the active one.
    fn spawn(self, commands: &mut Commands, assets: &BlockAssets, active: &mut ActivePiece) {
        for (index, offset) in self.shape.cells(0).into_iter().enumerate() {
            spawn_block(commands, assets, self.shape, SPAWN + offset, self.gates[index], index);
        }
        active.tetromino = self;
        active.rotation = 0;
        active.fall_timer.reset();
    }
}

/// The falling piece's identity and orientation, plus the gravity clock.
#[derive(Resource)]
pub struct ActivePiece {
    pub tetromino: Tetromino,
    pub rotation: usize,
    pub fall_timer: Timer,
}

impl Default for ActivePiece {
    fn default() -> Self {
        Self {
            tetromino: Tetromino::random(),
            rotation: 0,
            fall_timer: Timer::new(FALL_PERIOD, TimerMode::Repeating),
        }
    }
}

/// The stashed piece. Holding is allowed once per piece: `used` clears when the next piece
/// spawns after a lock.
#[derive(Resource, Default)]
pub struct Hold {
    pub piece: Option<Tetromino>,
    used: bool,
}

/// The blocks of the falling piece.
type Piece<'w, 's> = Query<'w, 's, (Entity, &'static mut Block, &'static Falling)>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    pub const ALL: [Shape; 7] = [
        Shape::I,
        Shape::J,
        Shape::L,
        Shape::O,
        Shape::S,
        Shape::T,
        Shape::Z,
    ];

    pub const fn color(self) -> Color {
        match self {
            Shape::I => Color::srgb_u8(250, 116, 166),
            Shape::J => Color::srgb_u8(5, 186, 182),
            Shape::L => Color::srgb_u8(187, 139, 255),
            Shape::O => Color::srgb_u8(111, 165, 255),
            Shape::S => Color::srgb_u8(169, 139, 171),
            Shape::T => Color::srgb_u8(88, 162, 177),
            Shape::Z => Color::srgb_u8(218, 128, 210),
        }
    }

    /// Which cell carries the control gate. The I piece has none: lying along a wire it would
    /// have nothing to point at.
    pub const fn control_index(self) -> Option<usize> {
        match self {
            Shape::I => None,
            Shape::J | Shape::S | Shape::T | Shape::Z => Some(1),
            Shape::L => Some(2),
            Shape::O => Some(0),
        }
    }

    /// Whether the control's target sits on the wire below it (otherwise above) in `rotation`.
    pub const fn control_target_below(self, rotation: usize) -> bool {
        match self {
            Shape::I => true,
            Shape::J => [false, true, true, false][rotation],
            Shape::L => [false, false, true, true][rotation],
            Shape::O => [true, true, false, false][rotation],
            Shape::S => [false, false, true, true][rotation],
            Shape::T => [false, true, true, true][rotation],
            Shape::Z => [true, false, false, true][rotation],
        }
    }

    /// Cell offsets of the four blocks in `rotation`, from the SRS tables at
    /// <https://tetris.fandom.com/wiki/SRS>. Gravity points along -x here, so the tables are
    /// transposed relative to upright Tetris.
    pub const fn cells(self, rotation: usize) -> [IVec2; 4] {
        CELLS[self as usize][rotation]
    }

    /// Offsets to try in order when rotating out of `rotation`, from the SRS wall-kick tables
    /// at the same source, rotated to this board's orientation.
    pub const fn kicks(self, rotation: usize, clockwise: bool) -> [IVec2; 5] {
        let table = match self {
            Shape::I => I_KICKS,
            _ => KICKS,
        };
        table[clockwise as usize][rotation]
    }
}

const fn v(x: i32, y: i32) -> IVec2 {
    IVec2::new(x, y)
}

/// `[shape][rotation][block]`, in [`Shape::ALL`] order.
const CELLS: [[[IVec2; 4]; 4]; 7] = [
    [
        [v(0, 2), v(1, 2), v(2, 2), v(3, 2)],
        [v(2, 3), v(2, 2), v(2, 1), v(2, 0)],
        [v(3, 1), v(2, 1), v(1, 1), v(0, 1)],
        [v(1, 0), v(1, 1), v(1, 2), v(1, 3)],
    ],
    [
        [v(0, 2), v(0, 1), v(1, 1), v(2, 1)],
        [v(2, 2), v(1, 2), v(1, 1), v(1, 0)],
        [v(2, 0), v(2, 1), v(1, 1), v(0, 1)],
        [v(0, 0), v(1, 0), v(1, 1), v(1, 2)],
    ],
    [
        [v(0, 1), v(1, 1), v(2, 1), v(2, 2)],
        [v(1, 2), v(1, 1), v(1, 0), v(2, 0)],
        [v(2, 1), v(1, 1), v(0, 1), v(0, 0)],
        [v(1, 0), v(1, 1), v(1, 2), v(0, 2)],
    ],
    [
        [v(0, 1), v(0, 0), v(1, 1), v(1, 0)],
        [v(1, 1), v(0, 1), v(1, 0), v(0, 0)],
        [v(1, 0), v(1, 1), v(0, 0), v(0, 1)],
        [v(0, 0), v(1, 0), v(0, 1), v(1, 1)],
    ],
    [
        [v(0, 1), v(1, 1), v(1, 2), v(2, 2)],
        [v(1, 2), v(1, 1), v(2, 1), v(2, 0)],
        [v(2, 1), v(1, 1), v(1, 0), v(0, 0)],
        [v(1, 0), v(1, 1), v(0, 1), v(0, 2)],
    ],
    [
        [v(0, 1), v(1, 1), v(1, 2), v(2, 1)],
        [v(1, 2), v(1, 1), v(2, 1), v(1, 0)],
        [v(2, 1), v(1, 1), v(1, 0), v(0, 1)],
        [v(1, 0), v(1, 1), v(0, 1), v(1, 2)],
    ],
    [
        [v(0, 2), v(1, 2), v(1, 1), v(2, 1)],
        [v(2, 2), v(2, 1), v(1, 1), v(1, 0)],
        [v(2, 0), v(1, 0), v(1, 1), v(0, 1)],
        [v(0, 0), v(0, 1), v(1, 1), v(1, 2)],
    ],
];

/// `[clockwise][rotation][attempt]` for every shape but I.
const KICKS: [[[IVec2; 5]; 4]; 2] = [
    [
        [v(0, 0), v(0, 1), v(-1, 1), v(2, 0), v(2, 1)],
        [v(0, 0), v(0, -1), v(1, -1), v(-2, 0), v(-2, -1)],
        [v(0, 0), v(0, -1), v(-1, -1), v(2, 0), v(2, -1)],
        [v(0, 0), v(0, 1), v(1, 1), v(-2, 0), v(-2, 1)],
    ],
    [
        [v(0, 0), v(0, 1), v(-1, 1), v(2, 0), v(2, 1)],
        [v(0, 0), v(0, 1), v(1, 1), v(-2, 0), v(-2, 1)],
        [v(0, 0), v(0, -1), v(-1, -1), v(2, 0), v(2, -1)],
        [v(0, 0), v(0, -1), v(1, -1), v(-2, 0), v(-2, -1)],
    ],
];

const I_KICKS: [[[IVec2; 5]; 4]; 2] = [
    [
        [v(0, 0), v(0, 2), v(0, -1), v(-1, 2), v(2, -1)],
        [v(0, 0), v(0, 1), v(0, -2), v(2, 1), v(-1, -2)],
        [v(0, 0), v(0, -2), v(0, 1), v(1, -2), v(-2, 1)],
        [v(0, 0), v(0, -1), v(0, 2), v(-2, -1), v(1, 2)],
    ],
    [
        [v(0, 0), v(0, -1), v(0, 2), v(-2, -1), v(1, 2)],
        [v(0, 0), v(0, 2), v(0, -1), v(-1, 2), v(2, -1)],
        [v(0, 0), v(0, 1), v(0, -2), v(2, 1), v(-1, -2)],
        [v(0, 0), v(0, -2), v(0, 1), v(1, -2), v(-2, 1)],
    ],
];

/// Spawns a fresh piece once nothing is falling, which also makes holding available again.
pub fn spawn_piece(
    mut commands: Commands,
    piece: Query<(), With<Falling>>,
    mut active: ResMut<ActivePiece>,
    mut hold: ResMut<Hold>,
    assets: Res<BlockAssets>,
) {
    if piece.is_empty() {
        Tetromino::random().spawn(&mut commands, &assets, &mut active);
        hold.used = false;
    }
}

/// Stashes the falling piece and brings out the held one, or a fresh one if nothing was held.
pub fn hold_piece(
    keys: Res<ButtonInput<KeyCode>>,
    mut hold: ResMut<Hold>,
    mut active: ResMut<ActivePiece>,
    piece: Query<Entity, With<Falling>>,
    mut commands: Commands,
    assets: Res<BlockAssets>,
) {
    if !keys.just_pressed(HOLD) || hold.used || piece.is_empty() {
        return;
    }
    for entity in &piece {
        commands.entity(entity).despawn();
    }
    let next = hold.piece.replace(active.tetromino).unwrap_or_else(Tetromino::random);
    next.spawn(&mut commands, &assets, &mut active);
    hold.used = true;
}

/// Nudges the piece one wire up or down.
pub fn shift_piece(
    keys: Res<ButtonInput<KeyCode>>,
    locked: Query<&Block, Without<Falling>>,
    mut piece: Piece,
) {
    let step = i32::from(keys.just_pressed(SHIFT_UP)) - i32::from(keys.just_pressed(SHIFT_DOWN));
    if step != 0 {
        try_move(&Grid::from_blocks(locked), &mut piece, IVec2::new(0, step));
    }
}

/// Rotates the piece, trying each SRS wall kick until one fits.
pub fn rotate_piece(
    keys: Res<ButtonInput<KeyCode>>,
    mut active: ResMut<ActivePiece>,
    locked: Query<&Block, Without<Falling>>,
    mut piece: Piece,
) {
    let clockwise = keys.just_pressed(ROTATE_CLOCKWISE);
    if clockwise == keys.just_pressed(ROTATE_COUNTERCLOCKWISE) {
        return; // neither pressed, or both cancelling out
    }
    let shape = active.tetromino.shape;
    let from = active.rotation;
    let to = (from + if clockwise { 1 } else { 3 }) % 4;
    let destination = |block: &Block, falling: &Falling, kick: IVec2| {
        block.cell - shape.cells(from)[falling.index] + shape.cells(to)[falling.index] + kick
    };
    let grid = Grid::from_blocks(locked);
    let fits = |kick: IVec2| {
        grid.fits(
            piece
                .iter()
                .map(|(_, block, falling)| destination(block, falling, kick)),
        )
    };
    let Some(kick) = shape.kicks(from, clockwise).into_iter().find(|&kick| fits(kick)) else {
        return;
    };
    for (_, mut block, falling) in &mut piece {
        block.cell = destination(&block, falling, kick);
        if let Gate::Control { target_below, .. } = &mut block.gate {
            *target_below = shape.control_target_below(to);
        }
    }
    active.rotation = to;
}

/// Gravity: the piece steps one column left every [`FALL_PERIOD`], faster while soft-dropping,
/// and locks once it cannot.
pub fn fall_piece(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut active: ResMut<ActivePiece>,
    locked: Query<&Block, Without<Falling>>,
    mut piece: Piece,
    mut commands: Commands,
) {
    let period = if keys.pressed(SOFT_DROP) { SOFT_DROP_PERIOD } else { FALL_PERIOD };
    active.fall_timer.set_duration(period);
    active.fall_timer.tick(time.delta());
    if keys.just_pressed(SOFT_DROP) {
        active.fall_timer.reset();
    } else if !active.fall_timer.just_finished() {
        return;
    }
    if !try_move(&Grid::from_blocks(locked), &mut piece, IVec2::NEG_X) {
        lock_piece(&mut commands, &piece);
    }
}

/// Drops the piece as far left as it goes and locks it at once.
pub fn hard_drop(
    keys: Res<ButtonInput<KeyCode>>,
    locked: Query<&Block, Without<Falling>>,
    mut piece: Piece,
    mut commands: Commands,
    sounds: Res<Sounds>,
) {
    // An empty piece fits everywhere, so the search below would never end.
    if !keys.just_pressed(HARD_DROP) || piece.is_empty() {
        return;
    }
    let grid = Grid::from_blocks(locked);
    let mut distance = 0;
    while grid.fits(
        piece
            .iter()
            .map(|(_, block, _)| block.cell - IVec2::new(distance + 1, 0)),
    ) {
        distance += 1;
    }
    for (_, mut block, _) in &mut piece {
        block.cell.x -= distance;
    }
    lock_piece(&mut commands, &piece);
    commands.spawn((AudioPlayer::new(sounds.drop.clone()), PlaybackSettings::DESPAWN));
}

/// Moves every falling block by `delta` if all destinations are free; returns whether it moved.
fn try_move(grid: &Grid, piece: &mut Piece, delta: IVec2) -> bool {
    if !grid.fits(piece.iter().map(|(_, block, _)| block.cell + delta)) {
        return false;
    }
    for (_, mut block, _) in piece.iter_mut() {
        block.cell += delta;
    }
    true
}

/// Turns the falling blocks into locked ones.
fn lock_piece(commands: &mut Commands, piece: &Piece) {
    for (entity, ..) in piece.iter() {
        commands.entity(entity).remove::<Falling>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_rotation_has_four_distinct_cells() {
        for shape in Shape::ALL {
            for rotation in 0..4 {
                let cells = shape.cells(rotation);
                for (i, cell) in cells.iter().enumerate() {
                    assert!(!cells[..i].contains(cell), "{shape:?} rotation {rotation}");
                }
            }
        }
    }

    #[test]
    fn control_cells_point_at_another_cell_of_the_same_piece() {
        for shape in Shape::ALL {
            let Some(index) = shape.control_index() else { continue };
            for rotation in 0..4 {
                let cells = shape.cells(rotation);
                let step = if shape.control_target_below(rotation) { IVec2::NEG_Y } else { IVec2::Y };
                assert!(cells.contains(&(cells[index] + step)), "{shape:?} rotation {rotation}");
            }
        }
    }
}
