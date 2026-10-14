//! The circuit board. Each row is a qubit wire and each column one circuit step; gates fall in
//! from the right and settle leftwards. Every placed gate is its own entity for its whole
//! lifetime, and the occupancy grid is derived from those entities whenever a system needs it.

use std::f32::consts::PI;

use bevy::{prelude::*, sprite::Anchor};

use crate::{
    GameState, Score, Sounds,
    piece::{Falling, Shape},
    quantum::Gate,
};

pub const COLUMNS: i32 = 15;
pub const WIRES: i32 = 8;

/// The world-space area the camera always keeps in view.
pub const VIEW_WIDTH: f32 = 1920.;
pub const VIEW_HEIGHT: f32 = 1080.;
pub const COLUMN_GAP: f32 = VIEW_WIDTH / (COLUMNS as f32 + 1.);
pub const WIRE_GAP: f32 = VIEW_HEIGHT / (WIRES as f32 + 1.);
pub const WIRE_WIDTH: f32 = 4.;

const OPERATOR_SIZE: f32 = 64.;
const OPERATOR_FONT_SIZE: f32 = 48.;
const CONTROL_RADIUS: f32 = 16.;
const ANTI_CONTROL_HOLE_RADIUS: f32 = 12.;
/// Blocks sit above the background wires; their labels and holes one step above that.
const BLOCK_Z: f32 = 1.;

/// A gate occupying one cell: `cell.x` is the column (circuit step), `cell.y` the wire.
///
/// Blocks are cleared by restarting, which re-enters [`GameState::Playing`].
#[derive(Component)]
#[require(Transform::from_xyz(0., 0., BLOCK_Z), DespawnOnEnter::<GameState>(GameState::Playing))]
pub struct Block {
    pub cell: IVec2,
    pub gate: Gate,
}

/// The gates on one column, from the bottom wire up.
pub type Column = [Option<Gate>; WIRES as usize];

/// World position of a cell's centre; columns run left to right and wires bottom to top.
pub fn cell_to_world(cell: IVec2) -> Vec2 {
    Vec2::new(
        (cell.x + 1) as f32 * COLUMN_GAP - VIEW_WIDTH / 2.,
        (cell.y + 1) as f32 * WIRE_GAP - VIEW_HEIGHT / 2.,
    )
}

/// Snapshot of the locked gates on the visible grid.
#[derive(Default)]
pub struct Grid {
    columns: [Column; COLUMNS as usize],
}

impl Grid {
    pub fn from_blocks<'a>(blocks: impl IntoIterator<Item = &'a Block>) -> Self {
        let mut grid = Self::default();
        for block in blocks {
            let (Ok(x), Ok(y)) = (usize::try_from(block.cell.x), usize::try_from(block.cell.y))
            else {
                continue;
            };
            if let Some(cell) = grid.columns.get_mut(x).and_then(|column| column.get_mut(y)) {
                *cell = Some(block.gate);
            }
        }
        grid
    }

    /// The columns from left to right.
    pub fn columns(&self) -> impl Iterator<Item = &Column> {
        self.columns.iter()
    }

    /// Falling blocks may enter any empty visible cell, or any cell past the right edge where
    /// pieces spawn.
    pub fn is_free(&self, cell: IVec2) -> bool {
        cell.x >= 0
            && (0..WIRES).contains(&cell.y)
            && self
                .columns
                .get(cell.x as usize)
                .is_none_or(|column| column[cell.y as usize].is_none())
    }

    pub fn fits(&self, cells: impl IntoIterator<Item = IVec2>) -> bool {
        cells.into_iter().all(|cell| self.is_free(cell))
    }
}

/// Meshes and materials shared by every block, created once.
#[derive(Resource)]
pub struct BlockAssets {
    control: Handle<Mesh>,
    anti_control_hole: Handle<Mesh>,
    white: Handle<ColorMaterial>,
    shape_materials: [Handle<ColorMaterial>; Shape::ALL.len()],
}

impl FromWorld for BlockAssets {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let control = meshes.add(Circle::new(CONTROL_RADIUS));
        let anti_control_hole = meshes.add(Circle::new(ANTI_CONTROL_HOLE_RADIUS));
        let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
        Self {
            control,
            anti_control_hole,
            white: materials.add(Color::WHITE),
            shape_materials: Shape::ALL.map(|shape| materials.add(shape.color())),
        }
    }
}

/// Spawns one falling block: a lettered square for an operator, or a dot with a wire stub
/// (and a hole when negated) for a control. The stub points up; [`sync_block_transforms`]
/// turns the whole control over when its target is below.
pub fn spawn_block(
    commands: &mut Commands,
    assets: &BlockAssets,
    shape: Shape,
    cell: IVec2,
    gate: Gate,
    index: usize,
) {
    let color = shape.color();
    if let Gate::Control { negated, .. } = gate {
        let mut block = commands.spawn((
            Block { cell, gate },
            Falling { index },
            Mesh2d(assets.control.clone()),
            MeshMaterial2d(assets.shape_materials[shape as usize].clone()),
            children![(
                Sprite::from_color(color, Vec2::new(WIRE_WIDTH, WIRE_GAP)),
                Anchor::BOTTOM_CENTER,
                Transform::from_xyz(0., 0., -0.5),
            )],
        ));
        if negated {
            block.with_child((
                Mesh2d(assets.anti_control_hole.clone()),
                MeshMaterial2d(assets.white.clone()),
                Transform::from_xyz(0., 0., 1.),
            ));
        }
    } else {
        commands.spawn((
            Block { cell, gate },
            Falling { index },
            Sprite::from_color(color, Vec2::splat(OPERATOR_SIZE)),
            children![(
                Text2d::new(gate.label()),
                TextFont::from_font_size(OPERATOR_FONT_SIZE),
                TextColor::BLACK,
                Transform::from_xyz(0., 0., 1.),
            )],
        ));
    }
}

/// Positions every moved block, hiding those still in the spawn area past the right edge, and
/// points each control's stub at its target.
pub fn sync_block_transforms(
    mut blocks: Query<(&Block, &mut Transform, &mut Visibility), Changed<Block>>,
) {
    for (block, mut transform, mut visibility) in &mut blocks {
        let Vec2 { x, y } = cell_to_world(block.cell);
        transform.translation.x = x;
        transform.translation.y = y;
        if let Gate::Control { target_below, .. } = block.gate {
            transform.rotation = Quat::from_rotation_z(if target_below { PI } else { 0. });
        }
        visibility.set_if_neq(if block.cell.x < COLUMNS {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

/// Clears every full column and scores it Tetris-style.
pub fn clear_full_columns(
    mut commands: Commands,
    blocks: Query<(Entity, &Block), Without<Falling>>,
    mut score: ResMut<Score>,
    sounds: Res<Sounds>,
) {
    let grid = Grid::from_blocks(blocks.iter().map(|(_, block)| block));
    let full: Vec<i32> = grid
        .columns()
        .enumerate()
        .filter(|(_, column)| column.iter().all(Option::is_some))
        .map(|(x, _)| x as i32)
        .collect();
    if full.is_empty() {
        return;
    }
    for (entity, block) in &blocks {
        if full.contains(&block.cell.x) {
            commands.entity(entity).despawn();
        }
    }
    score.0 += match full.len() {
        1 => 100,
        2 => 300,
        3 => 500,
        _ => 800,
    };
    let sound = if full.len() >= 4 { &sounds.quad_clear } else { &sounds.clear };
    commands.spawn((AudioPlayer::new(sound.clone()), PlaybackSettings::DESPAWN));
}

/// Slides blocks left to close any empty columns, as Tetris drops rows.
pub fn collapse_columns(mut blocks: Query<&mut Block, Without<Falling>>) {
    let grid = Grid::from_blocks(blocks.iter());
    // How far each column's blocks slide: the number of empty columns to its left.
    let mut slide = [0; COLUMNS as usize];
    let mut empty = 0;
    for (x, column) in grid.columns().enumerate() {
        slide[x] = empty;
        if column.iter().all(Option::is_none) {
            empty += 1;
        }
    }
    for mut block in &mut blocks {
        let distance = usize::try_from(block.cell.x)
            .ok()
            .and_then(|x| slide.get(x).copied())
            .unwrap_or(empty);
        if distance > 0 {
            block.cell.x -= distance;
        }
    }
}

/// A piece that locks while still past the right edge ends the game.
pub fn check_game_over(
    blocks: Query<&Block, Without<Falling>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if blocks.iter().any(|block| block.cell.x >= COLUMNS) {
        next_state.set(GameState::Lost);
    }
}
