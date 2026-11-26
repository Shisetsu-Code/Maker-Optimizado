const assert=require('node:assert/strict');
const {chromium}=require('playwright');
(async()=>{
 const browser=await chromium.launch({channel:process.env.PLAYWRIGHT_CHANNEL||'msedge',headless:true});
 try{
  const context=await browser.newContext();
  const origin=process.env.DEMO_ORIGIN||'http://127.0.0.1:8787';
  const [first,second]=await Promise.all([context.newPage(),context.newPage()]);
  await Promise.all([first.goto(origin+'/muestra/juego-demo.html'),second.goto(origin+'/muestra/juego-demo.html')]);
  const load=page=>page.evaluate(async()=>{const {loadElementSnapshot}=await import('/muestra/element-store.mjs');return loadElementSnapshot();});
  const [one,two]=await Promise.all([load(first),load(second)]);
  assert.equal(one.version,two.version);
  const state={version:1,assets:{},pages:{'azul:lobby':[{target:{type:'text',selector:'#title',tag:'h1',original:'Title',slot:0,label:'Title'},text:'First edit'}]}};
  const written=await first.evaluate(async({state,version})=>{const {saveElementSnapshot}=await import('/muestra/element-store.mjs');return saveElementSnapshot(state,version);},{state,version:one.version});
  assert.notEqual(written.version,one.version);
  const conflict=await second.evaluate(async({state,version})=>{const {saveElementSnapshot}=await import('/muestra/element-store.mjs');try{await saveElementSnapshot(state,version);return 'unexpected success';}catch(e){return e.name;}},{state:{version:1,pages:{},assets:{}},version:two.version});
  assert.equal(conflict,'ConflictError');
  const latest=await load(second);
  assert.equal(latest.version,written.version);
  assert.equal(latest.state.pages['azul:lobby'][0].text,'First edit');
  console.log('PASS IndexedDB multi-tab compare-and-swap');
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
