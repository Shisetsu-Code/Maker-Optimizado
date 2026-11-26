#![forbid(unsafe_code)]

pub mod model;
pub mod presets;
pub mod editor;
pub mod render;

#[cfg(all(target_arch = "wasm32", feature = "edge"))]
mod edge;
#[cfg(all(target_arch = "wasm32", feature = "browser"))]
mod browser;
