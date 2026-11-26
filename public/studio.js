import init,{Studio} from './wasm/maker_studio.js';
import {load,save} from './storage.js';

const $=id=>document.getElementById(id);
const status=$('status'),frame=$('preview'),themeForm=$('theme'),inspector=$('inspector');
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
  put(themeForm,data.theme);
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
}
function attachFrame(){
  const doc=frame.contentDocument;if(!doc?.getElementById('surface'))return;
  doc.addEventListener('click',event=>{
    const node=event.target.closest?.('[data-component]');
    if(editing&&node){event.preventDefault();event.stopPropagation();choose(node.dataset.component);return;}
    if(event.target.closest?.('[data-demo]')){event.preventDefault();say('Vista de demostración local. No se crean cuentas ni se realizan apuestas.');}
  },true);
  doc.addEventListener('input',event=>{
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
function readTheme(){const f=Object.fromEntries(new FormData(themeForm));return{...f,radius:Number(f.radius)};}
function readPatch(){const f=new FormData(inspector),number=name=>f.get(name)===''?null:Number(f.get(name)),color=name=>String(f.get(name)||'')||null;return{title:String(f.get('title')),text:String(f.get('text')),button:String(f.get('button')),hidden:inspector.elements.hidden.checked,art:Number(f.get('art')),style:{color:color('color'),background:color('background'),radius:number('radius'),padding:number('padding'),font_size:number('font_size'),columns:number('columns')}};}
function randomId(){return 'component-'+crypto.randomUUID();}
async function imageData(file){
  if(!['image/png','image/jpeg','image/webp'].includes(file.type)||file.size>8*1024*1024)throw Error('Elegí JPG, PNG o WebP de hasta 8 MB.');
  const bitmap=await createImageBitmap(file);
  try{
    if(bitmap.width*bitmap.height>40_000_000)throw Error('La imagen tiene demasiados píxeles.');
    for(const [size,quality] of [[1600,.8],[1200,.7],[900,.65],[640,.6]]){
      const ratio=Math.min(1,size/Math.max(bitmap.width,bitmap.height)),canvas=document.createElement('canvas');
      canvas.width=Math.max(1,Math.round(bitmap.width*ratio));canvas.height=Math.max(1,Math.round(bitmap.height*ratio));
      canvas.getContext('2d').drawImage(bitmap,0,0,canvas.width,canvas.height);
      const url=canvas.toDataURL('image/webp',quality);
      if(url.length<=450000&&/^data:image\/(?:webp|png|jpeg);base64,/.test(url))return url;
    }
    throw Error('La imagen supera el límite. Reducí su tamaño.');
  }finally{bitmap.close();}
}
function bind(){
  $('preset').addEventListener('change',()=>{selected='';dispatch({type:'switch',preset:$('preset').value});});
  for(const view of ['lobby','panel'])$(view).onclick=()=>{screen=view;selected='';refresh();};
  for(const button of document.querySelectorAll('button[data-size]'))button.onclick=()=>{ $('canvas').dataset.size=button.dataset.size;for(const b of document.querySelectorAll('button[data-size]'))b.setAttribute('aria-pressed',String(b===button)); };
  $('mode').onclick=()=>{editing=!editing;$('mode').setAttribute('aria-pressed',String(editing));$('mode').textContent=editing?'Editar':'Explorar';refresh();};
  themeForm.onsubmit=event=>{event.preventDefault();dispatch({type:'theme',theme:readTheme()});};
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
    try{const file=event.target.files?.[0];if(!file)return;if(file.size>8*1024*1024)throw Error('El proyecto supera 8 MiB.');studio.import_project(await file.text());selected='';refresh();persist();}
    catch(error){say(String(error),true);}finally{event.target.value='';}
  };
  $('export-json').onclick=()=>{try{download(studio.snapshot(),'application/json','maker-project.json');}catch(error){say(String(error),true);}};
  $('export-html').onclick=()=>{try{download(studio.html(screen,false),'text/html;charset=utf-8','maker-'+screen+'.html');}catch(error){say(String(error),true);}};
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
