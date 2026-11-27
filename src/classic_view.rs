//! Native layouts for the four original lobby families. No captured scripts run.
use crate::{
    gallery::{action_attr, attrs, font_name, generic, scaled_css, visible},
    model::*,
    render::{accent_ink, escape},
};
use std::fmt::Write as _;

const CSS: &str = include_str!("../frontend/classic-surface.css");
const CHIPS: &str = "87737f5b54ace29f1eb6f8696050ca893d62a2a6.webp";
const TEXTURE: &str = "5c6aacb2b61e491fc4d8ae2370dd6a0249b4df61.webp";
const CARD: &str = "96c5ca1fb373bb8bbb33efdd67d4ed6738c8b2bf.webp";
const BONUS: &str = "0ef2ef2b707c52976a7df5c29ee6b59bb24319e1.webp";

pub fn enabled(preset: &str, design: &Design) -> bool {
    matches!(preset, "ember" | "cobalt" | "flare" | "coal")
        && crate::presets::get(preset).is_some_and(|p| p.layout == design.theme.layout)
}

pub fn defaults(preset: &str, nodes: &mut [Component]) {
    if !matches!(preset, "ember" | "cobalt" | "flare" | "coal") {
        return;
    }
    for n in nodes {
        match n.id.as_str() {
            "header" => {
                n.title = "Inicio".into();
                n.text.clear();
                n.button = if preset == "cobalt" {
                    "Registrate"
                } else {
                    "Acceder"
                }
                .into();
            }
            "hero" => {
                let (title, body, action) = match preset {
                    "ember" => ("ARS 11.052,00", "Tu saldo de muestra", "Cargar fichas"),
                    "cobalt" => ("Bienvenido a", "", "Registrate"),
                    "flare" => (
                        "Hasta 300% y 100 giros gratis",
                        "Bono de bienvenida · muestra histórica",
                        "Ver demo",
                    ),
                    _ => (
                        "ALMIGHTY REELS – Power of Zeus",
                        "Destacado · demostración",
                        "Ver demo",
                    ),
                };
                n.title = title.into();
                n.text = body.into();
                n.button = action.into();
            }
            "categories" => {
                n.title.clear();
                n.text.clear();
                n.children.truncate(3);
                for (child, label) in n.children.iter_mut().zip(["En vivo", "Slots", "Deportes"]) {
                    child.title = label.into();
                    child.button = "Explorar".into();
                }
            }
            "catalog" => {
                n.title = match preset {
                    "flare" => "Juegos populares",
                    "coal" => "Todo el casino",
                    _ => "Recomendados",
                }
                .into();
                n.text.clear();
                n.style.placeholder = Some("Busca juegos o proveedores".into());
            }
            "live" => n.hidden = true,
            "footer" => {
                n.title = "DEMO".into();
                n.text.clear();
            }
            _ => {}
        }
    }
}

fn asset(out: &mut String, id: &str, class: &str, label: &str) {
    if crate::media::valid_asset(id) {
        write!(
            out,
            "<img class=\"{class}\" src=\"/media/objects/{id}\" alt=\"{}\" decoding=\"async\">",
            escape(label)
        )
        .unwrap();
    }
}
fn image(out: &mut String, n: &Component, class: &str) {
    if let Some(src) = &n.image {
        write!(
            out,
            "<img class=\"{class}\" src=\"{}\" alt=\"{}\" decoding=\"async\">",
            escape(src),
            escape(&n.title)
        )
        .unwrap();
    } else if let Some(id) = &n.asset {
        asset(out, id, class, &n.title);
    }
}
fn button(out: &mut String, label: &str, mode: &str, class: &str) {
    let action = if mode == "sports" {
        "data-experience=\"sports\""
    } else {
        action_attr(mode)
    };
    write!(
        out,
        "<button type=\"button\" class=\"{class}\" {action}>{}</button>",
        escape(label)
    )
    .unwrap();
}
fn brand(out: &mut String, n: &Component, theme: &Theme) {
    out.push_str("<span class=\"classic-brand\"><span class=\"brand-art\">");
    if n.asset.is_some() || n.image.is_some() {
        image(out, n, "classic-logo");
    } else {
        out.push_str("<span class=\"classic-mark\" aria-hidden=\"true\">◧</span>");
    }
    write!(
        out,
        "</span><strong>{}</strong></span>",
        escape(&theme.brand)
    )
    .unwrap();
}
fn nav(out: &mut String) {
    for (label, mode) in [
        ("Inicio", "all"),
        ("Slots", "slots"),
        ("En vivo", "live"),
        ("Deportes", "sports"),
        ("Favoritos", "favorites"),
    ] {
        button(out, label, mode, "classic-nav-button");
    }
}
fn header(out: &mut String, n: &Component, theme: &Theme, preset: &str) {
    write!(out,"<header class=\"classic-header\" {}><button type=\"button\" class=\"classic-menu\" data-gallery-menu=\"true\" aria-expanded=\"false\" aria-label=\"Menú\">☰</button>",attrs(n)).unwrap();
    brand(out, n, theme);
    out.push_str("<nav class=\"classic-topnav\" aria-label=\"Principal\">");
    nav(out);
    out.push_str("</nav><div class=\"classic-account\"><span>ARS 11.052,00 · DEMO</span>");
    if preset == "cobalt" {
        button(out, "Iniciar sesión", "demo", "classic-secondary");
    }
    button(out, &n.button, "demo", "classic-primary");
    out.push_str("</div></header>");
}
fn rail(out: &mut String, preset: &str) {
    if !matches!(preset, "ember" | "cobalt" | "flare") {
        return;
    }
    out.push_str("<aside class=\"classic-rail gallery-sidebar\" aria-label=\"Explorar\">");
    for (symbol, label, mode) in [
        ("⌂", "Inicio", "all"),
        ("▦", "Slots", "slots"),
        ("◉", "En vivo", "live"),
        ("♧", "Deportes", "sports"),
        ("♡", "Favoritos", "favorites"),
    ] {
        write!(out,"<button type=\"button\" {}><span aria-hidden=\"true\">{symbol}</span><small>{label}</small></button>",if mode=="sports"{"data-experience=\"sports\""}else{action_attr(mode)}).unwrap();
    }
    out.push_str("</aside>");
}
fn hero(out: &mut String, n: &Component, theme: &Theme, preset: &str) {
    write!(out, "<section class=\"classic-hero\" {}>", attrs(n)).unwrap();
    match preset {
        "ember" => {
            asset(out, TEXTURE, "classic-texture", "");
            image(out, n, "classic-hero-art");
            write!(out,"<div class=\"classic-hero-copy\"><small>{}</small><h1>{}</h1><div class=\"classic-actions\">",escape(&n.text),escape(&n.title)).unwrap();
            button(out, &n.button, "demo", "classic-primary");
            button(out, "Retirar", "demo", "classic-secondary");
            out.push_str("</div></div>");
        }
        "cobalt" => {
            image(out, n, "classic-hero-art");
            write!(
                out,
                "<div class=\"classic-hero-copy\"><h1>{}</h1><strong>{}</strong><p>{}</p>",
                escape(&n.title),
                escape(&theme.brand.to_uppercase()),
                escape(&n.text)
            )
            .unwrap();
            button(out, &n.button, "demo", "classic-primary");
            out.push_str("</div>");
        }
        "flare" => {
            out.push_str("<article class=\"classic-promo\"><div class=\"classic-hero-copy\">");
            write!(
                out,
                "<small>{}</small><h1>{}</h1>",
                escape(&n.text),
                escape(&n.title)
            )
            .unwrap();
            button(out, &n.button, "demo", "classic-primary");
            out.push_str("</div>");
            image(out, n, "classic-hero-art");
            out.push_str("</article><article class=\"classic-promo classic-cashback\"><div class=\"classic-hero-copy\"><small>Oferta especial · DEMO</small><h2>Cashback semanal 15%</h2>");
            button(out, "Ver más", "demo", "classic-secondary");
            out.push_str("</div>");
            asset(out, CHIPS, "classic-hero-art", "");
            out.push_str("</article>");
        }
        _ => {
            out.push_str("<article class=\"classic-carousel\"><small>DESTACADO · DEMO</small><div class=\"classic-carousel-art\">");
            asset(
                out,
                "467ca7712116430e862700170048d4746319f7ad.webp",
                "classic-side-cover",
                "Ruleta Mega",
            );
            image(out, n, "classic-main-cover");
            asset(
                out,
                "4fc121b1a7736714555eefeb74217003c5db60df.webp",
                "classic-side-cover",
                "Age of Olympus",
            );
            write!(
                out,
                "</div><div class=\"classic-carousel-caption\"><div><h1>{}</h1><p>{}</p></div>",
                escape(&n.title),
                escape(&n.text)
            )
            .unwrap();
            button(out, &n.button, "demo", "classic-primary");
            out.push_str("</div></article><div class=\"classic-wallet\"><div class=\"classic-wallet-header\"><strong>Tu cartera</strong><span>Saldo · Bono · Solicitud</span></div><div class=\"classic-wallet-card\"><small>SALDO DE MUESTRA · ARS</small><h2>$ 12.500</h2><div class=\"classic-actions\">");
            button(out, "Pedir depósito", "demo", "classic-primary");
            button(out, "Retirar", "demo", "classic-secondary");
            out.push_str("</div></div><div class=\"classic-wallet-track\"><i></i><small>$ 12.500 · DEMO</small></div></div>");
        }
    }
    out.push_str("</section>");
}
fn categories(out: &mut String, n: &Component, preset: &str, preview: bool) {
    write!(out, "<section class=\"classic-categories\" {}>", attrs(n)).unwrap();
    for (i, child) in n
        .children
        .iter()
        .enumerate()
        .filter(|(_, c)| visible(c, preview))
    {
        let mode = ["live", "slots", "sports"].get(i).copied().unwrap_or("all");
        let action = if mode == "sports" {
            "data-experience=\"sports\""
        } else {
            action_attr(mode)
        };
        write!(
            out,
            "<button class=\"classic-category\" type=\"button\" {action} {}>",
            attrs(child)
        )
        .unwrap();
        if preset == "ember" {
            asset(out, CARD, "classic-category-texture", "");
        }
        image(out, child, "classic-category-art");
        write!(
            out,
            "<span><strong>{}</strong><small>{}</small></span></button>",
            escape(&child.title),
            escape(&child.button)
        )
        .unwrap();
    }
    out.push_str("</section>");
}
fn search(out: &mut String, n: &Component) {
    write!(out,"<label class=\"classic-search\"><span aria-hidden=\"true\">⌕</span><input type=\"search\" data-search=\"true\" aria-label=\"Buscar juegos\" placeholder=\"{}\"></label>",escape(n.style.placeholder.as_deref().unwrap_or("Busca juegos"))).unwrap();
}
fn catalog(out: &mut String, n: &Component, preset: &str) {
    write!(out, "<section class=\"classic-catalog\" {}>", attrs(n)).unwrap();
    if matches!(preset, "ember" | "cobalt") {
        write!(out,"<div class=\"classic-catalog-heading\"><h2>{}</h2><button type=\"button\" data-catalog-mode=\"all\" data-catalog-scroll=\"catalog\">Ver todos ›</button></div><div class=\"classic-featured-layout\"><div class=\"classic-featured\" data-featured-group=\"recommended\"></div>",escape(&n.title)).unwrap();
        if preset == "cobalt" {
            out.push_str("<div class=\"classic-bonus\">");
            asset(out, BONUS, "classic-bonus-art", "");
            out.push_str("<div>Bono bienvenida<br><strong>DEMO</strong></div></div>");
        }
        out.push_str(
            "</div><h2 class=\"classic-all-heading\" data-catalog-title>Todos los juegos</h2>",
        );
    } else {
        write!(out, "<h2 data-catalog-title>{}</h2>", escape(&n.title)).unwrap();
    }
    out.push_str("<div class=\"classic-filters\">");
    for (label, mode) in [
        ("Todos", "all"),
        ("Populares", "top"),
        ("Slots", "slots"),
        ("En vivo", "live"),
        ("Favoritos", "favorites"),
    ] {
        button(out, label, mode, "classic-filter");
    }
    out.push_str("<span data-catalog-count></span></div><div class=\"classic-providers\" data-provider-rail></div><details class=\"classic-extra\"><summary>Más filtros</summary><div data-extra-filters></div></details><div class=\"items classic-games\"></div><p data-catalog-empty hidden>No encontramos juegos.</p><button class=\"classic-primary classic-more\" type=\"button\" data-catalog-more=\"true\">Cargar más</button></section>");
}
pub fn document(design: &Design, preset: &str, preview: bool, appearance: &Appearance) -> String {
    let t = &design.theme;
    let settings = &appearance.settings;
    let defaults = crate::presets::get(preset).expect("known classic preset");
    let font = settings
        .font
        .or_else(|| (t.font != defaults.font).then_some(t.font))
        .map(font_name)
        .unwrap_or("original");
    let bg = if t.background == defaults.background {
        match preset {
            "ember" => "#141414",
            "cobalt" => "#030326",
            "flare" => "#f7f7f8",
            _ => "#100b08",
        }
    } else {
        &t.background
    };
    let surface = if t.surface == defaults.surface {
        match preset {
            "ember" => "#1c1c1c",
            "cobalt" => "#080849",
            "flare" => "#ffffff",
            _ => "#201711",
        }
    } else {
        &t.surface
    };
    let ink = if t.ink == defaults.ink && preset == "flare" {
        "#101014"
    } else {
        &t.ink
    };
    let button_color = t.button_color.as_deref().unwrap_or(&t.accent);
    let backdrop = settings.enabled && appearance.image.is_some();
    let mut out = String::with_capacity(32_000);
    write!(out,"<!doctype html><html lang=\"es\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; style-src 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; base-uri 'none'; form-action 'none'\"><title>{}</title><style>",escape(&t.brand)).unwrap();
    for line in crate::render::SURFACE_CSS
        .lines()
        .take_while(|line| line.starts_with("@font-face"))
    {
        out.push_str(line);
    }
    scaled_css(&mut out, CSS, settings.scale);
    write!(out,"</style></head><body data-profile=\"{preset}\" data-gallery=\"true\" data-appearance-font=\"{font}\" data-global-weight=\"{}\" class=\"font-{font} {}\" style=\"--accent:{};--button:{button_color};--button-ink:{};--page:{bg};--surface:{surface};--ink:{ink};--global-weight:{};--global-radius:{}px;\">",settings.weight.is_some(),if backdrop{"classic-backdrop-enabled"}else{""},t.accent,accent_ink(button_color),settings.weight.unwrap_or(400),settings.radius.unwrap_or(t.radius)).unwrap();
    if backdrop {
        if let Some(image) = &appearance.image {
            write!(out,"<div class=\"page-backdrop\" aria-hidden=\"true\" style=\"--backdrop-fit:{};--backdrop-position:{};--backdrop-opacity:{}\"><img src=\"{}\" alt=\"\"></div>",settings.fit.css(),settings.position.css(),1.-f64::from(settings.shade)/100.,escape(image)).unwrap();
        }
    }
    out.push_str("<div id=\"surface\" class=\"lobby\">");
    if let Some(n) = design
        .lobby
        .first()
        .filter(|n| n.kind == Kind::Header && visible(n, preview))
    {
        header(&mut out, n, t, preset);
        rail(&mut out, preset);
    }
    out.push_str("<main class=\"classic-main\">");
    if preset == "flare" {
        out.push_str("<div class=\"classic-ticker\">● Grandes ganancias · resultados ficticios de demostración</div>");
    }
    if matches!(preset, "ember" | "cobalt") {
        if let Some(n) = design
            .lobby
            .iter()
            .find(|n| n.id == "catalog" && visible(n, preview))
        {
            search(&mut out, n);
        }
    }
    for n in design.lobby.iter().filter(|n| visible(n, preview)) {
        match (n.id.as_str(), n.kind) {
            ("header", Kind::Header) => {}
            ("hero", Kind::Hero) => hero(&mut out, n, t, preset),
            ("categories", Kind::Categories) => categories(&mut out, n, preset, preview),
            ("catalog", Kind::Catalog) => {
                if matches!(preset, "flare" | "coal") {
                    search(&mut out, n);
                }
                catalog(&mut out, n, preset);
            }
            ("footer", Kind::Footer) => {
                write!(
                    out,
                    "<footer class=\"classic-demo\" {}>{} · Sin apuestas reales</footer>",
                    attrs(n),
                    escape(&n.title)
                )
                .unwrap();
            }
            _ => generic(&mut out, n, preview),
        }
    }
    out.push_str("</main><nav class=\"classic-mobile\" aria-label=\"Navegación móvil\">");
    nav(&mut out);
    out.push_str("</nav></div></body></html>");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn classic_layouts_escape_customized_content() {
        for id in ["ember", "cobalt", "flare", "coal"] {
            let mut d = crate::presets::design(id).unwrap();
            d.lobby[1].title = "<script>test</script>".into();
            let html = document(&d, id, false, &Appearance::default());
            assert!(html.contains("id=\"surface\""));
            assert!(html.contains("id=\"catalog\""));
            assert!(html.contains("&lt;script&gt;"));
            assert!(!html.contains("<script>"));
        }
    }
}
