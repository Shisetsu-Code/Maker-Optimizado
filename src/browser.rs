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
    catalog: Option<crate::catalog::Catalog>,
    panel: crate::demo_panel::Panel,
}

#[wasm_bindgen]
impl Studio {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            editor: Editor::new(),
            catalog: None,
            panel: crate::demo_panel::Panel::default(),
        }
    }
    pub fn presets(&self) -> String {
        serde_json::to_string(&presets::PRESETS).expect("presets serializable")
    }
    pub fn describe(&self, screen: &str, id: &str) -> Result<String, JsValue> {
        let screen = view(screen)?;
        let mut data = self.editor.describe(screen, id);
        if screen == View::Lobby {
            if let Some(base) = self
                .current_catalog()
                .and_then(|c| c.default_component(id, self.editor.active()))
            {
                if data["selected"].is_null() {
                    data["selected"] = crate::editor::component_info(&base);
                }
                data["selected"]["catalog_item"] = true.into();
            }
        }
        Ok(data.to_string())
    }
    pub fn dispatch(&mut self, input: &str) -> Result<(), JsValue> {
        if input.len() > crate::model::MAX_BACKGROUND_BYTES + 16_384 {
            return Err(JsValue::from_str("Acción demasiado grande."));
        }
        let action: Action =
            serde_json::from_str(input).map_err(|_| JsValue::from_str("Acción inválida."))?;
        let base = action.target().and_then(|(view, id)| {
            if view != View::Lobby {
                return None;
            }
            self.current_catalog()
                .and_then(|c| c.default_component(id, self.editor.active()))
        });
        self.editor.apply_catalog(action, base).map_err(err)
    }
    pub fn set_catalog(&mut self, input: &str) -> Result<(), JsValue> {
        self.catalog = Some(
            crate::catalog::Catalog::parse(input, &self.editor.workspace().active).map_err(err)?,
        );
        Ok(())
    }
    pub fn catalog_metadata(&self) -> Result<String, JsValue> {
        self.current_catalog()
            .map(|c| c.metadata().to_string())
            .ok_or_else(|| JsValue::from_str("Catálogo no disponible."))
    }
    pub fn catalog_page(&self, input: &str) -> Result<String, JsValue> {
        if input.len() > 256 * 1024 {
            return Err(JsValue::from_str("Filtro demasiado grande."));
        }
        let query: crate::catalog::Query =
            serde_json::from_str(input).map_err(|_| JsValue::from_str("Filtro inválido."))?;
        let catalog = self
            .current_catalog()
            .ok_or_else(|| JsValue::from_str("Catálogo no disponible."))?;
        serde_json::to_string(&catalog.page(self.editor.active(), &query))
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    pub fn panel_action(&mut self,input:&str)->Result<(),JsValue>{
        if input.len()>8192{return Err(JsValue::from_str("Acción de demostración demasiado grande."));}
        let action:crate::demo_panel::Action=serde_json::from_str(input).map_err(|_|JsValue::from_str("Acción de panel inválida."))?;
        self.panel.apply(action).map_err(err)
    }
    pub fn panel_content(&self)->String{self.panel.content()}
    pub fn panel_navigation(&self)->String{self.panel.navigation()}
    pub fn panel_metadata(&self)->String{self.panel.metadata().to_string()}
    pub fn snapshot(&self) -> Result<String, JsValue> {
        serde_json::to_string(self.editor.workspace())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    pub fn load(&mut self, input: &str) -> Result<(), JsValue> {
        self.editor.load(input).map_err(err)
    }
    pub fn import_project(&mut self, input: &str) -> Result<String, JsValue> {
        if let Ok(workspace) = crate::model::decode(input) {
            self.editor.replace_workspace(workspace).map_err(err)?;
            return Ok(
                serde_json::json!({"converted":false,"applied":0,"unresolved":[]}).to_string(),
            );
        }
        let (workspace, report) =
            crate::migration::convert(input, self.editor.workspace()).map_err(err)?;
        self.editor.replace_workspace(workspace).map_err(err)?;
        serde_json::to_string(&report).map_err(|e| JsValue::from_str(&e.to_string()))
    }
    pub fn undo(&mut self) -> bool {
        self.editor.undo()
    }
    pub fn redo(&mut self) -> bool {
        self.editor.redo()
    }
    pub fn html(&self, screen: &str, preview: bool) -> Result<String, JsValue> {
        Ok(render::document_with_appearance(
            self.editor.active(),
            view(screen)?,
            preview,
            &self.editor.workspace().appearance,
        ))
    }
}
impl Studio {
    fn current_catalog(&self) -> Option<&crate::catalog::Catalog> {
        self.catalog
            .as_ref()
            .filter(|c| c.preset == self.editor.workspace().active)
    }
}
impl Default for Studio {
    fn default() -> Self {
        Self::new()
    }
}
