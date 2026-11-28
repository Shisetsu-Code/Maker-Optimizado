use super::{escape, node_style, render_node};
use crate::model::{Component, Design, Kind};
use serde::Deserialize;
use std::{collections::BTreeMap, fmt::Write, sync::OnceLock};

#[derive(Deserialize)]
struct Variant {
    header: String,
    hero: String,
    css: String,
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
    pub(super) fn style(&self, out: &mut String) {
        write!(
            out,
            "@media(min-width:769px){{{}}}@media(max-width:768px){{{}}}{}",
            self.desktop.css,
            self.mobile.css,
            include_str!("../../frontend/classic/presentation.css")
        )
        .unwrap();
    }
    pub(super) fn section(
        &self,
        out: &mut String,
        node: &Component,
        design: &Design,
        defaults: &Design,
        preview: bool,
    ) -> bool {
        let header = node.id == "header" && node.kind == Kind::Header;
        if !header && !(node.id == "hero" && node.kind == Kind::Hero) {
            return false;
        }
        if node.hidden && !preview {
            return true;
        }
        let Some(base) = defaults.lobby.iter().find(|n| n.id == node.id) else {
            return false;
        };
        // Keep authored replacements and previous projects editable through the normal component renderer.
        if node.title != base.title
            || node.text != base.text
            || node.button != base.button
            || node.asset != base.asset
            || node.image.is_some()
            || (header && design.theme.brand != defaults.theme.brand)
        {
            render_node(out, node, &design.theme, preview);
            return true;
        }
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
            write!(
                out,
                "<div class=\"classic-{mode}\">{}</div>",
                if header { &v.header } else { &v.hero }
            )
            .unwrap();
        }
        out.push_str("</div>");
        true
    }
}
