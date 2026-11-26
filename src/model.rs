use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

pub type ModelResult<T> = Result<T, String>;
pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_PROJECT_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_IMAGE_BYTES: usize = 450_000;
pub const MAX_IMAGES_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_ROOTS: usize = 32;
pub const MAX_NODES: usize = 160;
pub const MAX_CHILDREN: usize = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum View {
    Lobby,
    Panel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layout {
    Rail,
    Top,
    Compact,
    Editorial,
    Rooms,
    Immersive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Font {
    System,
    Condensed,
    Editorial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Header,
    Hero,
    Categories,
    Catalog,
    Live,
    Footer,
    Metrics,
    Activity,
    Tile,
    Text,
    Image,
}

impl Kind {
    pub fn group(self) -> bool {
        matches!(
            self,
            Self::Categories | Self::Catalog | Self::Live | Self::Metrics | Self::Activity
        )
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Header => "Cabecera",
            Self::Hero => "Portada",
            Self::Categories => "Categorías",
            Self::Catalog => "Catálogo",
            Self::Live => "En vivo",
            Self::Footer => "Pie de página",
            Self::Metrics => "Indicadores",
            Self::Activity => "Actividad",
            Self::Tile => "Tarjeta",
            Self::Text => "Texto",
            Self::Image => "Imagen",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Theme {
    pub brand: String,
    pub accent: String,
    pub background: String,
    pub surface: String,
    pub ink: String,
    pub radius: u8,
    pub font: Font,
    pub layout: Layout,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Style {
    pub color: Option<String>,
    pub background: Option<String>,
    pub radius: Option<u8>,
    pub padding: Option<u8>,
    pub font_size: Option<u8>,
    pub columns: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub id: String,
    pub kind: Kind,
    pub title: String,
    pub text: String,
    pub button: String,
    pub hidden: bool,
    pub art: u8,
    pub style: Style,
    pub image: Option<Rc<str>>,
    pub children: Vec<Component>,
}
impl Component {
    pub fn new(id: impl Into<String>, kind: Kind, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind,
            title: title.into(),
            text: String::new(),
            button: String::new(),
            hidden: false,
            art: 0,
            style: Style::default(),
            image: None,
            children: Vec::new(),
        }
    }
    pub fn find(&self, id: &str) -> Option<&Self> {
        if self.id == id {
            Some(self)
        } else {
            self.children.iter().find_map(|node| node.find(id))
        }
    }
    pub fn find_mut(&mut self, id: &str) -> Option<&mut Self> {
        if self.id == id {
            Some(self)
        } else {
            self.children.iter_mut().find_map(|node| node.find_mut(id))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Design {
    pub theme: Theme,
    pub lobby: Vec<Component>,
    pub panel: Vec<Component>,
}
impl Design {
    pub fn nodes(&self, view: View) -> &Vec<Component> {
        match view {
            View::Lobby => &self.lobby,
            View::Panel => &self.panel,
        }
    }
    pub fn nodes_mut(&mut self, view: View) -> &mut Vec<Component> {
        match view {
            View::Lobby => &mut self.lobby,
            View::Panel => &mut self.panel,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Workspace {
    pub version: u32,
    pub active: String,
    pub designs: BTreeMap<String, Design>,
}

pub fn valid_color(s: &str) -> bool {
    s.len() == 7 && s.starts_with('#') && s.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
}
pub fn valid_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 80
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
}
pub fn valid_text(s: &str, max: usize) -> bool {
    s.chars().count() <= max
        && !s
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\t' | '\r'))
}
pub fn valid_image(s: &str) -> bool {
    if s.len() > MAX_IMAGE_BYTES {
        return false;
    }
    let Some(data) = [
        "data:image/webp;base64,",
        "data:image/png;base64,",
        "data:image/jpeg;base64,",
    ]
    .iter()
    .find_map(|prefix| s.strip_prefix(prefix)) else {
        return false;
    };
    let padding = data.bytes().rev().take_while(|b| *b == b'=').count();
    data.len() >= 4
        && data.len().is_multiple_of(4)
        && padding <= 2
        && data.as_bytes()[..data.len() - padding]
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || *b == b'+' || *b == b'/')
}

fn validate_node(
    node: &Component,
    depth: usize,
    ids: &mut BTreeSet<String>,
    count: &mut usize,
    image_bytes: &mut usize,
) -> ModelResult<()> {
    *count += 1;
    if *count > MAX_NODES || depth > 1 || !valid_id(&node.id) || !ids.insert(node.id.clone()) {
        return Err("Identificador inválido, repetido o exceso de componentes.".into());
    }
    if (depth == 0 && node.kind == Kind::Tile)
        || (depth == 1 && node.kind != Kind::Tile)
        || (!node.kind.group() && !node.children.is_empty())
        || node.children.len() > MAX_CHILDREN
    {
        return Err("Jerarquía de componentes inválida.".into());
    }
    if !valid_text(&node.title, 120)
        || !valid_text(&node.text, 600)
        || !valid_text(&node.button, 60)
        || node.art > 11
    {
        return Err("Texto o estilo de ilustración fuera de límite.".into());
    }
    let s = &node.style;
    if s.color.as_ref().is_some_and(|s| !valid_color(s))
        || s.background.as_ref().is_some_and(|s| !valid_color(s))
        || s.radius.is_some_and(|n| n > 48)
        || s.padding.is_some_and(|n| n > 64)
        || s.font_size.is_some_and(|n| !(10..=64).contains(&n))
        || s.columns.is_some_and(|n| !(1..=8).contains(&n))
    {
        return Err("Estilo no permitido.".into());
    }
    if let Some(image) = &node.image {
        if !valid_image(image) {
            return Err("Imagen inválida o demasiado grande.".into());
        }
        *image_bytes += image.len();
        if *image_bytes > MAX_IMAGES_BYTES {
            return Err("El proyecto supera el presupuesto de imágenes.".into());
        }
    }
    for child in &node.children {
        validate_node(child, depth + 1, ids, count, image_bytes)?;
    }
    Ok(())
}

pub fn validate(workspace: &Workspace) -> ModelResult<()> {
    if workspace.version != SCHEMA_VERSION
        || !workspace.designs.contains_key(&workspace.active)
        || workspace.designs.is_empty()
        || workspace.designs.len() > crate::presets::PRESETS.len()
    {
        return Err("Versión o plantilla de proyecto inválida.".into());
    }
    let mut images = 0;
    for (id, design) in &workspace.designs {
        if crate::presets::get(id).is_none() {
            return Err("Plantilla desconocida.".into());
        }
        let theme = &design.theme;
        if theme.brand.trim().is_empty()
            || !valid_text(&theme.brand, 60)
            || theme.radius > 48
            || [&theme.accent, &theme.background, &theme.surface, &theme.ink]
                .iter()
                .any(|c| !valid_color(c))
        {
            return Err("Tema inválido.".into());
        }
        for view in [View::Lobby, View::Panel] {
            let nodes = design.nodes(view);
            if nodes.len() > MAX_ROOTS {
                return Err("Demasiadas secciones.".into());
            }
            let mut ids = BTreeSet::new();
            let mut count = 0;
            for node in nodes {
                validate_node(node, 0, &mut ids, &mut count, &mut images)?;
            }
        }
    }
    Ok(())
}

pub fn decode(input: &str) -> ModelResult<Workspace> {
    if input.len() > MAX_PROJECT_BYTES {
        return Err("Proyecto demasiado grande.".into());
    }
    let state = serde_json::from_str::<Workspace>(input)
        .map_err(|_| "Formato de proyecto inválido.".to_owned())?;
    validate(&state)?;
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn image_types_and_color() {
        assert!(valid_image("data:image/png;base64,YWJj"));
        assert!(!valid_image("data:image/svg+xml;base64,YWJj"));
        assert!(!valid_image("data:image/png;base64,A==="));
        assert!(!valid_image("data:image/png;base64,AA=A"));
        assert!(!valid_color("#abc;"));
        assert!(valid_color("#aabbcc"));
    }
    #[test]
    fn reject_duplicates() {
        let mut state = crate::presets::workspace("ember").unwrap();
        let design = state.designs.get_mut("ember").unwrap();
        design.lobby.push(design.lobby[0].clone());
        assert!(validate(&state).is_err());
    }
    #[test]
    fn reject_unknown_fields() {
        let value = serde_json::to_value(crate::presets::workspace("ember").unwrap()).unwrap();
        let mut value = value;
        value["foreign"] = true.into();
        assert!(decode(&value.to_string()).is_err());
    }
}
