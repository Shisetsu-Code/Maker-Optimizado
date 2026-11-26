use crate::contract::{self, Route, MAX_BODY_BYTES};
use futures_util::StreamExt;
use serde_json::{json, Value};
use worker::*;

fn security_headers(headers: &Headers) -> Result<()> {
    headers.set("x-content-type-options", "nosniff")?;
    headers.set("referrer-policy", "no-referrer")?;
    headers.set("x-frame-options", "SAMEORIGIN")?;
    headers.set("permissions-policy", "camera=(), microphone=(), geolocation=(), payment=()")?;
    headers.set("x-robots-tag", "noindex, nofollow")?;
    Ok(())
}

fn response(value: Value, status: u16, head: bool) -> Result<Response> {
    let mut result = if head { Response::empty()? } else { Response::from_json(&value)? };
    let headers = result.headers_mut();
    headers.set("content-type", "application/json; charset=utf-8")?;
    headers.set("cache-control", "no-store")?;
    headers.set("x-demo-mode", "true")?;
    security_headers(headers)?;
    Ok(result.with_status(status))
}

fn method_not_allowed(allowed: &str) -> Result<Response> {
    let mut result = response(contract::error("Método no permitido."),405,false)?;
    result.headers_mut().set("allow",allowed)?;
    Ok(result)
}

async fn bounded_body(req: &mut Request) -> std::result::Result<Vec<u8>, u16> {
    if let Ok(Some(length)) = req.headers().get("content-length") {
        if length.parse::<u64>().is_ok_and(|n| n > MAX_BODY_BYTES as u64) { return Err(413); }
    }
    let mut stream = req.stream().map_err(|_| 400u16)?;
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| 400u16)?;
        if chunk.len() > MAX_BODY_BYTES.saturating_sub(bytes.len()) { return Err(413); }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

async fn handle(mut req: Request, env: Env) -> Result<Response> {
    let path = req.path();
    let method = req.method();
    let head = method == Method::Head;
    match contract::route(&path) {
        Route::Health | Route::Winners => {
            if method != Method::Get && !head { return method_not_allowed("GET, HEAD"); }
            let value = if path == "/api/health" {
                json!({"ok":true,"demo":true,"version":"maker-rust-0.2.0"})
            } else { json!({"winners":[],"demo":true}) };
            response(value,200,head)
        }
        // Legacy panel explicitly probes this demo endpoint. No data is collected.
        Route::Monitor => {
            if !matches!(method, Method::Get | Method::Post | Method::Head) {
                return method_not_allowed("GET, HEAD, POST");
            }
            response(json!({"ok":false,"demo":true,"motivo":"demo"}),200,head)
        }
        Route::DesignEvent => {
            if method != Method::Post { return method_not_allowed("POST"); }
            response(json!({"ok":false,"demo":true,"mensaje":contract::DEMO_MESSAGE,"error":contract::DEMO_MESSAGE}),422,false)
        }
        Route::Launch => {
            if method != Method::Post { return method_not_allowed("POST"); }
            let launch = match bounded_body(&mut req).await.and_then(|bytes| contract::parse_launch(&bytes)) {
                Ok(launch) => launch,
                Err(status) => return response(contract::error(if status == 422 {
                    "En esta muestra solo se abren vistas de demostración."
                } else { "Solicitud de demostración inválida." }),status,false),
            };
            let mut url = req.url()?;
            url.set_path("/muestra/juego-demo.html");
            url.set_query(None);
            url.set_fragment(None);
            url.query_pairs_mut().append_pair("vendor",&launch.vendor)
                .append_pair("game",&launch.game).append_pair("channel",launch.channel);
            response(json!({"ok":true,"demo":true,"url":url.as_str()}),200,false)
        }
        Route::UnknownApi => response(contract::error("Ruta no encontrada."),404,head),
        Route::Asset => {
            let assets = env.assets("ASSETS")?;
            let alias = if path == "/" { Some("/index.html") } else if path.starts_with("/muestra/games-cartas/") {
                Some(&path["/muestra".len()..])
            } else { None };
            if let Some(target) = alias {
                if method != Method::Get && !head { return method_not_allowed("GET, HEAD"); }
                let mut url = req.url()?;
                url.set_path(target);
                let mut forwarded = Request::new(url.as_str(),method)?;
                for key in ["accept","range","if-range","if-match","if-none-match","if-modified-since","if-unmodified-since"] {
                    if let Some(value) = req.headers().get(key)? {
                        forwarded.headers_mut()?.set(key,&value)?;
                    }
                }
                let asset = assets.fetch_request(forwarded).await?;
                // Clone into a new mutable Headers: a binding response may have an immutable guard.
                let headers: Headers = asset.headers().entries().collect();
                security_headers(&headers)?;
                headers.set("content-security-policy","frame-ancestors 'self'; object-src 'none'; base-uri 'self'; form-action 'self'; connect-src 'self'")?;
                Ok(asset.with_headers(headers))
            } else {
                assets.fetch_request(req).await
            }
        }
    }
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    match handle(req,env).await {
        Ok(result) => Ok(result),
        Err(_) => {
            console_error!("maker edge request failed");
            response(contract::error("No se pudo cargar la demostración."),500,false)
        }
    }
}
