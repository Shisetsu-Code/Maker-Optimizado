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
 assert.equal(index.unique,deployed.size);
 assert.equal(provenance.unique_raster,deployed.size);
 assert.equal(provenance.archived_unique,archived.size);
 assert.equal([...deployed.values()].reduce((a,b)=>a+b,0),provenance.raster_bytes);
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
