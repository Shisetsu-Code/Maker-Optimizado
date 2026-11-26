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
    pub favorites_only: bool,
    pub recent_only: bool,
    pub favorites: Vec<String>,
    pub recent: Vec<String>,
    pub offset: usize,
    pub top: bool,
    pub preview: bool,
}
#[derive(Serialize)]
pub struct Page {
    pub html: String,
    pub total: usize,
    pub offset: usize,
    pub shown: usize,
}

impl Item {
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
        if input.len() > MAX_CATALOG_BYTES { return Err("Catálogo demasiado grande.".into()); }
        let value: Self = serde_json::from_str(input).map_err(|_| "Catálogo inválido.")?;
        if value.version != 1 || value.preset != preset || value.items.len() > 10_000
            || value.groups.len() > 100 || value.banners.len() > 100 {
            return Err("Catálogo incompatible.".into());
        }
        let mut ids = BTreeSet::new();
        let mut seeds = BTreeSet::new();
        for item in &value.items {
            if !item.id.starts_with("item-") || !model::valid_id(&item.id) || !ids.insert(item.id.as_str())
                || !model::valid_text(&item.title, 120) || item.title.is_empty()
                || !model::valid_text(&item.provider, 120) || !model::valid_text(&item.category, 80)
                || item.asset.as_deref().is_some_and(|id| !crate::media::valid_asset(id))
                || item.seed.as_ref().is_some_and(|s| !model::valid_id(s) || !s.starts_with("game-") || !seeds.insert(s.as_str())) {
                return Err("Tarjeta de catálogo inválida.".into());
            }
        }
        let mut group_ids = BTreeSet::new();
        for group in &value.groups {
            if !model::valid_id(&group.id) || !group_ids.insert(&group.id)
                || !model::valid_text(&group.title, 120) || group.items.iter().any(|id| !ids.contains(id.as_str())) {
                return Err("Colección inválida.".into());
            }
        }
        for banner in &value.banners {
            if !crate::media::valid_asset(&banner.desktop)
                || banner.mobile.as_deref().is_some_and(|id| !crate::media::valid_asset(id)) {
                return Err("Portada inválida.".into());
            }
        }
        Ok(value)
    }
    fn nodes(design: &Design) -> Option<&[Component]> {
        design.lobby.iter().find(|n| n.id == "catalog" && n.kind == Kind::Catalog)
            .map(|n| n.children.as_slice())
    }
    pub fn default_component(&self, id: &str, design: &Design) -> Option<Component> {
        Self::nodes(design)?;
        let item = self.items.iter().find(|i| i.id == id || i.seed.as_deref() == Some(id))?;
        let mut n = item.component();
        if item.seed.as_deref() == Some(id) { n.id = id.to_owned(); }
        Some(n)
    }
    pub fn metadata(&self) -> serde_json::Value {
        let providers: BTreeSet<_> = self.items.iter().map(|i| &i.provider).collect();
        let categories: BTreeSet<_> = self.items.iter().map(|i| &i.category).collect();
        serde_json::json!({"total":self.items.len(),"providers":providers,"categories":categories,
            "groups":self.groups.iter().map(|g|serde_json::json!({"id":g.id,"title":g.title})).collect::<Vec<_>>(),"banners":self.banners})
    }
    pub fn page(&self, design: &Design, query: &Query) -> Page {
        let Some(nodes) = Self::nodes(design) else { return Page {html:String::new(),total:0,offset:0,shown:0}; };
        let overrides: BTreeMap<_,_> = nodes.iter().map(|n|(n.id.as_str(),n)).collect();
        let favorites: BTreeSet<_> = query.favorites.iter().map(String::as_str).collect();
        let recent: BTreeSet<_> = query.recent.iter().map(String::as_str).collect();
        let group = self.groups.iter().find(|g|g.id==query.group)
            .map(|g|g.items.iter().map(String::as_str).collect::<BTreeSet<_>>());
        let search=query.search.trim().to_lowercase();
        let mut rows=Vec::new();
        let mut consumed=BTreeSet::new();
        // Keep existing project seed IDs intact. All other cards are virtual until edited.
        for item in &self.items {
            let effective=overrides.get(item.id.as_str()).copied().or_else(||item.seed.as_deref().and_then(|s|overrides.get(s).copied()));
            if let Some(n)=effective{consumed.insert(n.id.as_str());}
            let n=effective.cloned().unwrap_or_else(||item.component());
            if (!query.preview && n.hidden)
                || (query.favorites_only && !favorites.contains(item.id.as_str()))
                || (query.recent_only && !recent.contains(item.id.as_str()))
                || (!query.provider.is_empty() && query.provider!=item.provider)
                || (!query.category.is_empty() && query.category!=item.category)
                || (!query.group.is_empty() && !group.as_ref().is_some_and(|g|g.contains(item.id.as_str())))
                || (!search.is_empty() && !format!("{} {} {} {}",n.title,n.text,item.title,item.provider).to_lowercase().contains(&search)) {continue;}
            rows.push((item,n));
        }
        if query.top{rows.truncate(12);}
        if query.recent_only{rows.sort_by_key(|(i,_)|query.recent.iter().position(|s|s==&i.id).unwrap_or(usize::MAX));}
        let custom:Vec<_>=if query.provider.is_empty()&&query.category.is_empty()&&query.group.is_empty()
            &&!query.favorites_only&&!query.recent_only&&!query.top {
            nodes.iter().filter(|n|!consumed.contains(n.id.as_str())&&(query.preview||!n.hidden)
                &&(search.is_empty()||format!("{} {}",n.title,n.text).to_lowercase().contains(&search))).collect()
        }else{Vec::new()};
        let total=rows.len()+custom.len();
        let offset=query.offset.min(total);
        let end=(offset+PAGE_SIZE).min(total);
        let mut html=String::new();
        for index in offset..end {
            if let Some((item,n))=rows.get(index){html.push_str(&crate::render::catalog_card(n,&item.id,favorites.contains(item.id.as_str()),query.preview));}
            else if let Some(n)=custom.get(index-rows.len()){html.push_str(&crate::render::catalog_card(n,&n.id,false,query.preview));}
        }
        Page{html,total,offset,shown:end}
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn sample()->Catalog{
        Catalog::parse(&serde_json::json!({"version":1,"preset":"ember","items":[
            {"id":"item-one","title":"Golden Test","provider":"One","category":"slots","asset":null},
            {"id":"item-two","title":"Live Table","provider":"Two","category":"live","asset":null}
        ],"groups":[{"id":"collection-one","title":"Group","items":["item-two"]}],"banners":[]}).to_string(),"ember").unwrap()
    }
    #[test]
    fn composed_filters(){
        let catalog=sample();let design=crate::presets::design("ember").unwrap();
        let page=catalog.page(&design,&Query{provider:"Two".into(),group:"collection-one".into(),..Default::default()});
        assert_eq!(page.total,1);assert!(page.html.contains("Live Table"));assert!(!page.html.contains("Golden Test"));
        let page=catalog.page(&design,&Query{favorites_only:true,favorites:vec!["item-one".into()],..Default::default()});
        assert_eq!(page.total,1);assert!(page.html.contains("aria-pressed=\"true\""));
    }
    #[test]
    fn reject_untrusted_asset(){let mut json=serde_json::to_value(sample()).unwrap();json["items"][0]["asset"]="../bad.png".into();assert!(Catalog::parse(&json.to_string(),"ember").is_err());}
}
