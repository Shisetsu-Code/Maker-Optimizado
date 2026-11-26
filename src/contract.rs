use serde_json::{json, Value};

pub const MAX_BODY_BYTES: usize = 4096;
pub const DEMO_MESSAGE: &str = "Esta réplica es una demostración local. No se envían mensajes ni diseños. Podés descargar tu diseño con el botón de descarga.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Health,
    Winners,
    Monitor,
    DesignEvent,
    Launch,
    UnknownApi,
    Asset,
}

pub fn route(path: &str) -> Route {
    match path {
        "/api/health" => Route::Health,
        "/api/winners" => Route::Winners,
        "/api/monitoreo" => Route::Monitor,
        "/api/demo/evento" => Route::DesignEvent,
        "/api/muestra/juego-demo" => Route::Launch,
        "/api" => Route::UnknownApi,
        p if p.starts_with("/api/") => Route::UnknownApi,
        _ => Route::Asset,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Launch {
    pub vendor: String,
    pub game: String,
    pub channel: &'static str,
}

pub fn parse_launch(body: &[u8]) -> Result<Launch, u16> {
    if body.len() > MAX_BODY_BYTES {
        return Err(413);
    }
    let input: Value = serde_json::from_slice(body).map_err(|_| 400u16)?;
    fn code(value: &Value) -> Option<String> {
        let value = value.as_str()?.trim();
        (!value.is_empty() && value.encode_utf16().count() <= 120).then(|| value.to_owned())
    }
    if input.get("demo").and_then(Value::as_bool) != Some(true) {
        return Err(422);
    }
    Ok(Launch {
        vendor: code(&input["vendorCode"]).ok_or(422u16)?,
        game: code(&input["gameCode"]).ok_or(422u16)?,
        channel: if input["channel"] == "mobile" { "mobile" } else { "desktop" },
    })
}

pub fn error(message: &str) -> Value {
    json!({"ok":false,"demo":true,"error":message})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_are_explicit() {
        assert_eq!(route("/api/monitoreo"), Route::Monitor);
        assert_eq!(route("/api/missing"), Route::UnknownApi);
        assert_eq!(route("/api"), Route::UnknownApi);
        assert_eq!(route("/muestra/socio.html"), Route::Asset);
    }

    #[test]
    fn bounded_demo_only() {
        let valid = parse_launch(br#"{"demo":true,"vendorCode":" vendor ","gameCode":"game","channel":"mobile"}"#).unwrap();
        assert_eq!(valid, Launch { vendor:"vendor".into(), game:"game".into(), channel:"mobile" });
        assert_eq!(parse_launch(br#"{"demo":false,"vendorCode":"v","gameCode":"g"}"#), Err(422));
        assert_eq!(parse_launch(br#"{"demo":true,"vendorCode":1,"gameCode":"g"}"#), Err(422));
        assert_eq!(parse_launch(b"invalid"), Err(400));
        assert_eq!(parse_launch(&vec![b'x'; MAX_BODY_BYTES + 1]), Err(413));
    }

    #[test]
    fn codes_have_utf16_limit() {
        let data = json!({"demo":true,"vendorCode":"v","gameCode":"😀".repeat(61)});
        assert_eq!(parse_launch(data.to_string().as_bytes()), Err(422));
    }
}
