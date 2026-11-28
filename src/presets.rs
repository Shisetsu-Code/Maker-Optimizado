use crate::model::*;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Preset {
    pub id: &'static str,
    pub name: &'static str,
    pub accent: &'static str,
    pub background: &'static str,
    pub surface: &'static str,
    pub ink: &'static str,
    pub layout: Layout,
    pub font: Font,
    pub radius: u8,
    pub art: u8,
}
use Font::{Condensed, Editorial as EditorialFont, System};
use Layout::{Compact, Editorial, Immersive, Rail, Rooms, Top};

pub const PRESETS: [Preset; 17] = [
    Preset {
        id: "ember",
        name: "Ember Classic",
        accent: "#ff5500",
        background: "#141414",
        surface: "#242424",
        ink: "#fafafa",
        layout: Rail,
        font: Condensed,
        radius: 12,
        art: 0,
    },
    Preset {
        id: "cobalt",
        name: "Cobalt",
        accent: "#3988ff",
        background: "#091325",
        surface: "#16243b",
        ink: "#f4f7ff",
        layout: Rail,
        font: System,
        radius: 12,
        art: 1,
    },
    Preset {
        id: "flare",
        name: "Flare",
        accent: "#ff5b32",
        background: "#171113",
        surface: "#282028",
        ink: "#fafafa",
        layout: Immersive,
        font: Condensed,
        radius: 16,
        art: 2,
    },
    Preset {
        id: "coal",
        name: "Coal",
        accent: "#e36b37",
        background: "#151212",
        surface: "#24201d",
        ink: "#fff8f2",
        layout: Compact,
        font: System,
        radius: 8,
        art: 3,
    },
    Preset {
        id: "ivory",
        name: "Ivory Resort",
        accent: "#d5b66d",
        background: "#f5f3ed",
        surface: "#ffffff",
        ink: "#172423",
        layout: Editorial,
        font: EditorialFont,
        radius: 18,
        art: 4,
    },
    Preset {
        id: "violet",
        name: "Violet",
        accent: "#ba3cff",
        background: "#0d1117",
        surface: "#1b2028",
        ink: "#e6edf3",
        layout: Rail,
        font: System,
        radius: 12,
        art: 5,
    },
    Preset {
        id: "saffron",
        name: "Saffron",
        accent: "#ffc400",
        background: "#0c0d10",
        surface: "#202126",
        ink: "#fafafa",
        layout: Top,
        font: Condensed,
        radius: 12,
        art: 6,
    },
    Preset {
        id: "midnight",
        name: "Midnight",
        accent: "#4d8df7",
        background: "#090d18",
        surface: "#161e31",
        ink: "#f5f7fb",
        layout: Top,
        font: System,
        radius: 14,
        art: 7,
    },
    Preset {
        id: "amethyst",
        name: "Amethyst Rooms",
        accent: "#c34ee8",
        background: "#100a18",
        surface: "#21132d",
        ink: "#f5eafa",
        layout: Rooms,
        font: System,
        radius: 18,
        art: 8,
    },
    Preset {
        id: "jade",
        name: "Jade",
        accent: "#71c92e",
        background: "#012326",
        surface: "#012e32",
        ink: "#e5f6ef",
        layout: Compact,
        font: System,
        radius: 10,
        art: 9,
    },
    Preset {
        id: "rose",
        name: "Rose",
        accent: "#da4167",
        background: "#000032",
        surface: "#11112b",
        ink: "#fff5f8",
        layout: Immersive,
        font: System,
        radius: 14,
        art: 10,
    },
    Preset {
        id: "garnet",
        name: "Garnet",
        accent: "#da4168",
        background: "#000032",
        surface: "#14142f",
        ink: "#fff5f8",
        layout: Top,
        font: System,
        radius: 10,
        art: 11,
    },
    Preset {
        id: "brass",
        name: "Brass",
        accent: "#fdca55",
        background: "#1c1c1c",
        surface: "#29251c",
        ink: "#fff8df",
        layout: Rail,
        font: Condensed,
        radius: 8,
        art: 0,
    },
    Preset {
        id: "aurum",
        name: "Aurum",
        accent: "#f2bd3f",
        background: "#07080a",
        surface: "#101218",
        ink: "#fff9e8",
        layout: Immersive,
        font: Condensed,
        radius: 14,
        art: 4,
    },
    Preset {
        id: "verdant",
        name: "Verdant",
        accent: "#1ec463",
        background: "#080b09",
        surface: "#0d1a12",
        ink: "#eaffed",
        layout: Rail,
        font: System,
        radius: 12,
        art: 9,
    },
    Preset {
        id: "crimson",
        name: "Crimson",
        accent: "#c41e1e",
        background: "#080707",
        surface: "#191313",
        ink: "#fff1f1",
        layout: Compact,
        font: Condensed,
        radius: 10,
        art: 2,
    },
    Preset {
        id: "onyx",
        name: "Onyx Gold",
        accent: "#ffd700",
        background: "#0a0a0a",
        surface: "#181710",
        ink: "#fff8de",
        layout: Immersive,
        font: EditorialFont,
        radius: 16,
        art: 6,
    },
];

pub fn get(id: &str) -> Option<&'static Preset> {
    PRESETS.iter().find(|p| p.id == id)
}

fn section(id: &str, kind: Kind, title: &str, text: &str, button: &str, art: u8) -> Component {
    let mut node = Component::new(id, kind, title);
    node.text = text.into();
    node.button = button.into();
    node.art = art;
    node
}

const TITLES: [&str; 12] = [
    "Golden Horizon",
    "Crystal Garden",
    "Midnight Cards",
    "Solar Palace",
    "Jade Fortune",
    "Velvet Moon",
    "Ruby Crown",
    "Ocean Pearl",
    "Aurora Gems",
    "Amber Temple",
    "Royal Orbit",
    "Silver Mirage",
];

pub fn design(id: &str) -> ModelResult<Design> {
    let p = get(id).ok_or("Plantilla desconocida.")?;
    let theme = Theme {
        brand: "Mi casino".into(),
        accent: p.accent.into(),
        button_color: None,
        glow_color: None,
        background: p.background.into(),
        surface: p.surface.into(),
        ink: p.ink.into(),
        radius: p.radius,
        font: p.font,
        layout: p.layout,
    };
    let header = section(
        "header",
        Kind::Header,
        "Inicio",
        "Explor� el cat�logo",
        "Acceder",
        p.art,
    );
    let mut hero = section(
        "hero",
        Kind::Hero,
        "Un nuevo nivel de entretenimiento",
        "Descubr� un espacio hecho a tu medida. Una experiencia visual que comienza aqu�.",
        "Explorar juegos",
        p.art,
    );
    if matches!(
        id,
        "violet"
            | "saffron"
            | "midnight"
            | "jade"
            | "garnet"
            | "brass"
            | "verdant"
            | "crimson"
            | "onyx"
    ) {
        hero.title.clear();
        hero.text.clear();
        hero.button.clear();
    }
    let mut categories = section("categories", Kind::Categories, "Categor�as", "", "", p.art);
    for (index, title) in [
        "Destacados",
        "Slots",
        "Casino en vivo",
        "Cl�sicos",
        "Favoritos",
    ]
    .iter()
    .enumerate()
    {
        categories.children.push(section(
            &format!("category-{index}"),
            Kind::Tile,
            title,
            "",
            "",
            index as u8,
        ));
    }
    let mut catalog = section(
        "catalog",
        Kind::Catalog,
        "Juegos destacados",
        "Una selecci�n de demostraciones visuales.",
        "",
        p.art,
    );
    for (index, title) in TITLES.iter().enumerate() {
        catalog.children.push(section(
            &format!("game-{index}"),
            Kind::Tile,
            title,
            "Demo local",
            "Ver demo",
            (index as u8 + p.art) % 12,
        ));
    }
    let mut live = section(
        "live",
        Kind::Live,
        "Casino en vivo",
        "La experiencia visual de las mesas cl�sicas.",
        "",
        p.art,
    );
    for (index, title) in ["Mesa cl�sica", "Mesa premium", "Ruleta de sal�n"]
        .iter()
        .enumerate()
    {
        live.children.push(section(
            &format!("live-{index}"),
            Kind::Tile,
            title,
            "Vista de muestra",
            "Ver demo",
            (index as u8 + p.art + 3) % 12,
        ));
    }
    let footer = section(
        "footer",
        Kind::Footer,
        "Entretenimiento responsable",
        "Dise�o de demostraci�n. No se crean cuentas ni se realizan apuestas.",
        "",
        p.art,
    );
    let mut lobby = vec![
        header.clone(),
        hero,
        categories,
        catalog,
        live,
        footer.clone(),
    ];
    crate::media::apply_defaults(id, &mut lobby);
    if p.layout == Layout::Rooms {
        lobby.swap(1, 2);
    }
    let mut metrics = section(
        "metrics",
        Kind::Metrics,
        "Resumen",
        "Datos ilustrativos, no reales.",
        "",
        p.art,
    );
    for (index, (title, text)) in [
        ("Usuarios", "-"),
        ("Actividad", "-"),
        ("Sesiones", "-"),
        ("Balance", "-"),
    ]
    .into_iter()
    .enumerate()
    {
        metrics.children.push(section(
            &format!("metric-{index}"),
            Kind::Tile,
            title,
            text,
            "",
            index as u8,
        ));
    }
    let mut activity = section(
        "activity",
        Kind::Activity,
        "Actividad reciente",
        "Sin operaciones reales.",
        "",
        p.art,
    );
    for (index, title) in [
        "Sesiones de muestra",
        "Movimientos de ejemplo",
        "Configuraci�n",
    ]
    .iter()
    .enumerate()
    {
        activity.children.push(section(
            &format!("activity-{index}"),
            Kind::Tile,
            title,
            "Sin datos",
            "",
            index as u8,
        ));
    }
    Ok(Design {
        theme,
        lobby,
        panel: vec![
            header,
            section(
                "panel-title",
                Kind::Text,
                "Panel de administraci�n",
                "Previsualizaci�n de la interfaz.",
                "",
                p.art,
            ),
            metrics,
            activity,
            footer,
        ],
    })
}

pub fn workspace(id: &str) -> ModelResult<Workspace> {
    Ok(Workspace {
        version: SCHEMA_VERSION,
        active: id.into(),
        designs: BTreeMap::from([(id.into(), design(id)?)]),
        appearance: Appearance::default(),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_presets_valid() {
        for p in PRESETS {
            validate(&workspace(p.id).unwrap()).unwrap();
        }
    }
}
