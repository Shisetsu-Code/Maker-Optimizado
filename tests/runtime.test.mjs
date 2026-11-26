import {test} from 'node:test';
import assert from 'node:assert/strict';
const origin=process.env.DEMO_ORIGIN||'http://127.0.0.1:8787';
test('Worker HTTP contract and static assets',async()=>{
  const root=await fetch(origin);assert.equal(root.status,200);assert.match(await root.text(),/Maker Studio/);
  const health=await fetch(origin+'/api/v1/health');assert.equal(health.status,200);assert.equal((await health.json()).runtime,'rust-wasm');
  const templates=await fetch(origin+'/api/v1/templates');assert.equal((await templates.json()).templates.length,17);
  assert.equal((await fetch(origin+'/api/unknown')).status,404);
  assert.equal((await fetch(origin+'/api/v1/health',{method:'POST'})).status,405);
  const head=await fetch(origin+'/api/v1/health',{method:'HEAD'});assert.equal(head.status,200);assert.equal(await head.text(),'');
  assert.equal((await fetch(origin+'/api/v1/validate',{method:'POST',body:'x'.repeat(262145)})).status,413);
  assert.equal((await fetch(origin+'/api/v1/validate',{method:'POST',body:'{}'})).status,422);
  assert.equal((await fetch(origin+'/api/v1/preview?template=absent')).status,404);
  const preview=await fetch(origin+'/api/v1/preview?template=ivory&view=panel');assert.equal(preview.status,200);assert.match(await preview.text(),/Panel de administración/);
  const wasm=await fetch(origin+'/wasm/maker_studio_bg.wasm');assert.equal(wasm.status,200);assert.match(wasm.headers.get('content-type'),/application\/wasm/);
});
