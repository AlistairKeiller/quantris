//! Quantum-circuit simulation on a small state vector.
//!
//! Pure Rust with no engine dependency, so the physics is unit-testable headlessly.

use std::f64::consts::FRAC_1_SQRT_2;
use std::ops::{Add, Mul};

/// A complex amplitude.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub const ZERO: Self = Self::new(0., 0.);
    pub const ONE: Self = Self::new(1., 0.);

    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    pub fn norm_sqr(self) -> f64 {
        self.re * self.re + self.im * self.im
    }
}

impl Add for Complex {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.re + rhs.re, self.im + rhs.im)
    }
}

impl Mul for Complex {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self::new(
            self.re * rhs.re - self.im * rhs.im,
            self.re * rhs.im + self.im * rhs.re,
        )
    }
}

/// A single-wire operator, row-major.
pub type Operator = [[Complex; 2]; 2];

/// What a block does to the circuit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Gate {
    X,
    Y,
    Z,
    H,
    /// Makes the gate on the neighbouring wire conditional on this wire being |1⟩,
    /// or |0⟩ when `negated` (an anti-control).
    Control { negated: bool, target_below: bool },
}

impl Gate {
    /// The matrix this gate applies to its wire; a control applies none of its own.
    pub const fn operator(self) -> Option<Operator> {
        const fn c(re: f64, im: f64) -> Complex {
            Complex::new(re, im)
        }
        const R: f64 = FRAC_1_SQRT_2;
        Some(match self {
            Gate::X => [[c(0., 0.), c(1., 0.)], [c(1., 0.), c(0., 0.)]],
            Gate::Y => [[c(0., 0.), c(0., -1.)], [c(0., 1.), c(0., 0.)]],
            Gate::Z => [[c(1., 0.), c(0., 0.)], [c(0., 0.), c(-1., 0.)]],
            Gate::H => [[c(R, 0.), c(R, 0.)], [c(R, 0.), c(-R, 0.)]],
            Gate::Control { .. } => return None,
        })
    }

    pub const fn label(self) -> &'static str {
        match self {
            Gate::X => "X",
            Gate::Y => "Y",
            Gate::Z => "Z",
            Gate::H => "H",
            Gate::Control { negated: false, .. } => "C",
            Gate::Control { negated: true, .. } => "AC",
        }
    }
}

/// Amplitudes of a register of wires; wire `w` is bit `w` of each basis-state index.
pub struct StateVector {
    amplitudes: Vec<Complex>,
}

impl StateVector {
    /// Every wire starts in |0⟩.
    pub fn new(wires: usize) -> Self {
        let mut amplitudes = vec![Complex::ZERO; 1 << wires];
        amplitudes[0] = Complex::ONE;
        Self { amplitudes }
    }

    /// Applies `op` to `target` on every basis state where each control wire holds its
    /// required value.
    pub fn apply(&mut self, op: &Operator, target: usize, controls: &[(usize, bool)]) {
        let target_bit = 1 << target;
        for low in 0..self.amplitudes.len() {
            // Visit each (|target=0⟩, |target=1⟩) pair once, from its lower member.
            if low & target_bit != 0 {
                continue;
            }
            let satisfied = controls
                .iter()
                .all(|&(wire, value)| ((low >> wire) & 1 == 1) == value);
            if !satisfied {
                continue;
            }
            let high = low | target_bit;
            let (a, b) = (self.amplitudes[low], self.amplitudes[high]);
            self.amplitudes[low] = op[0][0] * a + op[0][1] * b;
            self.amplitudes[high] = op[1][0] * a + op[1][1] * b;
        }
    }

    /// Probability of each joint outcome of `wires`, where outcome bit `i` is the value of
    /// `wires[i]`.
    pub fn probabilities(&self, wires: &[usize]) -> Vec<f64> {
        let mut probabilities = vec![0.; 1 << wires.len()];
        for (index, amplitude) in self.amplitudes.iter().enumerate() {
            let outcome = wires
                .iter()
                .enumerate()
                .fold(0, |outcome, (bit, &wire)| outcome | ((index >> wire) & 1) << bit);
            probabilities[outcome] += amplitude.norm_sqr();
        }
        probabilities
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len());
        for (a, e) in actual.iter().zip(expected) {
            assert!((a - e).abs() < 1e-12, "{actual:?} != {expected:?}");
        }
    }

    fn op(gate: Gate) -> Operator {
        gate.operator().expect("operator gate")
    }

    #[test]
    fn x_flips_a_wire_and_marginals_ignore_the_others() {
        let mut state = StateVector::new(2);
        state.apply(&op(Gate::X), 1, &[]);
        assert_close(&state.probabilities(&[1]), &[0., 1.]);
        assert_close(&state.probabilities(&[0]), &[1., 0.]);
    }

    #[test]
    fn phases_are_invisible_alone_but_interfere_under_hadamard() {
        let mut state = StateVector::new(1);
        state.apply(&op(Gate::H), 0, &[]);
        state.apply(&op(Gate::Z), 0, &[]);
        assert_close(&state.probabilities(&[0]), &[0.5, 0.5]);
        state.apply(&op(Gate::H), 0, &[]);
        assert_close(&state.probabilities(&[0]), &[0., 1.]);
    }

    #[test]
    fn cnot_after_hadamard_is_a_bell_pair() {
        let mut state = StateVector::new(2);
        state.apply(&op(Gate::H), 0, &[]);
        state.apply(&op(Gate::X), 1, &[(0, true)]);
        assert_close(&state.probabilities(&[0, 1]), &[0.5, 0., 0., 0.5]);
    }

    #[test]
    fn controls_fire_on_one_and_anti_controls_on_zero() {
        let mut state = StateVector::new(2);
        state.apply(&op(Gate::X), 1, &[(0, true)]);
        assert_close(&state.probabilities(&[0, 1]), &[1., 0., 0., 0.]);
        state.apply(&op(Gate::X), 1, &[(0, false)]);
        assert_close(&state.probabilities(&[0, 1]), &[0., 0., 1., 0.]);
    }
}
