import { chromium } from 'playwright';
import { gzipSync } from 'node:zlib';
import { writeFile } from 'node:fs/promises';
const pages={ember:'vista-socio-lobby',cobalt:'vista-azul',flare:'vista-blaze',coal:'vista-brasa'};
const b=await chromium.launch({channel:'chrome'});
try{
 for(const [width,height] of [[1440,900],[393,798]])for(const [preset,path] of Object.entries(pages)){
  const context=await b.newContext({viewport:{width,height},locale:'es-AR',timezoneId:'UTC',reducedMotion:'reduce'});
  await context.route('**/*',r=>new URL(r.request().url()).hostname==='127.0.0.1'?r.continue():r.abort());
  await context.clock.setFixedTime(new Date('2026-11-26T15:00:00Z'));
  const page=await context.newPage();
  await page.goto('http://127.0.0.1:8788/muestra/'+path+'.html');
  await page.waitForFunction(()=>window.__demo?.listo===true);
  await page.waitForTimeout(2200);
  await page.waitForLoadState('networkidle',{timeout:10000}).catch(()=>{});
  const result=await page.evaluate(()=>{
    const root=document.body.cloneNode(true);
    root.querySelectorAll('script,style,link,iframe,object,embed,form').forEach(n=>n.remove());
    for(const el of root.querySelectorAll('*')){
      for(const a of [...el.attributes])if(a.name.startsWith('on')||a.name==='srcdoc')el.removeAttribute(a.name);
    }
    return {markup:root.innerHTML,styles:[...document.styleSheets].map(s=>({href:s.href,css:s.href?null:s.ownerNode.textContent})),root:document.documentElement.className,body:document.body.className,text:document.body.innerText};
  });
  const bytes=gzipSync(JSON.stringify(result));
  await writeFile(preset+'-'+width+'-source.json.gz',bytes);
  await context.close();
 }
}finally{await b.close();}
