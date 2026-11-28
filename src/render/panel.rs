use super::{SURFACE_CSS, artwork, escape, node_style, render_node};
use crate::model::{Appearance, Component, Design, Font, Kind};
use std::{collections::BTreeMap, fmt::Write, sync::OnceLock};

const CSS: &str = include_str!("../../frontend/panel/panel.css");
const BINDINGS: &str = include_str!("../../frontend/panel/panel-parts.json");

fn parts() -> &'static BTreeMap<String, String> {
    static PARTS: OnceLock<BTreeMap<String, String>> = OnceLock::new();
    PARTS.get_or_init(|| serde_json::from_str(BINDINGS).expect("validated panel fragments"))
}

fn property_style(n: &Component) -> String {
    let mut s = node_style(n);
    if let Some(v) = &n.style.color {
        write!(s, "color:{v};").unwrap();
    }
    if let Some(v) = &n.style.background {
        write!(s, "background:{v};").unwrap();
    }
    if let Some(v) = n.style.radius {
        write!(s, "border-radius:{v}px;").unwrap();
    }
    if let Some(v) = n.style.padding {
        write!(s, "padding:{v}px;").unwrap();
    }
    if let Some(v) = n.style.font_size {
        write!(s, "font-size:{v}px;").unwrap();
    }
    if let Some(v) = n.style.font_weight {
        write!(s, "font-weight:{v};").unwrap();
    }
    s
}

// Only placeholders in the compiled template are interpreted. User strings are
// escaped once and never scanned again for template syntax.
fn component(out: &mut String, key: &str, n: &Component, d: &Design, eyebrow: &str, preview: bool) {
    if n.hidden && !preview {
        return;
    }
    let Some(template) = parts().get(key) else {
        out.push_str("<div class=\"panel-extra\">");
        render_node(out, n, &d.theme, preview);
        out.push_str("</div>");
        return;
    };
    let mut rest = template.as_str();
    while let Some((before, tail)) = rest.split_once("{{") {
        out.push_str(before);
        let (field, after) = tail.split_once("}}").expect("closed panel placeholder");
        match field {
            "attrs" => {
                let id = escape(&n.id);
                let suffix = if key == "mobile_header" {
                    "-mobile"
                } else {
                    ""
                };
                write!(out, "id=\"{id}{suffix}\" data-component=\"{id}\"").unwrap();
                if n.hidden {
                    out.push_str(" data-hidden=\"true\"")
                }
            }
            "style" => out.push_str(&property_style(n)),
            "title" => out.push_str(&escape(&n.title)),
            "text" => out.push_str(&escape(&n.text)),
            "button" => out.push_str(&escape(&n.button)),
            "eyebrow" => out.push_str(&escape(eyebrow)),
            "logo" => {
                if n.image.is_some() || n.asset.is_some() {
                    out.push_str(&artwork(n).replacen(
                        "class=\"art-image\"",
                        "class=\"logo\" width=\"327\" height=\"74\"",
                        1,
                    ));
                } else {
                    out.push_str(&escape(&d.theme.brand));
                }
            }
            "amount" => {
                out.push_str("<span class=\"plata\"><span class=\"plata-txt\">");
                if let Some((amount, cents)) = n.text.rsplit_once(',') {
                    write!(out, "{}<small>,{}</small>", escape(amount), escape(cents)).unwrap();
                } else {
                    out.push_str(&escape(&n.text));
                }
                out.push_str("</span></span>");
            }
            "extra" => {
                if n.asset.is_some() || n.image.is_some() {
                    out.push_str(&artwork(n));
                }
                if (key.starts_with("metric-") || key.starts_with("activity-"))
                    && !n.text.is_empty()
                {
                    write!(out, "<p>{}</p>", escape(&n.text)).unwrap();
                }
            }
            "children" => {
                for child in &n.children {
                    // Only tiles of this group may use its built-in presentation.
                    let prefix = match key {
                        "metrics" => "metric-",
                        "totals" => "total-",
                        "activity" => "activity-",
                        _ => "",
                    };
                    let child_key = if !prefix.is_empty() && child.id.starts_with(prefix) {
                        child.id.as_str()
                    } else {
                        ""
                    };
                    component(out, child_key, child, d, eyebrow, preview);
                }
            }
            _ => unreachable!("unknown panel placeholder"),
        }
        rest = after;
    }
    out.push_str(rest);
}

pub(super) fn document(
    preset: &str,
    design: &Design,
    preview: bool,
    appearance: &Appearance,
) -> String {
    let theme = &design.theme;
    let defaults = crate::presets::get(preset).expect("validated preset");
    let mut vars = String::new();
    let accent = &theme.accent;
    if accent != defaults.accent {
        let rgb = [1, 3, 5].map(|i| u8::from_str_radix(&accent[i..i + 2], 16).unwrap_or_default());
        write!(
            vars,
            "--dm-p:{accent};--dm-p-v4:{accent};--dm-p-rgb:{},{},{};",
            rgb[0], rgb[1], rgb[2]
        )
        .unwrap();
    }
    if let Some(color) = &theme.button_color {
        write!(vars, "--dm-b:{color};--dm-b-grad:{color};").unwrap();
    }
    if theme.background != defaults.background {
        write!(
            vars,
            "--dm-f:{};--dm-calido:{};",
            theme.background, theme.background
        )
        .unwrap();
    }
    if theme.surface != defaults.surface {
        write!(vars, "--dm-v4-sup:{};", theme.surface).unwrap();
    }
    if theme.ink != defaults.ink {
        write!(vars, "--dm-tx:{};", theme.ink).unwrap();
    }
    let face = appearance
        .settings
        .font
        .or((theme.font != defaults.font).then_some(theme.font));
    if let Some(face) = face {
        let family = match face {
            Font::Barlow => "Barlow",
            Font::Archivo => "Archivo",
            Font::Oswald | Font::Condensed => "Oswald",
            Font::Editorial => "Georgia",
            Font::System => "system-ui",
        };
        write!(vars, "--font-figtree:{family};").unwrap();
    }
    let mut out = String::with_capacity(64000);
    write!(out,r#"<!doctype html><html lang="es" class="h-full antialiased" style="{vars}"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1,viewport-fit=cover"><meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; base-uri 'none'; form-action 'none'"><title>{}</title><style>{CSS}"#,escape(&theme.brand)).unwrap();
    for line in SURFACE_CSS
        .lines()
        .take_while(|l| l.starts_with("@font-face"))
    {
        out.push_str(line);
    }
    write!(out, "@scope (.panel-extra){{{SURFACE_CSS}}}").unwrap();
    if appearance.settings.enabled {
        if let Some(img) = &appearance.image {
            write!(out,".cash-panel .marco{{background-image:linear-gradient(#0000,#0000),url('{}');background-position:{};background-size:{};}}",escape(img),appearance.settings.position.css(),appearance.settings.fit.css()).unwrap();
        }
    }
    write!(out,r#"</style></head><body class="min-h-full flex flex-col" data-preset="{}"><div class="panel cash-panel fuente-demo-figtree" id="surface" data-estilo="1" aria-label="Panel de administración"><div class="marco">"#,escape(preset)).unwrap();
    let base_brand = crate::presets::catalog_brand(preset).unwrap_or(if preset == "ivory" {
        "Tu casino"
    } else {
        "Mi casino"
    });
    let eyebrow = if theme.brand == base_brand {
        "RUSHYBET · PANEL DE CAJA".into()
    } else {
        format!("{} · PANEL DE CAJA", theme.brand)
    };
    let header = design
        .panel
        .iter()
        .find(|n| n.id == "header" && n.kind == Kind::Header);
    if let Some(n) = header {
        component(&mut out, "header", n, design, &eyebrow, preview);
        component(&mut out, "mobile_header", n, design, &eyebrow, preview);
    }
    out.push_str("<main class=\"main\">");
    for n in &design.panel {
        if header.is_some_and(|h| h.id == n.id) {
            continue;
        }
        let known = matches!(
            (n.id.as_str(), n.kind),
            ("panel-title", Kind::Text)
                | ("metrics" | "totals", Kind::Metrics)
                | ("activity", Kind::Activity)
                | ("footer", Kind::Footer)
        );
        component(
            &mut out,
            if known { &n.id } else { "" },
            n,
            design,
            &eyebrow,
            preview,
        );
    }
    out.push_str("</main>");
    if header.is_some_and(|n| !n.hidden || preview) {
        out.push_str(&parts()["mobile_nav"]);
    }
    out.push_str(
        "</div></div><div class=\"panel-demo-ribbon\">DEMO — nada es real</div></body></html>",
    );
    out
}
