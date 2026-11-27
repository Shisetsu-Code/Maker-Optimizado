//! Shared administrative presentation; all displayed measurements are demo data.
use crate::{
    gallery::{attrs, font_name, generic, scaled_css, visible},
    model::*,
    render::escape,
};
use std::fmt::Write as _;

const CSS: &str = include_str!("../frontend/panel-surface.css");
const BARS: [u8; 60] = [
    4, 5, 6, 5, 5, 5, 4, 4, 2, 1, 1, 1, 2, 1, 2, 4, 5, 6, 5, 6, 6, 7, 7, 5, 5, 5, 6, 7, 6, 7, 8, 9,
    10, 9, 9, 9, 8, 7, 5, 5, 4, 5, 5, 4, 5, 5, 6, 6, 4, 4, 4, 3, 3, 5, 6, 7, 8, 9, 8, 8,
];

pub fn defaults(nodes: &mut [Component]) {
    for n in nodes {
        match n.id.as_str() {
            "header" => {
                n.title = "dueño".into();
                n.text = "DUEÑO".into();
                n.button = "Cargar / Descargar".into();
            }
            "panel-title" => {
                n.title = "Inicio".into();
                n.text = "Lo que pasó hoy y lo que te falta hacer.".into();
            }
            "metrics" => {
                n.title = "Monitoreo".into();
                n.text = "actualizado hace 0 s".into();
                for (child, (title, value, detail)) in n.children.iter_mut().zip([
                    ("Jugando ahora", "4", "apostaron en los últimos 5 min"),
                    ("Jugaron hoy", "4", "desde la medianoche"),
                    (
                        "Apuestas por minuto · última hora",
                        "323 apuestas en la última hora",
                        "pico 10/min",
                    ),
                    ("Servidor", "Todo bien", "Encendido hace 6 días"),
                ]) {
                    child.title = title.into();
                    child.text = value.into();
                    child.button = detail.into();
                }
            }
            "activity" => {
                n.title = "Lo que sigue".into();
                n.text.clear();
                for (child, (title, value, detail)) in n.children.iter_mut().zip([
                    ("CARGASTE HOY", "ARS 30.000,00", ""),
                    ("DESCARGASTE HOY", "ARS 0,00", ""),
                    (
                        "TE TENDRÍA QUE QUEDAR",
                        "ARS 30.000,00",
                        "lo que entró menos lo que pagaste, hoy",
                    ),
                ]) {
                    child.title = title.into();
                    child.text = value.into();
                    child.button = detail.into();
                }
            }
            "footer" => {
                n.title = "DEMO — nada es real".into();
                n.text.clear();
            }
            _ => {}
        }
    }
}

fn icon(name: &str) -> &'static str {
    match name {
        "overview" => r#"<path d="m3 10 9-8 9 8M5 9v12h14V9"/>"#,
        "support" => {
            r#"<path d="M21 11a9 9 0 0 1-9 9 9 9 0 0 1-4-.9L3 21l1.9-5A9 9 0 1 1 21 11Z"/>"#
        }
        "partners" => {
            r#"<rect x="3" y="3" width="18" height="18" rx="2"/><path d="M12 3v18M3 12h18"/>"#
        }
        "team" | "players" => {
            r#"<circle cx="12" cy="7" r="3"/><path d="M4 21v-2a8 8 0 0 1 16 0v2"/>"#
        }
        "requests" => r#"<circle cx="12" cy="12" r="9"/><path d="M12 7v5l4 3"/>"#,
        "records" => r#"<path d="M3 7h17l-4-4m4 4-4 4M21 17H4l4 4m-4-4 4-4"/>"#,
        "statistics" => r#"<path d="M5 20V9M12 20V4M19 20v-7"/>"#,
        "alerts" => r#"<path d="m12 3 10 18H2L12 3Zm0 6v5m0 3v1"/>"#,
        "prizes" => r#"<path d="m12 2 3 6 7 1-5 5 1 7-6-3-6 3 1-7-5-5 7-1 3-6Z"/>"#,
        "flyers" => {
            r#"<rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8" cy="8" r="1.5"/><path d="m3 17 6-6 4 4 3-3 5 5"/>"#
        }
        "settings" => {
            r#"<path d="M3 6h18M3 12h18M3 18h18"/><circle cx="8" cy="6" r="2" fill="var(--panel-surface)"/><circle cx="16" cy="12" r="2" fill="var(--panel-surface)"/><circle cx="11" cy="18" r="2" fill="var(--panel-surface)"/>"#
        }
        "money" => r#"<path d="M12 3v18M8 7l4-4 4 4M8 17l4 4 4-4"/>"#,
        "up" => r#"<path d="M12 20V4m-6 6 6-6 6 6"/>"#,
        "down" => r#"<path d="M12 4v16m-6-6 6 6 6-6"/>"#,
        "wallet" => r#"<rect x="3" y="5" width="18" height="14" rx="2"/><path d="M16 12h1"/>"#,
        "lock" => {
            r#"<rect x="5" y="10" width="14" height="11" rx="2"/><path d="M8 10V7a4 4 0 0 1 8 0v3"/>"#
        }
        "exit" => r#"<path d="M10 3H4v18h6M9 12h13m-5-5 5 5-5 5"/>"#,
        "tool" => r#"<path d="m14 6 4 4 3-3a7 7 0 0 1-9 9l-6 6-4-4 6-6a7 7 0 0 1 9-9l-3 3Z"/>"#,
        _ => r#"<path d="M4 6h16M4 12h16M4 18h16"/>"#,
    }
}
fn glyph(name: &str) -> String {
    format!(
        "<svg aria-hidden=\"true\" viewBox=\"0 0 24 24\" width=\"17\" height=\"17\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.6\" stroke-linecap=\"round\" stroke-linejoin=\"round\">{}</svg>",
        icon(name)
    )
}
fn mark() -> &'static str {
    r#"<svg viewBox="0 0 24 21" width="20" height="17" aria-hidden="true" fill="currentColor"><path d="M10 0h14v12h-5l5 9h-9l-5-8V0ZM0 10h8v11H0z"/></svg>"#
}

pub fn navigation(section: crate::demo_panel::Section) -> String {
    use crate::demo_panel::Section as S;
    let mut out =
        String::from("<nav class=\"panel-navigation\" aria-label=\"Secciones del panel\">");
    for (s, label, badge) in [
        (S::Overview, "Inicio", ""),
        (S::Support, "Soporte", "6"),
        (S::Partners, "Paneles", ""),
        (S::Team, "Equipo", ""),
        (S::Players, "Jugadores", ""),
        (S::Requests, "Pendientes", "6"),
        (S::Records, "Movimientos", ""),
        (S::Statistics, "Plata", ""),
        (S::Alerts, "Alertas", "2"),
        (S::Prizes, "Premios", "3"),
        (S::Flyers, "Flyers", ""),
        (S::Settings, "Ajustes", "2"),
    ] {
        let sub = matches!(s, S::Team | S::Players);
        let expanded = matches!(section, S::Partners | S::Team | S::Players);
        let hidden = if sub && !expanded { " hidden" } else { "" };
        write!(out,"<button type=\"button\" class=\"panel-link{}\" data-panel-section=\"{}\" aria-pressed=\"{}\"{hidden}><span class=\"panel-icon\">{}</span><span class=\"panel-link-label\">{label}</span>",if sub{" panel-sub"}else{""},s.id(),section==s,glyph(s.id())).unwrap();
        if !badge.is_empty() {
            write!(out, "<span class=\"panel-badge\">{badge}</span>").unwrap();
        }
        if matches!(s, S::Partners | S::Records) {
            out.push_str("<span class=\"panel-chevron\" aria-hidden=\"true\">⌄</span>");
        }
        out.push_str("</button>");
    }
    out.push_str("</nav>");
    out
}
fn mobile_nav(out: &mut String) {
    out.push_str("<nav class=\"panel-mobile\" aria-label=\"Navegación móvil\">");
    for (name, label, badge) in [
        ("overview", "Inicio", ""),
        ("partners", "Paneles", ""),
        ("money", "Cargar / Descargar", ""),
        ("requests", "Pendientes", "6"),
        ("menu", "Más", "13"),
    ] {
        let attr = match name {
            "money" => "data-demo=\"true\"".to_owned(),
            "menu" => "data-panel-menu=\"true\" aria-expanded=\"false\"".to_owned(),
            _ => format!(
                "data-panel-section=\"{name}\" aria-pressed=\"{}\"",
                name == "overview"
            ),
        };
        write!(out,"<button type=\"button\" class=\"panel-mobile-{name}\" {attr} aria-label=\"{label}\"><span class=\"panel-mobile-icon\">{}",glyph(name)).unwrap();
        if !badge.is_empty() {
            write!(out, "<span class=\"panel-badge\">{badge}</span>").unwrap();
        }
        write!(
            out,
            "</span><span class=\"panel-mobile-label\">{label}</span></button>"
        )
        .unwrap();
    }
    out.push_str("</nav>");
}
fn header(out: &mut String, n: &Component, theme: &Theme) {
    write!(out, "<header {} class=\"panel-identity\"><div class=\"panel-brand\"><span class=\"panel-brand-icon\">{}</span><span class=\"panel-brand-copy\">",attrs(n),mark()).unwrap();
    if let Some(src) = n
        .image
        .as_deref()
        .map(str::to_owned)
        .or_else(|| n.asset.as_ref().map(|id| format!("/media/objects/{id}")))
    {
        write!(
            out,
            "<img src=\"{}\" alt=\"{}\">",
            escape(&src),
            escape(&theme.brand)
        )
        .unwrap();
    } else {
        write!(
            out,
            "<span class=\"panel-brand-name\">{}<b>{}</b></span>",
            mark(),
            escape(&theme.brand)
        )
        .unwrap();
    }
    write!(out,"<span class=\"panel-role\">{} <strong>{}</strong></span></span></div><div class=\"panel-float\" aria-label=\"Saldo ficticio\"><i></i><small>FLOAT</small><b>ARS 1.407.365,00</b></div>",escape(&n.title),escape(&n.text)).unwrap();
    if !n.button.is_empty() {
        write!(
            out,
            "<button class=\"panel-cash-action\" type=\"button\" data-demo=\"true\">{} {}</button>",
            glyph("money"),
            escape(&n.button)
        )
        .unwrap();
    }
    out.push_str("</header>");
}
fn heading(out: &mut String, n: &Component, theme: &Theme) {
    write!(out,"<section class=\"panel-screen-heading\" {} data-panel-overview><div class=\"panel-kicker\"><i></i><span>{} · PANEL DE CAJA</span></div><h1>{}</h1><p>{}</p></section>",attrs(n),escape(&theme.brand.to_uppercase()),escape(&n.title),escape(&n.text)).unwrap();
}
fn chart(out: &mut String, n: &Component) {
    write!(out,"<div class=\"panel-chart\" {}><div class=\"panel-chart-heading\"><span>{}</span><small>{}</small></div><svg viewBox=\"0 0 600 48\" preserveAspectRatio=\"none\" role=\"img\" aria-label=\"Serie ficticia de actividad por minuto\" class=\"panel-chart-graph\" width=\"100%\" height=\"48\">",attrs(n),escape(&n.title),escape(&n.button)).unwrap();
    for (index, v) in BARS.iter().enumerate() {
        let h = f64::from(*v) * 4.6;
        let class = if index == 32 {
            " peak"
        } else if index == 59 {
            " current"
        } else {
            ""
        };
        write!(out,"<rect class=\"panel-chart-bar{class}\" x=\"{}\" y=\"{}\" width=\"7\" height=\"{h}\" rx=\"1\"/>",index as f64*10.+1.5,48.-h).unwrap();
    }
    write!(out,"</svg><div class=\"panel-chart-axis\"><span>23:07</span><span>ahora</span></div><p class=\"panel-chart-total\">{}</p></div>",escape(&n.text)).unwrap();
}
fn monitoring(out: &mut String, n: &Component, preview: bool) {
    write!(out,"<section class=\"panel-monitor panel-box\" {} data-panel-overview><div class=\"panel-box-heading\"><span>{}</span><small><i></i>{}</small></div><div class=\"panel-monitor-grid\"><section class=\"panel-monitor-cell\"><h2>Jugadores</h2><div class=\"panel-figures\">",attrs(n),escape(&n.title),escape(&n.text)).unwrap();
    for (i, child) in n
        .children
        .iter()
        .filter(|c| visible(c, preview))
        .filter(|c| matches!(c.id.as_str(), "metric-0" | "metric-1"))
        .enumerate()
    {
        write!(
            out,
            "<div class=\"panel-figure{}\" {}><strong>{}</strong><b>{}</b><small>{}</small></div>",
            if i == 0 { " green" } else { "" },
            attrs(child),
            escape(&child.text),
            escape(&child.title),
            escape(&child.button)
        )
        .unwrap();
    }
    out.push_str("</div>");
    if let Some(c) = n
        .children
        .iter()
        .find(|c| c.id == "metric-2" && visible(c, preview))
    {
        chart(out, c);
    }
    out.push_str("</section>");
    if let Some(c) = n
        .children
        .iter()
        .find(|c| c.id == "metric-3" && visible(c, preview))
    {
        write!(out,"<section class=\"panel-monitor-cell\" {}><h2>{}</h2><div class=\"panel-server-status\"><i></i>{}</div><div class=\"panel-server-grid\">",attrs(c),escape(&c.title),escape(&c.text)).unwrap();
        for (label, value, short, percent, detail) in [
            ("Procesador", "18%", "18%", 18.0, "tranquilo"),
            (
                "Memoria",
                "42%",
                "42%",
                42.0,
                "3,3 de 8,0 GB · memoria de reserva 0%",
            ),
            ("Disco", "38%", "38%", 38.0, "99 GB libres de 160"),
            (
                "Base de datos",
                "3 ms",
                "al instante",
                0.3,
                "responde al instante",
            ),
        ] {
            write!(out,"<div class=\"panel-server-measure\"><div class=\"panel-server-line\"><b>{label}</b><span class=\"panel-server-desktop\">{value}</span><span class=\"panel-server-mobile\">{short}</span></div><div class=\"panel-server-track\"><i style=\"width:{percent}%\"></i></div><small>{detail}</small></div>").unwrap();
        }
        write!(
            out,
            "</div><p class=\"panel-server-foot\">{}</p></section>",
            escape(&c.button)
        )
        .unwrap();
    }
    out.push_str("</div>");
    for c in n.children.iter().filter(|c| {
        visible(c, preview)
            && !matches!(
                c.id.as_str(),
                "metric-0" | "metric-1" | "metric-2" | "metric-3"
            )
    }) {
        generic(out, c, preview);
    }
    out.push_str("</section>");
}
fn cash(out: &mut String, n: &Component, preview: bool) {
    out.push_str("<div class=\"panel-money-grid\" data-panel-overview>");
    for c in n.children.iter().filter(|c| visible(c, preview)) {
        let (class, ic) = match c.id.as_str() {
            "activity-0" => ("green", "up"),
            "activity-1" => ("amber", "down"),
            _ => ("highlight", "wallet"),
        };
        write!(out,"<article class=\"panel-money {class}\" {}><div class=\"panel-money-head\"><span>{}</span><i>{}</i></div><div class=\"panel-money-value\">",attrs(c),escape(&c.title),glyph(ic)).unwrap();
        if let Some((whole, decimal)) = c.text.rsplit_once(',') {
            write!(out, "{}<small>,{}</small>", escape(whole), escape(decimal)).unwrap();
        } else {
            out.push_str(&escape(&c.text));
        }
        out.push_str("</div>");
        if !c.button.is_empty() {
            write!(out, "<p>{}</p>", escape(&c.button)).unwrap();
        }
        out.push_str("</article>");
    }
    out.push_str("</div>");
}

pub fn document(
    design: &Design,
    preset: Option<&str>,
    preview: bool,
    appearance: &Appearance,
) -> String {
    let t = &design.theme;
    let settings = &appearance.settings;
    let baseline = preset.and_then(crate::presets::get);
    let accent = if baseline.is_none_or(|p| p.accent == t.accent) {
        "#ff5a00"
    } else {
        &t.accent
    };
    let page = if baseline.is_none_or(|p| p.background == t.background) {
        "#0f0f0f"
    } else {
        &t.background
    };
    let surface = if baseline.is_none_or(|p| p.surface == t.surface) {
        "rgba(30,30,30,.92)"
    } else {
        &t.surface
    };
    let font = settings
        .font
        .or_else(|| baseline.filter(|p| p.font != t.font).map(|_| t.font))
        .map(font_name)
        .unwrap_or("original");
    let mut out = String::with_capacity(30000);
    write!(out,"<!doctype html><html lang=\"es\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; style-src 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; base-uri 'none'; form-action 'none'\"><title>{}</title><style>",escape(&t.brand)).unwrap();
    for line in crate::render::SURFACE_CSS
        .lines()
        .take_while(|line| line.starts_with("@font-face"))
    {
        out.push_str(line);
    }
    scaled_css(&mut out, CSS, settings.scale);
    write!(out,"</style></head><body class=\"panel-document font-{font}\" data-appearance-font=\"{font}\" data-global-weight=\"{}\" style=\"--panel-accent:{accent};--panel-page:{page};--panel-surface:{surface};--text-scale:{};--global-weight:{};--panel-radius:{}px;\">",settings.weight.is_some(),f64::from(settings.scale)/100.,settings.weight.unwrap_or(400),settings.radius.unwrap_or(18)).unwrap();
    if settings.enabled {
        if let Some(image) = &appearance.image {
            write!(out,"<div class=\"page-backdrop\" aria-hidden=\"true\" style=\"--backdrop-fit:{};--backdrop-position:{};--backdrop-opacity:{}\"><img alt=\"\" src=\"{}\"></div>",settings.fit.css(),settings.position.css(),1.-f64::from(settings.shade)/100.,escape(image)).unwrap();
        }
    }
    out.push_str("<div id=\"surface\" class=\"panel\" aria-label=\"Panel de administración\"><aside class=\"panel-aside\">");
    if let Some(n) = design
        .panel
        .iter()
        .find(|n| n.id == "header" && visible(n, preview))
    {
        header(&mut out, n, t);
    }
    out.push_str(&navigation(crate::demo_panel::Section::Overview));
    out.push_str("<div class=\"panel-aside-foot\">");
    for (ic, label) in [
        ("tool", "Cerrar el sitio"),
        ("lock", "Mi seguridad"),
        ("exit", "Cerrar sesión"),
    ] {
        write!(out,"<button type=\"button\" class=\"panel-link panel-foot-{ic}\" data-demo=\"true\"><span class=\"panel-icon\">{}</span><span class=\"panel-link-label\">{label}</span></button>",glyph(ic)).unwrap();
    }
    out.push_str("</div></aside><main class=\"panel-main\">");
    let mut workspace = false;
    for n in design.panel.iter().filter(|n| visible(n, preview)) {
        match n.id.as_str() {
            "header" | "footer" => {}
            "panel-title" => heading(&mut out, n, t),
            "metrics" => monitoring(&mut out, n, preview),
            "activity" => {
                write!(out, "<section class=\"panel-activity\" {}>", attrs(n)).unwrap();
                cash(&mut out, n, preview);
                out.push_str("<div id=\"panel-workspace\">");
                out.push_str(&crate::demo_panel::Panel::default().content());
                out.push_str("</div></section>");
                workspace = true;
            }
            _ => generic(&mut out, n, preview),
        }
    }
    if !workspace {
        out.push_str("<div id=\"panel-workspace\">");
        out.push_str(&crate::demo_panel::Panel::default().content());
        out.push_str("</div>");
    }
    out.push_str("</main>");
    mobile_nav(&mut out);
    if let Some(n) = design
        .panel
        .iter()
        .find(|n| n.id == "footer" && visible(n, preview))
    {
        write!(
            out,
            "<footer class=\"panel-demo-badge\" {}>{}</footer>",
            attrs(n),
            escape(&n.title)
        )
        .unwrap();
    }
    out.push_str("</div></body></html>");
    out
}
