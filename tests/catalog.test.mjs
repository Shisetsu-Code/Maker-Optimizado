import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import init,{Studio} from '../public/wasm/maker_studio.js';
await init({module_or_path:await readFile(new URL('../public/wasm/maker_studio_bg.wasm',import.meta.url))});
test('17 data catalogues validate and paginate in Rust',async()=>{
 const studio=new Studio();
 for(const preset of JSON.parse(studio.presets())){
   studio.dispatch(JSON.stringify({type:'switch',preset:preset.id}));
   const source=await readFile('public/catalogs/'+preset.id+'.json','utf8');
   studio.set_catalog(source);
   const data=JSON.parse(source),meta=JSON.parse(studio.catalog_metadata());
   assert.equal(meta.total,data.items.length);assert.ok(data.items.length>0,preset.id);
   const page=JSON.parse(studio.catalog_page('{}'));
   assert.ok(page.shown<=48);assert.equal(page.total,data.items.length);assert.match(page.html,/data-catalog-card/);
   const item=data.items.find(i=>!i.seed)??data.items[0];
   const id=item.seed??item.id;
   const selected=JSON.parse(studio.describe('lobby',id)).selected;assert.ok(selected.catalog_item);
   studio.dispatch(JSON.stringify({type:'edit',view:'lobby',id,patch:{title:'Título editado',text:'Texto',button:'Ver',hidden:false,art:0,style:{}}}));
   const filtered=JSON.parse(studio.catalog_page(JSON.stringify({search:'Título editado'})));
   assert.match(filtered.html,/Título editado/);
   studio.dispatch(JSON.stringify({type:'restore',view:'lobby',id}));
   const restored=JSON.parse(studio.describe('lobby',id)).selected;
   assert.equal(restored.title,item.title);
 }
 studio.free();
});

test('stored catalog selection intersects provider and game IDs before all queries',async()=>{
 const studio=new Studio();
 try{
  const source=await readFile('public/catalogs/violet.json','utf8');
  studio.dispatch(JSON.stringify({type:'switch',preset:'violet'}));
  studio.set_catalog(source);
  const data=JSON.parse(source),one=data.items[0],other=data.items.find(item=>item.provider!==one.provider);
  assert.ok(other);
  const select=selection=>studio.dispatch(JSON.stringify({type:'select_catalog',selection}));
  const total=(query={})=>JSON.parse(studio.catalog_page(JSON.stringify(query))).total;
  select({providers:[one.provider],games:[one.id,other.id]});
  assert.equal(total(),1);
  assert.equal(total({favorites_only:true,favorites:[other.id]}),0);
  assert.equal(total({recent_only:true,recent:[one.id,other.id]}),1);
  const saved=studio.snapshot();
  select({providers:[],games:null});
  assert.equal(total(),0);
  assert.equal(studio.undo(),true);
  assert.equal(total(),1);
  studio.load(saved);
  assert.equal(total(),1);
  select({providers:null,games:null});
  assert.equal(total(),data.items.length);
  const old=JSON.parse(saved);
  delete old.designs.violet.catalog_selection;
  studio.load(JSON.stringify(old));
  assert.equal(total(),data.items.length);
 }finally{studio.free();}
});
