//! Build script.
//!
//! `src/web/mod.rs` embeds the built web UI via `rust-embed` (`#[folder =
//! "web-ui/dist"]`). rust-embed requires that folder to EXIST at compile time, but
//! the built bundle is git-ignored and only produced by `make web-build` /
//! `scripts/build-release.sh`. To keep a fresh clone compiling (it serves a
//! "frontend not embedded" stub until the bundle is built), ensure the directory
//! exists here.
use std::fs;

fn main() {
    let _ = fs::create_dir_all("web-ui/dist");
    println!("cargo:rerun-if-changed=web-ui/dist");
}
