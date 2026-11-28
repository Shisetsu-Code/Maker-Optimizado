use super::{SURFACE_CSS, artwork, escape, node_style, render_node};
use crate::model::{Appearance, Component, Design, Font, Kind};
use serde::Deserialize;
use std::{collections::BTreeMap, fmt::Write, sync::OnceLock};

#[derive(Deserialize)]
struct Defaults {
    catalog_title: String,
    footer_title: String,
    footer_text: String,
}
#[derive(Deserialize)]
pub(crate) struct Frame {
    defaults: Defaults,
    styles: Vec<String>,
    header: String,
    hero: String,
    categories: String,
    catalog: String,
    footer: String,
    mobile: String,
}
fn frames() -> &'static BTreeMap<String, Frame> {
    static FRAMES: OnceLock<BTreeMap<String, Frame>> = OnceLock::new();
    FRAMES.get_or_init(|| {
        serde_json::from_str(include_str!("../../frontend/frames/lobbies.json"))
            .expect("compiled lobby layouts")
    })
}
fn sheets() -> &'static BTreeMap<String, String> {
    static SHEETS: OnceLock<BTreeMap<String, String>> = OnceLock::new();
    SHEETS.get_or_init(|| {
        serde_json::from_str(include_str!("../../frontend/frames/styles.json"))
            .expect("compiled lobby styles")
    })
}
pub(crate) fn available(id: &str, design: &Design) -> bool {
    crate::presets::catalog_brand(id).is_some()
        && crate::presets::get(id).is_some_and(|p| p.layout == design.theme.layout)
}
pub(crate) fn initialize(preset: &str, nodes: &mut Vec<Component>) {
    let Some(frame) = frames().get(preset) else {
        return;
    };
    nodes.retain(|n| n.id != "live" && !(preset == "violet" && n.id == "categories"));
    for n in nodes {
        match n.id.as_str() {
            "catalog" => {
                n.title.clone_from(&frame.defaults.catalog_title);
                n.text.clear();
            }
            "footer" => {
                n.title.clone_from(&frame.defaults.footer_title);
                n.text.clone_from(&frame.defaults.footer_text);
            }
            "categories"
                if !matches!(
                    preset,
                    "amethyst" | "garnet" | "verdant" | "crimson" | "onyx"
                ) =>
            {
                n.title.clear();
                n.children = [
                    "Casino",
                    "En vivo",
                    "Deportes",
                    "Destacados",
                    "Favoritos",
                    "Promociones",
                ]
                .iter()
                .enumerate()
                .map(|(i, title)| Component::new(&format!("category-{i}"), Kind::Tile, *title))
                .collect();
            }
            _ => {}
        }
    }
}
fn image_src(n: &Component) -> String {
    if let Some(data) = &n.image {
        escape(data)
    } else if let Some(asset) = &n.asset {
        format!("/media/objects/{}", escape(asset))
    } else {
        String::new()
    }
}
fn attrs(n: &Component, preview: bool) -> String {
    let id = escape(&n.id);
    let hidden = if n.hidden && preview {
        " data-hidden=\"true\""
    } else {
        ""
    };
    let mut style = node_style(n);
    if let Some(bg) = &n.style.background {
        write!(style, "background-color:{bg};").unwrap();
    }
    if let Some(fg) = &n.style.color {
        write!(style, "color:{fg};").unwrap();
    }
    if let Some(radius) = n.style.radius {
        write!(style, "border-radius:{radius}px;").unwrap();
    }
    if let Some(padding) = n.style.padding {
        write!(style, "padding:{padding}px;").unwrap();
    }
    format!("id=\"{id}\" data-component=\"{id}\" style=\"{style}\"{hidden}")
}
fn experience(out: &mut String, preset: &str, parent: &str, tile: &Component, preview: bool) {
    if tile.hidden && !preview {
        return;
    }
    let attr = attrs(tile, preview);
    let title = escape(&tile.title);
    let text = escape(&tile.text);
    let label = escape(&tile.button);
    let img = artwork(tile);
    if preset == "rose" && parent == "hero" {
        write!(out,r#"<button {attr} type="button" data-demo="true">{img}<div><span>{text}</span><h2>{title}</h2><strong>{label}</strong></div></button>"#).unwrap();
    } else if preset == "amethyst" {
        write!(out,r#"<button {attr} type="button" class="ms-room" data-demo="true">{img}<div><h3>{title}</h3><span>{text}</span><strong>{label}</strong></div></button>"#).unwrap();
    } else if matches!(preset, "garnet" | "verdant" | "crimson" | "onyx") {
        write!(out,r#"<button {attr} type="button" data-demo="true">{img}<div><span>{text}</span><h2>{title}</h2><strong>{label}</strong></div></button>"#).unwrap();
    } else {
        let (symbol, action) = match tile.id.as_str() {
            "category-0" => ("♠", "data-catalog-mode=\"all\""),
            "category-1" => ("◉", "data-demo=\"true\""),
            "category-2" => ("⚽", "data-demo=\"true\""),
            "category-3" => ("♛", "data-catalog-mode=\"top\""),
            "category-4" => ("♡", "data-catalog-mode=\"favorites\""),
            _ => ("✦", "data-demo=\"true\""),
        };
        let decoration = if tile.image.is_some() || tile.asset.is_some() {
            img
        } else {
            symbol.into()
        };
        write!(out,"<button {attr} type=\"button\" {action}><span>{decoration}</span>{title}{text}{label}</button>").unwrap();
    }
}
pub(crate) fn card(n: &Component, id: &str, favorite: bool, editing: bool) -> String {
    if n.hidden && !editing {
        return String::new();
    }
    let id = escape(id);
    let title = escape(&n.title);
    let mark = if favorite { "♥" } else { "♡" };
    let label = if n.button.is_empty() || n.button == "Ver demo" {
        String::new()
    } else {
        format!("<span>{}</span>", escape(&n.button))
    };
    format!(
        r#"<article class="lobby-game catalog-card" {} data-catalog-card="{id}"><button type="button" class="lobby-game-open" data-game="{id}">{}<h3>{title}</h3><p>{}</p>{label}</button><button type="button" class="lobby-fav" data-favorite="{id}" aria-label="Favorito: {title}" aria-pressed="{favorite}">{mark}</button></article>"#,
        attrs(n, editing),
        artwork(n),
        escape(&n.text)
    )
}
fn emit(out: &mut String, template: &str, n: &Component, d: &Design, preset: &str, preview: bool) {
    let mut remaining = template;
    while let Some((before, tail)) = remaining.split_once("{{") {
        out.push_str(before);
        let (slot, after) = tail.split_once("}}").expect("closed presentation slot");
        match slot {
            "src" => out.push_str(&image_src(n)),
            "image_src" => {
                let src = image_src(n);
                if src.is_empty() {
                    out.push_str("hidden");
                } else {
                    write!(out, "src=\"{src}\"").unwrap();
                }
            }
            "title" => out.push_str(&escape(&n.title)),
            "text" => out.push_str(&escape(&n.text)),
            "button" => out.push_str(&escape(&n.button)),
            "brand" => {
                if n.image.is_some() || n.asset.is_some() {
                    out.push_str(&artwork(n));
                } else {
                    write!(
                        out,
                        "<span>{}</span><strong>{}</strong>",
                        if matches!(preset, "violet" | "jade") {
                            "♠"
                        } else {
                            "✦"
                        },
                        escape(&d.theme.brand)
                    )
                    .unwrap();
                }
            }
            "placeholder" => out.push_str(&escape(
                n.style
                    .placeholder
                    .as_deref()
                    .unwrap_or("Buscar juegos o proveedores"),
            )),
            "caption" => {
                if !n.title.is_empty() || !n.text.is_empty() || !n.button.is_empty() {
                    write!(
                        out,
                        "<div class=\"user-caption\"><h1>{}</h1><p>{}</p>{}</div>",
                        escape(&n.title),
                        escape(&n.text),
                        super::button(n)
                    )
                    .unwrap();
                }
            }
            "children" => {
                for tile in &n.children {
                    experience(out, preset, &n.id, tile, preview);
                }
            }
            "seeds" => {
                for tile in &n.children {
                    out.push_str(&card(tile, &tile.id, false, preview));
                }
            }
            "extra" => {
                if n.asset.is_some() || n.image.is_some() {
                    out.push_str(&artwork(n));
                }
                out.push_str(&super::button(n));
            }
            _ => unreachable!("unknown compiled slot"),
        }
        remaining = after;
    }
    out.push_str(remaining);
}
fn section(
    out: &mut String,
    frame: &Frame,
    n: &Component,
    d: &Design,
    preset: &str,
    preview: bool,
) {
    if n.hidden && !preview {
        return;
    }
    let template = match (n.id.as_str(), n.kind) {
        ("header", Kind::Header) => &frame.header,
        ("hero", Kind::Hero | Kind::Categories) => &frame.hero,
        ("categories", Kind::Categories) => &frame.categories,
        ("catalog", Kind::Catalog) => &frame.catalog,
        ("footer", Kind::Footer) => &frame.footer,
        _ => {
            out.push_str("<div class=\"panel-extra\">");
            render_node(out, n, &d.theme, preview);
            out.push_str("</div>");
            return;
        }
    };
    write!(out, "<div {} data-native-section>", attrs(n, preview)).unwrap();
    emit(out, template, n, d, preset, preview);
    if n.id == "categories" && n.asset.is_some() {
        out.push_str(&artwork(n));
    }
    if n.id == "catalog" && !n.text.is_empty() {
        write!(out, "<p>{}</p>", escape(&n.text)).unwrap();
    }
    out.push_str("</div>");
}

pub(super) fn document(preset: &str, d: &Design, preview: bool, appearance: &Appearance) -> String {
    let frame = &frames()[preset];
    let theme = &d.theme;
    let defaults = crate::presets::get(preset).expect("known preset");
    let button = theme.button_color.as_ref().unwrap_or(&theme.accent);
    let mut vars = String::new();
    if theme.accent != defaults.accent {
        write!(vars, "--skin-p:{};--skin-b:{};", theme.accent, theme.accent).unwrap();
    }
    if theme.background != defaults.background {
        write!(vars, "--skin-f:{};", theme.background).unwrap();
    }
    if theme.surface != defaults.surface {
        write!(vars, "--skin-t:{};", theme.surface).unwrap();
    }
    if theme.button_color.is_some() || theme.accent != defaults.accent {
        write!(
            vars,
            "--skin-b:{button};--skin-b-ink:{};",
            super::accent_ink(button)
        )
        .unwrap();
    }
    let mut out = String::with_capacity(40000);
    write!(out,r#"<!doctype html><html lang="es" data-preset="{preset}" style="{vars}"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; base-uri 'none'; form-action 'none'"><title>{}</title><style>"#,escape(&theme.brand)).unwrap();
    for key in &frame.styles {
        out.push_str(&sheets()[key]);
    }
    out.push_str(include_str!("../../frontend/frames/editor.css"));
    let face = appearance
        .settings
        .font
        .or((theme.font != defaults.font).then_some(theme.font));
    if let Some(face) = face {
        for line in SURFACE_CSS
            .lines()
            .take_while(|l| l.starts_with("@font-face"))
        {
            out.push_str(line);
        }
        let font = match face {
            Font::Barlow => "Barlow",
            Font::Archivo => "Archivo",
            Font::Oswald | Font::Condensed => "Oswald",
            Font::Editorial => "Georgia",
            Font::System => "system-ui",
        };
        write!(out, "html{{font-family:{font},sans-serif}}").unwrap();
    }
    if theme.ink != defaults.ink {
        write!(out, "html{{color:{}}}", theme.ink).unwrap();
    }
    if let Some(weight) = appearance.settings.weight {
        write!(out, "body,button{{font-weight:{weight}}}").unwrap();
    }
    if appearance.settings.scale != 100 {
        write!(out, ".lobby{{zoom:{}%}}", appearance.settings.scale).unwrap();
    }
    if let Some(radius) = appearance
        .settings
        .radius
        .or((theme.radius != defaults.radius).then_some(theme.radius))
    {
        write!(
            out,
            ".lobby button,.lobby-banner,.lobby-game img{{border-radius:{radius}px}}"
        )
        .unwrap();
    }
    if appearance.settings.enabled {
        if let Some(image) = &appearance.image {
            write!(out,"body{{background-image:linear-gradient(rgb(0 0 0 / {0}%),rgb(0 0 0 / {0}%)),url('{1}');background-position:{2};background-size:{3};background-attachment:fixed;}}",appearance.settings.shade,escape(image),appearance.settings.position.css(),appearance.settings.fit.css()).unwrap();
        }
    }
    let active_font = super::font(appearance.settings.font.unwrap_or(theme.font));
    write!(out,"</style></head><body data-preset=\"{preset}\" class=\"font-{active_font}\"><div id=\"surface\" class=\"lobby\" data-original-layout>").unwrap();
    let mut nodes = d.lobby.iter().peekable();
    if nodes
        .peek()
        .is_some_and(|n| n.kind == Kind::Header && n.id == "header")
    {
        section(&mut out, frame, nodes.next().unwrap(), d, preset, preview);
    }
    out.push_str("<main>");
    for n in nodes {
        section(&mut out, frame, n, d, preset, preview);
    }
    out.push_str("</main>");
    if d.lobby
        .iter()
        .any(|n| n.id == "header" && (!n.hidden || preview))
    {
        out.push_str(&frame.mobile);
    }
    out.push_str("</div></body></html>");
    out
}
