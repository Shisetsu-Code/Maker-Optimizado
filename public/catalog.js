// DOM bridge for the Rust-owned catalogue. Session-only interaction state.
const states=new Map(),sources=new Map();
let current=null;
const installed=new WeakMap(),menuCache=new WeakMap();
const endpoint=document.querySelector('meta[name="maker-catalog-endpoint"]')?.content.trim();
async function catalogSource(preset){
  let url=new URL('/catalogs/'+encodeURIComponent(preset)+'.json',location.origin);
  if(endpoint){
    url=new URL(endpoint,location.origin);
    if(url.origin!==location.origin)throw Error('La API del catálogo debe compartir el origen del editor.');
    url.searchParams.set('preset',preset);
  }
  if(!sources.has(preset)){
    const pending=fetch(url,{credentials:'same-origin',cache:endpoint?'no-store':'default',redirect:'error'})
      .then(async response=>{
        if(!response.ok)throw Error('Catálogo no disponible ('+response.status+').');
        const reader=response.body?.getReader();
        if(!reader)throw Error('El catálogo llegó sin contenido.');
        const chunks=[];let size=0;
        try{
          for(;;){
            const {done,value}=await reader.read();
            if(done)break;
            size+=value.byteLength;
            if(size>8*1024*1024){await reader.cancel();throw Error('El catálogo supera 8 MiB.');}
            chunks.push(value);
          }
        }finally{reader.releaseLock();}
        return new Blob(chunks).text();
      });
    sources.set(preset,pending);
    pending.catch(()=>sources.delete(preset));
  }
  return sources.get(preset);
}
async function prepare(studio,preset,stillCurrent){
  const source=await catalogSource(preset);
  if(!stillCurrent())return false;
  if(installed.get(studio)!==preset){
    try{studio.set_catalog(source);}
    catch(error){sources.delete(preset);throw error;}
    installed.set(studio,preset);
    menuCache.delete(studio);
  }
  return true;
}
function stateFor(preset){
  if(!states.has(preset))states.set(preset,{search:'',provider:'',category:'',group:'',favorites:new Set(),recent:[],favorites_only:false,recent_only:false,top:false,banner:0,shown:48});
  return states.get(preset);
}
function button(doc,text,key,value=''){
  const b=doc.createElement('button');b.type='button';b.textContent=text;b.dataset[key]=value;return b;
}
function select(doc,key,label,options,value){
  const s=doc.createElement('select');s.dataset.catalogFilter=key;s.setAttribute('aria-label',label);
  s.add(new Option(label,''));
  for(const option of options)s.add(new Option(option.title??option,option.id??option));
  s.value=value;return s;
}
function request(state,preview,offset=0){
  return JSON.stringify({search:state.search,provider:state.provider,category:state.category,group:state.group,
    favorites_only:state.favorites_only,recent_only:state.recent_only,favorites:[...state.favorites],recent:state.recent,
    top:state.top,source_only:Boolean(endpoint),preview,offset});
}
function classicShelf(doc,studio,meta){
  for(const summary of doc.querySelectorAll('[data-classic-summary]'))summary.textContent='Casino online con '+meta.total+' juegos de '+meta.providers.length+' proveedores.';
  const root=doc.querySelector('[data-classic-shelf]');
  if(!root&&!doc.querySelector('.account-feature'))return;
  const authored=!endpoint&&meta.groups.some(group=>group.id==='original-recommendations');
  const result=JSON.parse(studio.catalog_page(JSON.stringify({source_only:Boolean(endpoint),group:authored?'original-recommendations':''})));
  const template=doc.createElement('template');template.innerHTML=result.html;
  const cards=[...template.content.querySelectorAll('[data-catalog-card]')];
  accountFeature(doc,cards.slice(0,8));
  if(!root)return;
  root.hidden=!cards.length;
  for(const row of root.querySelectorAll('[data-native-cards]')){
    const selected=cards.slice(0,Number(row.dataset.itemLimit)||0);
    const count=row.parentElement.querySelector('[data-native-count]');
    if(count)count.textContent=String(selected.length);
    row.replaceChildren(...selected.map(card=>{
      const name=card.querySelector('h3').textContent,open=button(doc,'','game',card.dataset.catalogCard);
      open.style.cssText=row.dataset.itemStyle||'';
      open.setAttribute('aria-label','Ver '+name);
      const img=card.querySelector('img');
      if(authored)open.classList.add('ember-original-tile');
      else if(img)open.append(img.cloneNode(true));
      const label=doc.createElement('span');label.textContent=name;open.append(label);return open;
    }));
  }
}
function resortFeatured(doc,state){
  const feature=doc.querySelector('[data-resort-featured]');if(!feature)return;
  const first=doc.querySelector('#catalog [data-catalog-card]');
  feature.hidden=!first||Boolean(state.search||state.category||state.provider||state.group||state.favorites_only||state.recent_only||state.top);
  if(feature.hidden)return;
  const img=first.querySelector('img'),art=feature.querySelector('img');art.hidden=!img;
  if(img)art.src=img.src;else art.removeAttribute('src');
  feature.querySelector('h3').textContent=first.querySelector('h3').textContent;
  feature.querySelector('button').dataset.game=first.dataset.catalogCard;
}
function populate(reset=true){
  if(!current)return;
  const {doc,studio,state,editing}=current;
  const group=doc.querySelector('#catalog'),items=group?.querySelector('.items');
  if(!items)return;
  const previous=reset?0:items.querySelectorAll('.catalog-card').length;
  const target=reset?state.shown:previous+48;let offset=reset?0:previous,total=0,shown=offset;
  if(reset)items.replaceChildren();
  do{
    const page=JSON.parse(studio.catalog_page(request(state,editing(),offset)));
    total=page.total;shown=page.shown;
    // Only Rust-generated markup, with escaped text and validated asset identifiers.
    items.insertAdjacentHTML('beforeend',page.html);
    if(page.shown===offset)break;
    offset=page.shown;
  }while(shown<Math.min(target,total));
  state.shown=Math.max(48,shown);
  group.querySelector('[data-catalog-count]').textContent=group.hasAttribute('data-native-section')?total+' juegos':total?shown+' de '+total+' juegos':'No encontramos juegos.';
  const empty=group.querySelector('[data-catalog-empty]');
  if(empty)empty.hidden=total!==0;
  group.querySelector('[data-catalog-more]').hidden=shown>=total;
  resortFeatured(doc,state);
  group.dataset.catalogReady='true';
}
function collectionMenu(doc,root,groups,studio){
  if(root.id==='pp-collections'){
    root.replaceChildren(...groups.slice(0,6).map((group,index)=>{
      const section=doc.createElement('section');section.className='pp-collection';
      const header=doc.createElement('div');header.className='pp-row-heading';
      const title=doc.createElement('h2');title.textContent=group.title;
      header.append(title,button(doc,'Ver todos →','catalogGroup',group.id));
      const row=doc.createElement('div');row.className='pp-row';
      const page=JSON.parse(studio.catalog_page(JSON.stringify({group:group.id,group_order:true,source_only:Boolean(endpoint)})));
      const fragment=doc.createElement('template');fragment.innerHTML=page.html;
      for(const [i,card] of [...fragment.content.querySelectorAll('[data-catalog-card]')].slice(0,12).entries()){
        const name=card.querySelector('h3').textContent,img=card.querySelector('img');
        const tile=button(doc,'','game',card.dataset.catalogCard);tile.className='pp-game';tile.setAttribute('aria-label','Ver '+name);
        const label=doc.createElement('span');label.textContent=name;if(img)tile.append(img);tile.append(label);
        if(index===0){const rank=doc.createElement('b');rank.textContent=String(i+1).padStart(2,'0');tile.append(rank);}row.append(tile);
      }
      section.append(header,row);return section;
    }));return;
  }
  root.replaceChildren(...groups.map(group=>{
    const b=button(doc,group.title,'catalogGroup',group.id);
    if(root.id==='ce-categories'){
      const icon=doc.createElement('span'),name=doc.createElement('span');name.textContent=group.title;
      icon.textContent=/vivo|live/i.test(group.title)?'◉':/fruta|moneda/i.test(group.title)?'◆':/crash/i.test(group.title)?'↗':'♠';b.replaceChildren(icon,name);
    }return b;
  }));
}
function nativeControls(doc,meta,state,studio){
  const search=doc.querySelector('[data-search]');
  if(search)search.value=state.search;
  const providers=doc.querySelector('[data-providers]');
  providers?.replaceChildren(...(meta.featured_providers??meta.providers).map(provider=>button(doc,provider,'catalogProvider',provider)));
  const groups=doc.querySelector('[data-collections]');
  if(groups)collectionMenu(doc,groups,meta.groups,studio);
  const sidebar=doc.querySelector('.lobby-sidebar');
  // All three filter dimensions are available even on themes with a short toolbar.
  if(sidebar){
    const fields=[['provider','Proveedor',meta.providers],['category','Categoría',meta.categories],['group','Colección',meta.groups]];
    const details=doc.createElement('details');
    const title=doc.createElement('summary');title.textContent='Filtrar catálogo';details.append(title);
    for(const [key,label,values] of fields)if(values.length)details.append(select(doc,key,label,values,state[key]));
    sidebar.append(details);
  }
  const recent=doc.querySelector('[data-catalog-mode=recent]');
  if(recent)recent.hidden=!state.recent.length;
}
function nativeBanner(doc,meta,state){
  const art=doc.querySelector('#hero [data-banner-art]');
  if(!art)return null;
  const initial=art.getAttribute('src');
  if(!initial)return null;
  const banners=meta.banners.map(b=>({desktop:'/media/objects/'+b.desktop,mobile:b.mobile?'/media/objects/'+b.mobile:null}));
  const slides=[banners.find(b=>b.desktop===initial)??{desktop:initial,mobile:null},...banners.filter(b=>b.desktop!==initial)];
  const dots=doc.querySelector('[data-banner-dots]');
  dots?.replaceChildren(...slides.map((_,i)=>{
    const b=button(doc,'','bannerIndex',String(i));b.setAttribute('aria-label','Portada '+(i+1));return b;
  }));
  const mobile=doc.querySelector('#hero [data-banner-mobile]');
  const draw=()=>{
    state.banner=((state.banner%slides.length)+slides.length)%slides.length;
    const slide=slides[state.banner];art.setAttribute('src',slide.desktop);
    if(mobile){if(slide.mobile)mobile.setAttribute('srcset',slide.mobile);else mobile.removeAttribute('srcset');}
    for(const b of dots?.children??[])b.classList.toggle('active',Number(b.dataset.bannerIndex)===state.banner);
  };
  draw();return {draw};
}
function controls(doc,meta,state,studio){
  const group=doc.querySelector('#catalog');if(!group)return;
  if(group.hasAttribute('data-native-section')){nativeControls(doc,meta,state,studio);return;}
  const root=doc.createElement('div');root.className='catalog-controls';
  root.append(select(doc,'provider','Todos los proveedores',meta.providers,state.provider),
    select(doc,'category','Todas las categorías',meta.categories,state.category),
    select(doc,'group','Todas las colecciones',meta.groups,state.group));
  for(const [label,key] of [['Todos','all'],['Destacados','top'],['Favoritos','favorites'],['Recientes','recent']])
    root.append(button(doc,label,'catalogMode',key));
  const count=doc.createElement('span');count.dataset.catalogCount='true';count.setAttribute('role','status');root.append(count);
  group.querySelector('.items')?.before(root);
  const more=button(doc,'Cargar más','catalogMore','true');more.className='action catalog-more';group.append(more);
  const search=group.querySelector('[data-search]');if(search)search.value=state.search;
}
function updateMode(doc,state){
  const mode=state.favorites_only?'favorites':state.recent_only?'recent':state.top?'top':'all';
  for(const b of doc.querySelectorAll('[data-catalog-mode]'))b.setAttribute('aria-pressed',String(b.dataset.catalogMode===mode));
}
function buildBanner(doc,meta,state){
  const root=doc.querySelector('#hero .hero-art'),img=root?.querySelector('img');if(!root||!img||!meta.banners.length)return;
  const initial=img.getAttribute('src'),original=meta.banners.find(b=>'/media/objects/'+b.desktop===initial);
  const slides=[{desktop:initial,mobile:original?.mobile?'/media/objects/'+original.mobile:null},
    ...meta.banners.filter(b=>'/media/objects/'+b.desktop!==initial).map(b=>({desktop:'/media/objects/'+b.desktop,mobile:b.mobile?'/media/objects/'+b.mobile:null}))];
  if(slides.length<2&&!slides[0].mobile)return;
  const picture=doc.createElement('picture'),source=doc.createElement('source');source.media='(max-width: 700px)';
  img.before(picture);picture.append(source,img);
  const nav=doc.createElement('div');nav.className='banner-navigation';
  if(slides.length>1){
    nav.append(button(doc,'‹','bannerStep','-1'));
    slides.forEach((_,i)=>{const b=button(doc,'•','bannerIndex',String(i));b.setAttribute('aria-label','Portada '+(i+1));nav.append(b);});
    nav.append(button(doc,'›','bannerStep','1'));root.append(nav);
  }
  const draw=()=>{
    const index=((state.banner%slides.length)+slides.length)%slides.length,slide=slides[index];
    state.banner=index;img.src=slide.desktop;
    if(slide.mobile)source.srcset=slide.mobile;else source.removeAttribute('srcset');
    for(const b of nav.querySelectorAll('[data-banner-index]'))b.setAttribute('aria-pressed',String(Number(b.dataset.bannerIndex)===index));
  };
  current.banner={draw};draw();
}
function restrictFixtureGames(doc){
  for(const original of doc.querySelectorAll('[data-fixture-only]')){
    if(doc.body.dataset.preset==='coal'&&original.parentElement?.id==='inicio'){
      const deck=doc.createElement('section');deck.className='account-feature';deck.hidden=true;
      deck.setAttribute('aria-label','Juegos destacados');original.replaceWith(deck);
    }else original.remove();
  }
}
function accountFeature(doc,cards){
  for(const deck of doc.querySelectorAll('.account-feature')){
    deck.hidden=!cards.length;deck.dataset.count=String(cards.length);if(!cards.length){deck.replaceChildren();continue;}
    const index=(Number(deck.dataset.index)||0)%cards.length;
    const current=cards[index],id=current.dataset.catalogCard,title=current.querySelector('h3').textContent;
    const caption=doc.createElement('small');caption.textContent='DESTACADOS · '+(index+1)+' / '+cards.length;
    const heading=doc.createElement('h2');heading.textContent=title;
    const details=doc.createElement('p');details.textContent=current.querySelector('p')?.textContent??'';
    const open=button(doc,'Ver juego','game',id),image=current.querySelector('img')?.cloneNode();
    const nav=doc.createElement('div');nav.append(open);
    if(cards.length>1){for(const [name,step] of [['Anterior',-1],['Siguiente',1]]){const next=button(doc,name,'featureStep',String(step));nav.append(next);}}
    deck.replaceChildren(caption,heading,...(image?[image]:[]),details,nav);
  }
}
export function previewDocument(html,restricted){
  if(!endpoint&&!restricted)return html;
  const doc=new DOMParser().parseFromString(html,'text/html');
  doc.querySelector('#catalog .items')?.replaceChildren();
  restrictFixtureGames(doc);
  return '<!doctype html>'+doc.documentElement.outerHTML;
}
export async function attachCatalog(doc,studio,preset,editing,stillCurrent){
  if(!doc?.querySelector('#catalog'))return;
  if(!await prepare(studio,preset,stillCurrent))return;
  const choice=JSON.stringify(JSON.parse(studio.describe('lobby','')).catalog_selection);
  let menu=menuCache.get(studio);
  if(!menu||menu.choice!==choice){
    menu={choice,meta:JSON.parse(studio.catalog_metadata())};
    menuCache.set(studio,menu);
  }
  const meta=menu.meta,state=stateFor(preset);
  if(!meta.providers.includes(state.provider))state.provider='';
  if(!meta.categories.includes(state.category))state.category='';
  if(!meta.groups.some(group=>group.id===state.group))state.group='';
  current={doc,studio,preset,state,editing,banner:null};
  classicShelf(doc,studio,meta);controls(doc,meta,state,studio);populate();updateMode(doc,state);
  if(doc.querySelector('[data-original-layout]'))current.banner=nativeBanner(doc,meta,state);
  else buildBanner(doc,meta,state);
}
export function catalogInput(doc,target){
  if(current?.doc!==doc)return false;
  const state=current.state;
  if(target.matches?.('[data-search],[data-classic-section] input[type=search],[data-classic-section] input[aria-label*="Buscar"]'))state.search=target.value;
  else if(target.dataset?.catalogFilter)state[target.dataset.catalogFilter]=target.value;
  else return false;
  state.shown=48;populate();return true;
}
export function catalogClick(doc,target,notice){
  if(current?.doc!==doc)return false;
  const state=current.state,b=target.closest?.('button');
  if(target.closest?.('[data-classic-section]')&&b){
    const label=(b.getAttribute('aria-label')||b.textContent).trim().toLocaleLowerCase('es');
    if(/^(buscar|búsqueda)/.test(label)){doc.querySelector('#catalog [data-search]')?.focus();return true;}
    if(/^(slots|casino en vivo|en vivo|deportes|deportivas)$/.test(label)){
      const pattern=label==='slots'?/slot/i:/vivo/.test(label)?/vivo|live/i:/sport|deport/i;
      state.category=JSON.parse(current.studio.catalog_metadata()).categories.find(name=>pattern.test(name))??label;
      state.search='';state.group='';state.provider='';state.top=false;state.favorites_only=false;state.recent_only=false;state.shown=48;
      populate();updateMode(doc,state);doc.querySelector('#catalog')?.scrollIntoView();return true;
    }
    if(/^(favoritos|recientes|todos|casino|inicio)$/.test(label)){
      b.dataset.catalogMode=label==='favoritos'?'favorites':label==='recientes'?'recent':'all';
      doc.querySelector('#catalog')?.scrollIntoView();
    }
  }
  if(b?.dataset.featureStep){
    const deck=b.closest('.account-feature'),items=Number(deck?.dataset.count)||0;
    if(deck&&items){deck.dataset.index=String(((Number(deck.dataset.index)||0)+Number(b.dataset.featureStep)+items)%items);
      classicShelf(doc,current.studio,JSON.parse(current.studio.catalog_metadata()));}return true;
  }
  if(b?.hasAttribute('data-catalog-menu')){
    const open=b.getAttribute('aria-expanded')!=='true';
    b.setAttribute('aria-expanded',String(open));doc.querySelector('.lobby-sidebar')?.classList.toggle('open',open);
    return true;
  }
  if(b?.dataset.resortCategory){const names={slots:['slots','slot'],vivo:['vivo','live','casino en vivo'],deportes:['deportes','sports'],crash:['crash']};const key=b.dataset.resortCategory;
    const options=JSON.parse(current.studio.catalog_metadata()).categories;
    state.category=options.find(name=>(names[key]??[]).includes(name.toLowerCase()))??key;
    state.favorites_only=false;state.recent_only=false;state.shown=48;populate();updateMode(doc,state);doc.querySelector('#catalog')?.scrollIntoView();return true;}
  if(b?.dataset.catalogProvider||b?.dataset.catalogGroup){
    const key=b.dataset.catalogProvider?'provider':'group',value=b.dataset.catalogProvider||b.dataset.catalogGroup;
    state[key]=state[key]===value?'':value;state.shown=48;populate();return true;
  }
  if(b?.dataset.catalogMode){
    const mode=b.dataset.catalogMode;state.favorites_only=mode==='favorites';state.recent_only=mode==='recent';state.top=mode==='top';
    if(mode==='all'){state.provider='';state.category='';state.group='';state.search='';for(const s of doc.querySelectorAll('[data-catalog-filter]'))s.value='';const q=doc.querySelector('[data-search]');if(q)q.value='';}
    state.shown=48;populate();updateMode(doc,state);return true;
  }
  if(b?.hasAttribute('data-catalog-more')){populate(false);return true;}
  if(b?.dataset.favorite){const id=b.dataset.favorite;state.favorites.has(id)?state.favorites.delete(id):state.favorites.add(id);populate();return true;}
  if(b?.hasAttribute('data-shelf-action')){doc.getElementById('catalog')?.scrollIntoView();return true;}
  if(b?.dataset.game){const id=b.dataset.game;state.recent=[id,...state.recent.filter(x=>x!==id)].slice(0,100);const recent=doc.querySelector('[data-catalog-mode=recent]');if(recent)recent.hidden=false;notice(b.closest('.catalog-card')?.querySelector('h3')?.textContent??'Juego','Vista de demostración local. No se inicia un motor de juego ni se realizan apuestas.');return true;}
  if(b?.dataset.bannerStep||b?.hasAttribute('data-banner-index')){
    if(current.banner){state.banner=b.dataset.bannerStep?state.banner+Number(b.dataset.bannerStep):Number(b.dataset.bannerIndex);current.banner.draw();}
    return true;
  }
  return false;
}
export async function exportCatalog(doc,studio,preset,stillCurrent){
  const items=doc.querySelector('#catalog .items');
  if(!items)return;
  if(!await prepare(studio,preset,stillCurrent))throw Error('La vista cambió durante la exportación. Volvé a exportar.');
  const selection=JSON.parse(studio.describe('lobby','')).catalog_selection;
  if(endpoint||selection?.games!=null||selection?.providers!=null)restrictFixtureGames(doc);
  const state=stateFor(preset),page=JSON.parse(studio.catalog_page(request(state,false,0)));
  items.innerHTML=page.html;
  if(doc.querySelector('[data-classic-section]'))classicShelf(doc,studio,JSON.parse(studio.catalog_metadata()));
  resortFeatured(doc,state);
  if(doc.querySelector('[data-original-layout]')){
    const meta=JSON.parse(studio.catalog_metadata());
    nativeControls(doc,meta,state,studio);nativeBanner(doc,meta,state);updateMode(doc,state);
    doc.querySelector('[data-catalog-count]').textContent=page.total+' juegos';
    doc.querySelector('[data-catalog-empty]').hidden=page.total!==0;
    doc.querySelector('[data-catalog-more]').hidden=true;
  }
}
