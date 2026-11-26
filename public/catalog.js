// DOM bridge for the Rust-owned catalogue. Session-only interaction state.
const states=new Map(),sources=new Map();
let current=null;
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
    top:state.top,preview,offset});
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
  group.querySelector('[data-catalog-count]').textContent=current.gallery?total+' juegos':total?shown+' de '+total+' juegos':'No encontramos juegos.';
  group.querySelector('[data-catalog-more]').hidden=shown>=total;
  const empty=group.querySelector('[data-catalog-empty]');if(empty)empty.hidden=total!==0;
  const title=group.querySelector('[data-catalog-title]');
  const resort=doc.body.hasAttribute('data-resort');
  if(title)title.textContent=state.favorites_only?(resort?'Tus favoritos':'Mis favoritos'):state.recent_only?'Recientes':state.top?'Juegos destacados':state.provider||current.meta.groups.find(g=>g.id===state.group)?.title||(resort&&state.category==='Crash'?'Una nueva emoción':state.category)||current.defaultTitle;
  if(resort){
    const spotlight=group.querySelector('#resort-spotlight');
    if(spotlight){spotlight.hidden=Boolean(state.search||state.provider||state.category||state.group||state.favorites_only||state.recent_only||state.top);spotlight.parentElement.classList.toggle('filtered',spotlight.hidden);}
  }
  group.dataset.catalogReady='true';group.dataset.catalogPreset=current.preset;
}
function controls(doc,meta,state){
  const group=doc.querySelector('#catalog');if(!group)return;
  const extra=group.querySelector('[data-extra-filters]');
  if(extra){
    extra.append(select(doc,'provider','Todos los proveedores',meta.providers,state.provider),
      select(doc,'category','Todas las categorías',meta.categories,state.category),
      select(doc,'group','Todas las colecciones',meta.groups,state.group),
      button(doc,'Recientes','catalogMode','recent'));
    const search=doc.querySelector('[data-search]');if(search)search.value=state.search;
    return;
  }
  const root=doc.createElement('div');root.className='catalog-controls';
  root.append(select(doc,'provider','Todos los proveedores',meta.providers,state.provider),
    select(doc,'category','Todas las categorías',meta.categories,state.category),
    select(doc,'group','Todas las colecciones',meta.groups,state.group));
  for(const [label,key] of [['Todos','all'],['Destacados','top'],['Favoritos','favorites'],['Recientes','recent']])
    root.append(button(doc,label,'catalogMode',key));
  const count=doc.createElement('span');count.dataset.catalogCount='true';count.setAttribute('role','status');root.append(count);
  group.querySelector('.items')?.before(root);
  const more=button(doc,'Cargar más','catalogMore','true');more.className='action catalog-more';group.append(more);
  const search=doc.querySelector('[data-search]');if(search)search.value=state.search;
}
function updateMode(doc,state){
  const mode=state.favorites_only?'favorites':state.recent_only?'recent':state.top?'top':'all';
  for(const b of doc.querySelectorAll('[data-catalog-mode]')){
    const active=b.dataset.catalogMode===mode&&!state.provider&&!state.category&&!state.group;
    b.setAttribute('aria-pressed',String(active));b.classList.toggle('active',active);
  }
  for(const b of doc.querySelectorAll('[data-experience]')){
    const active=(b.dataset.experience==='live'&&state.category==='En vivo')||(b.dataset.experience==='crash'&&state.category==='Crash')||(b.dataset.experience==='slots'&&state.category==='Slots')||(b.dataset.experience==='sports'&&state.category==='Deportes');
    b.setAttribute('aria-pressed',String(active));b.classList.toggle('active',active);
  }
  for(const b of doc.querySelectorAll('[data-provider-nav],[data-group-nav]')){
    const active=b.dataset.providerNav===state.provider||b.dataset.groupNav===state.group;
    b.setAttribute('aria-pressed',String(active));b.classList.toggle('active',active);
  }
}
function buildBanner(doc,meta,state){
  const gallery=doc.body.hasAttribute('data-gallery');
  const root=gallery?doc.querySelector('#hero.gallery-banner'):doc.querySelector('#hero .hero-art');
  const img=root?.querySelector('img');if(!root||!img||!meta.banners.length)return;
  const initial=img.getAttribute('src'),original=meta.banners.find(b=>'/media/objects/'+b.desktop===initial);
  const slides=[{desktop:initial,mobile:original?.mobile?'/media/objects/'+original.mobile:null},
    ...meta.banners.filter(b=>'/media/objects/'+b.desktop!==initial).map(b=>({desktop:'/media/objects/'+b.desktop,mobile:b.mobile?'/media/objects/'+b.mobile:null}))];
  if(!gallery&&slides.length<2&&!slides[0].mobile)return;
  let source=null;
  if(!gallery||slides.some(s=>s.mobile)){
    const picture=doc.createElement('picture');source=doc.createElement('source');source.media='(max-width: 700px)';
    img.before(picture);picture.append(source,img);
  }
  const nav=gallery?doc.querySelector('#gallery-dots'):doc.createElement('div');
  if(!nav)return;
  if(!gallery){
    nav.className='banner-navigation';
    nav.append(button(doc,'‹','bannerStep','-1'));
  }
  slides.forEach((_,i)=>{const b=button(doc,gallery?'':'•','bannerIndex',String(i));b.setAttribute('aria-label','Portada '+(i+1));nav.append(b);});
  if(!gallery){nav.append(button(doc,'›','bannerStep','1'));root.append(nav);}
  const draw=()=>{
    const index=((state.banner%slides.length)+slides.length)%slides.length,slide=slides[index];
    state.banner=index;img.src=slide.desktop;
    if(source){if(slide.mobile)source.srcset=slide.mobile;else source.removeAttribute('srcset');}
    for(const b of nav.querySelectorAll('[data-banner-index]')){
      const active=Number(b.dataset.bannerIndex)===index;b.setAttribute('aria-pressed',String(active));b.classList.toggle('active',active);
    }
  };
  current.banner={draw};draw();
}
function catalogSource(preset){
  if(!sources.has(preset)){
    const promise=fetch('/catalogs/'+preset+'.json').then(r=>{if(!r.ok)throw Error('Catálogo no disponible.');return r.text();});
    sources.set(preset,promise);promise.catch(()=>{if(sources.get(preset)===promise)sources.delete(preset);});
  }
  return sources.get(preset);
}
export async function attachCatalog(doc,studio,preset,editing,stillCurrent){
  if(!doc?.querySelector('#catalog'))return;
  const input=await catalogSource(preset);if(!stillCurrent())return;
  studio.set_catalog(input);
  const meta=JSON.parse(studio.catalog_metadata()),state=stateFor(preset);
  const gallery=doc.body.hasAttribute('data-gallery');
  current={doc,studio,preset,state,editing,meta,gallery,banner:null,defaultTitle:doc.querySelector('[data-catalog-title]')?.textContent??'Todos los juegos'};
  controls(doc,meta,state);
  const rail=doc.querySelector('[data-provider-rail]');
  if(rail)for(const provider of gallery?(meta.provider_order??meta.providers).slice(0,12):meta.providers)rail.append(button(doc,provider,'providerNav',provider));
  const groupRail=doc.querySelector('[data-group-rail]');
  if(groupRail){
    if(groupRail.id==='rows-collections'){
      // Collection cards are separate navigation summaries; only catalogue cards
      // have editable IDs. Rust validates and generates the displayed entries.
      for(const [index,group] of meta.groups.slice(0,6).entries()){
        const section=doc.createElement('section');section.className='rows-collection';
        const header=doc.createElement('div');header.className='rows-row-heading';
        const title=doc.createElement('h2');title.textContent=group.title;
        header.append(title,button(doc,'Ver todos →','groupNav',group.id));
        const row=doc.createElement('div');row.className='rows-row';
        const result=JSON.parse(studio.catalog_page(JSON.stringify({group:group.id,preview:false})));
        const fragment=doc.createElement('template');fragment.innerHTML=result.html;
        for(const [i,card] of [...fragment.content.querySelectorAll('[data-catalog-card]')].slice(0,12).entries()){
          const label=card.querySelector('h3')?.textContent??'Juego';
          const b=button(doc,'','game',card.dataset.catalogCard);b.className='rows-game';b.setAttribute('aria-label','Ver '+label);
          const img=card.querySelector('img');if(img)b.append(img.cloneNode(true));
          const span=doc.createElement('span');span.textContent=label;b.append(span);
          if(index===0){const num=doc.createElement('b');num.textContent=String(i+1).padStart(2,'0');b.append(num);}
          row.append(b);
        }
        section.append(header,row);groupRail.append(section);
      }
    }else for(const group of meta.groups)groupRail.append(button(doc,group.title,'groupNav',group.id));
  }
  populate();updateMode(doc,state);buildBanner(doc,meta,state);
}
export function catalogInput(doc,target){
  if(current?.doc!==doc)return false;
  const state=current.state;
  if(target.matches?.('[data-search]'))state.search=target.value;
  else if(target.dataset?.catalogFilter)state[target.dataset.catalogFilter]=target.value;
  else return false;
  state.shown=48;populate();updateMode(doc,state);return true;
}
function clearFilters(doc,state){
  state.provider='';state.category='';state.group='';state.search='';
  for(const s of doc.querySelectorAll('[data-catalog-filter]'))s.value='';
  const q=doc.querySelector('[data-search]');if(q)q.value='';
}
function scrollTarget(doc,where){
  if(where==='catalog')doc.querySelector('#catalog')?.scrollIntoView({block:'start'});
  else if(where==='top')doc.defaultView?.scrollTo(0,0);
}
export function catalogClick(doc,target,notice){
  if(current?.doc!==doc)return false;
  const {state,meta}=current,b=target.closest?.('button');
  if(b?.hasAttribute('data-gallery-menu')){
    const rail=doc.querySelector('.gallery-sidebar');
    if(rail){const open=rail.classList.toggle('open');b.setAttribute('aria-expanded',String(open));}
    return true;
  }
  if(b?.hasAttribute('data-group-nav')||b?.hasAttribute('data-experience')){
    const experience=b.dataset.experience;
    const pattern=experience==='live'?/vivo|live/i:experience==='crash'?/crash/i:experience==='sports'?/deportes|sports/i:/slot/i;
    const group=b.dataset.groupNav??(experience?meta.groups.find(g=>pattern.test(g.title))?.id:undefined);
    const category=experience==='live'?'En vivo':experience==='crash'?'Crash':experience==='sports'?'Deportes':'Slots';
    if(!group&&(!experience||(!meta.categories.includes(category)&&experience!=='sports'))){notice('Categoría','No hay una categoría equivalente en este catálogo.');return true;}
    clearFilters(doc,state);state.favorites_only=false;state.recent_only=false;state.top=false;
    if(group)state.group=group;else state.category=category;
    for(const s of doc.querySelectorAll('[data-catalog-filter]'))s.value=state[s.dataset.catalogFilter]??'';
    state.shown=48;populate();updateMode(doc,state);scrollTarget(doc,'catalog');return true;
  }
  if(b?.hasAttribute('data-category-nav')||b?.hasAttribute('data-provider-nav')){
    const label=b.dataset.categoryNav??'';
    const category=label==='Casino en vivo'?'En vivo':label;
    const modes={Destacados:'top',Favoritos:'favorites',Recientes:'recent',Todos:'all'};
    const mode=modes[label]??'all';
    if(b.hasAttribute('data-category-nav')&&!Object.hasOwn(modes,label)&&!meta.categories.includes(category)){
      notice('Categoría', 'No hay una categoría equivalente en este catálogo.');return true;
    }
    clearFilters(doc,state);
    state.favorites_only=mode==='favorites';state.recent_only=mode==='recent';state.top=mode==='top';
    if(b.hasAttribute('data-provider-nav'))state.provider=b.dataset.providerNav;
    else if(!Object.hasOwn(modes,label))state.category=category;
    for(const s of doc.querySelectorAll('[data-catalog-filter]'))s.value=state[s.dataset.catalogFilter]??'';
    state.shown=48;populate();updateMode(doc,state);scrollTarget(doc,'catalog');return true;
  }
  if(b?.dataset.catalogMode){
    const mode=b.dataset.catalogMode;state.favorites_only=mode==='favorites';state.recent_only=mode==='recent';state.top=mode==='top';
    if(mode==='all'||b.dataset.catalogScroll)clearFilters(doc,state);
    doc.querySelector('.gallery-sidebar')?.classList.remove('open');
    state.shown=48;populate();updateMode(doc,state);scrollTarget(doc,b.dataset.catalogScroll);return true;
  }
  if(b?.hasAttribute('data-catalog-more')){populate(false);return true;}
  if(b?.dataset.favorite){const id=b.dataset.favorite;state.favorites.has(id)?state.favorites.delete(id):state.favorites.add(id);populate();return true;}
  if(b?.dataset.game){const id=b.dataset.game;state.recent=[id,...state.recent.filter(x=>x!==id)].slice(0,100);notice(b.closest('.catalog-card')?.querySelector('h3')?.textContent??b.getAttribute('aria-label')?.replace(/^Ver /,'')??'Juego','Vista de demostración local. No se inicia un motor de juego ni se realizan apuestas.');return true;}
  if(b?.dataset.bannerStep||b?.hasAttribute('data-banner-index')){
    if(current.banner){state.banner=b.dataset.bannerStep?state.banner+Number(b.dataset.bannerStep):Number(b.dataset.bannerIndex);current.banner.draw();}
    return true;
  }
  return false;
}
// Capture the query before any await, independently of the visible iframe.
export function snapshotCatalogQuery(preset){
  return request(stateFor(preset),false,0);
}
export async function exportCatalog(doc,frozenStudio,preset,query){
  const items=doc.querySelector('#catalog .items');if(!items)return;
  frozenStudio.set_catalog(await catalogSource(preset));
  items.innerHTML=JSON.parse(frozenStudio.catalog_page(query)).html;
}
