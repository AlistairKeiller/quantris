<div align="center">

<img src="logo.svg" width="256px" />

# Quantris

</div>

Tetris where every block is a quantum gate. Pieces fall onto a circuit of eight qubit wires,
and every few pieces a measurement probe lands: the circuit to its left must produce the
requested outcome, or the game is over.

Video demonstration: https://www.youtube.com/watch?v=_Uonyu2nNHg

## Controls

+ Up/Down: move the piece up or down a wire
+ Left (hold): fall faster
+ Right: drop the piece
+ X/Z: rotate clockwise/counterclockwise
+ R: restart

## Rules

+ Measurement: after every ten tetrominoes a measurement probe spanning one or two wires falls. When it lands,
  the probed wires must show the current objective's outcome probabilities (phases are ignored).
  Success clears every block to the probe's left for 10 points each; failure ends the game.
+ Clearing columns: a column with a gate on every wire clears for 100, 300, 500 or 800 points for
  one to four columns at once, and the columns to its right slide left.
+ Falling: pieces step one column left every second, or five times a second while Left is held.
+ Rotation: standard SRS tetromino rotations with wall kicks, transposed so gravity points left.
+ Losing: a failed measurement, or a piece that locks while still past the right edge.
+ Pieces: a random tetromino carrying random X, Y, Z and H gates, with one cell (none on the I)
  holding a control or anti-control, 50/50, that conditions the gate next to it.

## Architecture

+ `src/quantum.rs`: the state-vector simulation (gates, controlled gates, outcome
  probabilities). Pure Rust with unit tests; it knows nothing about Bevy.
+ `src/board.rs`: every placed gate is one `Block` entity that lives until it is cleared; there is
  no array of cells. `Grid` is an occupancy snapshot built from the blocks whenever a system needs
  collision checks or whole-column logic. Also column clearing and collapsing.
+ `src/piece.rs`: tetromino shapes and SRS tables, spawning, steering, gravity and locking. The
  only difference between a falling and a locked block is the `Falling` component; locking removes
  it and sends `PieceLocked`.
+ `src/objective.rs`: the measurement objectives and the circuit evaluation when a probe lands.
+ `src/ui.rs`: score and objective labels, the game-over screen, restarting.
+ `src/main.rs`: app wiring. Each frame runs restart, spawn, steer, fall, hard drop; then, only in
  a frame where a piece locked: clear full columns, grade a landed probe, collapse empty columns,
  check for game over; then the labels and block transforms (only for blocks that moved).
+ Restarting is just re-entering the `Playing` state: blocks carry `DespawnOnEnter(Playing)`,
  and the enter schedule resets the score and the piece state.

## Quantum simulation

+ The eight wires are a 256-amplitude state vector starting in |00000000⟩; wire `y` is bit `y` of
  the basis-state index.
+ A column's gates act on distinct wires, so they are applied one at a time directly to the
  vector, costing O(2ⁿ) per gate. A control contributes no gate of its own; it conditions the gate
  on the wire it points at on its own wire being |1⟩ (|0⟩ for an anti-control).
+ A probe reads the joint outcome distribution of the wires it covers and compares it with the
  objective's target probabilities within a small tolerance.

## Developing

```
cargo run
cargo test
```
