import {test} from 'node:test';
import assert from 'node:assert/strict';
import {chromium} from 'playwright';
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
