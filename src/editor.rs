use crate::{model::*, presets};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::VecDeque, rc::Rc};

const HISTORY_LIMIT: usize = 32;
const HISTORY_BYTES: usize = 48 * 1024 * 1024;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Patch {
    pub title: String,
    pub text: String,
    pub button: String,
    pub hidden: bool,
    pub style: Style,
    pub art: u8,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Switch {
        preset: String,
    },
    Theme {
        theme: Theme,
    },
    Edit {
        view: View,
        id: String,
        patch: Patch,
    },
    Image {
        view: View,
        id: String,
        image: Option<Rc<str>>,
    },
    Asset {
        view: View,
        id: String,
        asset: Option<String>,
    },
    Add {
        view: View,
        parent: Option<String>,
        id: String,
        kind: Kind,
    },
    Remove {
        view: View,
        id: String,
    },
    Move {
        view: View,
        id: String,
        offset: i8,
    },
    MoveBefore {
        view: View,
        id: String,
        before: String,
    },
    Appearance {
        settings: AppearanceSettings,
    },
    Backdrop {
        image: Option<Rc<str>>,
    },
    ResetAppearance,
    Restore {
        view: View,
        id: String,
    },
    ResetView {
        view: View,
    },
    Reset,
}

impl Action {
    pub fn target(&self) -> Option<(View, &str)> {
        match self {
            Self::Edit { view, id, .. }
            | Self::Image { view, id, .. }
            | Self::Asset { view, id, .. }
            | Self::Remove { view, id }
            | Self::Restore { view, id } => Some((*view, id)),
            _ => None,
        }
    }
}
pub fn component_info(n: &Component) -> Value {
    json!({"id":n.id,"kind":n.kind,"title":n.title,"text":n.text,"button":n.button,"hidden":n.hidden,"style":n.style,"art":n.art,"has_image":n.image.is_some()||n.asset.is_some(),"asset":n.asset,"group":n.kind.group()})
}
#[derive(Serialize)]
struct Outline<'a> {
    id: &'a str,
    kind: Kind,
    title: &'a str,
    hidden: bool,
    children: Vec<Outline<'a>>,
}
fn outline(node: &Component) -> Outline<'_> {
    Outline {
        id: &node.id,
        kind: node.kind,
        title: &node.title,
        hidden: node.hidden,
        children: node.children.iter().map(outline).collect(),
    }
}
fn find_mut<'a>(nodes: &'a mut [Component], id: &str) -> ModelResult<&'a mut Component> {
    nodes
        .iter_mut()
        .find_map(|n| n.find_mut(id))
        .ok_or_else(|| "Componente no encontrado.".into())
}
fn siblings_mut<'a>(nodes: &'a mut Vec<Component>, id: &str) -> Option<&'a mut Vec<Component>> {
    if nodes.iter().any(|n| n.id == id) {
        return Some(nodes);
    }
    for node in nodes {
        if let Some(group) = siblings_mut(&mut node.children, id) {
            return Some(group);
        }
    }
    None
}

pub struct Editor {
    current: Workspace,
    current_size: usize,
    past: VecDeque<(Workspace, usize)>,
    future: Vec<(Workspace, usize)>,
}
impl Editor {
    pub fn new() -> Self {
        let current = presets::workspace("ember").expect("static preset");
        let current_size = serialized_size(&current).expect("static preset size");
        Self {
            current,
            current_size,
            past: VecDeque::new(),
            future: Vec::new(),
        }
    }
    pub fn workspace(&self) -> &Workspace {
        &self.current
    }
    pub fn active(&self) -> &Design {
        &self.current.designs[&self.current.active]
    }
    fn trim_history(&mut self) {
        while self.past.len() + self.future.len() > HISTORY_LIMIT
            || self
                .past
                .iter()
                .chain(self.future.iter())
                .map(|(_, size)| size)
                .sum::<usize>()
                > HISTORY_BYTES
        {
            if self.past.pop_front().is_none() {
                self.future.remove(0);
            }
        }
    }
    fn commit(&mut self, next: Workspace) -> ModelResult<()> {
        validate(&next)?;
        if next != self.current {
            let size = serialized_size(&next)?;
            let previous = std::mem::replace(&mut self.current, next);
            let previous_size = std::mem::replace(&mut self.current_size, size);
            self.past.push_back((previous, previous_size));
            self.future.clear();
            self.trim_history();
        }
        Ok(())
    }
    pub fn replace_workspace(&mut self, workspace: Workspace) -> ModelResult<()> {
        self.commit(workspace)
    }
    pub fn restore(&mut self, input: &str) -> ModelResult<()> {
        self.commit(decode(input)?)
    }
    pub fn load(&mut self, input: &str) -> ModelResult<()> {
        let current = decode(input)?;
        self.current_size = serialized_size(&current)?;
        self.current = current;
        self.past.clear();
        self.future.clear();
        Ok(())
    }
    pub fn undo(&mut self) -> bool {
        if let Some((previous, size)) = self.past.pop_back() {
            let old = std::mem::replace(&mut self.current, previous);
            let old_size = std::mem::replace(&mut self.current_size, size);
            self.future.push((old, old_size));
            self.trim_history();
            true
        } else {
            false
        }
    }
    pub fn redo(&mut self) -> bool {
        if let Some((next, size)) = self.future.pop() {
            let old = std::mem::replace(&mut self.current, next);
            let old_size = std::mem::replace(&mut self.current_size, size);
            self.past.push_back((old, old_size));
            self.trim_history();
            true
        } else {
            false
        }
    }
    pub fn apply(&mut self, action: Action) -> ModelResult<()> {
        self.apply_catalog(action, None)
    }
    pub fn apply_catalog(
        &mut self,
        action: Action,
        catalog_default: Option<Component>,
    ) -> ModelResult<()> {
        let mut next = self.current.clone();
        match action {
            Action::Switch { preset } => {
                if !next.designs.contains_key(&preset) {
                    next.designs
                        .insert(preset.clone(), presets::design(&preset)?);
                }
                next.active = preset;
            }
            Action::Reset => {
                next.designs
                    .insert(next.active.clone(), presets::design(&next.active)?);
            }
            Action::Appearance { settings } => next.appearance.settings = settings,
            Action::Backdrop { image } => next.appearance.image = image,
            Action::ResetAppearance => next.appearance = Appearance::default(),
            Action::ResetView { view } => {
                let original = presets::design(&next.active)?;
                let design = next.designs.get_mut(&next.active).ok_or("No hay diseño.")?;
                *design.nodes_mut(view) = original.nodes(view).clone();
            }
            other => {
                let original = if matches!(&other, Action::Restore { .. }) {
                    Some(presets::design(&next.active)?)
                } else {
                    None
                };
                let design = next.designs.get_mut(&next.active).ok_or("No hay diseño.")?;
                if let Some(base) = &catalog_default {
                    let nodes = design.nodes_mut(View::Lobby);
                    if !nodes.iter().any(|n| n.find(&base.id).is_some()) {
                        let parent = nodes
                            .iter_mut()
                            .find(|n| n.id == "catalog" && n.kind == Kind::Catalog)
                            .ok_or("Catálogo no encontrado.")?;
                        parent.children.push(base.clone());
                    }
                }
                match other {
                    Action::Theme { theme } => design.theme = theme,
                    Action::Edit { view, id, patch } => {
                        let node = find_mut(design.nodes_mut(view), &id)?;
                        node.title = patch.title;
                        node.text = patch.text;
                        node.button = patch.button;
                        node.hidden = patch.hidden;
                        node.style = patch.style;
                        node.art = patch.art;
                    }
                    Action::Image { view, id, image } => {
                        let node = find_mut(design.nodes_mut(view), &id)?;
                        node.image = image;
                        node.asset = None;
                    }
                    Action::Asset { view, id, asset } => {
                        let node = find_mut(design.nodes_mut(view), &id)?;
                        node.asset = asset;
                        node.image = None;
                    }
                    Action::Add {
                        view,
                        parent,
                        id,
                        kind,
                    } => {
                        let node = Component::new(id, kind, kind.label());
                        let nodes = design.nodes_mut(view);
                        if let Some(parent) = parent {
                            let target = find_mut(nodes, &parent)?;
                            if !target.kind.group() || kind != Kind::Tile {
                                return Err("Solo pueden agregarse tarjetas a grupos.".into());
                            }
                            target.children.push(node);
                        } else {
                            nodes.push(node);
                        }
                    }
                    Action::Restore { view, id } => {
                        let default = catalog_default
                            .as_ref()
                            .or_else(|| {
                                original
                                    .as_ref()
                                    .and_then(|d| d.nodes(view).iter().find_map(|n| n.find(&id)))
                            })
                            .ok_or("No existe un componente original.")?;
                        *find_mut(design.nodes_mut(view), &id)? = default.clone();
                    }
                    Action::Remove { view, id } => {
                        if catalog_default.is_some() {
                            find_mut(design.nodes_mut(view), &id)?.hidden = true;
                        } else {
                            let group = siblings_mut(design.nodes_mut(view), &id)
                                .ok_or("Componente no encontrado.")?;
                            group.retain(|n| n.id != id);
                        }
                    }
                    Action::Move { view, id, offset } => {
                        let group = siblings_mut(design.nodes_mut(view), &id)
                            .ok_or("Componente no encontrado.")?;
                        let index = group
                            .iter()
                            .position(|n| n.id == id)
                            .ok_or("Componente no encontrado.")?;
                        let dest = (index as isize + offset as isize)
                            .clamp(0, group.len() as isize - 1)
                            as usize;
                        group.swap(index, dest);
                    }
                    Action::MoveBefore { view, id, before } => {
                        if id == before {
                            return Ok(());
                        }
                        let group = siblings_mut(design.nodes_mut(view), &id)
                            .ok_or("Componente no encontrado.")?;
                        if !group.iter().any(|n| n.id == before) {
                            return Err("Los elementos deben pertenecer al mismo grupo.".into());
                        }
                        let index = group
                            .iter()
                            .position(|n| n.id == id)
                            .ok_or("Componente no encontrado.")?;
                        let node = group.remove(index);
                        let dest = group
                            .iter()
                            .position(|n| n.id == before)
                            .ok_or("Destino inválido.")?;
                        group.insert(dest, node);
                    }
                    Action::Switch { .. }
                    | Action::Appearance { .. }
                    | Action::Backdrop { .. }
                    | Action::ResetAppearance
                    | Action::ResetView { .. }
                    | Action::Reset => {
                        return Err("Acción inválida.".into());
                    }
                }
            }
        }
        self.commit(next)
    }
    pub fn describe(&self, view: View, id: &str) -> Value {
        let design = self.active();
        let nodes = design.nodes(view);
        let selected = nodes.iter().find_map(|n| n.find(id)).map(component_info);
        json!({"version":SCHEMA_VERSION,"active":self.current.active,"theme":design.theme,"appearance":{"settings":self.current.appearance.settings,"has_image":self.current.appearance.image.is_some()},"outline":nodes.iter().map(outline).collect::<Vec<_>>(),"selected":selected,"undo":!self.past.is_empty(),"redo":!self.future.is_empty()})
    }
}
impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn template_isolation_and_history() {
        let mut e = Editor::new();
        let mut t = e.active().theme.clone();
        t.brand = "Original".into();
        e.apply(Action::Theme { theme: t }).unwrap();
        e.apply(Action::Switch {
            preset: "cobalt".into(),
        })
        .unwrap();
        assert_eq!(e.active().theme.brand, "Mi casino");
        e.apply(Action::Switch {
            preset: "ember".into(),
        })
        .unwrap();
        assert_eq!(e.active().theme.brand, "Original");
        assert!(e.undo());
        assert_eq!(e.workspace().active, "cobalt");
        assert!(e.redo());
        assert_eq!(e.workspace().active, "ember");
    }
    #[test]
    fn history_has_a_combined_budget() {
        let mut e = Editor::new();
        for i in 0..100 {
            let mut theme = e.active().theme.clone();
            theme.brand = format!("Marca {i}");
            e.apply(Action::Theme { theme }).unwrap();
        }
        assert_eq!(e.past.len(), HISTORY_LIMIT);
        while e.undo() {}
        assert_eq!(e.future.len(), HISTORY_LIMIT);
        assert_eq!(serialized_size(e.workspace()).unwrap(), e.current_size);
    }
    #[test]
    fn actions_are_atomic() {
        let mut e = Editor::new();
        let before = e.workspace().clone();
        let mut t = e.active().theme.clone();
        t.accent = "red;display:none".into();
        assert!(e.apply(Action::Theme { theme: t }).is_err());
        assert_eq!(e.workspace(), &before);
        assert!(!e.undo());
    }
    #[test]
    fn stable_ids_and_reorder() {
        let mut e = Editor::new();
        e.apply(Action::MoveBefore {
            view: View::Lobby,
            id: "live".into(),
            before: "hero".into(),
        })
        .unwrap();
        assert_eq!(e.active().lobby[1].id, "live");
        assert!(
            e.apply(Action::Add {
                view: View::Lobby,
                parent: None,
                id: "hero".into(),
                kind: Kind::Text
            })
            .is_err()
        );
    }
    #[test]
    fn export_import_roundtrip() {
        let e = Editor::new();
        let data = serde_json::to_string(e.workspace()).unwrap();
        let mut copy = Editor::new();
        copy.load(&data).unwrap();
        assert_eq!(e.workspace(), copy.workspace());
    }
    #[test]
    fn view_isolation() {
        let mut e = Editor::new();
        e.apply(Action::Remove {
            view: View::Panel,
            id: "header".into(),
        })
        .unwrap();
        assert!(e.active().lobby.iter().any(|x| x.id == "header"));
        assert!(!e.active().panel.iter().any(|x| x.id == "header"));
    }
}
