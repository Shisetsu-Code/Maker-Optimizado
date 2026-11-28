// DOM bridge for the Rust-owned catalogue. Session-only interaction state.
const states=new Map(),sources=new Map();
let current=null;
const installed=new WeakMap();
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
    studio.set_catalog(source);
    installed.set(studio,preset);
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
  group.querySelector('[data-catalog-count]').textContent=total?shown+' de '+total+' juegos':'No encontramos juegos.';
  group.querySelector('[data-catalog-more]').hidden=shown>=total;
  group.dataset.catalogReady='true';
}
function controls(doc,meta,state){
  const group=doc.querySelector('#catalog');if(!group)return;
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
export function previewDocument(html,restricted){
  if(!endpoint&&!restricted)return html;
  const doc=new DOMParser().parseFromString(html,'text/html');
  doc.querySelector('#catalog .items')?.replaceChildren();
  return '<!doctype html>'+doc.documentElement.outerHTML;
}
export async function attachCatalog(doc,studio,preset,editing,stillCurrent){
  if(!doc?.querySelector('#catalog'))return;
  if(!await prepare(studio,preset,stillCurrent))return;
  const meta=JSON.parse(studio.catalog_metadata()),state=stateFor(preset);
  current={doc,studio,preset,state,editing,banner:null};
  controls(doc,meta,state);populate();updateMode(doc,state);buildBanner(doc,meta,state);
}
export function catalogInput(doc,target){
  if(current?.doc!==doc)return false;
  const state=current.state;
  if(target.matches?.('[data-search]'))state.search=target.value;
  else if(target.dataset?.catalogFilter)state[target.dataset.catalogFilter]=target.value;
  else return false;
  state.shown=48;populate();return true;
}
export function catalogClick(doc,target,notice){
  if(current?.doc!==doc)return false;
  const state=current.state,b=target.closest?.('button');
  if(b?.hasAttribute('data-catalog-menu')){
    b.setAttribute('aria-expanded',String(b.getAttribute('aria-expanded')!=='true'));
    return true;
  }
  if(b?.dataset.catalogMode){
    const mode=b.dataset.catalogMode;state.favorites_only=mode==='favorites';state.recent_only=mode==='recent';state.top=mode==='top';
    if(mode==='all'){state.provider='';state.category='';state.group='';state.search='';for(const s of doc.querySelectorAll('[data-catalog-filter]'))s.value='';const q=doc.querySelector('[data-search]');if(q)q.value='';}
    state.shown=48;populate();updateMode(doc,state);return true;
  }
  if(b?.hasAttribute('data-catalog-more')){populate(false);return true;}
  if(b?.dataset.favorite){const id=b.dataset.favorite;state.favorites.has(id)?state.favorites.delete(id):state.favorites.add(id);populate();return true;}
  if(b?.dataset.game){const id=b.dataset.game;state.recent=[id,...state.recent.filter(x=>x!==id)].slice(0,100);notice(b.closest('.catalog-card')?.querySelector('h3')?.textContent??'Juego','Vista de demostración local. No se inicia un motor de juego ni se realizan apuestas.');return true;}
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
  items.innerHTML=JSON.parse(studio.catalog_page(request(stateFor(preset),false,0))).html;
}
