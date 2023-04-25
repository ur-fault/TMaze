# TMaze

Simple multiplatform maze solving game for terminal written entirely in  Rust

### Install with `cargo install tmaze` and run with `tmaze`
---

![](https://img.shields.io/crates/d/tmaze)
![Crates.io](https://img.shields.io/crates/v/tmaze)

![Screenshot of in-game](https://github.com/ur-fault/tmaze/blob/master/readme_assets/screenshot_ingame.png?raw=true)

## What's this

### Features
- Responsive to terminal size
- Configurable maze sizes through config file
- Configurable colors
- Various maze generation algorithms: Randomized Kruskal's, Depth-First Search
- Timer and move counter
- Show visited places
- Spectator mode, where you can fly and see the gameboard
- Floors and 3D mazes (that's what spectator mode is mainly for)


### Rationale
Since I'm a student, I've got to attend classes, but even when I'm listening I wanted to do something more than sit there. Also at the time this project came to life, I started to learn Rust, so it seemed to make sense to make some kind of game, but since my notebook is not the newest and I wanted to make it as lightweight as possible, I decided to make it for a terminal. It's also pretty cool.

Another requirement was that it would be multiplatform so that I could play it anywhere. A bonus was that I could play it on the server.

And it ended up as maze solving game because I just couldn't find any other.

### To do
- Better settings UI
- Render path (you will be able to disable this)
- Maybe multiplayer
- Saving and exporting and importing game state, mazes

## How to run
- You can either download from GitHub releases,  they are built automatically now, using GitHub Actions ~~althought there are old builds, and I won't update them so frequently, maybe in the future~~
- or build from source, you need cargo installed on your system

### How to build from source
1. Make sure you have [cargo](https://crates.io/) installed
1. Clone GitHub repository or download it as zip, then extract it
1. Go to that folder
1. Run command `cargo run --release` to run (or you can just build it with `cargo build --release` without runing it)
1. You can find compiled executable in the folder `./target/release/`, which you can move or link somewhere else

#### Other option is to just install it using cargo
1. Make sure you have [cargo](https://crates.io/) installed
1. Run `cargo install tmaze`
1. If you want, make sure that `~/.cargo/bin` is in the PATH and enjoy
