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

pub fn catalog_brand(id: &str) -> Option<&'static str> {
    Some(match id {
        "violet" => "Tu casino",
        "saffron" => "Cobraya",
        "midnight" => "Lunabet",
        "amethyst" => "Multisala",
        "jade" => "Pinaplay",
        "rose" => "Celuapuestas40",
        "garnet" => "Bet30",
        "brass" => "CasinoTower30",
        "aurum" => "Fenarys",
        "verdant" => "Fortuna SJK",
        "crimson" => "RavenBet",
        "onyx" => "LuckRex",
        _ => return None,
    })
}
fn restore_lobby_sections(preset: &str, nodes: &mut [Component]) {
    let hero = nodes
        .iter_mut()
        .find(|n| n.id == "hero")
        .expect("preset hero");
    match preset {
        "ivory" => {
            hero.title = "La suerte tiene\nun nuevo lugar.".into();
            hero.text =
                "Grandes clásicos, nuevas emociones.\nElegí tu juego y hacé tuya la noche.".into();
            hero.button = "Explorar juegos ↗".into();
        }
        "amethyst" => {
            hero.title = "Las salas más poderosas".into();
            hero.text = "En un solo lugar".into();
            hero.button = "Entrar ahora →".into();
        }
        "aurum" => {
            hero.title = "Renace la emoción.".into();
            hero.text = "Explorá tus juegos favoritos en un mundo de nuevas experiencias.".into();
            hero.button = "Descubrir juegos →".into();
        }
        _ => {}
    }
    let cards: &[(&str, &str, &str)] = match preset {
        "amethyst" => &[
            (
                "Celu Apuestas",
                "La sala original",
                "6569761ec760a2a1b425e4d0ac3092ece4f90855.webp",
            ),
            (
                "Ganamos.net",
                "Pura adrenalina",
                "61dd5e13abf8a66c5a4f6594acce5b5ed08f56fc.webp",
            ),
            (
                "Tío Rico Juegos",
                "Tradición y confianza",
                "e892b940175070588140998d1de6cc101fcfe69f.webp",
            ),
            (
                "Betados",
                "Más juegos, más emoción",
                "7c00da3fca519c4a5a7807d6c956f1bebc60e43e.webp",
            ),
            (
                "FutureBet",
                "Casino galáctico",
                "0962b43954921f67c86296fa10f45f8acce57d24.webp",
            ),
        ],
        "rose" => &[
            (
                "Casino",
                "Descubrí tus favoritos",
                "7280119d39a2af5372c1e98ede79fcef57135ffa.webp",
            ),
            (
                "Casino en vivo",
                "La emoción de las mesas",
                "60d00af24df6cf157e7a043bb5bff597df522e1a.webp",
            ),
            (
                "Deportes",
                "Tu pasión, en un lugar",
                "8e01ff3945cd652de89b7085eb89a675f9217bbb.webp",
            ),
        ],
        "garnet" => &[
            (
                "Casino",
                "",
                "7280119d39a2af5372c1e98ede79fcef57135ffa.webp",
            ),
            (
                "Casino en vivo",
                "",
                "60d00af24df6cf157e7a043bb5bff597df522e1a.webp",
            ),
            (
                "Deportes",
                "",
                "8e01ff3945cd652de89b7085eb89a675f9217bbb.webp",
            ),
        ],
        "verdant" => &[
            (
                "Casino",
                "Todos tus favoritos",
                "6a2f39223cfacfc51e6077320e755cf9a93d3f27.webp",
            ),
            (
                "Casino en vivo",
                "La emoción de las mesas",
                "c93e3e79ce04a246723758a45176c3d0b5bbe5fa.webp",
            ),
            (
                "Crash",
                "Descubrí lo instantáneo",
                "8e341e9fd699c3c86c0e572ef1e2f11608a0c1a1.webp",
            ),
        ],
        "crimson" => &[
            (
                "Slots",
                "Elegí tu próxima aventura",
                "64e5d9a5b50f39f15fe5cbcdc5022262152f1258.webp",
            ),
            (
                "Crash Games",
                "Juegos instantáneos",
                "8aef33dd76a246c8279b22063921c3dcead75bbd.webp",
            ),
            (
                "Casino en vivo",
                "Mesas y experiencias",
                "3e72af84c0c587102605ec12cd0ef83243b1dd4d.webp",
            ),
        ],
        "onyx" => &[
            (
                "Casino",
                "Descubrí todos los juegos",
                "86da59273abc925a85f69ba3d9ccf3960516ff93.webp",
            ),
            (
                "Casino en vivo",
                "Entrá a las mesas",
                "8706574f446a4f0a2a14defb6ebdf1681a978971.webp",
            ),
        ],
        _ => return,
    };
    let target = if preset == "rose" {
        "hero"
    } else {
        "categories"
    };
    let group = nodes
        .iter_mut()
        .find(|n| n.id == target)
        .expect("preset group");
    group.kind = Kind::Categories;
    group.title = if preset == "amethyst" {
        "Nuestras salas"
    } else {
        ""
    }
    .into();
    group.text = if preset == "amethyst" {
        "Distintas experiencias dentro de esta demostración."
    } else {
        ""
    }
    .into();
    group.button.clear();
    group.asset = None;
    group.children = cards
        .iter()
        .enumerate()
        .map(|(i, (title, detail, asset))| {
            let label = match (preset, i) {
                ("rose", 0) => "Explorar juegos →",
                ("rose", 1) => "Ver mesas →",
                ("rose", _) => "Explorar demo →",
                ("garnet", _) => "",
                _ => "Explorar →",
            };
            let mut tile = section(
                &format!("{target}-art-{i}"),
                Kind::Tile,
                title,
                detail,
                label,
                0,
            );
            tile.asset = Some((*asset).into());
            tile
        })
        .collect();
}
pub fn design(id: &str) -> ModelResult<Design> {
    let p = get(id).ok_or("Plantilla desconocida.")?;
    let theme = Theme {
        brand: catalog_brand(id)
            .unwrap_or(if id == "ivory" {
                "Tu casino"
            } else {
                "Mi casino"
            })
            .into(),
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
    let mut header = section(
        "header",
        Kind::Header,
        "Inicio",
        "Explorá el catálogo",
        "Acceder",
        p.art,
    );
    let panel_header = header.clone();
    if id == "ivory" {
        header.title = "Descubrir".into();
        header.text = "MI SALDO".into();
        header.button = "ARS 11.052 ＋".into();
    }
    if catalog_brand(id).is_some() {
        header.title = "Casino".into();
        header.text = if id == "violet" { "Entrar" } else { "Ingresar" }.into();
        header.button = "Registrarse".into();
    }
    let mut hero = section(
        "hero",
        Kind::Hero,
        "Un nuevo nivel de entretenimiento",
        "Descubrí un espacio hecho a tu medida. Una experiencia visual que comienza aquí.",
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
    let mut categories = section("categories", Kind::Categories, "Categorías", "", "", p.art);
    for (index, title) in [
        "Destacados",
        "Slots",
        "Casino en vivo",
        "Clásicos",
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
        "Una selección de demostraciones visuales.",
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
        "La experiencia visual de las mesas clásicas.",
        "",
        p.art,
    );
    for (index, title) in ["Mesa clásica", "Mesa premium", "Ruleta de salón"]
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
        "Diseño de demostración. No se crean cuentas ni se realizan apuestas.",
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
    restore_lobby_sections(id, &mut lobby);
    let mut panel_header = panel_header;
    panel_header.title = "dueno".into();
    panel_header.text = "DUEÑO".into();
    panel_header.button = "Cargar / Descargar".into();
    panel_header.asset = Some("c7d60c43ba450618f022de96049c769a2cf81e87.webp".into());
    let mut metrics = section("metrics", Kind::Metrics, "MONITOREO", "", "", p.art);
    for (i, title) in ["Jugadores", "Servidor"].into_iter().enumerate() {
        metrics.children.push(section(
            &format!("metric-{i}"),
            Kind::Tile,
            title,
            "",
            "",
            i as u8,
        ));
    }
    let mut totals = section("totals", Kind::Metrics, "Totales", "", "", p.art);
    for (i, (title, amount, note)) in [
        ("CARGASTE HOY", "ARS 20.000,00", ""),
        ("DESCARGASTE HOY", "ARS 0,00", ""),
        (
            "TE TENDRÍA QUE QUEDAR",
            "ARS 20.000,00",
            "lo que entró menos lo que pagaste, hoy",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        totals.children.push(section(
            &format!("total-{i}"),
            Kind::Tile,
            title,
            amount,
            note,
            i as u8,
        ));
    }
    let mut activity = section("activity", Kind::Activity, "LO QUE SIGUE", "", "", p.art);
    for (i, (title, button)) in [
        (
            "4 jugadores están esperando que les resuelvas un pedido",
            "Ver pendientes",
        ),
        (
            "3 premios frenados por el tope, esperando que decidas",
            "Ver premios",
        ),
        ("2 grupos de cuentas se ven juntos", "Ver alertas"),
        (
            "1 socio no terminó de arrancar: les falta el acceso o las fichas",
            "Ver socios",
        ),
        (
            "2 del equipo todavía no configuraron el autenticador",
            "Ver equipo",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        activity.children.push(section(
            &format!("activity-{i}"),
            Kind::Tile,
            title,
            "",
            button,
            i as u8,
        ));
    }
    let footer = section(
        "footer",
        Kind::Footer,
        "MI LINK",
        "Todavía no tenés link. Crealo y mandáselo a la gente que querés que juegue con vos.",
        "Crear mi link",
        p.art,
    );
    Ok(Design {
        catalog_selection: CatalogSelection::default(),
        theme,
        lobby,
        panel: vec![
            panel_header,
            section(
                "panel-title",
                Kind::Text,
                "Inicio",
                "Lo que pasó hoy y lo que te falta hacer.",
                "",
                p.art,
            ),
            metrics,
            totals,
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
