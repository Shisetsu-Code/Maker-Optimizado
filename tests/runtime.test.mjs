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
  assert.equal((await fetch(origin+'/api/v1/validate',{method:'POST',headers:{'content-type':'application/json'},body:'x'.repeat(262145)})).status,413);
  assert.equal((await fetch(origin+'/api/v1/validate',{method:'POST',headers:{'content-type':'application/json'},body:'{}'})).status,422);
  assert.equal((await fetch(origin+'/api/v1/preview?template=absent')).status,404);
  const preview=await fetch(origin+'/api/v1/preview?template=ivory&view=panel');assert.equal(preview.status,200);assert.match(await preview.text(),/Panel de administración/);
  const wasm=await fetch(origin+'/wasm/maker_studio_bg.wasm');assert.equal(wasm.status,200);assert.match(wasm.headers.get('content-type'),/application\/wasm/);
});

test('security headers and strict validation contract',async()=>{
 const root=await fetch(origin);
 const csp=root.headers.get('content-security-policy')||'';
 assert.match(csp,/default-src 'none'/);
 assert.match(csp,/frame-ancestors 'self'/);
 assert.doesNotMatch(csp,/script-src[^;]*'unsafe-inline'/);
 assert.equal(root.headers.get('x-content-type-options'),'nosniff');
 assert.equal(root.headers.get('cross-origin-resource-policy'),'same-origin');
 const health=await fetch(origin+'/api/v1/health');
 assert.equal(health.headers.get('x-frame-options'),'DENY');
 assert.equal(health.headers.get('cache-control'),'no-store');
 assert.equal(health.headers.get('cross-origin-resource-policy'),'same-origin');
 assert.match(health.headers.get('content-security-policy'),/default-src 'none'/);
 const preview=await fetch(origin+'/api/v1/preview?template=ember');
 assert.equal(preview.headers.get('x-frame-options'),'SAMEORIGIN');
 assert.match(preview.headers.get('content-security-policy'),/default-src 'none'/);
 assert.equal((await fetch(origin+'/api/v1/validate',{method:'POST',body:'{}'})).status,415);
 const invalid=await fetch(origin+'/api/v1/validate',{method:'POST',headers:{'content-type':'application/json'},body:'{'});
 assert.equal(invalid.status,422);
});
