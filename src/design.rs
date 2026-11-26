//! Portable rules used by the browser WASM module; image payloads stay on the JS side.
use serde_json::{json, Value};

const SECTIONS: [&str; 4] = ["hero", "discover", "games", "lounge"];

fn text(value: &Value, fallback: &str, max: usize) -> String {
    let input = value.as_str().unwrap_or(fallback);
    let mut units = 0;
    input.chars().filter(|c| !matches!(*c, '\u{0000}'..='\u{0008}' | '\u{000b}'..='\u{000c}' | '\u{000e}'..='\u{001f}'))
        .take_while(|c| { units += c.len_utf16(); units <= max }).collect()
}

fn sections(value: &Value, complete: bool) -> Vec<&'static str> {
    let mut out = Vec::with_capacity(SECTIONS.len());
    if let Some(items) = value.as_array() {
        for item in items {
            if let Some(section) = SECTIONS.iter().copied().find(|s| Some(*s) == item.as_str()) {
                if !out.contains(&section) { out.push(section); }
            }
        }
    }
    if complete {
        for section in SECTIONS {
            if !out.contains(&section) { out.push(section); }
        }
    }
    out
}

pub fn normalize_content(value: &Value) -> Value {
    json!({
        "title":text(&value["title"], "La suerte tiene\nun nuevo lugar.", 100),
        "description":text(&value["description"], "Grandes clásicos, nuevas emociones.\nElegí tu juego y hacé tuya la noche.", 220),
        "button":text(&value["button"], "Explorar juegos ↗", 45),
        "sections":sections(&value["sections"], true),
        "hidden":sections(&value["hidden"], false)
    })
}

fn choice<'a>(value: &'a Value, allowed: &[&str], fallback: &'a str) -> &'a str {
    value.as_str().filter(|s| allowed.contains(s)).unwrap_or(fallback)
}
fn clamp(value: &Value, min: f64, max: f64, fallback: f64) -> f64 {
    value.as_f64().filter(|n| n.is_finite()).map(|n| n.clamp(min,max)).unwrap_or(fallback)
}

pub fn normalize_design(value: &Value) -> Value {
    let radius = if value["radius"].is_null() || value["radius"] == "original" {
        json!("original")
    } else {
        json!(clamp(&value["radius"], 0., 30., 16.))
    };
    json!({
        "font":choice(&value["font"], &["original","system","barlow","archivo","oswald","editorial"], "original"),
        "scale":clamp(&value["scale"],85.,120.,100.),
        "weight":choice(&value["weight"], &["original","400","600","700"], "original"),
        "radius":radius,
        "imageEnabled":value["imageEnabled"] != false,
        "shade":clamp(&value["shade"],0.,90.,55.),
        "position":choice(&value["position"], &["center","top","bottom"], "center"),
        "fit":choice(&value["fit"], &["cover","contain"], "cover")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_order_and_controls() {
        let v = normalize_content(&json!({"title":"A\u{0001}B","sections":["games","games","unknown","hero"],"hidden":["x","hero","hero"]}));
        assert_eq!(v["title"], "AB");
        assert_eq!(v["sections"], json!(["games","hero","discover","lounge"]));
        assert_eq!(v["hidden"], json!(["hero"]));
    }

    #[test]
    fn utf16_limit_does_not_split_surrogate_pair() {
        let v = normalize_content(&json!({"title":"a".repeat(99) + "😀"}));
        assert_eq!(v["title"].as_str().unwrap().encode_utf16().count(),99);
    }

    #[test]
    fn no_css_injection_and_bounds() {
        let v = normalize_design(&json!({"font":"Arial; color:red","scale":999,"radius":-1,"shade":null,"position":"top;display:none","weight":"900","imageEnabled":false}));
        assert_eq!(v["font"],"original");
        assert_eq!(v["scale"].as_f64(),Some(120.));
        assert_eq!(v["radius"].as_f64(),Some(0.));
        assert_eq!(v["shade"].as_f64(),Some(55.));
        assert_eq!(v["position"],"center");
        assert_eq!(v["weight"],"original");
        assert_eq!(v["imageEnabled"],false);
    }
}
