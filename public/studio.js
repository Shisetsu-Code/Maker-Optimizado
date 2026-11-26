import init,{Studio} from './wasm/maker_studio.js';
import {load,save} from './storage.js';
import {attachCatalog,catalogInput,catalogClick,exportCatalog} from './catalog.js';

const $=id=>document.getElementById(id);
const status=$('status'),frame=$('preview'),themeForm=$('theme'),appearanceForm=$('appearance'),inspector=$('inspector');
let mediaIndex,mediaItems=[],mediaPage=0,mediaTarget=null,mediaToken=0;
let studio,screen='lobby',selected='',editing=true,revision=null,queue=Promise.resolve(),conflict=false,dragged='',paintToken=0;
const say=(message,error=false)=>{status.textContent=message;status.dataset.error=String(error);};
const describe=()=>JSON.parse(studio.describe(screen,selected));
const dispatch=action=>{
  try{studio.dispatch(JSON.stringify(action));refresh();persist();}
  catch(error){say(String(error),true);}
};
function persist(){
  if(conflict){say('Conflicto local: exportá el proyecto y recargá.',true);return;}
  let snapshot;
  try{snapshot=studio.snapshot();}catch(error){say(String(error),true);return;}
  say('Guardando…');
  queue=queue.catch(()=>{}).then(async()=>{
    if(conflict)return;
    try{revision=await save(snapshot,revision);say('Proyecto guardado localmente.');}
    catch(error){if(error.name==='ConflictError')conflict=true;say(error.message,true);}
  });
}
function download(text,type,name){
  const url=URL.createObjectURL(new Blob([text],{type}));
  const link=document.createElement('a');link.href=url;link.download=name;link.click();
  setTimeout(()=>URL.revokeObjectURL(url),1500);
}
function put(form,values){for(const [name,value] of Object.entries(values)){const input=form.elements.namedItem(name);if(!input)continue;if(input.type==='checkbox')input.checked=Boolean(value);else input.value=value??'';}}
function choose(id){selected=id;paintControls();highlight();}
function highlight(){
  const doc=frame.contentDocument;if(!doc)return;
  for(const node of doc.querySelectorAll('[data-selected]'))node.removeAttribute('data-selected');
  if(editing&&selected)doc.getElementById(selected)?.setAttribute('data-selected','true');
}
function paintOutline(nodes,container,depth=0){
  for(const n of nodes){
    const button=document.createElement('button');button.type='button';button.className='layer';
    button.dataset.id=n.id;button.dataset.depth=String(depth);button.dataset.active=String(n.id===selected);button.dataset.hidden=String(n.hidden);
    button.draggable=true;
    const title=document.createElement('span');title.textContent=n.title||n.kind;
    const kind=document.createElement('span');kind.textContent=n.kind;
    button.append(title,kind);
    button.addEventListener('click',()=>choose(n.id));
    button.addEventListener('dragstart',event=>{dragged=n.id;event.dataTransfer.setData('text/plain',n.id);event.dataTransfer.effectAllowed='move';});
    button.addEventListener('dragover',event=>{event.preventDefault();button.dataset.drop='true';});
    button.addEventListener('dragleave',()=>delete button.dataset.drop);
    button.addEventListener('drop',event=>{event.preventDefault();delete button.dataset.drop;if(dragged)dispatch({type:'move_before',view:screen,id:dragged,before:n.id});dragged='';});
    button.addEventListener('dragend',()=>{dragged='';delete button.dataset.drop;});
    container.append(button);
    paintOutline(n.children,container,depth+1);
  }
}
function paintControls(){
  const data=describe();
  $('preset').value=data.active;
  put(themeForm,data.theme);put(appearanceForm,data.appearance.settings);$('clear-background').disabled=!data.appearance.has_image;
  $('undo').disabled=!data.undo;$('redo').disabled=!data.redo;
  $('lobby').setAttribute('aria-pressed',String(screen==='lobby'));$('panel').setAttribute('aria-pressed',String(screen==='panel'));
  const outline=$('outline');outline.replaceChildren();paintOutline(data.outline,outline);
  const n=data.selected;
  inspector.hidden=!n;$('empty').hidden=Boolean(n);
  if(!n){selected='';return;}
  $('component-name').textContent=n.kind+' · '+n.id;
  put(inspector,{title:n.title,text:n.text,button:n.button,hidden:n.hidden,art:n.art,...n.style});
  $('clear-image').disabled=!n.has_image;
  $('add-tile').hidden=!n.group;
  $('up').disabled=Boolean(n.catalog_item);$('down').disabled=Boolean(n.catalog_item);
}
function notice(title,message){
  $('demo-title').textContent=title;$('demo-message').textContent=message;
  if(!$('demo-dialog').open)$('demo-dialog').showModal();
}
function attachFrame(){
  const doc=frame.contentDocument;if(!doc?.getElementById('surface'))return;
  const preset=describe().active;
  if(screen==='lobby')attachCatalog(doc,studio,preset,()=>editing,()=>frame.contentDocument===doc&&screen==='lobby'&&describe().active===preset)
    .then(()=>highlight()).catch(error=>say(error.message??String(error),true));
  doc.addEventListener('click',event=>{
    const node=event.target.closest?.('[data-component]');
    if(editing&&node){event.preventDefault();event.stopPropagation();choose(node.dataset.component);return;}
    if(catalogClick(doc,event.target,notice)){event.preventDefault();return;}
    if(event.target.closest?.('[data-demo]')){event.preventDefault();notice('Demostración','Esta opción es una demostración. No se crean cuentas ni se realizan apuestas.');}
  },true);
  doc.addEventListener('change',event=>{if(event.target.matches?.('[data-catalog-filter]'))catalogInput(doc,event.target);});
  doc.addEventListener('input',event=>{
    if(event.target.matches?.('[data-search]')&&catalogInput(doc,event.target))return;
    if(!event.target.matches?.('[data-search]'))return;
    const term=event.target.value.trim().toLocaleLowerCase('es');
    const group=event.target.closest('.catalog');
    for(const tile of group?.querySelectorAll('.tile')??[])tile.hidden=!tile.textContent.toLocaleLowerCase('es').includes(term);
  });
  highlight();
}
frame.addEventListener('load',attachFrame);
function refresh(){
  if(!studio)return;
  const token=++paintToken;
  paintControls();
  const html=studio.html(screen,editing);
  if(token===paintToken)frame.srcdoc=html;
}
function readTheme(){const f=Object.fromEntries(new FormData(themeForm));return{...f,radius:Number(f.radius),button_color:f.button_color||null,glow_color:f.glow_color||null};}
function readPatch(){const f=new FormData(inspector),number=name=>f.get(name)===''?null:Number(f.get(name)),color=name=>String(f.get(name)||'')||null;return{title:String(f.get('title')),text:String(f.get('text')),button:String(f.get('button')),hidden:inspector.elements.hidden.checked,art:Number(f.get('art')),style:{color:color('color'),background:color('background'),radius:number('radius'),padding:number('padding'),font_size:number('font_size'),columns:number('columns'),font_weight:number('font_weight'),align:color('align'),border_color:color('border_color'),border_width:number('border_width'),width:number('width'),height:number('height'),fit:color('fit'),position:color('position'),placeholder:color('placeholder')}};}
function randomId(){return 'component-'+crypto.randomUUID();}
async function imageData(file,limit=600000){
  if(!['image/png','image/jpeg','image/webp'].includes(file.type)||file.size>8*1024*1024)throw Error('Elegí JPG, PNG o WebP de hasta 8 MB.');
  const bitmap=await createImageBitmap(file);
  try{
    if(bitmap.width*bitmap.height>40_000_000)throw Error('La imagen tiene demasiados píxeles.');
    for(const [size,quality] of [[1600,.8],[1200,.7],[900,.65],[640,.6]]){
      const ratio=Math.min(1,size/Math.max(bitmap.width,bitmap.height)),canvas=document.createElement('canvas');
      canvas.width=Math.max(1,Math.round(bitmap.width*ratio));canvas.height=Math.max(1,Math.round(bitmap.height*ratio));
      canvas.getContext('2d').drawImage(bitmap,0,0,canvas.width,canvas.height);
      const url=canvas.toDataURL('image/webp',quality);
      if(url.length<=limit&&/^data:image\/(?:webp|png|jpeg);base64,/.test(url))return url;
    }
    throw Error('La imagen supera el límite. Reducí su tamaño.');
  }finally{bitmap.close();}
}
const ASSET_ID=/^[a-f0-9]{40}\.(?:webp|png|avif|jpg|gif)$/;
const MEDIA_PAGE_SIZE=24;
function paintMedia(){
  const grid=$('media-grid');grid.replaceChildren();
  const start=mediaPage*MEDIA_PAGE_SIZE;
  for(const id of mediaItems.slice(start,start+MEDIA_PAGE_SIZE)){
    if(!ASSET_ID.test(id))continue;
    const button=document.createElement('button');button.type='button';button.className='media-card';
    button.setAttribute('aria-label','Seleccionar imagen '+id.slice(0,8));
    const img=document.createElement('img');img.src='/media/objects/'+id;img.alt='';img.loading='lazy';img.decoding='async';
    button.append(img);button.onclick=()=>{
      const target=mediaTarget;
      if(!target||selected!==target.id||screen!==target.view||describe().active!==target.template){$('media-dialog').close();say('La selección cambió.',true);return;}
      $('media-dialog').close();
      dispatch({type:'asset',view:target.view,id:target.id,asset:id});
    };grid.append(button);
  }
  const pages=Math.max(1,Math.ceil(mediaItems.length/MEDIA_PAGE_SIZE));
  $('media-page').textContent=(mediaPage+1)+' / '+pages+' · '+mediaItems.length+' imágenes';
  $('media-prev').disabled=mediaPage===0;$('media-next').disabled=mediaPage+1>=pages;
}
async function loadCollection(){
  const token=++mediaToken,id=$('media-collection').value;
  if(!/^[a-z0-9-]{1,40}$/.test(id))return;
  $('media-grid').replaceChildren();$('media-prev').disabled=true;$('media-next').disabled=true;
  $('media-page').textContent='Cargando…';
  const res=await fetch('/media/collections/'+id+'.json');
  if(!res.ok)throw Error('Colección no disponible.');
  const items=await res.json();
  if(token!==mediaToken)return;
  if(!Array.isArray(items))throw Error('Colección inválida.');
  mediaItems=items.filter(id=>typeof id==='string'&&ASSET_ID.test(id));mediaPage=0;paintMedia();
}
async function showLibrary(){
  if(!selected)return;
  mediaTarget={id:selected,view:screen,template:describe().active};
  try{
    if(!mediaIndex){const res=await fetch('/media/index.json');if(!res.ok)throw Error('Biblioteca no disponible.');mediaIndex=await res.json();}
    if(!mediaIndex.collections.length)throw Error('Biblioteca todavía no importada.');
    const select=$('media-collection');select.replaceChildren();
    for(const entry of mediaIndex.collections)select.add(new Option(entry.name+' ('+entry.count+')',entry.id));
    select.value=mediaIndex.collections.some(c=>c.id===mediaTarget.template)?mediaTarget.template:'all';
    await loadCollection();
    $('media-dialog').showModal();
  }catch(error){say(error.message??String(error),true);}
}
async function readAsset(id,budget){
  const response=await fetch('/media/objects/'+id);
  if(!response.ok)throw Error('No se encontró un recurso gráfico: '+id);
  const type=response.headers.get('content-type')?.split(';')[0]??'';
  if(!/^image\/(?:png|jpeg|webp|avif|gif)$/.test(type))throw Error('Formato de recurso inválido.');
  const length=Number(response.headers.get('content-length'));
  if(length>budget)throw Error('El HTML supera el límite de 24 MiB de imágenes.');
  const reader=response.body?.getReader();
  if(!reader)throw Error('Respuesta gráfica vacía.');
  let size=0;const chunks=[];
  try{
    while(true){
      const {value,done}=await reader.read();if(done)break;
      size+=value.byteLength;
      if(size>budget)throw Error('El HTML supera el límite de 24 MiB de imágenes.');
      chunks.push(value);
    }
  }finally{await reader.cancel().catch(()=>{});reader.releaseLock();}
  const blob=new Blob(chunks,{type});
  const url=await new Promise((resolve,reject)=>{const r=new FileReader();r.onload=()=>resolve(r.result);r.onerror=()=>reject(r.error);r.readAsDataURL(blob);});
  return {url,size};
}
async function standaloneHtml(view){
  // Work exclusively on trusted Rust-rendered HTML; never interpret imported text as markup.
  const doc=new DOMParser().parseFromString(studio.html(view,false),'text/html');
  if(view==='lobby')exportCatalog(doc,studio,describe().active);
  const images=new Map();
  for(const img of doc.querySelectorAll('img[src]')){
    const src=img.getAttribute('src');
    if(!src.startsWith('/media/objects/'))continue;
    const id=src.slice('/media/objects/'.length);
    if(!ASSET_ID.test(id))throw Error('Identificador gráfico inválido.');
    if(!images.has(id))images.set(id,[]);
    images.get(id).push(img);
  }
  let total=0;
  for(const [id,nodes] of images){
    const {url,size}=await readAsset(id,24*1024*1024-total);
    total+=size;
    for(const img of nodes)img.setAttribute('src',url);
  }
  // Embed the licensed local fonts so the static document also works offline.
  for(const style of doc.querySelectorAll('style')){
    const urls=[...new Set([...style.textContent.matchAll(/\/fonts\/[a-z0-9-]+-[a-f0-9]{8}\.woff2/g)].map(m=>m[0]))];
    for(const url of urls){
      const response=await fetch(url);if(!response.ok)throw Error('Fuente local no disponible.');
      const bytes=await response.arrayBuffer();if(bytes.byteLength>256000)throw Error('Fuente local demasiado grande.');
      const encoded=await new Promise((resolve,reject)=>{const r=new FileReader();r.onload=()=>resolve(r.result);r.onerror=()=>reject(r.error);r.readAsDataURL(new Blob([bytes],{type:'font/woff2'}));});
      style.textContent=style.textContent.replaceAll(url,encoded);
    }
    if(urls.length){
      const notices=await Promise.all(['barlow','archivo','oswald'].map(async name=>{const r=await fetch('/fonts/licenses/'+name+'.txt');if(!r.ok)throw Error('Licencia tipográfica no disponible.');return r.text();}));
      doc.head.append(doc.createComment(notices.join('\n\n').replaceAll('--','—')));
    }
  }
  const policy=doc.querySelector('meta[http-equiv="Content-Security-Policy"]');
  if(policy)policy.content=policy.content.replace("img-src 'self' data:","img-src data:").replace("font-src 'self' data:","font-src data:");
  return '<!doctype html>\n'+doc.documentElement.outerHTML;
}
function bind(){
  $('close-demo').onclick=()=>$('demo-dialog').close();
  $('restore-component').onclick=()=>{if(selected)dispatch({type:'restore',view:screen,id:selected});};
  $('reset-view').onclick=()=>{if(confirm('¿Restaurar esta pantalla?')){selected='';dispatch({type:'reset_view',view:screen});}};
  $('full-preview').onclick=()=>{document.body.dataset.fullPreview='true';editing=false;$('mode').setAttribute('aria-pressed','false');$('mode').textContent='Explorar';refresh();};
  $('exit-preview').onclick=()=>{delete document.body.dataset.fullPreview;};
  addEventListener('keydown',event=>{if(event.key==='Escape'&&!$('demo-dialog').open&&!$('media-dialog').open){delete document.body.dataset.fullPreview;if(editing){editing=false;$('mode').setAttribute('aria-pressed','false');$('mode').textContent='Explorar';refresh();}}});
  $('library').onclick=showLibrary;
  $('close-library').onclick=()=>$('media-dialog').close();
  $('media-collection').onchange=()=>loadCollection().catch(e=>say(String(e),true));
  $('media-prev').onclick=()=>{if(mediaPage>0){mediaPage--;paintMedia();}};
  $('media-next').onclick=()=>{if((mediaPage+1)*MEDIA_PAGE_SIZE<mediaItems.length){mediaPage++;paintMedia();}};

  $('preset').addEventListener('change',()=>{selected='';dispatch({type:'switch',preset:$('preset').value});});
  for(const view of ['lobby','panel'])$(view).onclick=()=>{screen=view;selected='';refresh();};
  for(const button of document.querySelectorAll('button[data-size]'))button.onclick=()=>{ $('canvas').dataset.size=button.dataset.size;for(const b of document.querySelectorAll('button[data-size]'))b.setAttribute('aria-pressed',String(b===button)); };
  $('mode').onclick=()=>{editing=!editing;$('mode').setAttribute('aria-pressed',String(editing));$('mode').textContent=editing?'Editar':'Explorar';refresh();};
  themeForm.onsubmit=event=>{event.preventDefault();dispatch({type:'theme',theme:readTheme()});};
  appearanceForm.onsubmit=event=>{
    event.preventDefault();const f=new FormData(appearanceForm);
    const optionalNumber=name=>f.get(name)===''?null:Number(f.get(name));
    dispatch({type:'appearance',settings:{font:f.get('font')||null,scale:Number(f.get('scale')),weight:optionalNumber('weight'),radius:optionalNumber('radius'),enabled:appearanceForm.elements.enabled.checked,shade:Number(f.get('shade')),fit:f.get('fit'),position:f.get('position')}});
  };
  $('background-file').onchange=async event=>{
    try{const file=event.target.files?.[0];if(file)dispatch({type:'backdrop',image:await imageData(file,1600000)});}
    catch(error){say(error.message??String(error),true);}finally{event.target.value='';}
  };
  $('clear-background').onclick=()=>dispatch({type:'backdrop',image:null});
  $('reset-appearance').onclick=()=>dispatch({type:'reset_appearance'});

  inspector.onsubmit=event=>{event.preventDefault();if(selected)dispatch({type:'edit',view:screen,id:selected,patch:readPatch()});};
  $('add-form').onsubmit=event=>{event.preventDefault();const id=randomId();selected=id;dispatch({type:'add',view:screen,parent:null,id,kind:$('kind').value});};
  $('add-tile').onclick=()=>{const parent=selected,id=randomId();selected=id;dispatch({type:'add',view:screen,parent,id,kind:'tile'});};
  $('delete').onclick=()=>{if(!selected)return;const id=selected;selected='';dispatch({type:'remove',view:screen,id});};
  $('up').onclick=()=>{if(selected)dispatch({type:'move',view:screen,id:selected,offset:-1});};
  $('down').onclick=()=>{if(selected)dispatch({type:'move',view:screen,id:selected,offset:1});};
  $('clear-image').onclick=()=>{if(selected)dispatch({type:'image',view:screen,id:selected,image:null});};
  $('image-file').onchange=async event=>{
    const file=event.target.files?.[0],id=selected,view=screen,template=describe().active;
    try{if(!file||!id)return;const data=await imageData(file);if(id!==selected||view!==screen||template!==describe().active)throw Error('Volvé a seleccionar el componente.');dispatch({type:'image',view,id,image:data});}
    catch(error){say(error.message??String(error),true);}finally{event.target.value='';}
  };
  $('undo').onclick=()=>{if(studio.undo()){selected='';refresh();persist();}};
  $('redo').onclick=()=>{if(studio.redo()){selected='';refresh();persist();}};
  $('reset').onclick=()=>{if(confirm('¿Restaurar la plantilla actual? Podés deshacer este cambio.')){selected='';dispatch({type:'reset'});}};
  $('import').onclick=()=>$('file').click();
  $('file').onchange=async event=>{
    try{const file=event.target.files?.[0];if(!file)return;if(file.size>24*1024*1024)throw Error('El proyecto supera 24 MiB.');studio.import_project(await file.text());selected='';refresh();persist();}
    catch(error){say(String(error),true);}finally{event.target.value='';}
  };
  $('export-json').onclick=()=>{try{download(studio.snapshot(),'application/json','maker-project.json');}catch(error){say(String(error),true);}};
  $('export-html').onclick=async()=>{const view=screen,button=$('export-html');button.disabled=true;try{say('Preparando HTML autónomo…');download(await standaloneHtml(view),'text/html;charset=utf-8','maker-'+view+'.html');say('HTML exportado.');}catch(error){say(String(error),true);}finally{button.disabled=false;}};
}
async function boot(){
  try{
    await init();
    studio=new Studio();
    for(const p of JSON.parse(studio.presets()))$('preset').add(new Option(p.name,p.id));
    for(let i=0;i<12;i++)inspector.elements.art.add(new Option('Estilo '+(i+1),String(i)));
    try{const saved=await load();revision=saved.revision;if(saved.json)studio.load(saved.json);}
    catch(error){say('No se pudo recuperar el proyecto: '+String(error),true);conflict=true;}
    bind();$('preset').disabled=false;refresh();
    document.documentElement.dataset.ready='true';
    if(!conflict)say('Motor Rust/WASM listo. Guardado local.');
  }catch(error){say('No se pudo iniciar Maker: '+String(error),true);}
}
boot();
