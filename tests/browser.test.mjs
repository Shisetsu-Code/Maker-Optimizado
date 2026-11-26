import {test} from 'node:test';
import assert from 'node:assert/strict';
import {chromium} from 'playwright';
import {readFile} from 'node:fs/promises';
const origin=process.env.DEMO_ORIGIN||'http://127.0.0.1:8787';
test('all presets, editing, mobile, persistence, export and conflict',{timeout:120000},async()=>{
 const browser=await chromium.launch({headless:true});
 try{
  const context=await browser.newContext({viewport:{width:1440,height:900},acceptDownloads:true});
  const page=await context.newPage(),errors=[],external=[];
  page.on('pageerror',e=>errors.push(e.message));page.on('request',r=>{if(/^https?:/.test(r.url())&&new URL(r.url()).origin!==origin)external.push(r.url());});
  await page.goto(origin);await page.locator('html[data-ready=true]').waitFor();
  const preview=page.frameLocator('#preview');
  const ids=await page.locator('#preset option').evaluateAll(options=>options.map(o=>o.value));
  assert.equal(ids.length,17);
  for(const id of ids){await page.locator('#preset').selectOption(id);await preview.locator('#surface').waitFor();await page.locator('#status').filter({hasText:'guardado localmente'}).waitFor();}
  await page.locator('#preset').selectOption('ember');
  await page.locator('#outline [data-id=hero]').click();
  await page.locator('#inspector [name=title]').fill('Edición propia');
  await page.locator('#inspector button[type=submit]').click();
  await preview.getByRole('heading',{name:'Edición propia',exact:true}).waitFor();
  await page.locator('#undo').click();await preview.getByRole('heading',{name:'Un nuevo nivel de entretenimiento',exact:true}).waitFor();
  await page.locator('#redo').click();await preview.getByRole('heading',{name:'Edición propia',exact:true}).waitFor();
  await page.locator('#status').filter({hasText:'guardado localmente'}).waitFor();
  await page.locator('[data-size=mobile]').click();
  assert.ok(await page.locator('#preview').evaluate(el=>el.getBoundingClientRect().width<=394));
  await page.locator('#panel').click();await preview.getByRole('heading',{name:'Panel de administración',exact:true}).waitFor();
  await page.locator('#lobby').click();await preview.getByRole('heading',{name:'Edición propia',exact:true}).waitFor();
  const downloadPromise=page.waitForEvent('download');await page.locator('#export-html').click();assert.equal((await downloadPromise).suggestedFilename(),'maker-lobby.html');
  await page.reload();await page.locator('html[data-ready=true]').waitFor();await preview.getByRole('heading',{name:'Edición propia',exact:true}).waitFor();
  const other=await context.newPage();await other.goto(origin);await other.locator('html[data-ready=true]').waitFor();
  await page.locator('#outline [data-id=hero]').click();await page.locator('#inspector [name=title]').fill('Primera pestaña');await page.locator('#inspector button[type=submit]').click();await page.locator('#status').filter({hasText:'guardado localmente'}).waitFor();
  await other.locator('#outline [data-id=hero]').click();await other.locator('#inspector [name=title]').fill('Segunda pestaña');await other.locator('#inspector button[type=submit]').click();await other.locator('#status').filter({hasText:'Otra pestaña'}).waitFor();
  assert.deepEqual(errors,[]);assert.deepEqual(external,[]);
 }finally{await browser.close();}
});

test('all 17 graphic presets, library selection and offline HTML',{timeout:180000},async()=>{
 const browser=await chromium.launch({headless:true});
 try{
  const defaults=JSON.parse(await readFile('assets/preset-artwork.json','utf8'));
  const context=await browser.newContext({viewport:{width:1440,height:900},acceptDownloads:true});
  const page=await context.newPage(),errors=[],failed=[];
  page.on('pageerror',error=>errors.push(error.message));
  page.on('response',response=>{if(response.url().includes('/media/objects/')&&response.status()>=400)failed.push(response.url());});
  await page.goto(origin);
  await page.locator('html[data-ready=true]').waitFor();
  const preview=page.frameLocator('#preview');
  for(const [preset,art] of Object.entries(defaults)){
   await page.locator('#preset').selectOption(preset);
   const hero=preview.locator('#hero img[src="/media/objects/'+art.hero+'"]').first();
   await hero.waitFor();
   assert.ok(await hero.evaluate(async img=>{await img.decode();return img.naturalWidth>0&&img.naturalHeight>0;}),preset);
   await preview.locator('#catalog[data-catalog-ready=true][data-catalog-preset="'+preset+'"]').waitFor();
   const source=JSON.parse(await readFile('public/catalogs/'+preset+'.json','utf8'));
   const first=source.items.slice(0,48);
   const cards=preview.locator('#catalog .catalog-card');
   assert.equal(await cards.count(),first.length,preset);
   for(const [index,item] of first.entries()){
     const card=cards.nth(index);assert.equal(await card.getAttribute('data-catalog-card'),item.id,preset);
     if(item.asset){
       const img=card.locator('img');assert.equal(await img.getAttribute('src'),'/media/objects/'+item.asset,preset+': '+item.title);
       assert.ok(await img.evaluate(async el=>{await el.decode();return el.naturalWidth>0;}),preset+': '+item.title);
     }else{
       assert.equal(await card.locator('img').count(),0,preset+': missing original thumbnail');
     }
   }
   await page.locator('#status').filter({hasText:'guardado localmente'}).waitFor();
  }
  await page.locator('#preset').selectOption('ember');
  await page.locator('#outline [data-id=hero]').click();
  await page.locator('#library').click();
  await page.locator('#media-dialog[open]').waitFor();
  await page.locator('#media-collection').selectOption('cinematic');
  await page.locator('#media-page').filter({hasText:'28 imágenes'}).waitFor();
  assert.equal(await page.locator('#media-grid .media-card').count(),24);
  await page.locator('#media-next').click();
  assert.equal(await page.locator('#media-grid .media-card').count(),4);
  const card=page.locator('#media-grid .media-card').first();
  const src=await card.locator('img').getAttribute('src');
  await card.click();
  await preview.locator('#hero img[src="'+src+'"]').waitFor();
  await page.locator('#status').filter({hasText:'guardado localmente'}).waitFor();
  // Test that header graphic controls actually render the selected image.
  await page.locator('#outline [data-id=header]').click();
  await page.locator('#library').click();
  await page.locator('#media-dialog[open]').waitFor();
  await page.locator('#media-collection').selectOption('cinematic');
  await page.locator('#media-page').filter({hasText:'28 imágenes'}).waitFor();
  await page.locator('#media-grid .media-card').first().click();
  await preview.locator('#header .brand-art img').waitFor();
  await page.locator('#status').filter({hasText:'guardado localmente'}).waitFor();
  const pending=page.waitForEvent('download');
  await page.locator('#export-html').click();
  const download=await pending;
  const html=await readFile(await download.path(),'utf8');
  assert.match(html,/src="data:image\//);
  assert.doesNotMatch(html,/<img[^>]+src="\/media\/objects\//);
  await context.setOffline(true);
  const offline=await context.newPage();
  await offline.setContent(html);
  assert.ok(await offline.locator('#hero img').evaluate(async img=>{await img.decode();return img.naturalWidth>0;}));
  assert.ok(await offline.locator('#header img').evaluate(async img=>{await img.decode();return img.naturalWidth>0;}));
  assert.deepEqual(errors,[]);assert.deepEqual(failed,[]);
 }finally{await browser.close();}
});

test('full catalogue interactions and sparse editing',{timeout:120000},async()=>{
 const browser=await chromium.launch({headless:true});
 try{
  const page=await browser.newPage({viewport:{width:1440,height:900}});await page.goto(origin);
  await page.locator('html[data-ready=true]').waitFor();
  await page.locator('#preset').selectOption('violet');
  const frame=page.frameLocator('#preview');
  await frame.locator('#catalog[data-catalog-ready=true][data-catalog-preset=violet]').waitFor();
  const cards=frame.locator('#catalog .catalog-card');assert.equal(await cards.count(),48);
  const virtual=cards.nth(25);await virtual.locator('h3').click();
  await page.locator('#inspector [name=title]').fill('Tarjeta personalizada');
  await page.locator('#inspector button[type=submit]').click();
  await frame.locator('#catalog[data-catalog-ready=true]').waitFor();
  await frame.getByRole('heading',{name:'Tarjeta personalizada',exact:true}).waitFor();
  await page.locator('#mode').click();
  await frame.locator('#catalog[data-catalog-ready=true]').waitFor();
  await frame.locator('[data-catalog-more]').click();assert.equal(await cards.count(),96);
  await cards.first().locator('[data-favorite]').click();
  await frame.locator('#catalog .catalog-controls [data-catalog-mode=favorites]').click();assert.equal(await cards.count(),1);
  await cards.first().locator('[data-game]').click();await page.locator('#demo-dialog[open]').waitFor();
  await page.locator('#close-demo').click();
  const details=frame.locator('.gallery-extra-filters summary');
  if(await details.count())await details.click();
  await frame.locator('#catalog [data-catalog-mode=recent]').click();assert.equal(await cards.count(),1);
  await frame.locator('#catalog .catalog-controls [data-catalog-mode=all]').click();
  await frame.locator('[data-search]').fill('Tarjeta personalizada');assert.equal(await cards.count(),1);
  await page.locator('#full-preview').click();assert.equal(await page.locator('body').getAttribute('data-full-preview'),'true');
  await page.locator('#exit-preview').click();assert.equal(await page.locator('body').getAttribute('data-full-preview'),null);
 }finally{await browser.close();}
});

test('administrative local preview, notes, review, history and maintenance',{timeout:120000},async()=>{
 const browser=await chromium.launch({headless:true});
 try{
  const context=await browser.newContext({viewport:{width:1440,height:900}});
  const page=await context.newPage();page.on('dialog',dialog=>dialog.accept());
  await page.goto(origin);await page.locator('html[data-ready=true]').waitFor();
  await page.locator('#panel').click();await page.locator('#mode').click();
  const frame=page.frameLocator('#preview');
  await frame.locator('html[data-screen=panel][data-editing=false]').waitFor();
  await frame.locator('#surface[data-panel-ready=true]').waitFor();
  const nav=frame.locator('.panel-navigation');
  assert.equal(await nav.locator('button').count(),12);
  await nav.locator('[data-panel-section=support]').click();
  await frame.locator('[data-panel-row=conversation-1]').click();
  await frame.locator('[data-panel-note=conversation-1]').fill('Respuesta ficticia');
  await frame.locator('[data-panel-save-note=conversation-1]').click();
  assert.match(await frame.locator('.panel-note').textContent(),/Respuesta ficticia/);
  await frame.locator('[data-panel-toggle=conversation-1]').click();
  assert.match(await frame.locator('[data-panel-toggle=conversation-1]').textContent(),/Reabrir/);
  await nav.locator('[data-panel-section=requests]').click();
  await frame.locator('[data-panel-row=request-1]').click();
  await frame.locator('[data-panel-review=request-1]').click();
  assert.match(await frame.locator('[data-panel-row=request-1]').textContent(),/Revisada/);
  await nav.locator('[data-panel-section=records]').click();
  assert.match(await frame.locator('#panel-workspace').textContent(),/Revisión visual/);
  await nav.locator('[data-panel-section=settings]').click();
  await frame.locator('[data-panel-maintenance=true]').click();
  await page.locator('#lobby').click();
  await frame.getByRole('heading',{name:'Estamos en mantenimiento',exact:true}).waitFor();
  await page.locator('#panel').click();
  await frame.locator('[data-panel-maintenance=false]').click();
  await page.locator('#lobby').click();
  await frame.locator('#catalog[data-catalog-preset=ember]').waitFor();
  assert.equal(await frame.locator('.maintenance-preview').count(),0);
 }finally{await browser.close();}
});

test('HTML export freezes the selected project and waits for its catalogue',{timeout:120000},async()=>{
 const browser=await chromium.launch({headless:true});
 let release;
 try{
  const context=await browser.newContext({viewport:{width:1440,height:900},acceptDownloads:true});
  const page=await context.newPage();
  let requested;
  const started=new Promise(resolve=>{requested=resolve;});
  const gate=new Promise(resolve=>{release=resolve;});
  await page.route('**/catalogs/cobalt.json',async route=>{requested();await gate;await route.continue();});
  await page.goto(origin);await page.locator('html[data-ready=true]').waitFor();
  await page.locator('#preset').selectOption('cobalt');
  await started;
  const downloadPromise=page.waitForEvent('download');
  await page.locator('#export-html').click();
  await page.locator('#preset').selectOption('violet');
  release();
  const download=await downloadPromise;
  const html=await readFile(await download.path(),'utf8');
  const catalog=JSON.parse(await readFile('public/catalogs/cobalt.json','utf8'));
  assert.ok(html.includes('data-catalog-card="'+catalog.items[0].id+'"'));
  const other=JSON.parse(await readFile('public/catalogs/violet.json','utf8'));
  assert.ok(!html.includes('data-catalog-card="'+other.items[0].id+'"'));
  assert.equal(await page.locator('#preset').inputValue(),'violet');
  assert.doesNotMatch(html,/<img[^>]+src="\/media\/objects\//);
 }finally{release?.();await browser.close();}
});

test('reference cover profiles and mobile navigation operate in the new renderer',{timeout:120000},async()=>{
 const browser=await chromium.launch({headless:true});
 try{
  const page=await browser.newPage({viewport:{width:1440,height:900}});await page.goto(origin);
  await page.locator('html[data-ready=true]').waitFor();
  await page.locator('#preset').selectOption('violet');
  await page.locator('#full-preview').click();
  const frame=page.frameLocator('#preview');
  await frame.locator('#catalog[data-catalog-preset=violet]').waitFor();
  assert.equal(await frame.locator('body').getAttribute('data-profile'),'violet');
  assert.equal(await frame.locator('body').getAttribute('data-gallery'),'true');
  assert.equal(await frame.locator('#hero').evaluate(el=>getComputedStyle(el).aspectRatio),'2.75 / 1');
  const provider=frame.locator('.lobby-rail [data-provider-nav]').first();
  await provider.click();
  assert.equal(await provider.getAttribute('aria-pressed'),'true');
  await page.locator('[data-size=mobile]').click();
  await frame.locator('.lobby-mobile').waitFor({state:'visible'});
  await frame.locator('.lobby-mobile [data-catalog-mode=all][data-catalog-scroll=catalog]').click();
  assert.equal(await provider.getAttribute('aria-pressed'),'false');
  await frame.locator('.lobby-mobile [data-catalog-mode=favorites]').click();
  assert.equal(await frame.locator('.catalog-controls [data-catalog-mode=favorites]').getAttribute('aria-pressed'),'true');
 }finally{await browser.close();}
});

test('offline thumbnail import re-encodes a validated image and rejects invalid headers',{timeout:30000},async()=>{
 const {encodeThumbnail}=await import('../scripts/import-thumbnails.mjs');
 const {default:init,Studio}=await import('../public/wasm/maker_studio.js');
 await init({module_or_path:await readFile('public/wasm/maker_studio_bg.wasm')});
 const studio=new Studio(),browser=await chromium.launch({headless:true});
 try{
  const context=await browser.newContext({offline:true});
  const page=await context.newPage();
  const png=Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAIAAAADCAIAAAA2iEnWAAAAFElEQVR4nGNcZSTCwMDAxMDAgKAAEroA9oAvBx4AAAAASUVORK5CYII=','base64');
  const result=await encodeThumbnail(page,studio,png);
  assert.equal(result.subarray(8,12).toString(),'WEBP');
  for(const mime of ['image/jpeg','image/webp']){
    const encoded=await page.evaluate(type=>{const c=document.createElement('canvas');c.width=2;c.height=3;const ctx=c.getContext('2d');ctx.fillStyle='#ab3214';ctx.fillRect(0,0,2,3);return c.toDataURL(type).split(',')[1];},mime);
    const bytes=Buffer.from(encoded,'base64');
    studio.check_upload(bytes,mime);
    assert.equal((await encodeThumbnail(page,studio,bytes)).subarray(8,12).toString(),'WEBP');
  }
  const huge=Buffer.from(png);huge.writeUInt32BE(100000,16);
  await assert.rejects(encodeThumbnail(page,studio,huge));
 }finally{studio.free();await browser.close();}
});
