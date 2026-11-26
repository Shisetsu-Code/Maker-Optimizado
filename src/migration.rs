//! Explicit migration of the original JSON formats. Ambiguous DOM rules are
//! reported, never silently guessed. The legacy source file remains unchanged.
use crate::{model::*, presets};
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Debug, Default, Serialize)]
pub struct Report {
    pub converted: bool,
    pub applied: usize,
    pub unresolved: Vec<Value>,
}
impl Report {
    fn unresolved(&mut self, scope: &str, label: &str, reason: &str) {
        self.unresolved
            .push(json!({"scope":scope,"label":label,"reason":reason}));
    }
}
fn preset(value: &str) -> Option<&'static str> {
    Some(match value {
        "clasica" | "ember" => "ember",
        "azul" | "cobalt" => "cobalt",
        "blaze" | "flare" => "flare",
        "brasa" | "coal" => "coal",
        "duplicada" | "resort" | "ivory" => "ivory",
        "bateupix" | "violet" => "violet",
        "cobraya" | "saffron" => "saffron",
        "lunabet" | "midnight" => "midnight",
        "multisala" | "amethyst" => "amethyst",
        "pinaplay" | "jade" => "jade",
        "celuapuestas40" | "rose" => "rose",
        "bet30" | "garnet" => "garnet",
        "casinotower30" | "brass" => "brass",
        "fenarys" | "aurum" => "aurum",
        "fortunasjk" | "verdant" => "verdant",
        "ravenbet" | "crimson" => "crimson",
        "luckrex" | "onyx" => "onyx",
        _ => return None,
    })
}
fn ensure<'a>(w: &'a mut Workspace, id: &str) -> ModelResult<&'a mut Design> {
    if !w.designs.contains_key(id) {
        w.designs.insert(id.to_owned(), presets::design(id)?);
    }
    w.designs
        .get_mut(id)
        .ok_or_else(|| "Plantilla no encontrada.".into())
}
fn number(value: &Value, min: u16, max: u16) -> Option<u16> {
    let value = value.as_f64().or_else(|| value.as_str()?.parse().ok())?;
    value
        .is_finite()
        .then(|| value.round().clamp(f64::from(min), f64::from(max)) as u16)
}
fn color(value: &Value) -> Option<String> {
    value
        .as_str()
        .filter(|s| valid_color(s))
        .map(|s| s.to_ascii_lowercase())
}
fn text(value: &Value, max: usize) -> Option<String> {
    value.as_str().map(|s| {
        s.chars()
            .filter(|c| !c.is_control() || matches!(c, '\n' | '\t' | '\r'))
            .take(max)
            .collect()
    })
}
fn font(value: &Value) -> Option<Font> {
    match value.as_str()? {
        "system" => Some(Font::System),
        "barlow" => Some(Font::Barlow),
        "archivo" => Some(Font::Archivo),
        "oswald" => Some(Font::Oswald),
        "editorial" => Some(Font::Editorial),
        _ => None,
    }
}
fn fit(value: &Value) -> Option<Fit> {
    match value.as_str()? {
        "cover" => Some(Fit::Cover),
        "contain" => Some(Fit::Contain),
        _ => None,
    }
}
fn position(value: &Value) -> Option<Position> {
    match value.as_str()? {
        "center" => Some(Position::Center),
        "top" => Some(Position::Top),
        "bottom" => Some(Position::Bottom),
        "left" => Some(Position::Left),
        "right" => Some(Position::Right),
        _ => None,
    }
}
fn alignment(value: &Value) -> Option<Align> {
    match value.as_str()? {
        "left" => Some(Align::Left),
        "center" => Some(Align::Center),
        "right" => Some(Align::Right),
        _ => None,
    }
}

fn apply_style(style: &mut Style, rule: &Value) {
    if let Some(v) = color(&rule["color"]) {
        style.color = Some(v);
    }
    if let Some(v) = color(&rule["background"]) {
        style.background = Some(v);
    }
    if let Some(v) = color(&rule["borderColor"]) {
        style.border_color = Some(v);
    }
    if let Some(v) = number(&rule["borderWidth"], 0, 12) {
        style.border_width = Some(v as u8);
    }
    if let Some(v) = number(&rule["radius"], 0, 80) {
        style.radius = Some(v as u8);
    }
    if let Some(v) = number(&rule["padding"], 0, 100) {
        style.padding = Some(v as u8);
    }
    if let Some(v) = number(&rule["fontSize"], 8, 110) {
        style.font_size = Some(v as u8);
    }
    if let Some(v) =
        number(&rule["weight"], 400, 900).filter(|v| [400, 500, 600, 700, 800, 900].contains(v))
    {
        style.font_weight = Some(v);
    }
    if let Some(v) = number(&rule["width"], 10, 100) {
        style.width = Some(v as u8);
    }
    if let Some(v) = number(&rule["height"], 20, 1200) {
        style.height = Some(v);
    }
    if let Some(v) = alignment(&rule["align"]) {
        style.align = Some(v);
    }
    if let Some(v) = fit(&rule["fit"]) {
        style.fit = Some(v);
    }
    if let Some(v) = position(&rule["position"]) {
        style.position = Some(v);
    }
}
fn find_mut<'a>(nodes: &'a mut [Component], id: &str) -> Option<&'a mut Component> {
    nodes.iter_mut().find_map(|n| n.find_mut(id))
}
fn match_text(nodes: &[Component], original: &str, found: &mut Vec<(String, &'static str)>) {
    for n in nodes {
        for (name, value) in [
            ("title", &n.title),
            ("text", &n.text),
            ("button", &n.button),
        ] {
            if value == original && !value.is_empty() {
                found.push((n.id.clone(), name));
            }
        }
        match_text(&n.children, original, found);
    }
}
fn selector_id(selector: &str) -> Option<(&'static str, &'static str)> {
    match selector {
        "#bp-banner" | "#hero-image" => Some(("hero", "image")),
        "#logo" | "#bp-logo" => Some(("header", "image")),
        "#bp-search" | "#search" => Some(("catalog", "placeholder")),
        "#bp-title" => Some(("catalog", "title")),
        "#bp-games" | "#games" => Some(("catalog", "block")),
        "#hero" => Some(("hero", "block")),
        _ => None,
    }
}
fn has_style(rule: &Value) -> bool {
    [
        "color",
        "background",
        "borderColor",
        "borderWidth",
        "radius",
        "padding",
        "fontSize",
        "weight",
        "width",
        "height",
        "align",
        "fit",
        "position",
    ]
    .iter()
    .any(|key| rule.get(key).is_some())
}

pub fn convert(input: &str, current: &Workspace) -> ModelResult<(Workspace, Report)> {
    if input.len() > MAX_PROJECT_BYTES {
        return Err("Archivo anterior demasiado grande.".into());
    }
    let root: Value = serde_json::from_str(input).map_err(|_| "JSON inválido.")?;
    let legacy_design = root["formato"] == "rushybet-diseno-1";
    let settings = root.get("ajustes").unwrap_or(&root);
    let recognized = legacy_design
        || [
            "font",
            "scale",
            "weight",
            "radius",
            "image",
            "imageEnabled",
            "shade",
            "position",
            "fit",
            "content",
        ]
        .iter()
        .any(|k| settings.get(k).is_some());
    if !recognized {
        return Err("Formato de proyecto no reconocido.".into());
    }
    let mut next = current.clone();
    let mut report = Report {
        converted: true,
        ..Default::default()
    };
    if legacy_design {
        let id = root["plantilla"]
            .as_str()
            .and_then(preset)
            .ok_or("Plantilla anterior desconocida.")?;
        next.active = id.into();
        let design = ensure(&mut next, id)?;
        if let Some(name) = text(&root["nombre"], 60).filter(|v| !v.trim().is_empty()) {
            design.theme.brand = name;
            report.applied += 1;
        }
        let colors = &root["coloresQueSePintan"];
        if let Some(v) = color(&colors["marca"]) {
            design.theme.accent = v;
            report.applied += 1;
        }
        if let Some(v) = color(&colors["botones"]) {
            design.theme.button_color = Some(v);
            report.applied += 1;
        }
        if let Some(v) = color(&colors["brillos"]) {
            design.theme.glow_color = Some(v);
            report.applied += 1;
        }
        if let Some(v) = color(&colors["fondo"]) {
            design.theme.background = v;
            report.applied += 1;
        }
        if let Some(v) = color(&root["variables"]["--dm-t"]) {
            design.theme.surface = v;
            report.applied += 1;
        }
        if !root["logo"].is_null() {
            report.unresolved("marca","Logo","El formato anterior exportaba el logo en un archivo separado. Volvé a cargar ese archivo.");
        }
        if !root["perillas"].is_null() {
            report.unresolved(
                "marca",
                "Perillas originales",
                "Los parámetros sin equivalente tipado requieren revisión visual.",
            );
        }
    }
    if let Some(v) = settings.get("font") {
        next.appearance.settings.font = font(v);
        report.applied += 1;
    }
    if let Some(v) = number(&settings["scale"], 85, 120) {
        next.appearance.settings.scale = v as u8;
        report.applied += 1;
    }
    if let Some(v) = settings.get("weight") {
        next.appearance.settings.weight =
            number(v, 400, 700).filter(|w| [400, 600, 700].contains(w));
        report.applied += 1;
    }
    if let Some(v) = settings.get("radius") {
        next.appearance.settings.radius = number(v, 0, 30).map(|v| v as u8);
        report.applied += 1;
    }
    if let Some(v) = settings["imageEnabled"].as_bool() {
        next.appearance.settings.enabled = v;
        report.applied += 1;
    }
    if let Some(v) = number(&settings["shade"], 0, 90) {
        next.appearance.settings.shade = v as u8;
        report.applied += 1;
    }
    if let Some(v) = fit(&settings["fit"]) {
        next.appearance.settings.fit = v;
        report.applied += 1;
    }
    if let Some(v) = position(&settings["position"]) {
        next.appearance.settings.position = v;
        report.applied += 1;
    }
    if let Some(v) = settings["image"].as_str() {
        if v.is_empty() {
            next.appearance.image = None;
            report.applied += 1;
        } else if valid_raster(v, MAX_BACKGROUND_BYTES) {
            next.appearance.image = Some(v.into());
            report.applied += 1;
        } else {
            report.unresolved("apariencia", "Fondo", "Formato o tamaño no permitido.");
        }
    }
    if let Some(content) = settings.get("content").filter(|v| v.is_object()) {
        let d = ensure(&mut next, "ivory")?;
        if let Some(hero) = find_mut(&mut d.lobby, "hero") {
            if let Some(v) = text(&content["title"], 1800) {
                hero.title = v;
                report.applied += 1;
            }
            if let Some(v) = text(&content["description"], 1800) {
                hero.text = v;
                report.applied += 1;
            }
            if let Some(v) = text(&content["button"], 1800) {
                hero.button = v;
                report.applied += 1;
            }
            if let Some(v) = content["image"].as_str() {
                if v.is_empty() {
                    hero.image = None;
                    report.applied += 1;
                } else if valid_image(v) {
                    hero.image = Some(v.into());
                    hero.asset = None;
                    report.applied += 1;
                } else {
                    report.unresolved(
                        "ivory:lobby",
                        "Portada",
                        "Imagen excede el límite por componente.",
                    );
                }
            }
        }
        let sections = [
            ("hero", "hero"),
            ("discover", "categories"),
            ("games", "catalog"),
            ("lounge", "live"),
        ];
        if let Some(order) = content["sections"].as_array() {
            let mut rank = std::collections::BTreeMap::new();
            for (i, old) in order.iter().enumerate() {
                if let Some((_, id)) = sections
                    .iter()
                    .find(|(name, _)| old.as_str() == Some(*name))
                {
                    rank.insert(*id, i + 1);
                }
            }
            d.lobby.sort_by_key(|n| {
                if n.id == "header" {
                    0
                } else if n.id == "footer" {
                    usize::MAX
                } else {
                    rank.get(n.id.as_str()).copied().unwrap_or(100)
                }
            });
            report.applied += 1;
        }
        if let Some(hidden) = content["hidden"].as_array() {
            for (old, id) in sections {
                if let Some(n) = find_mut(&mut d.lobby, id) {
                    n.hidden = hidden.iter().any(|v| v.as_str() == Some(old));
                }
            }
            report.applied += 1;
        }
    }
    let elements = root.get("elementos").or_else(|| settings.get("elementos"));
    if let Some(elements) = elements {
        if let Some(pages) = elements["pages"].as_object() {
            for (page, rules) in pages {
                let Some((old, screen)) = page.split_once(':') else {
                    report.unresolved(page, "Página", "Identificador anterior inválido.");
                    continue;
                };
                let Some(id) = preset(old) else {
                    report.unresolved(page, "Página", "Plantilla sin correspondencia.");
                    continue;
                };
                let view = match screen {
                    "lobby" => View::Lobby,
                    "panel" => View::Panel,
                    _ => {
                        report.unresolved(page, "Página", "Pantalla sin correspondencia.");
                        continue;
                    }
                };
                let d = ensure(&mut next, id)?;
                let Some(rules) = rules.as_array() else {
                    report.unresolved(page, "Reglas", "Formato inválido.");
                    continue;
                };
                for rule in rules {
                    let target = &rule["target"];
                    let label = target["label"].as_str().unwrap_or("Elemento");
                    let kind = target["type"].as_str().unwrap_or("");
                    let selector = target["selector"].as_str().unwrap_or("");
                    let mut matched =
                        selector_id(selector).map(|(id, field)| (id.to_owned(), field));
                    if matched.is_none() && kind == "text" {
                        let mut candidates = Vec::new();
                        if let Some(original) = target["original"].as_str() {
                            match_text(d.nodes(view), original, &mut candidates);
                        }
                        if candidates.len() == 1 {
                            matched = candidates.pop();
                        }
                    }
                    let Some((id, field)) = matched else {
                        report.unresolved(
                            page,
                            label,
                            "No existe correspondencia semántica única con el componente nuevo.",
                        );
                        continue;
                    };
                    let Some(node) = find_mut(d.nodes_mut(view), &id) else {
                        report.unresolved(
                            page,
                            label,
                            "Componente no disponible en esta pantalla.",
                        );
                        continue;
                    };
                    let mut applied = false;
                    if let Some(v) = text(&rule["text"], 1800) {
                        match field {
                            "title" => {
                                node.title = v;
                                applied = true;
                            }
                            "text" => {
                                node.text = v;
                                applied = true;
                            }
                            "button" => {
                                node.button = v;
                                applied = true;
                            }
                            "placeholder" => {
                                node.style.placeholder = Some(v);
                                applied = true;
                            }
                            _ => {}
                        }
                    }
                    if let Some(key) = rule["image"].as_str() {
                        if kind == "image" && field == "image" {
                            if let Some(value) = elements["assets"]
                                .get(key)
                                .and_then(Value::as_str)
                                .filter(|v| valid_image(v))
                            {
                                node.image = Some(value.into());
                                node.asset = None;
                                applied = true;
                            } else {
                                report.unresolved(page, label, "Imagen no disponible o inválida.");
                            }
                        } else {
                            report.unresolved(
                                page,
                                label,
                                "El fondo/subelemento anterior requiere selección manual.",
                            );
                        }
                    }
                    if has_style(rule) || rule["hidden"] == true {
                        if field == "block" || field == "image" {
                            apply_style(&mut node.style, rule);
                            if rule["hidden"] == true {
                                node.hidden = true;
                            }
                            applied = true;
                        } else {
                            report.unresolved(page,label,"El estilo u ocultamiento afectaba un subelemento; no se amplía automáticamente a todo el bloque.");
                        }
                    }
                    if applied {
                        report.applied += 1;
                    }
                }
            }
        }
    }
    validate(&next)?;
    Ok((next, report))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn imports_shared_appearance_and_content_without_guessing() {
        let current = presets::workspace("ember").unwrap();
        let old = json!({"version":2,"ajustes":{"font":"barlow","scale":110,"weight":"700","content":{"title":"Portada anterior","sections":["games","hero"],"hidden":["lounge"]}},"elementos":{"version":1,"assets":{},"pages":{"bateupix:lobby":[{"target":{"type":"field","selector":"#bp-search","label":"Buscar"},"text":"Encontrar"},{"target":{"type":"text","selector":"body>div:nth-of-type(8)","original":"No coincide","label":"Texto"}}]}}});
        let (next, report) = convert(&old.to_string(), &current).unwrap();
        assert_eq!(next.appearance.settings.font, Some(Font::Barlow));
        assert_eq!(next.appearance.settings.scale, 110);
        let d = &next.designs["ivory"];
        assert_eq!(
            d.lobby.iter().find(|n| n.id == "hero").unwrap().title,
            "Portada anterior"
        );
        let d = &next.designs["violet"];
        assert_eq!(
            d.lobby
                .iter()
                .find(|n| n.id == "catalog")
                .unwrap()
                .style
                .placeholder
                .as_deref(),
            Some("Encontrar")
        );
        assert_eq!(report.unresolved.len(), 1);
    }
    #[test]
    fn reject_unrecognized() {
        assert!(convert("{}", &presets::workspace("ember").unwrap()).is_err());
    }
}
