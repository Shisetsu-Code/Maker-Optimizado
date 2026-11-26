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
 studio.dispatch(JSON.stringify({type:'backdrop',image:'data:image/png;base64,YWJj'}));
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
