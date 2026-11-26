#![forbid(unsafe_code)]

pub mod contract;
pub mod design;

#[cfg(all(target_arch = "wasm32", feature = "edge"))]
mod edge;

#[cfg(all(target_arch = "wasm32", feature = "browser"))]
mod browser {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn normalize_content_json(input: &str) -> String {
        super::design::normalize_content(&serde_json::from_str(input).unwrap_or_default()).to_string()
    }

    #[wasm_bindgen]
    pub fn normalize_design_json(input: &str) -> String {
        super::design::normalize_design(&serde_json::from_str(input).unwrap_or_default()).to_string()
    }
}
