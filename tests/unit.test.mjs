import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import init,{Studio} from '../public/wasm/maker_studio.js';
await init({module_or_path:await readFile(new URL('../public/wasm/maker_studio_bg.wasm',import.meta.url))});
test('WASM renders all 17 independent presets',()=>{
  const studio=new Studio(),presets=JSON.parse(studio.presets());
  assert.equal(presets.length,17);
  for(const preset of presets){
    studio.dispatch(JSON.stringify({type:'switch',preset:preset.id}));
    assert.equal(JSON.parse(studio.describe('lobby','')).active,preset.id);
    for(const view of ['lobby','panel'])assert.match(studio.html(view,false),/id="surface"/);
  }
  studio.free();
});
test('WASM editing, export, undo, redo and atomic rejection',()=>{
  const studio=new Studio(),original=studio.snapshot();
  studio.dispatch(JSON.stringify({type:'add',view:'lobby',id:'test-text',kind:'text',parent:null}));
  assert.match(studio.html('lobby',false),/id="test-text"/);
  const patch={title:'<script>alert(1)</script>',text:'Prueba',button:'',hidden:false,art:0,style:{}};
  studio.dispatch(JSON.stringify({type:'edit',view:'lobby',id:'test-text',patch}));
  assert.match(studio.html('lobby',false),/&lt;script&gt;/);
  assert.doesNotMatch(studio.html('lobby',false),/<script>alert/);
  const state=studio.snapshot();
  assert.throws(()=>studio.dispatch(JSON.stringify({type:'image',view:'lobby',id:'test-text',image:'https://example.com/image.png'})));
  assert.equal(studio.snapshot(),state);
  assert.equal(studio.undo(),true);
  assert.equal(studio.redo(),true);
  studio.load(original);assert.equal(studio.snapshot(),original);
  studio.free();
});

test('library asset IDs cannot escape the static asset namespace',()=>{
  const studio=new Studio(),id='a'.repeat(40)+'.webp';
  const original=studio.snapshot();
  assert.throws(()=>studio.dispatch(JSON.stringify({type:'asset',view:'lobby',id:'hero',asset:'../../private.png'})));
  assert.equal(studio.snapshot(),original);
  studio.dispatch(JSON.stringify({type:'asset',view:'lobby',id:'hero',asset:id}));
  assert.match(studio.html('lobby',true),new RegExp('/media/objects/'+id));
  studio.free();
});

test('advanced appearance is shared, bounded and rendered by Rust',()=>{
 const studio=new Studio();
 studio.dispatch(JSON.stringify({type:'appearance',settings:{font:'barlow',scale:120,weight:700,radius:20,enabled:true,shade:60,fit:'contain',position:'top'}}));
 studio.dispatch(JSON.stringify({type:'backdrop',image:'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAIAAAADCAIAAAA2iEnWAAAAFElEQVR4nGNcZSTCwMDAxMDAgKAAEroA9oAvBx4AAAAASUVORK5CYII='}));
 studio.dispatch(JSON.stringify({type:'switch',preset:'cobalt'}));
 assert.match(studio.html('lobby',false),/font-barlow/);
 assert.match(studio.html('panel',false),/--text-scale:1\.2/);
 assert.match(studio.html('lobby',false),/page-backdrop/);
 const before=studio.snapshot();
 assert.throws(()=>studio.dispatch(JSON.stringify({type:'appearance',settings:{scale:200}})));
 assert.equal(studio.snapshot(),before);
 studio.dispatch(JSON.stringify({type:'edit',view:'lobby',id:'catalog',patch:{title:'Catálogo',text:'',button:'',hidden:false,art:0,style:{font_size:110,font_weight:900,align:'right',border_color:'#abcdef',border_width:12,width:80,height:500,fit:'contain',position:'top',placeholder:'Buscar a medida'}}}));
 const html=studio.html('lobby',false);assert.match(html,/--node-weight:900/);assert.match(html,/placeholder="Buscar a medida"/);assert.match(html,/min-height:500px/);
 studio.free();
});

test('legacy JSON import reports ambiguous rules instead of silently discarding them',()=>{
 const studio=new Studio(),old={version:2,ajustes:{font:'archivo',scale:115,weight:'600',shade:30},elementos:{version:1,assets:{},pages:{'bateupix:lobby':[{target:{type:'field',selector:'#bp-search',label:'Buscar'},text:'Buscar a medida'},{target:{type:'text',selector:'body>div:nth-of-type(19)',original:'Texto no encontrado',label:'Texto perdido'},text:'Nuevo'}]}}};
 const report=JSON.parse(studio.import_project(JSON.stringify(old)));
 assert.equal(report.converted,true);assert.equal(report.unresolved.length,1);
 studio.dispatch(JSON.stringify({type:'switch',preset:'violet'}));assert.match(studio.html('lobby',false),/placeholder="Buscar a medida"/);
 assert.match(studio.html('lobby',false),/font-archivo/);
 studio.free();
});

test('administrative preview uses isolated synthetic Rust state',()=>{
 const studio=new Studio();
 const navigate=section=>studio.panel_action(JSON.stringify({type:'navigate',section}));
 assert.equal((studio.panel_navigation().match(/data-panel-section=/g)||[]).length,12);
 navigate('support');
 studio.panel_action(JSON.stringify({type:'select',id:'conversation-1'}));
 studio.panel_action(JSON.stringify({type:'note',id:'conversation-1',text:'<script>local</script>'}));
 assert.match(studio.panel_content(),/&lt;script&gt;local&lt;\/script&gt;/);
 studio.panel_action(JSON.stringify({type:'toggle',id:'conversation-1'}));
 assert.match(studio.panel_content(),/Reabrir conversación/);
 navigate('settings');
 studio.panel_action(JSON.stringify({type:'maintenance',enabled:true}));
 assert.equal(JSON.parse(studio.panel_metadata()).maintenance,true);
 studio.panel_action(JSON.stringify({type:'reset'}));
 assert.equal(JSON.parse(studio.panel_metadata()).maintenance,false);
 studio.free();
});

test('untrusted raster headers and malformed legacy projects are rejected atomically',()=>{
 const studio=new Studio(),original=studio.snapshot();
 const pixel=Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAIAAAADCAIAAAA2iEnWAAAAFElEQVR4nGNcZSTCwMDAxMDAgKAAEroA9oAvBx4AAAAASUVORK5CYII=','base64');
 studio.check_upload(pixel,'image/png');
 assert.throws(()=>studio.check_upload(pixel,'image/webp'));
 const huge=Buffer.from(pixel);huge.writeUInt32BE(100000,16);
 assert.throws(()=>studio.check_upload(huge,'image/png'));
 assert.throws(()=>studio.dispatch(JSON.stringify({type:'backdrop',image:'data:image/png;base64,'+huge.toString('base64')})));
 assert.throws(()=>studio.dispatch(JSON.stringify({type:'image',view:'lobby',id:'hero',image:'data:image/png;base64,YWJj'})));
 const current=JSON.parse(original);current.appearance.settings.scale=200;current.font='barlow';
 assert.throws(()=>studio.import_project(JSON.stringify(current)));
 const legacy={font:'archivo',elementos:{pages:{'bateupix:lobby':Array.from({length:151},()=>({}))}}};
 assert.throws(()=>studio.import_project(JSON.stringify(legacy)));
 assert.equal(studio.snapshot(),original);
 studio.free();
});


test('reference geometry keeps valid CSS alignment properties',async()=>{
 for(const name of ['base','rail','rooms','rows','family','experience','editorial']){
  const css=await readFile('frontend/gallery/'+name+'.css','utf8');
  assert.doesNotMatch(css,/plaexperience-|spaexperience-/);
 }
});
