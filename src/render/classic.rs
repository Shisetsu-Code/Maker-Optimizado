use super::{artwork, escape, node_style};
use crate::model::{Appearance, Component, Design, Font, Kind};
use serde::Deserialize;
use std::{collections::BTreeMap, fmt::Write, sync::OnceLock};

#[derive(Deserialize)]
struct Variant {
    header: String,
    hero: String,
    shelf: String,
    footer: String,
    css: String,
    values: BTreeMap<String, String>,
}
#[derive(Deserialize)]
pub(super) struct Classic {
    desktop: Variant,
    mobile: Variant,
}
pub(super) fn get(preset: &str, design: &Design) -> Option<&'static Classic> {
    static DATA: OnceLock<BTreeMap<String, Classic>> = OnceLock::new();
    if !matches!(preset, "ember" | "cobalt" | "flare" | "coal")
        || crate::presets::get(preset)?.layout != design.theme.layout
    {
        return None;
    }
    DATA.get_or_init(|| {
        serde_json::from_str(include_str!("../../frontend/classic/layouts.json"))
            .expect("compiled classic presentations")
    })
    .get(preset)
}
impl Classic {
    pub(super) fn style(
        &self,
        out: &mut String,
        d: &Design,
        defaults: &Design,
        appearance: &Appearance,
    ) {
        out.push_str(include_str!("../../frontend/classic/fonts.css"));
        write!(
            out,
            "@media(min-width:769px){{{}}}@media(max-width:768px){{{}}}{}",
            self.desktop.css,
            self.mobile.css,
            include_str!("../../frontend/classic/presentation.css")
        )
        .unwrap();

        let theme = &d.theme;
        out.push_str(":root{");
        for (value, base, hex, rgb) in [
            (
                &theme.accent,
                &defaults.theme.accent,
                ["--dm-p", "--bb-primary", "--br-p"],
                ["--dm-p-rgb", "--rc-marca", "--br-p-rgb"],
            ),
            (
                &theme.background,
                &defaults.theme.background,
                ["--dm-f", "--bb-bg", "--br-fondo"],
                ["--dm-f-rgb", "--rc-fondo", "--br-fondo-rgb"],
            ),
            (
                &theme.surface,
                &defaults.theme.surface,
                ["--dm-t", "--bb-sidebar", "--br-hondo"],
                ["--dm-t-rgb", "--rc-tarjeta", "--br-hondo-rgb"],
            ),
        ] {
            if value != base {
                for name in hex {
                    write!(out, "{name}:{value};").unwrap();
                }
                if let Some(bytes) = value
                    .strip_prefix('#')
                    .filter(|s| s.len() == 6 && s.is_ascii())
                {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        u8::from_str_radix(&bytes[..2], 16),
                        u8::from_str_radix(&bytes[2..4], 16),
                        u8::from_str_radix(&bytes[4..], 16),
                    ) {
                        for name in rgb {
                            write!(out, "{name}:{r},{g},{b};").unwrap();
                        }
                    }
                }
            }
        }
        if let Some(button) = &theme.button_color {
            write!(out, "--dm-b:{button};--dm-b-grad:{button};").unwrap();
        }
        if theme.ink != defaults.theme.ink {
            write!(out, "--bb-text:{};--br-texto:{};", theme.ink, theme.ink).unwrap();
        }
        out.push('}');
        if let Some(font) = appearance
            .settings
            .font
            .or((theme.font != defaults.theme.font).then_some(theme.font))
        {
            let family = match font {
                Font::Archivo => "Archivo",
                Font::Barlow => "Barlow",
                Font::Oswald | Font::Condensed => "Oswald",
                Font::Editorial => "Georgia",
                Font::System => "system-ui",
            };
            write!(out,"[data-classic-section],[data-classic-section] :is(div,span,button,h1,h2,p){{font-family:{family},sans-serif!important}}").unwrap();
        }
        if appearance.settings.scale != 100 {
            write!(
                out,
                "[data-classic-section]{{zoom:{}%}}",
                appearance.settings.scale
            )
            .unwrap();
        }
    }
    pub(super) fn section(
        &self,
        out: &mut String,
        node: &Component,
        design: &Design,
        _preset: &str,
        defaults: &Design,
        preview: bool,
    ) -> bool {
        if node.id == "categories"
            && node.kind == Kind::Categories
            && defaults.lobby.iter().any(|n| n == node)
        {
            if !node.hidden || preview {
                out.push_str(
                    r#"<section id="categories" data-component="categories" data-classic-shelf>"#,
                );
                for (mode, variant) in [("desktop", &self.desktop), ("mobile", &self.mobile)] {
                    write!(out, "<div class=\"classic-{mode}\">{}</div>", variant.shelf).unwrap();
                }
                out.push_str("</section>");
            }
            return true;
        }
        let header = node.id == "header" && node.kind == Kind::Header;
        let footer = node.id == "footer" && node.kind == Kind::Footer;
        if !header && !footer && !(node.id == "hero" && node.kind == Kind::Hero) {
            return false;
        }
        if node.hidden && !preview {
            return true;
        }
        let Some(base) = defaults.lobby.iter().find(|n| n.id == node.id) else {
            return false;
        };
        write!(
            out,
            "<div id=\"{}\" data-component=\"{}\" data-classic-section style=\"{}\" {}>",
            escape(&node.id),
            escape(&node.id),
            node_style(node),
            if node.hidden {
                "data-hidden=\"true\""
            } else {
                ""
            }
        )
        .unwrap();
        for (mode, v) in [("desktop", &self.desktop), ("mobile", &self.mobile)] {
            write!(out, "<div class=\"classic-{mode}\">").unwrap();
            let mut rest = if header {
                v.header.as_str()
            } else if footer {
                v.footer.as_str()
            } else {
                v.hero.as_str()
            };
            while let Some((before, tail)) = rest.split_once("{{") {
                out.push_str(before);
                let (key, after) = tail.split_once("}}").expect("closed classic field");
                let original = v.values.get(key).map(String::as_str).unwrap_or("");
                let (_, field) = key.split_once('_').unwrap_or(("", key));
                match (key, field) {
                    ("extra_image", _) if node.image.is_some() || node.asset != base.asset => {
                        out.push_str(&artwork(node))
                    }
                    ("brand_text", _) if design.theme.brand != defaults.theme.brand => {
                        out.push_str(&escape(&design.theme.brand))
                    }
                    ("brand_text" | "extra_image", _) => {}
                    (_, "src") if node.image.is_some() => {
                        out.push_str(&escape(node.image.as_deref().unwrap_or("")))
                    }
                    (_, "src") if node.asset != base.asset => {
                        if let Some(asset) = &node.asset {
                            write!(out, "/media/objects/{}", escape(asset)).unwrap();
                        }
                    }
                    (_, "title") if node.title != base.title => write!(
                        out,
                        "<span role=\"heading\" aria-level=\"1\">{}</span>",
                        escape(&node.title)
                    )
                    .unwrap(),
                    (_, "text") if node.text != base.text => out.push_str(&escape(&node.text)),
                    (_, "button") if node.button != base.button => {
                        out.push_str(&escape(&node.button))
                    }
                    _ => out.push_str(&escape(original)),
                }
                rest = after;
            }
            out.push_str(rest);
            out.push_str("</div>");
        }
        out.push_str("</div>");
        true
    }
}
