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
   assert.equal(page.shown,Math.min(48,data.items.length));assert.equal(page.total,data.items.length);assert.match(page.html,/data-catalog-card/);
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

test('collection rows preserve the explicit manifest order',async()=>{
 const studio=new Studio();
 studio.dispatch(JSON.stringify({type:'switch',preset:'jade'}));
 const catalog=JSON.parse(await readFile('public/catalogs/jade.json','utf8'));
 studio.set_catalog(JSON.stringify(catalog));
 const group=catalog.groups[0];
 const page=JSON.parse(studio.catalog_page(JSON.stringify({group:group.id,group_order:true})));
 const ids=[...page.html.matchAll(/data-catalog-card="([^"]+)"/g)].map(m=>m[1]);
 assert.deepEqual(ids,group.items.slice(0,48));
 studio.free();
});
