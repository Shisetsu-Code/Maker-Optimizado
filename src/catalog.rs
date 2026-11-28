//! A bounded, immutable catalogue with sparse editable card overrides.
use crate::model::{self, Component, Design, Kind, ModelResult};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const PAGE_SIZE: usize = 48;
pub const MAX_CATALOG_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub provider: String,
    pub category: String,
    pub asset: Option<String>,
    #[serde(default)]
    pub seed: Option<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    pub id: String,
    pub title: String,
    pub items: Vec<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Banner {
    pub desktop: String,
    pub mobile: Option<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub version: u32,
    pub preset: String,
    pub items: Vec<Item>,
    pub groups: Vec<Group>,
    pub banners: Vec<Banner>,
}
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Query {
    pub search: String,
    pub provider: String,
    pub category: String,
    pub group: String,
    pub group_order: bool,
    pub favorites_only: bool,
    pub recent_only: bool,
    pub favorites: Vec<String>,
    pub recent: Vec<String>,
    pub offset: usize,
    pub top: bool,
    pub preview: bool,
    pub source_only: bool,
}
#[derive(Serialize)]
pub struct Page {
    pub html: String,
    pub total: usize,
    pub offset: usize,
    pub shown: usize,
}

impl Item {
    fn selected(&self, design: &Design) -> bool {
        let selection = &design.catalog_selection;
        selection
            .providers
            .as_ref()
            .is_none_or(|list| list.contains(&self.provider))
            && selection
                .games
                .as_ref()
                .is_none_or(|list| list.contains(&self.id))
    }

    pub fn component(&self) -> Component {
        let mut n = Component::new(&self.id, Kind::Tile, &self.title);
        n.text = self.provider.clone();
        n.button = "Ver demo".into();
        n.asset = self.asset.clone();
        n
    }
}

impl Catalog {
    pub fn parse(input: &str, preset: &str) -> ModelResult<Self> {
        if input.len() > MAX_CATALOG_BYTES {
            return Err("Catálogo demasiado grande.".into());
        }
        let value: Self = serde_json::from_str(input).map_err(|_| "Catálogo inválido.")?;
        if value.version != 1
            || value.preset != preset
            || value.items.len() > 10_000
            || value.groups.len() > 100
            || value.banners.len() > 100
        {
            return Err("Catálogo incompatible.".into());
        }
        let mut ids = BTreeSet::new();
        let mut seeds = BTreeSet::new();
        for item in &value.items {
            if !item.id.starts_with("item-")
                || !model::valid_id(&item.id)
                || !ids.insert(item.id.as_str())
                || !model::valid_text(&item.title, 120)
                || item.title.is_empty()
                || !model::valid_text(&item.provider, 120)
                || !model::valid_text(&item.category, 80)
                || item
                    .asset
                    .as_deref()
                    .is_some_and(|id| !crate::media::valid_asset(id))
                || item.seed.as_ref().is_some_and(|s| {
                    !model::valid_id(s) || !s.starts_with("game-") || !seeds.insert(s.as_str())
                })
            {
                return Err("Tarjeta de catálogo inválida.".into());
            }
        }
        let mut group_ids = BTreeSet::new();
        for group in &value.groups {
            if !model::valid_id(&group.id)
                || !group_ids.insert(&group.id)
                || !model::valid_text(&group.title, 120)
                || group.items.iter().any(|id| !ids.contains(id.as_str()))
            {
                return Err("Colección inválida.".into());
            }
        }
        for banner in &value.banners {
            if !crate::media::valid_asset(&banner.desktop)
                || banner
                    .mobile
                    .as_deref()
                    .is_some_and(|id| !crate::media::valid_asset(id))
            {
                return Err("Portada inválida.".into());
            }
        }
        Ok(value)
    }
    fn nodes(design: &Design) -> Option<&[Component]> {
        design
            .lobby
            .iter()
            .find(|n| n.id == "catalog" && n.kind == Kind::Catalog)
            .map(|n| n.children.as_slice())
    }
    pub fn default_component(&self, id: &str, design: &Design) -> Option<Component> {
        Self::nodes(design)?;
        let item = self
            .items
            .iter()
            .find(|i| i.id == id || i.seed.as_deref() == Some(id))?;
        let mut n = item.component();
        if item.seed.as_deref() == Some(id) {
            n.id = id.to_owned();
        }
        Some(n)
    }
    pub fn metadata(&self, design: &Design) -> serde_json::Value {
        let mut ids = BTreeSet::new();
        let mut providers = BTreeSet::new();
        let mut featured_providers = Vec::new();
        let mut categories = BTreeSet::new();
        for item in &self.items {
            if item.selected(design) {
                ids.insert(item.id.as_str());
                if providers.insert(&item.provider) && featured_providers.len() < 12 {
                    featured_providers.push(&item.provider);
                }
                categories.insert(&item.category);
            }
        }
        let groups = self
            .groups
            .iter()
            .filter(|group| group.items.iter().any(|id| ids.contains(id.as_str())))
            .map(|group| serde_json::json!({"id":group.id,"title":group.title}))
            .collect::<Vec<_>>();
        serde_json::json!({"total":ids.len(),"providers":providers,"featured_providers":featured_providers,"categories":categories,"groups":groups,"banners":self.banners})
    }
    pub fn page(&self, design: &Design, query: &Query) -> Page {
        let Some(nodes) = Self::nodes(design) else {
            return Page {
                html: String::new(),
                total: 0,
                offset: 0,
                shown: 0,
            };
        };
        let overrides: BTreeMap<_, _> = nodes.iter().map(|n| (n.id.as_str(), n)).collect();
        let favorites: BTreeSet<_> = query.favorites.iter().map(String::as_str).collect();
        let recent: BTreeSet<_> = query.recent.iter().map(String::as_str).collect();
        let group = self
            .groups
            .iter()
            .find(|g| g.id == query.group)
            .map(|g| g.items.iter().map(String::as_str).collect::<BTreeSet<_>>());
        let search = query.search.trim().to_lowercase();
        let mut rows = Vec::new();
        let mut consumed = BTreeSet::new();
        // Keep existing project seed IDs intact. All other cards are virtual until edited.
        for item in &self.items {
            let effective = overrides
                .get(item.id.as_str())
                .copied()
                .or_else(|| item.seed.as_deref().and_then(|s| overrides.get(s).copied()));
            if let Some(n) = effective {
                consumed.insert(n.id.as_str());
            }
            if !item.selected(design) {
                continue;
            }
            let title = effective.map_or(item.title.as_str(), |n| n.title.as_str());
            let detail = effective.map_or(item.provider.as_str(), |n| n.text.as_str());
            if (!query.preview && effective.is_some_and(|n| n.hidden))
                || (query.favorites_only && !favorites.contains(item.id.as_str()))
                || (query.recent_only && !recent.contains(item.id.as_str()))
                || (!query.provider.is_empty() && query.provider != item.provider)
                || (!query.category.is_empty() && query.category != item.category)
                || (!query.group.is_empty()
                    && !group.as_ref().is_some_and(|g| g.contains(item.id.as_str())))
                || (!search.is_empty()
                    && !format!("{title} {detail} {} {}", item.title, item.provider)
                        .to_lowercase()
                        .contains(&search))
            {
                continue;
            }
            rows.push((item.id.as_str(), Some(item), effective));
        }
        if query.group_order && !query.group.is_empty() {
            if let Some(collection) = self.groups.iter().find(|g| g.id == query.group) {
                let order: BTreeMap<_, _> = collection
                    .items
                    .iter()
                    .enumerate()
                    .map(|(i, id)| (id.as_str(), i))
                    .collect();
                rows.sort_by_key(|(id, _, _)| order.get(id).copied());
            }
        }
        if query.top {
            rows.truncate(12);
        }
        if !query.source_only
            && !query.top
            && query.group.is_empty()
            && query.category.is_empty()
            && query.provider.is_empty()
        {
            let defaults = crate::presets::design(&self.preset).ok();
            let original = defaults.as_ref().and_then(Self::nodes).unwrap_or_default();
            for n in nodes {
                let id = n.id.as_str();
                if design.catalog_selection.providers.is_some()
                    || design
                        .catalog_selection
                        .games
                        .as_ref()
                        .is_some_and(|list| !list.contains(id))
                    || consumed.contains(id)
                    || (id.starts_with("game-") && original.contains(n))
                    || (!query.preview && n.hidden)
                    || (query.favorites_only && !favorites.contains(id))
                    || (query.recent_only && !recent.contains(id))
                    || (!search.is_empty()
                        && !format!("{} {}", n.title, n.text)
                            .to_lowercase()
                            .contains(&search))
                {
                    continue;
                }
                rows.push((id, None, Some(n)));
            }
        }
        if query.recent_only {
            let positions: BTreeMap<_, _> = query
                .recent
                .iter()
                .enumerate()
                .map(|(i, id)| (id.as_str(), i))
                .collect();
            rows.sort_by_key(|(id, _, _)| positions.get(id).copied());
        }
        let total = rows.len();
        let offset = query.offset.min(total);
        let end = (offset + PAGE_SIZE).min(total);
        let mut html = String::new();
        for &(id, item, edited) in &rows[offset..end] {
            let base;
            let node = match (edited, item) {
                (Some(n), _) => n,
                (None, Some(item)) => {
                    base = item.component();
                    &base
                }
                (None, None) => unreachable!(),
            };
            let render = if self.preset == "ivory"
                && crate::render::frames::available(&self.preset, design)
            {
                crate::render::frames::resort_card
            } else if crate::render::frames::available(&self.preset, design) {
                crate::render::frames::card
            } else {
                crate::render::catalog_card
            };
            html.push_str(&render(node, id, favorites.contains(id), query.preview));
        }
        Page {
            html,
            total,
            offset,
            shown: end,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> Catalog {
        Catalog::parse(&serde_json::json!({"version":1,"preset":"ember","items":[
            {"id":"item-one","title":"Golden Test","provider":"One","category":"slots","asset":null},
            {"id":"item-two","title":"Live Table","provider":"Two","category":"live","asset":null}
        ],"groups":[{"id":"collection-one","title":"Group","items":["item-two"]}],"banners":[]}).to_string(),"ember").unwrap()
    }
    #[test]
    fn composed_filters() {
        let catalog = sample();
        let design = crate::presets::design("ember").unwrap();
        let page = catalog.page(
            &design,
            &Query {
                provider: "Two".into(),
                group: "collection-one".into(),
                ..Default::default()
            },
        );
        assert_eq!(page.total, 1);
        assert!(page.html.contains("Live Table"));
        assert!(!page.html.contains("Golden Test"));
        let page = catalog.page(
            &design,
            &Query {
                favorites_only: true,
                favorites: vec!["item-one".into()],
                ..Default::default()
            },
        );
        assert_eq!(page.total, 1);
        assert!(page.html.contains("aria-pressed=\"true\""));
    }
    #[test]
    fn custom_cards_participate_in_session_lists() {
        let catalog = sample();
        let mut design = crate::presets::design("ember").unwrap();
        let group = design.lobby.iter_mut().find(|n| n.id == "catalog").unwrap();
        group
            .children
            .push(Component::new("component-demo", Kind::Tile, "Propia"));
        let mut query = Query {
            favorites_only: true,
            favorites: vec!["component-demo".into()],
            ..Default::default()
        };
        let page = catalog.page(&design, &query);
        assert_eq!(page.total, 1);
        assert!(page.html.contains("aria-pressed=\"true\""));
        query.favorites_only = false;
        query.recent_only = true;
        query.recent = vec!["component-demo".into(), "item-two".into()];
        let page = catalog.page(&design, &query);
        assert_eq!(page.total, 2);
        assert!(page.html.find("Propia").unwrap() < page.html.find("Live Table").unwrap());
        query.source_only = true;
        assert_eq!(catalog.page(&design, &query).total, 1);
    }
    #[test]
    fn reject_untrusted_asset() {
        let mut json = serde_json::to_value(sample()).unwrap();
        json["items"][0]["asset"] = "../bad.png".into();
        assert!(Catalog::parse(&json.to_string(), "ember").is_err());
    }
}
