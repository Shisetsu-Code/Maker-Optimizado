import {test} from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,writeFile,realpath,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {createHash} from 'node:crypto';
import {inventory,planBindings} from '../scripts/thumbnail-manifest.mjs';
import {authorizedFile} from '../scripts/import-thumbnails.mjs';

const make=()=>new Map([
 ['a',{items:[{id:'item-a',provider:'Studio',title:'Game',asset:null},{id:'item-b',provider:'Studio',title:'Game',asset:'existing.webp'}]}],
 ['b',{items:[{id:'item-c',provider:'Studio',title:'Game',asset:null}]}]
]);
const entry={provider:'Studio',title:'Game',file:'game.png',sha256:'a'.repeat(64),targets:[{preset:'a',id:'item-a'},{preset:'b',id:'item-c'}]};
test('inventory and bindings require explicit matching identities and never overwrite',()=>{
 const catalogues=make();
 assert.deepEqual(inventory(catalogues).entries,[{provider:'Studio',title:'Game',targets:entry.targets}]);
 assert.equal(planBindings(catalogues,{version:1,entries:[entry]})[0].targets.length,2);
 assert.throws(()=>planBindings(catalogues,{version:1,entries:[{...entry,title:'Wrong'}]}));
 assert.throws(()=>planBindings(catalogues,{version:1,entries:[{...entry,targets:[{preset:'a',id:'item-b'}]}]}));
 assert.throws(()=>planBindings(catalogues,{version:1,entries:[entry,entry]}));
 assert.equal(catalogues.get('a').items[0].asset,null);
});
test('local source files require a matching digest and cannot escape the authorized directory',async()=>{
 const dir=await mkdtemp(join(tmpdir(),'maker-thumbnail-'));
 try{
  const root=await realpath(dir),bytes=Buffer.from('fixture');
  await writeFile(join(dir,'game.png'),bytes);
  const hash=createHash('sha256').update(bytes).digest('hex');
  assert.deepEqual(await authorizedFile(root,{file:'game.png',sha256:hash}),bytes);
  await assert.rejects(authorizedFile(root,{file:'../game.png',sha256:hash}));
  await assert.rejects(authorizedFile(root,{file:join(dir,'game.png'),sha256:hash}));
  await assert.rejects(authorizedFile(root,{file:'game.png',sha256:'a'.repeat(64)}));
 }finally{await rm(dir,{recursive:true,force:true});}
});
