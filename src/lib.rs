#![forbid(unsafe_code)]

pub mod editor;
pub mod model;
pub mod presets;
pub mod render;

#[cfg(all(target_arch = "wasm32", feature = "browser"))]
mod browser;
#[cfg(all(target_arch = "wasm32", feature = "edge"))]
mod edge;
