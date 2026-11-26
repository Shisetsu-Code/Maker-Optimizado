use crate::{model, presets, render};
use futures_util::StreamExt;
use serde_json::{Value,json};
use worker::*;

const MAX_REQUEST_BYTES:usize=256*1024;

fn json_response(value:Value,status:u16,head:bool)->Result<Response>{
    let mut response=if head{Response::empty()?}else{Response::from_json(&value)?};
    let headers=response.headers_mut();
    headers.set("content-type","application/json; charset=utf-8")?;
    headers.set("cache-control","no-store")?;
    headers.set("x-content-type-options","nosniff")?;
    headers.set("referrer-policy","no-referrer")?;
    Ok(response.with_status(status))
}
fn error(message:&str,status:u16,head:bool)->Result<Response>{json_response(json!({"ok":false,"error":message}),status,head)}
fn allow(methods:&str)->Result<Response>{let mut r=error("Método no permitido.",405,false)?;r.headers_mut().set("allow",methods)?;Ok(r)}
async fn bounded_body(req:&mut Request)->std::result::Result<Vec<u8>,u16>{
    if let Ok(Some(length))=req.headers().get("content-length"){
        if length.parse::<u64>().is_ok_and(|n|n>MAX_REQUEST_BYTES as u64){return Err(413);}
    }
    let mut stream=req.stream().map_err(|_|400u16)?;
    let mut bytes=Vec::new();
    while let Some(chunk)=stream.next().await{
        let chunk=chunk.map_err(|_|400u16)?;
        if chunk.len()>MAX_REQUEST_BYTES.saturating_sub(bytes.len()){return Err(413);}
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}
async fn handle(mut req:Request,env:Env)->Result<Response>{
    let method=req.method();
    let head=method==Method::Head;
    match req.path().as_str(){
        "/api/v1/health"=>{if method!=Method::Get&&!head{return allow("GET, HEAD");}json_response(json!({"ok":true,"version":env!("CARGO_PKG_VERSION"),"runtime":"rust-wasm","schema":model::SCHEMA_VERSION}),200,head)}
        "/api/v1/templates"=>{if method!=Method::Get&&!head{return allow("GET, HEAD");}json_response(json!({"templates":presets::PRESETS}),200,head)}
        "/api/v1/validate"=>{if method!=Method::Post{return allow("POST");}let bytes=match bounded_body(&mut req).await{Ok(b)=>b,Err(code)=>return error("Solicitud inválida o demasiado grande.",code,false)};
            let Ok(body)=std::str::from_utf8(&bytes)else{return error("UTF-8 inválido.",400,false);};
            match model::decode(body){Ok(_)=>json_response(json!({"ok":true,"schema":model::SCHEMA_VERSION}),200,false),Err(_)=>error("Proyecto inválido.",422,false)}
        }
        "/api/v1/preview"=>{if method!=Method::Get&&!head{return allow("GET, HEAD");}
            let url=req.url()?;
            let mut id="ember".to_owned();let mut view=model::View::Lobby;
            for (key,value) in url.query_pairs(){if key=="template"{id=value.into_owned();}else if key=="view"&&value=="panel"{view=model::View::Panel;}}
            let Ok(design)=presets::design(&id)else{return error("Plantilla desconocida.",404,head);};
            let mut response=if head{Response::empty()?}else{Response::from_html(render::document(&design,view,false))?};
            response.headers_mut().set("content-type","text/html; charset=utf-8")?;
            response.headers_mut().set("content-security-policy","default-src 'none'; style-src 'unsafe-inline'; img-src data:; frame-ancestors 'self'; base-uri 'none'; form-action 'none'")?;
            response.headers_mut().set("cache-control","public, max-age=300")?;
            response.headers_mut().set("x-content-type-options","nosniff")?;
            Ok(response)
        }
        path if path=="/api"||path.starts_with("/api/")=>error("Ruta no encontrada.",404,head),
        _=>env.assets("ASSETS")?.fetch_request(req).await,
    }
}
#[event(fetch)]
pub async fn main(req:Request,env:Env,_ctx:Context)->Result<Response>{
    match handle(req,env).await{Ok(response)=>Ok(response),Err(_)=>{console_error!("maker edge error");error("Error interno.",500,false)}}
}
