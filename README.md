# TMaze

Simple multiplatform maze solving game for terminal written entirely in  Rust

## What it can do

### Features

- Responsive to terminal size and resize events
- Variable maze sizes
- Various maze generation algorithms: Randomized Kruskal's, Depth-First Search
- Timer and move counter
- Show visited places
- Spectator mode, where you can fly and see the map
- Floors, basically 3D mazes (that's what spectator mode is mainly for)

### To do

- Add compiled executables
- Settings, Controls and About screen
- Render path (you will be able to disable this)
- Maybe multiplayer (depends on difficulty of adding it)
- Separate UI to different crate
- Saving and exporting game state, mazes and their generators

## How to run

- Download from GitHub, althought there are old builds, and I won't update them so frequently, maybe in the future
- Build from source, you need cargo installed on your system

### How to build from source

1. Make sure you have [cargo](https://crates.io/) installed
1. Clone GitHub repository or download it as zip, then extract it
1. Go to that folder
1. Run `cargo run --release` to run
1. Alternatively you can just build it with `cargo build --release`
1. Compiled executable is in the folder `./target/release/`

#### Other option is to just install it using cargo

1. Make sure you have [cargo](https://crates.io/) installed
1. Run `cargo install --git https://github.com/ur-fault/TMaze.git`
1. If you want, make sure that `~/.cargo/bin` is in the PATH and enjoy