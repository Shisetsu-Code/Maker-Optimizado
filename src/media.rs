//! Local, content-addressed image library. All references are same-origin IDs.
use crate::model::{Component, Kind};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize, Default)]
struct Artwork {
    #[serde(default)]
    hero: Option<String>,
    #[serde(default)]
    cards: Vec<Card>,
    #[serde(default)]
    live: Vec<Card>,
    #[serde(default)]
    categories: Vec<String>,
}

#[derive(Deserialize)]
struct Card {
    asset: String,
    title: String,
    #[serde(default)]
    provider: String,
}

pub fn valid_asset(value: &str) -> bool {
    let Some((hash, extension)) = value.split_once('.') else {
        return false;
    };
    hash.len() == 40
        && hash
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        && matches!(extension, "png" | "jpg" | "webp" | "avif" | "gif")
}

fn assign_cards(nodes: &mut [Component], cards: &[Card]) {
    for (node, card) in nodes.iter_mut().zip(cards) {
        if valid_asset(&card.asset) {
            node.asset = Some(card.asset.clone());
            if crate::model::valid_text(&card.title, 120) && !card.title.is_empty() {
                node.title.clone_from(&card.title);
            }
            if !card.provider.is_empty() && crate::model::valid_text(&card.provider, 120) {
                node.text.clone_from(&card.provider);
            }
        }
    }
}

pub fn apply_defaults(preset: &str, nodes: &mut [Component]) {
    let Ok(defaults) = serde_json::from_str::<BTreeMap<String, Artwork>>(include_str!(
        "../assets/preset-artwork.json"
    )) else {
        return;
    };
    let Some(artwork) = defaults.get(preset) else {
        return;
    };
    for node in nodes {
        match node.kind {
            Kind::Hero => {
                node.asset = artwork.hero.as_ref().filter(|id| valid_asset(id)).cloned();
            }
            Kind::Catalog => assign_cards(&mut node.children, &artwork.cards),
            Kind::Live => assign_cards(&mut node.children, &artwork.live),
            Kind::Categories => {
                for (child, id) in node.children.iter_mut().zip(&artwork.categories) {
                    if valid_asset(id) {
                        child.asset = Some(id.clone());
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_content_addressed_raster_names() {
        let good = format!("{}.webp", "a".repeat(40));
        assert!(valid_asset(&good));
        for bad in [
            "../private.png",
            "https://example.com/test.png",
            "a.svg",
            "/test.png",
            "bad",
        ] {
            assert!(!valid_asset(bad));
        }
    }
}
