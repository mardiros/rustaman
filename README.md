# Rustaman

<!--
[![Build Status](https://travis-ci.org/mardiros/rustaman.svg?branch=master)](https://travis-ci.org/mardiros/rustaman)
[![Current Crates.io Version](https://img.shields.io/crates/v/rustaman.svg)](https://crates.io/crates/rustaman)
[![Latests Documentation](https://docs.rs/rustaman/badge.svg)](https://docs.rs/crate/rustaman)
-->

[![dependency status](https://deps.rs/repo/github/mardiros/rustaman/status.svg)](https://deps.rs/repo/github/mardiros/rustaman)



A Template based HTTP client.


## Status

unstable/under development


## Install on Archlinux

You can install the package `rustaman-git` from [AUR](https://aur.archlinux.org/packages/rustaman-git).


## Install from source

You must have GTK 3 installed on your OS to get it working.
The GTK SourveView has to be installed too.


### Clone the repository

```
    git clone https://github.com/mardiros/rustaman.git
    cd rustaman
```

### Copy assets for syntax highlighting

#### On Linux

```
    mkdir -p ~/.config/rustaman
    cp assets/* ~/.config/rustaman
```


#### On MacOS

```
    mkdir $HOME/Library/Preferences/rustaman
    cp assets/* $HOME/Library/Preferences/rustaman
```

### Build
```
    cargo build
```

Or in release mode:

```
    cargo build --release
```


### Run with logging info

```
    RUST_BACKTRACE=1 RUST_LOG=rustaman=debug cargo run
```

Or in release mode with the error level:

```
    RUST_BACKTRACE=1 RUST_LOG=rustaman=error cargo run --release
```
