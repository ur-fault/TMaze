# TMaze

Simple multiplatform maze solving game for terminal, written entirely in Rust

### Install with `cargo install tmaze` and run with `tmaze`
---

![](https://img.shields.io/crates/d/tmaze)
![Crates.io](https://img.shields.io/crates/v/tmaze)

[![Packaging status](https://repology.org/badge/vertical-allrepos/tmaze.svg)](https://repology.org/project/tmaze/versions)

![Screenshot of in-game](https://github.com/ur-fault/tmaze/blob/master/readme_assets/screenshot_ingame.png?raw=true)

# Features

- Random mazes using powered by several algorithms
- Configurable - colors, features and maze presets
- Sound and OST
- Multiplatform - Windows, Linux, MacOS and Termux\*, even more untested
- 3D mazes (yes, it's unplayable) and towers
- Binaries using CI
- Playable on most screens, ie. terminal windows, even mobile


# Credits and thanks
- Music and OST - [step](https://github.com/StepGamesOfficial)
- Marketing - [PhntD](https://github.com/PhntD), Inženýr
- Random but important stuff - [filip2cz](https://github.com/filip2cz/)
- Playtest - everyone
- Everything else - [ur-fault (me)](https://github.com/ur-fault)


# How to run
- There are several options:
    - Download from binaries [Github releases](https://github.com/ur-fault/TMaze/releases/latest), these are built automatically, using GitHub CI
    - Install it with your favorite package manager, see [Repology](https://repology.org/project/tmaze/packages)
    - Build from source, `cargo` is needed for that, as for most Rust project
        - You can even build and run it inside included `Dockerfile`

## Scoop
>TMaze is available on official `games` repository, but also on [henshouse-scoop](https://github.com/henshouse/henshouse-scoop), check that for instructions on how to add it
1. Make sure you have the latest version of Scoop installed
2. Add games bucket using `scoop bucket add games` if you did not before
3. And finally, install tmaze with `scoop install games/tmaze`

# Building from source
Install [cargo](https://github.com/rust-lang/cargo), recommended way is installing `rustup`, which will install `cargo` too

## Compile flags
>Rust programs are typically flaged using `cargo features`, which are flags you can enable to add/remove specific functionality from programs and libraries. Enable them by adding `-F <features separated by comma>` to the build/install command. As said, all\* flags are enabled by default, so to specify only subset, add `--no-default-features` flag to disable them all

### Features in TMaze
TMaze has several of these flags and all of them are enabled by default. Note: this is not guaranteed to be true in the future, for example when we add debug flag or something similar

- updates - TMaze can check [crates.io](https://crates.io) and notify you about new version
- sound - background music and in the future other sound effects, note: you need alsa developement headers during build, on Debian its `libasound2-dev`

## From crates.io
[crates.io](https://crates.io) is package/library registry for Rust and TMaze is there too
1. Just run `cargo install tmaze`, optionally specify compile flags
1. If you have `~/.cargo/bin/` in your path, simply run TMaze with `tmaze`

## From Github
1. Clone the repo using git with `git clone https://github.com/ur-fault/tmaze`
1. `cargo install --path ./tmaze/tmaze^`
1. If you have `~/.cargo/bin/` in your path, simply run TMaze with `tmaze`

#### If you are Docker enjoyer, you may use it too
1. Build the image with `docker build -t tmaze . --tag tmaze` inside the repository folder, image is not published on Docker Hub yet
1. Then there are mostly options
    1. Run it one-time only: `docker run --rm -it tmaze`
    1. Run it with persistent storage for config and saved data: `docker run -it --rm -v tmaze_data:/root/.config/tmaze tmaze`
        - In this case you can edit config using somthing like `docker run --rm -it -v tmaze_data:/root thinca/vim:latest`
