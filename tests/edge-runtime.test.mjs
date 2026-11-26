import {test} from 'node:test';
import assert from 'node:assert/strict';
const origin=process.env.DEMO_ORIGIN||'http://127.0.0.1:8787';
const endpoint='/api/muestra/juego-demo';
test('Rust HTTP contract and security headers',async()=>{
    const health=await fetch(origin+'/api/health');
    assert.equal(health.status,200);
    assert.match((await health.json()).version,/maker-rust/);
    assert.equal(health.headers.get('cache-control'),'no-store');
    assert.equal(health.headers.get('x-content-type-options'),'nosniff');
    assert.equal((await fetch(origin+'/api/unknown')).status,404);
    assert.equal((await fetch(origin+'/api/monitoreo')).status,200);
    const wrong=await fetch(origin+endpoint);
    assert.equal(wrong.status,405);
    assert.equal(wrong.headers.get('allow'),'POST');
    const head=await fetch(origin+'/api/health',{method:'HEAD'});
    assert.equal(head.status,200);
    assert.equal(await head.text(),'');
    assert.equal((await fetch(origin+endpoint,{method:'POST',body:'x'.repeat(4097)})).status,413);
    assert.equal((await fetch(origin+endpoint,{method:'POST',body:'not-json'})).status,400);
    assert.equal((await fetch(origin+endpoint,{method:'POST',body:JSON.stringify({demo:false,vendorCode:'v',gameCode:'g'})})).status,422);
    const response=await fetch(origin+endpoint,{method:'POST',body:JSON.stringify({demo:true,vendorCode:'v&x',gameCode:'g?x',channel:'mobile'})});
    assert.equal(response.status,200);
    const url=new URL((await response.json()).url);
    assert.equal(url.origin,origin);
    assert.equal(url.searchParams.get('vendor'),'v&x');
    assert.equal(url.searchParams.get('game'),'g?x');
});
test('WASM and all 17 template resources served',async()=>{
    const wasm=await fetch(origin+'/muestra/wasm/maker_browser_bg.wasm');
    assert.equal(wasm.status,200);
    assert.match(wasm.headers.get('content-type'),/application\/wasm/);
    const bytes=new Uint8Array(await wasm.arrayBuffer());
    assert.deepEqual([...bytes.slice(0,4)],[0,97,115,109]);
    for(const view of ['vista-socio-lobby','vista-socio-panel','vista-azul','vista-blaze','vista-brasa','resort','bateupix','cobraya','lunabet','multisala','pinaplay','celuapuestas40','bet30','casinotower30','fenarys','fortunasjk','ravenbet','luckrex']){
        const r=await fetch(origin+'/muestra/'+view+'.html');
        assert.equal(r.status,200,view);
        assert.match(r.headers.get('content-type'),/text\/html/,view);
    }
});
