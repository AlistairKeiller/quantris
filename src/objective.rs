//! Measurement objectives: what the circuit must produce, and measuring it on demand.

use std::time::Duration;

use bevy::prelude::*;

use crate::{
    Score,
    board::{Block, Grid, VIEW_WIDTH, WIRES, cell_to_world},
    pick,
    piece::Falling,
    quantum::{Gate, StateVector},
};

const MEASURE: KeyCode = KeyCode::Space;
const MEASURE_COOLDOWN: Duration = Duration::from_secs(8);
const POINTS_PER_BLOCK: u32 = 10;
const TOLERANCE: f64 = 1e-6;
/// Distance of the measurement icons from the right edge of the view.
const ICON_INSET: f32 = 48.;

/// The outcome distribution the measured wires must show, ignoring phases.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Objective {
    #[default]
    Zero,
    One,
    /// The Bell state Φ⁺: both wires agree.
    Phi,
    /// The Bell state Ψ⁺: the wires disagree.
    Psi,
}

impl Objective {
    pub const ALL: [Objective; 4] = [
        Objective::Zero,
        Objective::One,
        Objective::Phi,
        Objective::Psi,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Objective::Zero => "Measure 0",
            Objective::One => "Measure 1",
            Objective::Phi => "Measure Phi",
            Objective::Psi => "Measure Psi",
        }
    }

    /// Required outcome probabilities, indexed by outcome with the bottom wire as bit 0.
    pub const fn target(self) -> &'static [f64] {
        match self {
            Objective::Zero => &[1., 0.],
            Objective::One => &[0., 1.],
            Objective::Phi => &[0.5, 0., 0., 0.5],
            Objective::Psi => &[0., 0.5, 0.5, 0.],
        }
    }

    /// How many wires are measured, counted up from the bottom wire.
    pub const fn wires(self) -> usize {
        self.target().len().ilog2() as usize
    }

    pub fn accepts(self, outcome: &[f64]) -> bool {
        outcome.len() == self.target().len()
            && outcome
                .iter()
                .zip(self.target())
                .all(|(p, t)| (p - t).abs() < TOLERANCE)
    }
}

/// Time until the next measurement is allowed; starts ready.
#[derive(Resource, Deref, DerefMut)]
pub struct MeasureCooldown(Timer);

impl Default for MeasureCooldown {
    fn default() -> Self {
        let mut timer = Timer::new(MEASURE_COOLDOWN, TimerMode::Once);
        timer.finish();
        Self(timer)
    }
}

/// The icon at the right end of a wire showing that it is measured.
#[derive(Component)]
pub struct MeasureIcon(i32);

pub fn setup_measure_icons(mut commands: Commands, asset_server: Res<AssetServer>) {
    let image = asset_server.load("measure.png");
    for wire in 0..WIRES {
        commands.spawn((
            MeasureIcon(wire),
            Sprite::from_image(image.clone()),
            Transform::from_xyz(
                VIEW_WIDTH / 2. - ICON_INSET,
                cell_to_world(IVec2::new(0, wire)).y,
                1.,
            ),
        ));
    }
}

/// Shows an icon on each measured wire, fading it in as the cooldown runs out.
pub fn sync_measure_icons(
    objective: Res<Objective>,
    cooldown: Res<MeasureCooldown>,
    mut icons: Query<(&MeasureIcon, &mut Sprite, &mut Visibility)>,
) {
    let color = Color::WHITE.with_alpha(0.25 + 0.75 * cooldown.fraction());
    for (icon, mut sprite, mut visibility) in &mut icons {
        visibility.set_if_neq(if (icon.0 as usize) < objective.wires() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        if sprite.color != color {
            sprite.color = color;
        }
    }
}

/// Runs the locked circuit left to right, every wire starting in |0⟩.
pub fn simulate(grid: &Grid) -> StateVector {
    let mut state = StateVector::new(WIRES as usize);
    for column in grid.columns() {
        // Each control conditions the gate on the neighbouring wire it points at.
        let controls: Vec<(usize, (usize, bool))> = column
            .iter()
            .enumerate()
            .filter_map(|(wire, gate)| match *gate {
                Some(Gate::Control { negated, target_below }) => {
                    let target = if target_below { wire.checked_sub(1)? } else { wire + 1 };
                    Some((target, (wire, !negated)))
                }
                _ => None,
            })
            .collect();
        for (wire, gate) in column.iter().enumerate() {
            if let Some(op) = gate.and_then(Gate::operator) {
                let own: Vec<(usize, bool)> = controls
                    .iter()
                    .filter(|(target, _)| *target == wire)
                    .map(|(_, control)| *control)
                    .collect();
                state.apply(&op, wire, &own);
            }
        }
    }
    state
}

/// On demand, grades the locked circuit against the objective. Success scores every block and
/// clears the board for the next objective; failure only costs the cooldown.
pub fn measure(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut cooldown: ResMut<MeasureCooldown>,
    blocks: Query<(Entity, &Block), Without<Falling>>,
    mut objective: ResMut<Objective>,
    mut score: ResMut<Score>,
    mut commands: Commands,
) {
    cooldown.tick(time.delta());
    if !keys.just_pressed(MEASURE) || !cooldown.is_finished() {
        return;
    }
    cooldown.reset();
    let grid = Grid::from_blocks(blocks.iter().map(|(_, block)| block));
    let wires: Vec<usize> = (0..objective.wires()).collect();
    if !objective.accepts(&simulate(&grid).probabilities(&wires)) {
        return;
    }
    for (entity, _) in &blocks {
        score.0 += POINTS_PER_BLOCK;
        commands.entity(entity).despawn();
    }
    *objective = pick(&Objective::ALL);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(x: i32, y: i32, gate: Gate) -> Block {
        Block {
            cell: IVec2::new(x, y),
            gate,
        }
    }

    fn control(x: i32, y: i32, target_below: bool) -> Block {
        block(x, y, Gate::Control { negated: false, target_below })
    }

    #[test]
    fn every_objective_spans_the_wires_its_target_describes() {
        for objective in Objective::ALL {
            assert_eq!(1 << objective.wires(), objective.target().len());
            assert!(objective.accepts(objective.target()));
        }
    }

    #[test]
    fn an_empty_circuit_measures_zero() {
        let outcome = simulate(&Grid::default()).probabilities(&[0]);
        assert!(Objective::Zero.accepts(&outcome));
        assert!(!Objective::One.accepts(&outcome));
    }

    #[test]
    fn hadamard_then_cnot_is_phi() {
        let grid = Grid::from_blocks(&[
            block(0, 0, Gate::H),
            control(1, 0, false),
            block(1, 1, Gate::X),
        ]);
        let outcome = simulate(&grid).probabilities(&[0, 1]);
        assert!(Objective::Phi.accepts(&outcome));
        assert!(!Objective::Psi.accepts(&outcome));
    }

    #[test]
    fn flipping_the_target_first_makes_psi() {
        let grid = Grid::from_blocks(&[
            block(0, 0, Gate::H),
            block(0, 1, Gate::X),
            control(1, 0, false),
            block(1, 1, Gate::X),
        ]);
        assert!(Objective::Psi.accepts(&simulate(&grid).probabilities(&[0, 1])));
    }

    #[test]
    fn controls_can_point_down() {
        let grid = Grid::from_blocks(&[
            block(0, 1, Gate::X),
            control(1, 1, true),
            block(1, 0, Gate::X),
        ]);
        assert!(Objective::One.accepts(&simulate(&grid).probabilities(&[0])));
    }
}
