pub(crate) mod frames;
mod panel;
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
fn catalog_header(out: &mut String, n: &Component, theme: &Theme, preset: &str, preview: bool) {
    if n.hidden && !preview {
        return;
    }
    let violet = preset == "violet";
    let symbol = if violet || preset == "jade" {
        "♠"
    } else {
        "✦"
    };
    let label = if violet {
        "PLANTILLA DE DEMOSTRACIÓN · SIN APUESTAS NI OFERTAS REALES"
    } else {
        "DEMO · Sin apuestas reales ni promociones vigentes"
    };
    let logo = if n.image.is_some() || n.asset.is_some() {
        artwork(n)
    } else {
        format!(
            "<span>{symbol}</span><strong>{}</strong>",
            escape(&theme.brand)
        )
    };
    let live = if violet {
        ""
    } else {
        "<a href=\"#live\">En vivo</a>"
    };
    let hidden = if n.hidden {
        " data-hidden=\"true\""
    } else {
        ""
    };
    write!(out,r##"<div class="catalog-mast" id="{}" data-component="{}" style="{}"{}><div class="demo-strip">{label}</div><header><button type="button" data-catalog-menu aria-label="Mostrar menú" aria-expanded="false">☰</button><a class="identity" href="#surface">{logo}</a><nav><a href="#catalog">{}</a>{live}<button data-demo="true">Deportes</button></nav><div class="access"><button data-demo="true">{}</button>{}</div></header><aside class="catalog-sidebar"><h3>EXPLORAR</h3><button data-catalog-mode="all">Todos los juegos</button><button data-catalog-mode="top">Destacados</button><button data-catalog-mode="favorites">Mis favoritos</button><button data-demo="true">Soporte demo</button></aside></div>"##,escape(&n.id),escape(&n.id),node_style(n),hidden,escape(&n.title),escape(&n.text),button(n)).unwrap();
}
fn resort_header(out: &mut String, n: &Component, theme: &Theme, preview: bool) {
    if n.hidden && !preview {
        return;
    }
    let hidden = if n.hidden {
        " data-hidden=\"true\""
    } else {
        ""
    };
    let brand = if n.image.is_some() || n.asset.is_some() {
        artwork(n)
    } else {
        format!(
            r#"<span class="star">✦</span><span><strong>{}</strong><small>CASINO &amp; CLUB</small></span>"#,
            escape(&theme.brand)
        )
    };
    write!(out,r##"<header id="{}" data-component="{}" class="component resort-header" style="{}"{}><a href="#surface" class="resort-brand">{brand}</a><nav><a href="#catalog">{}</a><a href="#catalog">Slots</a><a href="#live">En vivo</a><button data-demo="true">Deportes</button></nav><button data-demo="true" class="wallet"><small>{}</small><strong>{}</strong></button></header>"##,escape(&n.id),escape(&n.id),node_style(n),hidden,escape(&n.title),escape(&n.text),escape(&n.button)).unwrap();
}
fn resort_hero(out: &mut String, n: &Component, preview: bool) {
    if n.hidden && !preview {
        return;
    }
    let hidden = if n.hidden {
        " data-hidden=\"true\""
    } else {
        ""
    };
    let art = if n.image.is_none()
        && n.asset.as_deref() == Some("bced3a505c90b295e30a6c749ba043d65a73e9f5.webp")
    {
        r#"<div class="orbit"></div><div class="orbit second"></div><span class="suit">♠</span><img class="poster one" alt="Book of Dead" src="/media/objects/bced3a505c90b295e30a6c749ba043d65a73e9f5.webp"><img class="poster two" alt="Fortune Tiger" src="/media/objects/5838f407e17892f42a6210429cce32b0a304ef8a.webp"><div class="art-label"><small>EL PLAN DE ESTA NOCHE</small><strong>Jugar. Descubrir. Disfrutar.</strong></div>"#.to_owned()
    } else {
        artwork(n)
    };
    write!(out,r#"<section class="component resort-hero" id="{}" data-component="{}" style="{}"{}><div class="resort-copy"><span>BIENVENIDO A TU NUEVO LUGAR FAVORITO</span><h1>{}</h1><p>{}</p>{}<div class="resort-meta">✦ Una experiencia diferente <span>Modo demo</span></div></div><div class="resort-art">{art}</div></section>"#,escape(&n.id),escape(&n.id),node_style(n),hidden,escape(&n.title),escape(&n.text),button(n)).unwrap();
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
                write!(out,"<label class=\"search-label\">Buscar <input type=\"search\" class=\"catalog-search\" placeholder=\"{}\" data-search=\"true\"></label>",escape(n.style.placeholder.as_deref().unwrap_or("Buscar juegos"))).unwrap();
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
pub fn document(preset: &str, design: &Design, view: View, preview: bool) -> String {
    document_with_appearance(preset, design, view, preview, &Appearance::default())
}
pub fn document_with_appearance(
    preset: &str,
    design: &Design,
    view: View,
    preview: bool,
    appearance: &Appearance,
) -> String {
    if view == View::Panel {
        return panel::document(preset, design, preview, appearance);
    }
    if frames::available(preset, design) {
        return frames::document(preset, design, preview, appearance);
    }
    let t = &design.theme;
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
    write!(out,"<!doctype html><html lang=\"es\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; style-src 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; base-uri 'none'; form-action 'none'\"><title>{}</title><style>{SURFACE_CSS}</style></head><body data-preset=\"{}\" class=\"layout-{} font-{}\" style=\"{css}\">",escape(&t.brand),escape(preset),layout(t.layout),font(settings.font.unwrap_or(t.font))).unwrap();
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
    for n in design.nodes(view) {
        if view == View::Lobby
            && n.kind == Kind::Header
            && crate::presets::catalog_brand(preset).is_some()
        {
            catalog_header(&mut out, n, t, preset, preview);
        } else if view == View::Lobby && preset == "ivory" && n.kind == Kind::Hero {
            resort_hero(&mut out, n, preview);
        } else if view == View::Lobby && preset == "ivory" && n.kind == Kind::Header {
            resort_header(&mut out, n, t, preview);
        } else {
            render_node(&mut out, n, t, preview);
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
        let html = document("ember", &d, View::Lobby, false);
        assert!(!html.contains("<script>"));
        assert!(!html.contains("<img src=x"));
        assert!(html.contains("&lt;script&gt;"));
    }
    #[test]
    fn hidden_not_exported() {
        let mut d = crate::presets::design("ember").unwrap();
        d.lobby[1].hidden = true;
        assert!(!document("ember", &d, View::Lobby, false).contains("id=\"hero\""));
        assert!(document("ember", &d, View::Lobby, true).contains("data-hidden=\"true\""));
    }
    #[test]
    fn all_templates_render_both_views() {
        for p in crate::presets::PRESETS {
            let d = crate::presets::design(p.id).unwrap();
            for view in [View::Lobby, View::Panel] {
                assert!(document(p.id, &d, view, false).contains("id=\"surface\""));
            }
        }
    }
}
