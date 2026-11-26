use crate::model::*;
use std::fmt::Write as _;

pub const SURFACE_CSS: &str = include_str!("../frontend/surface.css");

pub fn escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}
fn accent_ink(color: &str) -> &'static str {
    let channel = |start| {
        let n = u8::from_str_radix(&color[start..start + 2], 16).unwrap_or(0) as f64 / 255.;
        if n <= 0.04045 {
            n / 12.92
        } else {
            ((n + 0.055) / 1.055).powf(2.4)
        }
    };
    if 0.2126 * channel(1) + 0.7152 * channel(3) + 0.0722 * channel(5) > 0.179 {
        "#111111"
    } else {
        "#ffffff"
    }
}
fn layout(l: Layout) -> &'static str {
    match l {
        Layout::Rail => "rail",
        Layout::Top => "top",
        Layout::Compact => "compact",
        Layout::Editorial => "editorial",
        Layout::Rooms => "rooms",
        Layout::Immersive => "immersive",
    }
}
fn font(f: Font) -> &'static str {
    match f {
        Font::System => "system",
        Font::Condensed => "condensed",
        Font::Editorial => "editorial",
        Font::Barlow => "barlow",
        Font::Archivo => "archivo",
        Font::Oswald => "oswald",
    }
}
fn node_style(n: &Component) -> String {
    let mut css = String::new();
    if let Some(s) = &n.style.color {
        write!(css, "--node-ink:{s};").unwrap();
    }
    if let Some(s) = &n.style.background {
        write!(css, "--node-bg:{s};").unwrap();
    }
    if let Some(s) = n.style.radius {
        write!(css, "--node-radius:{s}px;").unwrap();
    }
    if let Some(s) = n.style.padding {
        write!(css, "--node-padding:{s}px;").unwrap();
    }
    if let Some(s) = n.style.font_size {
        write!(css, "--node-size:{s}px;").unwrap();
    }
    if let Some(s) = n.style.columns {
        write!(css, "--columns:{s};").unwrap();
    }
    if let Some(s) = n.style.font_weight {
        write!(css, "--node-weight:{s};").unwrap();
    }
    if let Some(s) = n.style.align {
        write!(css, "text-align:{};", s.css()).unwrap();
    }
    if let Some(s) = &n.style.border_color {
        write!(css, "border-color:{s};border-style:solid;").unwrap();
    }
    if let Some(s) = n.style.border_width {
        write!(css, "border-width:{s}px;border-style:solid;").unwrap();
    }
    if let Some(s) = n.style.width {
        write!(css, "width:{s}%;").unwrap();
    }
    if let Some(s) = n.style.height {
        write!(css, "min-height:{s}px;").unwrap();
    }
    if let Some(s) = n.style.fit {
        write!(css, "--node-fit:{};", s.css()).unwrap();
    }
    if let Some(s) = n.style.position {
        write!(css, "--node-position:{};", s.css()).unwrap();
    }
    css
}
fn artwork(n: &Component) -> String {
    if let Some(image) = &n.image {
        format!(
            "<img class=\"art-image\" alt=\"{}\" src=\"{}\" loading=\"lazy\" decoding=\"async\">",
            escape(&n.title),
            escape(image)
        )
    } else if let Some(asset) = &n.asset {
        format!(
            "<img class=\"art-image\" alt=\"{}\" src=\"/media/objects/{}\" loading=\"lazy\" decoding=\"async\">",
            escape(&n.title),
            escape(asset)
        )
    } else {
        let symbols = ["✦", "◆", "♠", "◈", "☀", "✧", "♛", "◉", "◇", "❖", "●", "✺"];
        format!(
            "<span class=\"art art-{}\" aria-hidden=\"true\"><span>{}</span></span>",
            n.art,
            symbols[usize::from(n.art)]
        )
    }
}
fn optional_artwork(n: &Component, class: &str) -> String {
    if n.image.is_some() || n.asset.is_some() {
        format!("<div class=\"{class}\">{}</div>", artwork(n))
    } else {
        String::new()
    }
}
fn button(n: &Component) -> String {
    if n.button.is_empty() {
        String::new()
    } else {
        format!(
            "<button type=\"button\" class=\"action\" data-demo=\"true\">{}</button>",
            escape(&n.button)
        )
    }
}
fn render_node(out: &mut String, n: &Component, theme: &Theme, preview: bool, category_tile: bool) {
    if n.hidden && !preview {
        return;
    }
    let hidden = if n.hidden {
        " data-hidden=\"true\""
    } else {
        ""
    };
    let attr = format!(
        "id=\"{}\" data-component=\"{}\" style=\"{}\"{}",
        escape(&n.id),
        escape(&n.id),
        node_style(n),
        hidden
    );
    let title = escape(&n.title);
    let text = escape(&n.text);
    match n.kind {
        Kind::Header => {
            write!(out,"<header class=\"component header\" {attr}><a class=\"brand\" href=\"#surface\">{}{}</a><nav><a href=\"#catalog\">{title}</a><a href=\"#categories\">Categorías</a><a href=\"#live\">En vivo</a></nav><span class=\"header-caption\">{text}</span>{}</header>",if n.image.is_some() || n.asset.is_some() {optional_artwork(n,"brand-art")} else {"<span class=\"brand-mark\">M</span>".to_owned()},escape(&theme.brand),button(n)).unwrap();
        }
        Kind::Hero => {
            write!(out, "<section class=\"component hero\" {attr}>").unwrap();
            if !n.title.is_empty() || !n.text.is_empty() || !n.button.is_empty() {
                write!(out,"<div class=\"hero-copy\"><span class=\"eyebrow\">EXPERIENCIA · DEMO</span><h1>{title}</h1><p>{text}</p>").unwrap();
                if !n.button.is_empty() {
                    write!(out,"<button type=\"button\" class=\"action\" data-catalog-mode=\"all\" data-catalog-scroll=\"catalog\">{}</button>",escape(&n.button)).unwrap();
                }
                out.push_str("</div>");
            }
            write!(
                out,
                "<div class=\"hero-art\">{}</div></section>",
                artwork(n)
            )
            .unwrap();
        }
        Kind::Categories | Kind::Catalog | Kind::Live | Kind::Metrics | Kind::Activity => {
            let group = match n.kind {
                Kind::Categories => "categories",
                Kind::Catalog => "catalog",
                Kind::Live => "live",
                Kind::Metrics => "metrics",
                _ => "activity",
            };
            write!(out,"<section class=\"component group {group}\" {attr}><div class=\"section-head\"><div><h2>{title}</h2><p>{text}</p></div>{}</div>",button(n)).unwrap();
            out.push_str(&optional_artwork(n, "section-art"));
            if n.kind == Kind::Catalog {
                write!(out,"<label class=\"search-label\">Buscar <input type=\"search\" class=\"catalog-search\" placeholder=\"{}\" data-search=\"true\"></label>",escape(n.style.placeholder.as_deref().unwrap_or("Buscar juegos"))).unwrap();
            }
            if n.id == "activity" && n.kind == Kind::Activity {
                out.push_str("<div id=\"panel-workspace\">");
                out.push_str(&crate::demo_panel::Panel::default().content());
                out.push_str("</div>");
            }
            out.push_str("<div class=\"items\">");
            for child in &n.children {
                render_node(out, child, theme, preview, n.kind == Kind::Categories);
            }
            out.push_str("</div></section>");
        }
        Kind::Tile => {
            write!(out,"<article class=\"component tile\" {attr}><div class=\"tile-art{}\">{}</div><div class=\"tile-copy\"><h3>{title}</h3><p>{text}</p>{}</div>",if n.image.is_some() || n.asset.is_some() {" has-artwork"} else {""},artwork(n),button(n)).unwrap();
            if category_tile {
                write!(out,"<button type=\"button\" class=\"category-hit\" data-category-nav=\"{title}\" aria-label=\"{title}\"></button>").unwrap();
            }
            out.push_str("</article>");
        }
        Kind::Footer => {
            write!(out,"<footer class=\"component footer\" {attr}><strong>{title}</strong><p>{text}</p>{}{}</footer>",optional_artwork(n,"section-art"),button(n)).unwrap();
        }
        Kind::Text => {
            write!(out,"<section class=\"component text-block\" {attr}><h2>{title}</h2><p>{text}</p>{}{}</section>",optional_artwork(n,"section-art"),button(n)).unwrap();
        }
        Kind::Image => {
            write!(out,"<section class=\"component image-block\" {attr}><h2>{title}</h2><div class=\"image-content\">{}</div><p>{text}</p>{}</section>",artwork(n),button(n)).unwrap();
        }
    }
}

pub fn catalog_card(n: &Component, game: &str, favorite: bool, preview: bool) -> String {
    if n.hidden && !preview {
        return String::new();
    }
    let hidden = if n.hidden {
        " data-hidden=\"true\""
    } else {
        ""
    };
    let id = escape(&n.id);
    let game = escape(game);
    let pressed = if favorite { "true" } else { "false" };
    let mark = if favorite { "♥" } else { "♡" };
    let action = if n.button.is_empty() {
        "Ver demo".to_owned()
    } else {
        escape(&n.button)
    };
    format!(
        "<article class=\"component tile catalog-card\" id=\"{id}\" data-component=\"{id}\" data-catalog-card=\"{game}\" style=\"{}\"{hidden}><div class=\"tile-art\">{}</div><div class=\"tile-copy\"><h3>{}</h3><p>{}</p><button type=\"button\" class=\"action\" data-game=\"{game}\">{action}</button></div><button type=\"button\" class=\"favorite\" data-favorite=\"{game}\" aria-label=\"Favorito: {}\" aria-pressed=\"{pressed}\">{mark}</button></article>",
        node_style(n),
        artwork(n),
        escape(&n.title),
        escape(&n.text),
        escape(&n.title)
    )
}
pub fn document(design: &Design, view: View, preview: bool) -> String {
    document_with_appearance(design, view, preview, &Appearance::default())
}
pub fn document_with_appearance(
    design: &Design,
    view: View,
    preview: bool,
    appearance: &Appearance,
) -> String {
    document_with_profile(design, view, preview, appearance, None)
}
pub fn document_with_profile(
    design: &Design,
    view: View,
    preview: bool,
    appearance: &Appearance,
    preset: Option<&str>,
) -> String {
    let t = &design.theme;
    let profile = preset
        .filter(|id| crate::presets::get(id).is_some_and(|p| p.layout == t.layout))
        .unwrap_or("custom");
    let settings = &appearance.settings;
    let mut out = String::with_capacity(16_000);
    let button = t.button_color.as_deref().unwrap_or(&t.accent);
    let glow = t.glow_color.as_deref().unwrap_or(&t.accent);
    let mut css = format!(
        "--accent:{};--accent-ink:{};--button:{};--button-ink:{};--glow:{};--page:{};--surface:{};--ink:{};--radius:{}px;--text-scale:{};",
        t.accent,
        accent_ink(&t.accent),
        button,
        accent_ink(button),
        glow,
        t.background,
        t.surface,
        t.ink,
        settings.radius.unwrap_or(t.radius),
        f64::from(settings.scale) / 100.
    );
    if let Some(weight) = settings.weight {
        write!(css, "--global-weight:{weight};").unwrap();
    }
    write!(out,"<!doctype html><html lang=\"es\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; style-src 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; base-uri 'none'; form-action 'none'\"><title>{}</title><style>{SURFACE_CSS}</style></head><body data-profile=\"{profile}\" class=\"layout-{} font-{}\" style=\"{css}\">",escape(&t.brand),layout(t.layout),font(settings.font.unwrap_or(t.font))).unwrap();
    if settings.enabled {
        if let Some(image) = &appearance.image {
            write!(out,"<div class=\"page-backdrop\" aria-hidden=\"true\" style=\"--backdrop-fit:{};--backdrop-position:{};--backdrop-opacity:{}\"><img alt=\"\" src=\"{}\"></div>",settings.fit.css(),settings.position.css(),1.-f64::from(settings.shade)/100.,escape(image)).unwrap();
        }
    }
    write!(
        out,
        "<main id=\"surface\" class=\"{}\">",
        if view == View::Lobby {
            "lobby"
        } else {
            "panel"
        }
    )
    .unwrap();
    if view == View::Panel {
        out.push_str(&crate::demo_panel::Panel::default().navigation());
        out.push_str("<div class=\"panel-body\">");
    }
    for n in design.nodes(view) {
        render_node(&mut out, n, t, preview, false);
    }
    if view == View::Panel {
        out.push_str("</div>");
    }
    if view == View::Lobby && design.lobby.iter().any(|n| n.kind == Kind::Header) {
        out.push_str(r#"<nav class="lobby-mobile" aria-label="Navegación móvil"><button type="button" data-catalog-mode="all" data-catalog-scroll="top"><span aria-hidden="true">⌂</span>Inicio</button><button type="button" data-catalog-mode="all" data-catalog-scroll="catalog"><span aria-hidden="true">▦</span>Juegos</button><button type="button" data-catalog-mode="favorites" data-catalog-scroll="catalog"><span aria-hidden="true">♡</span>Favoritos</button><button type="button" data-demo="true"><span aria-hidden="true">◉</span>Ayuda</button></nav>"#);
        if profile == "violet" {
            out.push_str(r#"<aside class="lobby-rail" aria-label="Explorar"><strong>EXPLORAR</strong><button type="button" data-catalog-mode="all" data-catalog-scroll="catalog">Todos los juegos</button><button type="button" data-catalog-mode="top" data-catalog-scroll="catalog">Destacados</button><button type="button" data-catalog-mode="favorites" data-catalog-scroll="catalog">Mis favoritos</button><button type="button" data-catalog-mode="recent" data-catalog-scroll="catalog">Recientes</button><strong>PROVEEDORES</strong><div data-provider-rail></div><strong>AYUDA</strong><button type="button" data-demo="true">Soporte demo</button></aside>"#);
        }
    }
    out.push_str("</main></body></html>");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rendering_escapes_user_text() {
        let mut d = crate::presets::design("ember").unwrap();
        d.theme.brand = "<script>alert(1)</script>".into();
        d.lobby[1].title = "<img src=x onerror=alert(1)>".into();
        let html = document(&d, View::Lobby, false);
        assert!(!html.contains("<script>"));
        assert!(!html.contains("<img src=x"));
        assert!(html.contains("&lt;script&gt;"));
    }
    #[test]
    fn hidden_not_exported() {
        let mut d = crate::presets::design("ember").unwrap();
        d.lobby[1].hidden = true;
        assert!(!document(&d, View::Lobby, false).contains("id=\"hero\""));
        assert!(document(&d, View::Lobby, true).contains("data-hidden=\"true\""));
    }
    #[test]
    fn all_templates_render_both_views() {
        for p in crate::presets::PRESETS {
            let d = crate::presets::design(p.id).unwrap();
            for view in [View::Lobby, View::Panel] {
                assert!(document(&d, view, false).contains("id=\"surface\""));
            }
        }
    }
}
