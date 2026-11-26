use crate::{
    editor::{Action, Editor},
    model::View,
    presets, render,
};
use wasm_bindgen::prelude::*;

fn err(message: String) -> JsValue {
    JsValue::from_str(&message)
}
fn view(value: &str) -> Result<View, JsValue> {
    match value {
        "lobby" => Ok(View::Lobby),
        "panel" => Ok(View::Panel),
        _ => Err(JsValue::from_str("Pantalla inválida.")),
    }
}

#[wasm_bindgen]
pub struct Studio {
    editor: Editor,
}

#[wasm_bindgen]
impl Studio {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            editor: Editor::new(),
        }
    }
    pub fn presets(&self) -> String {
        serde_json::to_string(&presets::PRESETS).expect("presets serializable")
    }
    pub fn describe(&self, screen: &str, id: &str) -> Result<String, JsValue> {
        Ok(self.editor.describe(view(screen)?, id).to_string())
    }
    pub fn dispatch(&mut self, input: &str) -> Result<(), JsValue> {
        if input.len() > crate::model::MAX_IMAGE_BYTES + 16_384 {
            return Err(JsValue::from_str("Acción demasiado grande."));
        }
        let action: Action =
            serde_json::from_str(input).map_err(|_| JsValue::from_str("Acción inválida."))?;
        self.editor.apply(action).map_err(err)
    }
    pub fn snapshot(&self) -> Result<String, JsValue> {
        serde_json::to_string(self.editor.workspace())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    pub fn load(&mut self, input: &str) -> Result<(), JsValue> {
        self.editor.load(input).map_err(err)
    }
    pub fn import_project(&mut self, input: &str) -> Result<(), JsValue> {
        self.editor.restore(input).map_err(err)
    }
    pub fn undo(&mut self) -> bool {
        self.editor.undo()
    }
    pub fn redo(&mut self) -> bool {
        self.editor.redo()
    }
    pub fn html(&self, screen: &str, preview: bool) -> Result<String, JsValue> {
        Ok(render::document(
            self.editor.active(),
            view(screen)?,
            preview,
        ))
    }
}
impl Default for Studio {
    fn default() -> Self {
        Self::new()
    }
}
