//! Editorial layout: the original composition rendered by the current model.
use crate::{
    gallery::{action_attr, attrs, font_name, generic, scaled_css, visible},
    model::*,
    render::{accent_ink, escape},
};
use std::fmt::Write as _;

const CSS: &str = include_str!("../frontend/resort.css");
const BOOK: &str = "bced3a505c90b295e30a6c749ba043d65a73e9f5.webp";
const TIGER: &str = "5838f407e17892f42a6210429cce32b0a304ef8a.webp";
const BOOK_ID: &str = "item-710387237f101db8d72a69fa";

pub fn enabled(preset: &str, design: &Design) -> bool {
    preset == "ivory" && design.theme.layout == Layout::Editorial
}
const NAV: [(&str, &str, &str); 6] = [
    ("✦", "Todos", "all"),
    ("♠", "Slots", "slots"),
    ("◉", "Casino en vivo", "live"),
    ("↗", "Crash", "crash"),
    ("⚽", "Deportes", "sports"),
    ("♡", "Mis favoritos", "favorites"),
];
fn navigation(action: &str) -> &'static str {
    if action == "sports" {
        "data-experience=\"sports\""
    } else {
        action_attr(action)
    }
}
pub fn defaults(theme: &mut Theme, nodes: &mut [Component]) {
    theme.brand = "Tu casino".into();
    for n in nodes {
        match n.kind {
            Kind::Header => {
                n.title = "Descubrir".into();
                n.text = "CASINO & CLUB".into();
                n.button = "MI SALDO · DEMO".into();
            }
            Kind::Hero => {
                n.asset = Some(BOOK.into());
                n.title = "La suerte tiene\nun nuevo lugar.".into();
                n.text = "Grandes clásicos, nuevas emociones.\nElegí tu juego y hacé tuya la noche.".into();
                n.button = "Explorar juegos ↗".into();
            }
            Kind::Categories => {
                n.title = "¿Qué jugamos hoy?".into();
                n.text = "ENCONTRÁ TU PRÓXIMO FAVORITO".into();
                n.style.placeholder = Some("Buscar un juego".into());
                n.children = NAV
                    .iter()
                    .enumerate()
                    .map(|(i, (_, title, _))| Component::new(format!("category-{i}"), Kind::Tile, *title))
                    .collect();
            }
            Kind::Catalog => {
                n.title = "Los favoritos de la casa".into();
                n.text.clear();
            }
            Kind::Live => {
                n.title = "Tu lugar en primera fila.".into();
                n.text = "La emoción de una mesa de casino, desde donde estés.".into();
                n.button = "Descubrir casino en vivo ↗".into();
            }
            Kind::Footer => {
                n.title = "✦ Casino & Club".into();
                n.text = "18+ · DEMO · Sin apuestas reales".into();
            }
            _ => {}
        }
    }
}
fn img(n: &Component, class: &str, primary: bool) -> String {
    if let Some(source) = n.image.as_deref().map(str::to_owned).or_else(|| n.asset.as_ref().map(|a| format!("/media/objects/{a}"))) {
        format!("<img class=\"{class}\" {} src=\"{}\" alt=\"{}\" loading=\"lazy\" decoding=\"async\">",if primary{"data-primary-art=\"true\""}else{""},escape(&source),escape(&n.title))
    } else {
        String::new()
    }
}
fn header(out: &mut String, n: &Component, theme: &Theme) {
    write!(out, "<header {}><a class=\"brand\" href=\"#surface\">", attrs(n)).unwrap();
    if n.image.is_some() || n.asset.is_some() {
        out.push_str(&img(n, "", false));
    } else {
        write!(out,"<span class=\"star\">✦</span><span><strong>{}</strong><small>{}</small></span>",escape(&theme.brand),escape(&n.text)).unwrap();
    }
    out.push_str("</a><nav class=\"desktop\">");
    for (label, action) in [(&*n.title,"all"),("Slots","slots"),("En vivo","live"),("Deportes","sports")] {
        write!(out,"<button type=\"button\" {}>{}</button>",navigation(action),escape(label)).unwrap();
    }
    write!(out,"</nav><button type=\"button\" class=\"wallet\" data-demo=\"true\"><small>{}</small><strong>ARS 11.052 ＋</strong></button></header>",escape(&n.button)).unwrap();
}
fn hero(out: &mut String, n: &Component) {
    let has_art = n.image.is_some() || n.asset.is_some();
    write!(out,"<section class=\"hero{}\" {}><div class=\"hero-copy\"><div class=\"eyebrow\">BIENVENIDO A TU NUEVO LUGAR FAVORITO</div><h1>{}</h1><p>{}</p>",if has_art{""}else{" no-art"},attrs(n),escape(&n.title),escape(&n.text)).unwrap();
    if !n.button.is_empty() {
        write!(out,"<button type=\"button\" class=\"primary\" data-catalog-mode=\"all\" data-catalog-scroll=\"catalog\">{}</button>",escape(&n.button)).unwrap();
    }
    out.push_str("<div class=\"hero-meta\">✦ Una experiencia diferente <span>Modo demo</span></div></div>");
    if has_art {
        out.push_str("<div class=\"hero-art\" aria-hidden=\"true\">");
        if n.image.is_none() && n.asset.as_deref() == Some(BOOK) {
            out.push_str("<div class=\"orbit\"></div><div class=\"orbit second\"></div><span class=\"suit\">♠</span>");
            out.push_str(&img(n, "poster one", true));
            write!(out,"<img class=\"poster two\" src=\"/media/objects/{TIGER}\" alt=\"\"><div class=\"art-label\"><small>EL PLAN DE ESTA NOCHE</small><strong>Jugar. Descubrir. Disfrutar.</strong></div>").unwrap();
        } else {
            out.push_str(&img(n, "hero-custom-image", true));
        }
        out.push_str("</div>");
    }
    out.push_str("</section>");
}
fn categories(out: &mut String, n: &Component, preview: bool) {
    write!(out,"<section {}><div class=\"section-top\"><div><div class=\"eyebrow muted\">{}</div><h2>{}</h2></div><label class=\"search\">⌕ <input type=\"search\" data-search=\"true\" maxlength=\"120\" aria-label=\"Buscar un juego\" placeholder=\"{}\"></label></div><div class=\"categories\">",attrs(n),escape(&n.text),escape(&n.title),escape(n.style.placeholder.as_deref().unwrap_or("Buscar un juego"))).unwrap();
    for c in n.children.iter().filter(|c| visible(c, preview)) {
        let (symbol, _, action) = c.id.strip_prefix("category-").and_then(|s|s.parse::<usize>().ok()).and_then(|i|NAV.get(i).copied()).unwrap_or(("✦","","demo"));
        write!(out,"<button type=\"button\" {} {}>{symbol} {}</button>",attrs(c),navigation(action),escape(&c.title)).unwrap();
    }
    out.push_str("</div></section>");
}
fn catalog(out: &mut String, n: &Component) {
    write!(out,"<section class=\"games-section\" {}><div class=\"section-top\"><h2 data-catalog-title>{}</h2><span class=\"muted\" data-catalog-count></span></div>",attrs(n),escape(&n.title)).unwrap();
    if !n.text.is_empty() {
        write!(out,"<p>{}</p>",escape(&n.text)).unwrap();
    }
    write!(out,"<div class=\"layout\"><article id=\"resort-spotlight\" class=\"spotlight\"><div class=\"spotlight-label\">SELECCIÓN DEL CLUB <span>01 / 03</span></div><img src=\"/media/objects/{BOOK}\" alt=\"Book of Dead\"><div class=\"spotlight-copy\"><small>UN CLÁSICO QUE SIEMPRE VUELVE</small><h3>Book of Dead</h3><button type=\"button\" class=\"primary\" data-game=\"{BOOK_ID}\" aria-label=\"Ver Book of Dead\">Ver juego ↗</button></div></article><div class=\"grid items\"></div></div><p data-catalog-empty hidden>No encontramos juegos.</p><button class=\"resort-more\" type=\"button\" data-catalog-more=\"true\" hidden>Ver más juegos</button><details class=\"gallery-extra-filters\"><summary>Más filtros y recientes</summary><div data-extra-filters></div></details></section>").unwrap();
}
fn mobile(out: &mut String) {
    out.push_str("<nav class=\"mobile lobby-mobile\" aria-label=\"Navegación móvil\">");
    for (symbol,label,action) in [("⌂","Inicio","all"),("♠","Juegos","all"),("♡","Favoritos","favorites"),("◉","Mi cuenta","demo")] {
        write!(out,"<button type=\"button\" {}><span>{symbol}</span>{label}</button>",navigation(action)).unwrap();
    }
    out.push_str("</nav>");
}
pub fn document(design: &Design, preview: bool, appearance: &Appearance) -> String {
    let t = &design.theme;
    let settings = &appearance.settings;
    let font = settings.font.or_else(||(t.font != Font::Editorial).then_some(t.font)).map(font_name).unwrap_or("original");
    let radius = settings.radius.or_else(||(t.radius != 18).then_some(t.radius));
    let button = t.button_color.as_deref().unwrap_or(&t.accent);
    let backdrop = settings.enabled && appearance.image.is_some();
    let mut out = String::with_capacity(24_000);
    write!(out,"<!doctype html><html lang=\"es\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; style-src 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; base-uri 'none'; form-action 'none'\"><title>{}</title><style>",escape(&t.brand)).unwrap();
    for line in crate::render::SURFACE_CSS.lines().take_while(|line|line.starts_with("@font-face")){
        out.push_str(line);
    }
    scaled_css(&mut out,CSS,settings.scale);
    write!(out,"</style></head><body data-profile=\"ivory\" data-gallery=\"true\" data-resort=\"true\" data-appearance-font=\"{font}\" data-global-weight=\"{}\" data-global-radius=\"{}\" class=\"{}\" style=\"--paper:{};--ink:{};--accent:{};--button:{button};--button-ink:{};--global-weight:{};--global-radius:{}px;\">",settings.weight.is_some(),radius.is_some(),if backdrop{"gallery-background-enabled"}else{""},t.background,t.ink,t.accent,accent_ink(button),settings.weight.unwrap_or(400),radius.unwrap_or(t.radius)).unwrap();
    if backdrop {
        if let Some(image)=&appearance.image{
            write!(out,"<div class=\"gallery-backdrop\" aria-hidden=\"true\" style=\"--backdrop-fit:{};--backdrop-position:{};--backdrop-opacity:{}\"><img alt=\"\" src=\"{}\"></div>",settings.fit.css(),settings.position.css(),1.-f64::from(settings.shade)/100.,escape(image)).unwrap();
        }
    }
    out.push_str("<div id=\"surface\" class=\"lobby\">");
    let first = design.lobby.first().filter(|n|n.kind==Kind::Header);
    if let Some(n)=first.filter(|n|visible(n,preview)){header(&mut out,n,t);}
    out.push_str("<main>");
    for n in design.lobby.iter().skip(usize::from(first.is_some())).filter(|n|visible(n,preview)){
        match (n.id.as_str(),n.kind){
            ("hero",Kind::Hero)=>hero(&mut out,n),
            ("categories",Kind::Categories)=>categories(&mut out,n,preview),
            ("catalog",Kind::Catalog)=>catalog(&mut out,n),
            ("live",Kind::Live)=>{
                write!(out,"<section class=\"lounge\" {}><div><div class=\"eyebrow\">LA EXPERIENCIA CONTINÚA</div><h2>{}</h2><p>{}</p></div><button type=\"button\" data-experience=\"live\">{}</button><span aria-hidden=\"true\">♣ ♦ ♠</span></section>",attrs(n),escape(&n.title),escape(&n.text),escape(&n.button)).unwrap();
            }
            ("footer",Kind::Footer)=>{
                write!(out,"<footer {}><span>{}</span><span>{}</span></footer>",attrs(n),escape(&n.title),escape(&n.text)).unwrap();
            }
            _=>generic(&mut out,n,preview),
        }
    }
    out.push_str("</main>");
    if first.is_some_and(|n|visible(n,preview)){mobile(&mut out);}
    out.push_str("</div></body></html>");
    out
}
pub fn card(n: &Component, game: &str, favorite: bool, preview: bool) -> String {
    if !visible(n,preview){return String::new();}
    let title=escape(&n.title);
    let id=escape(game);
    let tag=match game{
        BOOK_ID=>"CLÁSICO",
        "item-5944db7cf50ef41f1f4fcd73"=>"POPULAR",
        "item-e59cdb885c720a608b952392"=>"CRASH",
        "item-8561d997fedcf3414dbaff6c"=>"EN VIVO",
        _=>"",
    };
    let artwork=if n.image.is_none()&&n.asset.is_none(){"<span class=\"resort-missing\">Sin imagen</span>".into()}else{img(n,"",false)};
    let extra=if n.button.is_empty()||n.button=="Ver demo"{String::new()}else{format!("<span>{}</span>",escape(&n.button))};
    let badge=if tag.is_empty(){String::new()}else{format!("<span class=\"tag\">{tag}</span>")};
    let mark=if favorite{"♥"}else{"♡"};
    format!("<article class=\"game catalog-card\" {} data-catalog-card=\"{id}\"><button type=\"button\" class=\"game-open\" data-game=\"{id}\" aria-label=\"Ver {title}\"><div class=\"game-image\">{artwork}{badge}</div><h3>{title}</h3><p>{}</p>{extra}</button><button type=\"button\" class=\"favorite\" data-favorite=\"{id}\" aria-label=\"Favorito: {title}\" aria-pressed=\"{favorite}\">{mark}</button></article>",attrs(n),escape(&n.text))
}
