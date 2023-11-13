Link: [http://quantris.pages.dev](http://quantris.pages.dev)

Controls:
+ up/down: move piece up/down
+ left: make piece fall faster
+ right: drop piece
+ X/Z: rotate piece clockwise/counterclockwise

Run: `cargo run`

Build for web:
```
  cargo build --release --target wasm32-unknown-unknown
  wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/quantris.wasm
```
