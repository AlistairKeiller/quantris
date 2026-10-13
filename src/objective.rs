//! Measurement objectives: what the circuit must produce when a probe lands.

use bevy::prelude::*;

use crate::{
    GameState, Score,
    board::{Block, Grid, WIRES},
    pick,
    piece::Falling,
    quantum::{Gate, StateVector},
};

/// Tetrominoes between measurement pieces.
pub const MEASUREMENT_PERIOD: u32 = 10;
const POINTS_PER_BLOCK: u32 = 10;
const TOLERANCE: f64 = 1e-6;

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

    /// Required outcome probabilities, indexed by outcome with the bottom probed wire as bit 0.
    pub const fn target(self) -> &'static [f64] {
        match self {
            Objective::Zero => &[1., 0.],
            Objective::One => &[0., 1.],
            Objective::Phi => &[0.5, 0., 0., 0.5],
            Objective::Psi => &[0., 0.5, 0.5, 0.],
        }
    }

    /// How many wires the measurement piece spans, counted up from the bottom wire.
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

/// Runs the locked circuit from the leftmost column through `column`, every wire starting in |0⟩.
pub fn simulate(grid: &Grid, column: i32) -> StateVector {
    let mut state = StateVector::new(WIRES as usize);
    for gates in (0..=column).filter_map(|x| grid.column(x)) {
        // Each control conditions the gate on the neighbouring wire it points at.
        let controls: Vec<(usize, (usize, bool))> = gates
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
        for (wire, gate) in gates.iter().enumerate() {
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

/// Once a measurement piece has landed, grades the circuit to its left: success clears it for
/// points and draws the next objective, failure ends the game.
pub fn check_measurement(
    mut commands: Commands,
    blocks: Query<(Entity, &Block), Without<Falling>>,
    mut objective: ResMut<Objective>,
    mut score: ResMut<Score>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let probes: Vec<IVec2> = blocks
        .iter()
        .filter(|(_, block)| block.gate == Gate::Measure)
        .map(|(_, block)| block.cell)
        .collect();
    let Some(column) = probes.first().map(|cell| cell.x) else {
        return;
    };
    let mut wires: Vec<usize> = probes.iter().map(|cell| cell.y as usize).collect();
    wires.sort_unstable();
    let grid = Grid::from_blocks(blocks.iter().map(|(_, block)| block));
    if !objective.accepts(&simulate(&grid, column).probabilities(&wires)) {
        next_state.set(GameState::Lost);
        return;
    }
    for (entity, block) in &blocks {
        if block.cell.x < column {
            score.0 += POINTS_PER_BLOCK;
            commands.entity(entity).despawn();
        } else if block.gate == Gate::Measure {
            commands.entity(entity).despawn();
        }
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
    fn untouched_wires_measure_zero() {
        let outcome = simulate(&Grid::default(), 3).probabilities(&[0]);
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
        let outcome = simulate(&grid, 1).probabilities(&[0, 1]);
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
        assert!(Objective::Psi.accepts(&simulate(&grid, 1).probabilities(&[0, 1])));
    }

    #[test]
    fn controls_can_point_down() {
        let grid = Grid::from_blocks(&[
            block(0, 1, Gate::X),
            control(1, 1, true),
            block(1, 0, Gate::X),
        ]);
        assert!(Objective::One.accepts(&simulate(&grid, 1).probabilities(&[0])));
    }

    #[test]
    fn gates_past_the_probe_column_do_not_count() {
        let grid = Grid::from_blocks(&[block(1, 0, Gate::X)]);
        assert!(Objective::Zero.accepts(&simulate(&grid, 0).probabilities(&[0])));
        assert!(Objective::One.accepts(&simulate(&grid, 1).probabilities(&[0])));
    }
}
