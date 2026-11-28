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
  await page.locator('#panel').click();await preview.getByRole('heading',{name:'Inicio',exact:true}).waitFor();
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
   const hero=preview.locator('#hero img').first();
   await hero.waitFor();
   await preview.locator('#hero img[src="/media/objects/'+art.hero+'"]').waitFor();
   assert.ok(await hero.evaluate(async img=>{await img.decode();return img.naturalWidth>0&&img.naturalHeight>0;}),preset);
   await preview.locator('#catalog[data-catalog-ready=true]').waitFor();
   const cards=preview.locator('#catalog img');
   assert.ok(await cards.count()>0,preset);
   assert.ok(await cards.evaluateAll(async imgs=>{await Promise.all(imgs.map(img=>img.decode()));return imgs.every(img=>img.naturalWidth>0);}),preset);
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
  await frame.locator('#catalog[data-catalog-ready=true]').waitFor();
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
  await frame.locator('#catalog [data-catalog-mode=favorites]').click();assert.equal(await cards.count(),1);
  await cards.first().locator('[data-game]').click();await page.locator('#demo-dialog[open]').waitFor();
  await page.locator('#close-demo').click();
  await frame.locator('#catalog [data-catalog-mode=recent]').click();assert.equal(await cards.count(),1);
  await frame.locator('#catalog [data-catalog-mode=all]').click();
  await frame.locator('[data-search]').fill('Tarjeta personalizada');assert.equal(await cards.count(),1);
  await page.locator('#full-preview').click();assert.equal(await page.locator('body').getAttribute('data-full-preview'),'true');
  await page.locator('#exit-preview').click();assert.equal(await page.locator('body').getAttribute('data-full-preview'),null);
 }finally{await browser.close();}
});

test('export waits for session catalogue and respects an empty selection',async()=>{
 const browser=await chromium.launch();
 try{
  const page=await browser.newPage({acceptDownloads:true});
  const actual=await readFile('public/index.html','utf8');
  await page.route(origin+'/',route=>route.fulfill({contentType:'text/html',body:actual.replace('name="maker-catalog-endpoint" content=""','name="maker-catalog-endpoint" content="/api/account/catalog"')}));
  let release;
  const pending=new Promise(resolve=>{release=resolve;});
  await page.route('**/api/account/catalog?*',async route=>{
   const preset=new URL(route.request().url()).searchParams.get('preset');
   await pending;
   await route.fulfill({json:{version:1,preset,items:[],groups:[],banners:[]}});
  });
  await page.goto(origin,{waitUntil:'domcontentloaded'});
  await page.locator('html[data-ready=true]').waitFor();
  const download=page.waitForEvent('download');
  await page.locator('#export-html').click();
  assert.equal(await page.locator('#export-html').isDisabled(),true);
  release();
  const file=await download;
  const html=await readFile(await file.path(),'utf8');
  const count=await page.evaluate(source=>new DOMParser().parseFromString(source,'text/html').querySelector('#catalog .items').children.length,html);
  assert.equal(count,0);
 }finally{await browser.close();}
});
