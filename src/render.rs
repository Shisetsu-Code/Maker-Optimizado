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
fn render_node(out: &mut String, n: &Component, theme: &Theme, preview: bool) {
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
            write!(out,"<section class=\"component hero\" {attr}><div class=\"hero-copy\"><span class=\"eyebrow\">EXPERIENCIA · DEMO</span><h1>{title}</h1><p>{text}</p>{}</div><div class=\"hero-art\">{}</div></section>",button(n),artwork(n)).unwrap();
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
                out.push_str("<label class=\"search-label\">Buscar <input type=\"search\" class=\"catalog-search\" placeholder=\"Buscar juegos\" data-search=\"true\"></label>");
            }
            out.push_str("<div class=\"items\">");
            for child in &n.children {
                render_node(out, child, theme, preview);
            }
            out.push_str("</div></section>");
        }
        Kind::Tile => {
            write!(out,"<article class=\"component tile\" {attr}><div class=\"tile-art{}\">{}</div><div class=\"tile-copy\"><h3>{title}</h3><p>{text}</p>{}</div></article>",if n.image.is_some() || n.asset.is_some() {" has-artwork"} else {""},artwork(n),button(n)).unwrap();
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

pub fn catalog_card(n:&Component,game:&str,favorite:bool,preview:bool)->String{
    if n.hidden&&!preview{return String::new();}
    let hidden=if n.hidden{" data-hidden=\"true\""}else{""};
    let id=escape(&n.id);let game=escape(game);let pressed=if favorite{"true"}else{"false"};let mark=if favorite{"♥"}else{"♡"};
    let action=if n.button.is_empty(){"Ver demo".to_owned()}else{escape(&n.button)};
    format!("<article class=\"component tile catalog-card\" id=\"{id}\" data-component=\"{id}\" data-catalog-card=\"{game}\" style=\"{}\"{hidden}><div class=\"tile-art\">{}</div><div class=\"tile-copy\"><h3>{}</h3><p>{}</p><button type=\"button\" class=\"action\" data-game=\"{game}\">{action}</button></div><button type=\"button\" class=\"favorite\" data-favorite=\"{game}\" aria-label=\"Favorito: {}\" aria-pressed=\"{pressed}\">{mark}</button></article>",node_style(n),artwork(n),escape(&n.title),escape(&n.text),escape(&n.title))
}
pub fn document(design: &Design, view: View, preview: bool) -> String {
    let t = &design.theme;
    let mut out = String::with_capacity(16_000);
    write!(out,"<!doctype html><html lang=\"es\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; style-src 'unsafe-inline'; img-src 'self' data:; base-uri 'none'; form-action 'none'\"><title>{}</title><style>{SURFACE_CSS}</style></head><body class=\"layout-{} font-{}\" style=\"--accent:{};--accent-ink:{};--page:{};--surface:{};--ink:{};--radius:{}px\"><main id=\"surface\" class=\"{}\">",escape(&t.brand),layout(t.layout),font(t.font),t.accent,accent_ink(&t.accent),t.background,t.surface,t.ink,t.radius,if view==View::Lobby{"lobby"}else{"panel"}).unwrap();
    for n in design.nodes(view) {
        render_node(&mut out, n, t, preview);
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
