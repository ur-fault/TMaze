# TMaze

Simple multiplatform maze solvnig game for terminal written in entirely Rust

## What it can do

### Features

- Responsive to terminal size and resize events
- Variable maze sizes
- Variable maze generation algorithms: Randomized Kruskal's, Depth-First Search
- Timer and move counter
- Show visited places
- Spectator mode, where you can fly and see the map
- Floors, basically 3D mazes (that's what spectator mode is for)

### To do

- Add compiled executables
- Settings, Controls and About screen
- Render path (you will be able to disable this)
- Maybe multiplayer (depends on difficulty of adding it)
- Separate UI to different crate
- Saving and exporting game state, mazes and their generators

## How to run

- Download from GitHub, althought there are old builds, and I won't update them so frequently, maybe in the future
- Build from source

### How to build from source

1. Clone GitHub repository or download as zip, then extract it
2. Go to that folder
3. Run `cargo run --release` to run
4. Alternatively you can only build it with `cargo build --release`
5. Compiled executable is in the folder `./target/release/`
