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

### Credits and thanks
- Music and OST - [step](https://github.com/PhntD)
- Marketing - [PhntD](https://github.com/PhntD)
- Marketing - Inženýr
- Random stuff - [filip2cz](https://github.com/filip2cz/)
- Playtest - everyone
- Everything else - [me, ie. ur-fault](https://github.com/ur-fault)

## How to run
- You can either download from GitHub releases,  they are built automatically now, using GitHub Actions ~~althought there are old builds, and I won't update them so frequently, maybe in the future~~
- Install it with your favorite package manager
- Build from source, you need cargo installed on your system

### Using package managers
#### Scoop - Scoop's official repository
1. Make sure you have the latest version of Scoop installed
2. Add games bucket using `scoop bucket add games` if you did not before
3. And finally install tmaze with `scoop install games/tmaze`

#### Scoop - Henshouse repository
1. Make sure you have the latest version of Scoop installed
2. Add games bucket using `scoop bucket add henshouse https://github.com/henshouse/henshouse-scoop` if you did not before
3. And finally install tmaze with `scoop install henshouse/tmaze`

### Feature flags
TMaze uses cargo features to enable/disable some features. In Github release binaries they are all enabled, ~~but not all of them are enabled by default when building from source.~~ From version 1.14.0 all features are enabled by default and should be disabled manually. To disable them, use `--no-default-features` flag. After disabling them, enable specific ones you want with `--features <feature1>,<feature2>,...` flag.

The features are:

- hashbrown - uses hashbrown instead of std hashmap, which is faster
- updates - enables checking for updates, which is done on startup, can be disabled (this **doesn't** install new version)

### How to build from source
#### Enabling/disabling features
After `cargo` command add `--features` to enable features, such as `updates`. To disable default features, such as `hashbrown`, add `--no-default-features`. To enable all featueres add `--all-features`.

#### Install it using cargo from crates.io
1. Make sure you have [cargo](https://crates.io/) installed
1. Run `cargo install tmaze`
1. It's recommended that you have `~/.cargo/bin` in the PATH, so that you don't need full path to run it

#### Or directly from Github
1. Make sure you have [cargo](https://crates.io/) installed
1. Clone GitHub repository or download it as zip, then extract it
1. Go to that folder
1. Run command `cargo run --release` to run (or you can just build it with `cargo build --release` without runing it)
1. You can find compiled executable in the directory `./target/release/` with name `tmaze` or `tmaze.exe` , which you can move or link somewhere else

#### If you are Docker enjoyer, you may use it too
1. Make sure you have [Docker](https://www.docker.com/) installed
1. Build the image with `docker build -t tmaze . --tag tmaze` inside the repository folder, image is not published on Docker Hub yet
1. Then you have multiple options of actually running it (and ofc more)
    1. Run it one time only: `docker run --rm -it tmaze`
    1. Run it with persistent storage for config and saved data: `docker run -it --rm -v tmaze_data:/root/.config/tmaze tmaze`
        - In this case you can edit config using somthing like `docker run --rm -it -v tmaze_data:/root thinca/vim:latest`
