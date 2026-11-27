//! Native presentation recipes for the twelve catalogue-based layouts.
//! No legacy executable code is used; all editable data come from the Rust model.
use crate::{
    model::*,
    render::{accent_ink, escape, node_style},
};
use serde::Deserialize;
use std::{collections::BTreeMap, fmt::Write as _, sync::OnceLock};

const RAIL: &str = include_str!("../frontend/gallery/rail.css");
const BASE: &str = include_str!("../frontend/gallery/base.css");
const COMPAT: &str = include_str!("../frontend/gallery/compat.css");

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Composition {
    Rail,
    Cover,
    Rooms,
    Experiences,
    Mascot,
}
#[derive(Deserialize)]
struct Tile {
    title: String,
    text: String,
    asset: String,
    action: String,
}
#[derive(Deserialize)]
struct Profile {
    kind: Composition,
    hero: String,
    tiles: Vec<Tile>,
}
fn profiles() -> &'static BTreeMap<String, Profile> {
    static PROFILES: OnceLock<BTreeMap<String, Profile>> = OnceLock::new();
    PROFILES.get_or_init(|| {
        serde_json::from_str(include_str!("../assets/gallery-layouts.json"))
            .expect("static gallery profiles")
    })
}
pub fn enabled(preset: &str, design: &Design) -> bool {
    profiles().contains_key(preset)
        && crate::presets::get(preset).is_some_and(|p| p.layout == design.theme.layout)
}
pub fn has_profile(preset: &str) -> bool {
    profiles().contains_key(preset)
}

const QUICK: [(&str, &str, &str); 6] = [
    ("♠", "Casino", "all"),
    ("◉", "En vivo", "live"),
    ("⚽", "Deportes", "demo"),
    ("♛", "Destacados", "top"),
    ("♡", "Favoritos", "favorites"),
    ("✦", "Promociones", "demo"),
];

pub fn defaults(preset: &str, theme: &mut Theme, nodes: &mut [Component]) {
    let Some(profile) = profiles().get(preset) else {
        return;
    };
    theme.brand = "Tu casino".into();
    theme.ink = if preset == "violet" {
        "#e6edf3".into()
    } else {
        "#f6f6f6".into()
    };
    for node in nodes {
        match node.kind {
            Kind::Header => {
                node.title = "Casino".into();
                node.text.clear();
                node.button = if preset == "violet" {
                    "Entrar".into()
                } else {
                    "Ingresar".into()
                };
            }
            Kind::Hero => {
                node.asset = Some(profile.hero.clone());
                let (title, text, button) = match preset {
                    "amethyst" => (
                        "Las salas más poderosas",
                        "Tu mundo de juegos",
                        "Entrar ahora →",
                    ),
                    "rose" => ("Casino", "Descubrí tus favoritos", "Explorar juegos →"),
                    "aurum" => (
                        "Renace\nla emoción.",
                        "Explorá tus juegos favoritos en un mundo de nuevas experiencias.",
                        "Descubrir juegos →",
                    ),
                    "verdant" => (
                        "Donde empieza\ntu próxima aventura.",
                        "",
                        "Explorar casino →",
                    ),
                    "crimson" => ("Entra a tu\npróxima partida.", "", "Explorar casino →"),
                    _ => ("", "", ""),
                };
                node.title = title.into();
                node.text = text.into();
                node.button = button.into();
            }
            Kind::Categories => {
                node.title.clear();
                node.text.clear();
                node.children.clear();
                for (index, tile) in profile.tiles.iter().enumerate() {
                    let mut n =
                        Component::new(format!("category-{index}"), Kind::Tile, &tile.title);
                    n.text.clone_from(&tile.text);
                    n.button = match (preset, index) {
                        ("rose", 0) => "Ver mesas →",
                        ("rose", 1) => "Explorar demo →",
                        _ => "Explorar →",
                    }
                    .into();
                    n.asset = Some(tile.asset.clone());
                    node.children.push(n);
                }
                if preset != "violet" {
                    for (index, (_, title, _)) in QUICK.iter().enumerate() {
                        node.children.push(Component::new(
                            format!("quick-{index}"),
                            Kind::Tile,
                            *title,
                        ));
                    }
                }
            }
            Kind::Catalog => {
                node.title = "Todos los juegos".into();
                node.text.clear();
                node.style.placeholder = Some("Buscar juegos o proveedores".into());
            }
            Kind::Live => node.hidden = true,
            Kind::Footer => {
                node.title = "Tu casino · Demo".into();
                node.text = "Interfaz de demostración con recursos locales. Ingreso, registro, soporte y deportes se muestran como ejemplos; no se procesan pagos ni apuestas.".into();
            }
            _ => {}
        }
    }
}

pub(crate) fn visible(n: &Component, preview: bool) -> bool {
    !n.hidden || preview
}
pub(crate) fn attrs(n: &Component) -> String {
    format!(
        "id=\"{}\" data-component=\"{}\" style=\"{}\"{}",
        escape(&n.id),
        escape(&n.id),
        escape(&node_style(n)),
        if n.hidden {
            " data-hidden=\"true\""
        } else {
            ""
        }
    )
}
fn image(n: &Component, primary: bool) -> String {
    let src = n
        .image
        .as_deref()
        .map(str::to_owned)
        .or_else(|| n.asset.as_ref().map(|a| format!("/media/objects/{a}")));
    if let Some(src) = src {
        format!(
            "<img {} alt=\"{}\" src=\"{}\" loading=\"lazy\" decoding=\"async\">",
            if primary {
                "data-primary-art=\"true\""
            } else {
                ""
            },
            escape(&n.title),
            escape(&src)
        )
    } else {
        "<span class=\"gallery-missing\" role=\"img\" aria-label=\"Imagen no disponible\">Sin imagen</span>".into()
    }
}
pub(crate) fn action_attr(action: &str) -> &'static str {
    match action {
        "all" => "data-catalog-mode=\"all\" data-catalog-scroll=\"catalog\"",
        "top" => "data-catalog-mode=\"top\" data-catalog-scroll=\"catalog\"",
        "favorites" => "data-catalog-mode=\"favorites\" data-catalog-scroll=\"catalog\"",
        "recent" => "data-catalog-mode=\"recent\" data-catalog-scroll=\"catalog\"",
        "live" => "data-experience=\"live\"",
        "crash" => "data-experience=\"crash\"",
        "slots" => "data-experience=\"slots\"",
        _ => "data-demo=\"true\"",
    }
}
fn action_button(out: &mut String, label: &str, action: &str, class: &str) {
    if !label.is_empty() {
        write!(
            out,
            "<button type=\"button\" class=\"{class}\" {}>{}</button>",
            action_attr(action),
            escape(label)
        )
        .unwrap();
    }
}
fn header(out: &mut String, n: &Component, theme: &Theme, preset: &str) {
    let disclaimer = if preset == "violet" {
        "PLANTILLA DE DEMOSTRACIÓN · SIN APUESTAS NI OFERTAS REALES"
    } else {
        "DEMO · Sin apuestas reales ni promociones vigentes"
    };
    let icon = if matches!(preset, "violet" | "jade") {
        "♠"
    } else {
        "✦"
    };
    write!(out,"<div class=\"gallery-demo\">{disclaimer}</div><header {}><button type=\"button\" class=\"gallery-menu\" data-gallery-menu aria-label=\"Mostrar menú\">☰</button><a class=\"gallery-brand\" href=\"#surface\">",attrs(n)).unwrap();
    if n.image.is_some() || n.asset.is_some() {
        out.push_str(&image(n, false));
    } else {
        write!(
            out,
            "<span>{icon}</span><strong>{}</strong>",
            escape(&theme.brand)
        )
        .unwrap();
    }
    out.push_str("</a><nav>");
    action_button(out, &n.title, "all", "");
    if preset != "violet" {
        action_button(out, "En vivo", "live", "");
    }
    action_button(out, "Deportes", "demo", "");
    out.push_str("</nav>");
    if !n.text.is_empty() {
        write!(out, "<small>{}</small>", escape(&n.text)).unwrap();
    }
    out.push_str("<div class=\"gallery-access\">");
    action_button(out, &n.button, "demo", "");
    action_button(out, "Registrarse", "demo", "gallery-primary");
    out.push_str("</div></header>");
    out.push_str("<aside class=\"gallery-sidebar lobby-rail\">");
    if preset == "violet" {
        action_button(out, "✦ Premio diario", "demo", "gallery-promo");
        action_button(out, "♛ Torneos", "demo", "gallery-promo");
        action_button(out, "◈ Promociones", "demo", "gallery-promo");
        out.push_str("<h3>JUEGOS</h3>");
        action_button(out, "▦ Todos los juegos", "all", "");
        action_button(out, "♡ Mis favoritos", "favorites", "");
        action_button(out, "◉ Casino en vivo", "live", "");
        out.push_str("<h3>PROVEEDORES</h3><div id=\"gallery-providers\" data-provider-rail></div><h3>AYUDA</h3>");
        action_button(out, "☏ Soporte demo", "demo", "");
        action_button(out, "18+ Juego responsable", "demo", "");
    } else {
        out.push_str("<h3>EXPLORAR</h3>");
        action_button(out, "Todos los juegos", "all", "");
        action_button(out, "Destacados", "top", "");
        action_button(out, "Mis favoritos", "favorites", "");
        action_button(out, "Soporte demo", "demo", "");
    }
    out.push_str("</aside>");
}
fn copy(out: &mut String, n: &Component, eyebrow: &str) {
    if !eyebrow.is_empty() {
        write!(out, "<span class=\"editorial-eyebrow\">{eyebrow}</span>").unwrap();
    }
    if !n.title.is_empty() {
        write!(out, "<h1>{}</h1>", escape(&n.title)).unwrap();
    }
    if !n.text.is_empty() {
        write!(out, "<p>{}</p>", escape(&n.text)).unwrap();
    }
    action_button(out, &n.button, "all", "gallery-primary");
}
fn image_tile(out: &mut String, n: &Component, action: &str, preset: &str) {
    let class = if preset == "amethyst" {
        "rooms-room"
    } else {
        ""
    };
    write!(
        out,
        "<button type=\"button\" class=\"{class}\" {} {}>{}",
        attrs(n),
        action_attr(action),
        image(n, false)
    )
    .unwrap();
    if preset == "garnet" {
        write!(out, "<strong>{}</strong>", escape(&n.title)).unwrap();
    } else if preset == "amethyst" {
        write!(
            out,
            "<div><h3>{}</h3><span>{}</span><strong>{}</strong></div>",
            escape(&n.title),
            escape(&n.text),
            escape(&n.button)
        )
        .unwrap();
    } else {
        write!(
            out,
            "<div><span>{}</span><h2>{}</h2><strong>{}</strong></div>",
            escape(&n.text),
            escape(&n.title),
            escape(&n.button)
        )
        .unwrap();
    }
    out.push_str("</button>");
}
fn hero(
    out: &mut String,
    n: &Component,
    design: &Design,
    profile: &Profile,
    preset: &str,
    preview: bool,
) {
    match profile.kind {
        Composition::Rail | Composition::Cover => {
            write!(out,"<section class=\"gallery-banner hero-art\" {}>{}<button type=\"button\" id=\"gallery-prev\" data-banner-step=\"-1\" aria-label=\"Portada anterior\">‹</button><button type=\"button\" id=\"gallery-next\" data-banner-step=\"1\" aria-label=\"Portada siguiente\">›</button>",attrs(n),image(n,true)).unwrap();
            if !n.title.is_empty() || !n.text.is_empty() || !n.button.is_empty() {
                out.push_str("<div class=\"editorial-hero-copy gallery-cover-copy\">");
                let eyebrow = match preset {
                    "verdant" => "BIENVENIDO A TU NUEVA EXPERIENCIA",
                    "crimson" => "ELIGE TU CATEGORÍA",
                    _ => "",
                };
                copy(out, n, eyebrow);
                out.push_str("</div>");
            }
            out.push_str("<div class=\"gallery-banner-note\">Imagen de referencia · promociones no vigentes</div></section><div id=\"gallery-dots\" aria-label=\"Portadas\"></div>");
        }
        Composition::Mascot => {
            write!(out, "<section class=\"editorial-fenarys-hero\" {}><div><p class=\"editorial-eyebrow\">EL UNIVERSO DE TU CASINO</p>", attrs(n)).unwrap();
            if n.title == "Renace\nla emoción." {
                out.push_str("<h1>Renace<br><em>la emoción.</em></h1>");
                write!(out, "<p>{}</p>", escape(&n.text)).unwrap();
                action_button(out, &n.button, "all", "gallery-primary");
            } else {
                copy(out, n, "");
            }
            out.push_str("<div class=\"editorial-points\"><span>✦ Slots</span><span>◉ Casino en vivo</span><span>↗ Crash</span></div></div>");
            out.push_str(&image(n, true));
            out.push_str("</section>");
        }
        Composition::Rooms => {
            write!(
                out,
                "<section class=\"rooms-hero\" {}><div><p>{}</p>",
                attrs(n),
                escape(&n.text)
            )
            .unwrap();
            if n.title == "Las salas más poderosas" {
                out.push_str("<h1>Las salas más <strong>poderosas</strong></h1>");
            } else {
                write!(out, "<h1>{}</h1>", escape(&n.title)).unwrap();
            }
            out.push_str("<h2>En un solo lugar</h2><ul><li>Más juegos</li><li>Más experiencias</li><li>Más diversión</li></ul>");
            action_button(out, &n.button, "all", "gallery-primary");
            write!(out,"</div><div class=\"rooms-mark\">{}<span>Todas tus salas. Un solo lugar.</span></div><div class=\"rooms-values\"><span>♛ Jugá</span><span>✦ Explorá</span><span>♣ Compartí</span><span>♥ Disfrutá</span></div></section>",image(n,true)).unwrap();
        }
        Composition::Experiences => {
            write!(out,"<section class=\"experience-home\" {}><button type=\"button\" {}>{}<div><span>{}</span><h1>{}</h1><strong>{}</strong></div></button>",attrs(n),action_attr("all"),image(n,true),escape(&n.text),escape(&n.title),escape(&n.button)).unwrap();
            if let Some(categories) = design
                .lobby
                .iter()
                .find(|c| c.id == "categories" && visible(c, preview))
            {
                for (i, tile) in profile.tiles.iter().enumerate() {
                    if let Some(child) = categories
                        .children
                        .iter()
                        .find(|c| c.id == format!("category-{i}") && visible(c, preview))
                    {
                        image_tile(out, child, &tile.action, preset);
                    }
                }
            }
            out.push_str("</section>");
        }
    }
}
fn categories(out: &mut String, n: &Component, profile: &Profile, preset: &str, preview: bool) {
    write!(out, "<div {}>", attrs(n)).unwrap();
    if !n.title.is_empty() || !n.text.is_empty() {
        write!(
            out,
            "<h2>{}</h2><p>{}</p>",
            escape(&n.title),
            escape(&n.text)
        )
        .unwrap();
    }
    if !profile.tiles.is_empty() && preset != "rose" {
        if preset == "amethyst" {
            out.push_str("<section class=\"rooms-rooms\"><div class=\"rooms-section-heading\"><div><h2>Nuestras salas</h2><p>Distintas experiencias dentro de esta demostración.</p></div><span>Elegí tu zona</span></div><div class=\"rooms-room-grid\">");
        } else {
            let class = if preset == "garnet" {
                "family-experiences"
            } else {
                "editorial-experiences"
            };
            write!(
                out,
                "<section class=\"{class}\" aria-label=\"Experiencias\">"
            )
            .unwrap();
        }
        for child in n.children.iter().filter(|c| visible(c, preview)) {
            let index = child
                .id
                .strip_prefix("category-")
                .and_then(|i| i.parse::<usize>().ok());
            if let Some(tile) = index.and_then(|i| profile.tiles.get(i)) {
                image_tile(out, child, &tile.action, preset);
            }
        }
        if preset == "amethyst" {
            out.push_str("</div>");
        }
        out.push_str("</section>");
    }
    if preset == "rose" {
        out.push_str("<div class=\"experience-heading\"><span>EXPLORÁ EL CASINO</span><p>Categorías para cada forma de jugar</p></div><nav id=\"experience-categories\" data-group-rail aria-label=\"Categorías\"></nav>");
    }
    let quick: Vec<_> = n
        .children
        .iter()
        .filter(|c| c.id.starts_with("quick-") && visible(c, preview))
        .collect();
    if !quick.is_empty() {
        out.push_str("<section class=\"quick-links\" aria-label=\"Explorar casino\">");
        for child in quick {
            let (icon, _, action) = child
                .id
                .strip_prefix("quick-")
                .and_then(|i| i.parse::<usize>().ok())
                .and_then(|i| QUICK.get(i).copied())
                .unwrap_or(("✦", "", "demo"));
            write!(
                out,
                "<button type=\"button\" {} {}>",
                attrs(child),
                action_attr(action)
            )
            .unwrap();
            if child.asset.is_some() || child.image.is_some() {
                out.push_str(&image(child, false));
            } else {
                write!(out, "<span>{icon}</span>").unwrap();
            }
            out.push_str(&escape(&child.title));
            out.push_str("</button>");
        }
        out.push_str("</section>");
    }
    for child in n.children.iter().filter(|c| visible(c, preview)) {
        let is_profile_tile = child
            .id
            .strip_prefix("category-")
            .and_then(|i| i.parse::<usize>().ok())
            .is_some_and(|i| i < profile.tiles.len());
        if !child.id.starts_with("quick-") && !is_profile_tile {
            write!(
                out,
                "<div class=\"gallery-custom-section\" {}>{}<strong>{}</strong></div>",
                attrs(child),
                image(child, false),
                escape(&child.title)
            )
            .unwrap();
        }
    }
    out.push_str("</div>");
}
fn catalog(out: &mut String, n: &Component, preset: &str) {
    write!(out, "<section {}>", attrs(n)).unwrap();
    match preset {
        "jade" => out.push_str("<div id=\"rows-collections\" data-group-rail aria-label=\"Colecciones de juegos\"></div>"),
        "rose" => {},
        "garnet" | "brass" => out.push_str("<nav id=\"family-categories\" data-group-rail aria-label=\"Categorías\"></nav>"),
        "aurum" | "verdant" | "crimson" | "onyx" => out.push_str("<nav id=\"gallery-groups\" data-group-rail aria-label=\"Categorías\"></nav>"),
        _ => {}
    }
    if preset != "violet" {
        out.push_str("<section class=\"provider-strip\"><span>PROVEEDORES</span><div id=\"gallery-providers\" data-provider-rail></div></section>");
    }
    out.push_str("<section class=\"gallery-discover\"><div class=\"gallery-tabs catalog-controls\"><button type=\"button\" data-catalog-mode=\"all\" class=\"active\">Todos</button>");
    action_button(
        out,
        if preset == "violet" {
            "Destacados"
        } else {
            "Populares"
        },
        "top",
        "",
    );
    action_button(out, "Favoritos", "favorites", "");
    write!(out,"</div><label>⌕ <input type=\"search\" id=\"gallery-search\" data-search=\"true\" maxlength=\"120\" aria-label=\"Buscar juegos o proveedores\" placeholder=\"{}\"></label></section><div class=\"gallery-section-head\"><h1 data-catalog-title>{}</h1><span id=\"gallery-count\" data-catalog-count></span></div>",escape(n.style.placeholder.as_deref().unwrap_or("Buscar juegos o proveedores")),escape(&n.title)).unwrap();
    if !n.text.is_empty() {
        write!(out, "<p>{}</p>", escape(&n.text)).unwrap();
    }
    out.push_str("<div class=\"gallery-grid items\"></div><p data-catalog-empty hidden>No encontramos juegos para esta selección.</p><button type=\"button\" class=\"gallery-more\" data-catalog-more=\"true\">Ver más juegos</button><details class=\"gallery-extra-filters\"><summary>Más filtros y recientes</summary><div data-extra-filters></div></details></section>");
}
fn footer(out: &mut String, n: &Component) {
    write!(out,"<footer {}><strong>{}</strong><p>{}</p><span>18+ · Sin conexión al casino o proveedor original</span></footer>",attrs(n),escape(&n.title),escape(&n.text)).unwrap();
}
pub(crate) fn generic(out: &mut String, n: &Component, preview: bool) {
    write!(
        out,
        "<section class=\"gallery-custom-section\" {}><h2>{}</h2><p>{}</p>",
        attrs(n),
        escape(&n.title),
        escape(&n.text)
    )
    .unwrap();
    if n.image.is_some() || n.asset.is_some() {
        out.push_str(&image(n, false));
    }
    action_button(out, &n.button, "demo", "gallery-primary");
    if !n.children.is_empty() {
        out.push_str("<div class=\"gallery-custom-grid\">");
        for child in n.children.iter().filter(|c| visible(c, preview)) {
            write!(
                out,
                "<article {}>{}<h3>{}</h3><p>{}</p></article>",
                attrs(child),
                image(child, false),
                escape(&child.title),
                escape(&child.text)
            )
            .unwrap();
        }
        out.push_str("</div>");
    }
    out.push_str("</section>");
}
fn mobile(out: &mut String, preset: &str) {
    out.push_str("<nav class=\"gallery-mobile lobby-mobile\" aria-label=\"Navegación móvil\">");
    let links = if preset == "violet" {
        vec![
            ("⌂", "Inicio", "all"),
            ("♠", "Juegos", "top"),
            ("♡", "Favoritos", "favorites"),
            ("◉", "Cuenta", "demo"),
        ]
    } else {
        vec![
            ("⌂", "Inicio", "all"),
            ("♠", "Casino", "top"),
            ("⚽", "Deportes", "demo"),
            ("♡", "Favoritos", "favorites"),
            ("◉", "Cuenta", "demo"),
        ]
    };
    for (symbol, label, action) in links {
        write!(
            out,
            "<button type=\"button\" {}>{symbol}<small>{label}</small></button>",
            action_attr(action)
        )
        .unwrap();
    }
    out.push_str("</nav>");
}
fn css_for(preset: &str) -> &'static str {
    match preset {
        "amethyst" => include_str!("../frontend/gallery/rooms.css"),
        "jade" => include_str!("../frontend/gallery/rows.css"),
        "rose" => include_str!("../frontend/gallery/experience.css"),
        "garnet" | "brass" => include_str!("../frontend/gallery/family.css"),
        "aurum" | "verdant" | "crimson" | "onyx" => {
            include_str!("../frontend/gallery/editorial.css")
        }
        _ => "",
    }
}
pub(crate) fn font_name(font: Font) -> &'static str {
    match font {
        Font::System => "system",
        Font::Condensed => "condensed",
        Font::Editorial => "editorial",
        Font::Barlow => "barlow",
        Font::Archivo => "archivo",
        Font::Oswald => "oswald",
    }
}
pub(crate) fn scaled_css(out: &mut String, css: &str, scale: u8) {
    if scale == 100 {
        out.push_str(css);
        return;
    }
    let mut parts = css.split("font-size:");
    out.push_str(parts.next().unwrap_or(""));
    for part in parts {
        out.push_str("font-size:");
        let end = part
            .bytes()
            .take_while(|b| b.is_ascii_digit() || *b == b'.')
            .count();
        if end > 0 && part[end..].starts_with("px") {
            write!(
                out,
                "calc({}px * {}){}",
                &part[..end],
                f64::from(scale) / 100.,
                &part[end + 2..]
            )
            .unwrap();
        } else {
            out.push_str(part);
        }
    }
}
pub fn document(design: &Design, preset: &str, preview: bool, appearance: &Appearance) -> String {
    let profile = &profiles()[preset];
    let theme = &design.theme;
    let settings = &appearance.settings;
    let defaults = crate::presets::get(preset).expect("known gallery preset");
    let button = theme.button_color.as_deref().unwrap_or(&theme.accent);
    let button_ink = if theme.button_color.is_none() && theme.accent == defaults.accent {
        match preset {
            "violet" | "midnight" | "rose" | "garnet" | "crimson" | "amethyst" => "#fff",
            "jade" => "#071b10",
            "verdant" => "#04180a",
            "aurum" => "#171009",
            "onyx" => "#161003",
            _ => "#111",
        }
    } else {
        accent_ink(button)
    };
    let font = settings
        .font
        .or_else(|| (theme.font != defaults.font).then_some(theme.font))
        .map(font_name)
        .unwrap_or("original");
    let radius = settings
        .radius
        .or_else(|| (theme.radius != defaults.radius).then_some(theme.radius));
    let backdrop = settings.enabled && appearance.image.is_some();
    let mut out = String::with_capacity(48_000);
    write!(out,"<!doctype html><html lang=\"es\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; style-src 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; base-uri 'none'; form-action 'none'\"><title>{}</title><style>",escape(&theme.brand)).unwrap();
    for line in crate::render::SURFACE_CSS
        .lines()
        .take_while(|line| line.starts_with("@font-face"))
    {
        out.push_str(line);
    }
    scaled_css(
        &mut out,
        if preset == "violet" { RAIL } else { BASE },
        settings.scale,
    );
    scaled_css(&mut out, css_for(preset), settings.scale);
    scaled_css(&mut out, COMPAT, settings.scale);
    write!(out,"</style></head><body data-profile=\"{preset}\" data-gallery=\"true\" data-appearance-font=\"{font}\" data-global-weight=\"{}\" data-global-radius=\"{}\" class=\"font-{font} {}\" style=\"--accent:{};--accent-ink:{};--button:{button};--button-ink:{button_ink};--page:{};--surface:{};--ink:{};--global-weight:{};--global-radius:{}px;\">",settings.weight.is_some(),radius.is_some(),if backdrop{"gallery-background-enabled"}else{""},theme.accent,accent_ink(&theme.accent),theme.background,theme.surface,theme.ink,settings.weight.unwrap_or(400),radius.unwrap_or(theme.radius)).unwrap();
    if backdrop {
        if let Some(image) = &appearance.image {
            write!(out,"<div class=\"gallery-backdrop page-backdrop\" aria-hidden=\"true\" style=\"--backdrop-fit:{};--backdrop-position:{};--backdrop-opacity:{}\"><img alt=\"\" src=\"{}\"></div>",settings.fit.css(),settings.position.css(),1.-f64::from(settings.shade)/100.,escape(image)).unwrap();
        }
    }
    out.push_str("<div id=\"surface\" class=\"lobby\">");
    let header_node = design.lobby.first().filter(|n| n.kind == Kind::Header);
    if let Some(n) = header_node.filter(|n| visible(n, preview)) {
        header(&mut out, n, theme, preset);
    }
    out.push_str("<main>");
    for n in design
        .lobby
        .iter()
        .skip(usize::from(header_node.is_some()))
        .filter(|n| visible(n, preview))
    {
        match (n.id.as_str(), n.kind) {
            ("hero", Kind::Hero) => hero(&mut out, n, design, profile, preset, preview),
            ("categories", Kind::Categories) => categories(&mut out, n, profile, preset, preview),
            ("catalog", Kind::Catalog) => catalog(&mut out, n, preset),
            ("footer", Kind::Footer) => footer(&mut out, n),
            _ => generic(&mut out, n, preview),
        }
    }
    out.push_str("</main>");
    if header_node.is_some_and(|n| visible(n, preview)) {
        mobile(&mut out, preset);
    }
    out.push_str("</div></body></html>");
    out
}
pub fn card(n: &Component, game: &str, favorite: bool, preview: bool) -> String {
    if !visible(n, preview) {
        return String::new();
    }
    let title = escape(&n.title);
    let game = escape(game);
    let mark = if favorite { "♥" } else { "♡" };
    let extra = if !n.button.is_empty() && n.button != "Ver demo" {
        format!(
            "<span class=\"gallery-edit-action\">{}</span>",
            escape(&n.button)
        )
    } else {
        String::new()
    };
    format!(
        "<article class=\"gallery-game catalog-card\" {} data-catalog-card=\"{game}\"><button type=\"button\" class=\"gallery-game-open\" data-game=\"{game}\" aria-label=\"Ver {title}\">{}<h3>{title}</h3><p>{}</p>{extra}</button><button type=\"button\" class=\"gallery-fav\" data-favorite=\"{game}\" aria-label=\"Favorito: {title}\" aria-pressed=\"{favorite}\">{mark}</button></article>",
        attrs(n),
        image(n, false),
        escape(&n.text)
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gallery_profiles_have_editable_structure_and_no_scripts() {
        assert_eq!(profiles().len(), 12);
        for (id, profile) in profiles() {
            let mut design = crate::presets::design(id).unwrap();
            let hero = design.lobby.iter_mut().find(|n| n.id == "hero").unwrap();
            hero.title = "<script>test</script>".into();
            let html = document(&design, id, false, &Appearance::default());
            assert!(crate::media::valid_asset(&profile.hero));
            assert!(html.contains("id=\"surface\""));
            assert!(html.contains("id=\"catalog\""));
            assert!(html.contains("data-primary-art"));
            assert!(html.contains("&lt;script&gt;"));
            assert!(!html.contains("<script>"));
        }
    }
}
