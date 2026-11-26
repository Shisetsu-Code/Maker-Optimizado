import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFile,readdir,stat} from 'node:fs/promises';
import {createHash} from 'node:crypto';
const readJson=async(path)=>JSON.parse(await readFile(path,'utf8'));
const idPattern=/^[a-f0-9]{40}\.(?:webp|png|avif|jpg|gif)$/;
const ROOT='public/media';
test('Every original graphic is byte-preserved, deduplicated and indexed',{timeout:120000},async()=>{
 const index=await readJson(ROOT+'/index.json'),provenance=await readJson('docs/asset-provenance.json'),defaults=await readJson('assets/preset-artwork.json');
 const originals=Object.values(provenance.files);
 assert.equal(originals.length,provenance.source_paths);
 const deployed=new Map(),archived=new Map();
 for(const entry of originals){
   const target=entry.served?deployed:archived;
   assert.equal(entry.git_blob,entry.id.split('.')[0]);
   if(target.has(entry.id))assert.equal(target.get(entry.id),entry.bytes);
   target.set(entry.id,entry.bytes);
 }
 assert.equal(provenance.unique_raster,deployed.size);
 assert.equal(provenance.archived_unique,archived.size);
 assert.equal([...deployed.values()].reduce((a,b)=>a+b,0),provenance.raster_bytes);
 const imported=await readJson('docs/imported-thumbnails.json');
 assert.equal(imported.version,1);
 for(const [id,meta] of Object.entries(imported.objects)){
   assert.match(id,idPattern);
   assert.ok(!deployed.has(id),'Imported object cannot overwrite original');
   assert.match(meta.sha256,/^[a-f0-9]{64}$/);
   assert.match(meta.source_sha256,/^[a-f0-9]{64}$/);
   const bytes=await readFile(ROOT+'/objects/'+id);
   assert.equal(createHash('sha256').update(bytes).digest('hex'),meta.sha256);
   deployed.set(id,meta.bytes);
 }
 assert.equal(index.unique,deployed.size);
 assert.deepEqual((await readdir(ROOT+'/objects')).sort(),[...deployed.keys()].sort());
 assert.deepEqual((await readdir('assets/archive')).sort(),[...archived.keys()].sort());
 async function verify(directory,entries){
   for(const [id,size] of entries){
     assert.equal((await stat(directory+'/'+id)).size,size,id);
     const content=await readFile(directory+'/'+id);
     const hash=createHash('sha1').update('blob '+content.length+'\0').update(content).digest('hex');
     assert.equal(hash,id.split('.')[0],id);
   }
 }
 await verify(ROOT+'/objects',deployed);
 await verify('assets/archive',archived);
 assert.equal(Object.keys(defaults).length,17);
 assert.equal(index.collections.length,19);
 for(const col of index.collections){
   assert.match(col.id,/^[a-z0-9-]{1,40}$/);
   const list=await readJson(ROOT+'/collections/'+col.id+'.json');
   assert.equal(new Set(list).size,list.length);
   assert.equal(col.count,list.length);
   for(const id of list){assert.match(id,idPattern);assert.ok(deployed.has(id),id);}
   if(col.id==='all')assert.deepEqual([...list].sort(),[...deployed.keys()].sort());
 }
 for(const [preset,art] of Object.entries(defaults)){
   const ids=[art.hero,...art.categories,...art.cards.map(c=>c.asset),...art.live.map(c=>c.asset)];
   for(const id of ids)assert.ok(deployed.has(id),preset+': '+id);
 }
 console.log('Verified '+originals.length+' original paths, '+deployed.size+' raster objects, '+archived.size+' archived objects.');
});

test('All catalogue references resolve to local graphics; missing coverage is explicit',async()=>{
 const inventory=await readJson('docs/catalog-parity.json');
 const ids=new Set(await readdir(ROOT+'/objects'));
 const paths=(await readdir('public/catalogs')).filter(p=>p.endsWith('.json')).sort();
 assert.deepEqual(paths,Object.keys(inventory.catalogs).map(p=>p+'.json').sort());
 const missingIdentities=new Set();let missing=0,records=0;
 for(const path of paths){
   const catalogue=await readJson('public/catalogs/'+path);
   const report=inventory.catalogs[catalogue.preset];
   assert.equal(catalogue.items.length,report.normalized_records,path);
   assert.equal(catalogue.items.length,report.source_records,path);
   assert.equal(catalogue.items.filter(i=>i.asset===null).length,report.without_local_image,path);
   for(const item of catalogue.items){
     records++;
     if(item.asset===null){
       missing++;missingIdentities.add(JSON.stringify([item.provider,item.title]));
       continue;
     }
     assert.match(item.asset,idPattern);
     assert.ok(ids.has(item.asset),path+': '+item.id+' has a dangling graphic');
   }
   for(const banner of catalogue.banners){
     assert.ok(ids.has(banner.desktop),path+': missing desktop banner');
     if(banner.mobile!==null)assert.ok(ids.has(banner.mobile),path+': missing mobile banner');
   }
 }
 console.log('Catalogue coverage: '+records+' records, '+missing+' missing graphic references, '+missingIdentities.size+' distinct exact title/provider pairs.');
});
