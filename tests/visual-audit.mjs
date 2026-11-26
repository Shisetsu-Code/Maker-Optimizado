import {chromium} from 'playwright';
import {mkdir,writeFile} from 'node:fs/promises';
import assert from 'node:assert/strict';

const ORIGIN=process.env.DEMO_ORIGIN||'http://127.0.0.1:8787';
const LEGACY=process.env.LEGACY_ORIGIN||'http://127.0.0.1:8788';
const DIR=process.env.VISUAL_OUTPUT||'work/visual-parity';
const templates=[
 ['ember','vista-socio-lobby'],['cobalt','vista-azul'],['flare','vista-blaze'],['coal','vista-brasa'],['ivory','resort'],
 ['violet','bateupix'],['saffron','cobraya'],['midnight','lunabet'],['amethyst','multisala'],['jade','pinaplay'],
 ['rose','celuapuestas40'],['garnet','bet30'],['brass','casinotower30'],['aurum','fenarys'],['verdant','fortunasjk'],
 ['crimson','ravenbet'],['onyx','luckrex']
];
const sizes=[['desktop',1440,900],['mobile',393,852]];
await mkdir(DIR,{recursive:true});
const browser=await chromium.launch({headless:true});
const report={reference:'9bc456f6ed1b4344a0cf4fb2e84ca4791061ba10',revision:process.env.GITHUB_SHA??'local',threshold:0.01,cases:[]};
const worker=await browser.newPage();
async function difference(before,after){
 return worker.evaluate(async({before,after})=>{
  async function pixels(encoded){
   const im=new Image();im.src='data:image/png;base64,'+encoded;await im.decode();
   const canvas=document.createElement('canvas');canvas.width=im.width;canvas.height=im.height;
   const ctx=canvas.getContext('2d',{willReadFrequently:true});ctx.drawImage(im,0,0);
   return {width:im.width,height:im.height,data:ctx.getImageData(0,0,im.width,im.height).data};
  }
  const a=await pixels(before),b=await pixels(after);
  if(a.width!==b.width||a.height!==b.height)return {dimensionMismatch:true,changedFraction:null};
  const canvas=document.createElement('canvas');canvas.width=a.width;canvas.height=a.height;
  const ctx=canvas.getContext('2d'),out=ctx.createImageData(a.width,a.height);let changed=0,sum=0;
  for(let i=0;i<a.data.length;i+=4){
   let max=0;for(let c=0;c<3;c++){const d=Math.abs(a.data[i+c]-b.data[i+c]);max=Math.max(max,d);sum+=d;}
   if(max>16){changed++;out.data[i]=255;out.data[i+1]=40;out.data[i+2]=65;}
   else{const gray=Math.round((a.data[i]+a.data[i+1]+a.data[i+2])/9);out.data[i]=gray;out.data[i+1]=gray;out.data[i+2]=gray;}
   out.data[i+3]=255;
  }
  ctx.putImageData(out,0,0);
  return {dimensionMismatch:false,changedFraction:changed/(a.width*a.height),meanChannelDifference:sum/(a.width*a.height*3),diff:canvas.toDataURL('image/png').split(',')[1]};
 },{before:before.toString('base64'),after:after.toString('base64')});
}
async function prepare(doc){
 await doc.evaluate(async()=>{
  await document.fonts.ready;
  const visible=[...document.images].filter(img=>{const r=img.getBoundingClientRect();return r.top<innerHeight&&r.bottom>0&&r.width>0&&r.height>0;});
  await Promise.race([Promise.allSettled(visible.map(img=>img.decode())),new Promise(r=>setTimeout(r,4000))]);
  window.scrollTo(0,0);
 });
}
try{
 for(const [size,width,height] of sizes){
  const context=await browser.newContext({viewport:{width,height},deviceScaleFactor:1,reducedMotion:'reduce',locale:'es-AR',serviceWorkers:'block'});
  let external=new Set(),errors=[];
  await context.route('**/*',async route=>{
   const url=new URL(route.request().url());
   if(!['http:','https:'].includes(url.protocol)||[ORIGIN,LEGACY].includes(url.origin))await route.continue();
   else{external.add(url.hostname);await route.abort();}
  });
  const original=await context.newPage(),fresh=await context.newPage();
  original.on('pageerror',e=>errors.push('reference: '+e.message));fresh.on('pageerror',e=>errors.push('new: '+e.message));
  for(const [preset,legacy] of templates){
   for(const view of ['lobby','panel']){
    external=new Set();errors=[];
    const key=preset+'-'+view+'-'+size;
    const result={preset,view,size,width,height,key};
    try{
     // Reference is the original local page, never an external site.
     const legacyPath=view==='panel'?'vista-socio-panel':legacy;
     await original.goto(LEGACY+'/muestra/'+legacyPath+'.html',{waitUntil:'domcontentloaded',timeout:15000});
     await original.waitForFunction(()=>document.body&&document.body.innerText.trim().length>60,{},{timeout:10000});
     await prepare(original);
     const before=await original.screenshot({animations:'disabled',caret:'hide',timeout:15000});
     await writeFile(DIR+'/'+key+'-reference.png',before);
     await fresh.goto(ORIGIN,{waitUntil:'domcontentloaded'});
     await fresh.locator('html[data-ready=true]').waitFor({timeout:15000});
     await fresh.locator('#preset').selectOption(preset);
     if(view==='panel')await fresh.locator('#panel').click();
     await fresh.locator('#full-preview').click();
     // Match the visible viewport exactly, independent of editor chrome.
     await fresh.addStyleTag({content:'body,html{margin:0!important;overflow:hidden!important}.studio,.workspace,.canvas{display:block!important;width:100vw!important;height:100vh!important;padding:0!important;margin:0!important;overflow:hidden!important}.canvas-toolbar{display:none!important}#preview{display:block!important;position:fixed!important;inset:0!important;width:100vw!important;height:100vh!important;max-width:none!important;border:0!important;border-radius:0!important;box-shadow:none!important}'});
     const iframe=fresh.frameLocator('#preview');
     await iframe.locator('html[data-preset="'+preset+'"][data-screen="'+view+'"]').waitFor({timeout:15000});
     if(view==='lobby')await iframe.locator('#catalog[data-catalog-preset="'+preset+'"]').waitFor({timeout:15000});
     const content=await fresh.locator('#preview').contentFrame();
     // frameLocator provides locator/evaluate access in all supported Playwright versions.
     await iframe.locator('html').evaluate(async()=>{
       await document.fonts.ready;
       await Promise.race([Promise.allSettled([...document.images].filter(img=>img.getBoundingClientRect().top<innerHeight).map(img=>img.decode())),new Promise(r=>setTimeout(r,4000))]);
       window.scrollTo(0,0);
     });
     const after=await fresh.screenshot({animations:'disabled',caret:'hide',timeout:15000});
     await writeFile(DIR+'/'+key+'-new.png',after);
     const delta=await difference(before,after);
     if(delta.diff){await writeFile(DIR+'/'+key+'-diff.png',Buffer.from(delta.diff,'base64'));delete delta.diff;}
     Object.assign(result,delta);
     result.pass=!delta.dimensionMismatch&&delta.changedFraction<=report.threshold;
    }catch(error){result.error=String(error);result.pass=false;}
    result.blockedReferenceHosts=[...external].sort();result.pageErrors=errors.slice(0,20);
    report.cases.push(result);
    console.log(key,result.error??((result.changedFraction??1)*100).toFixed(2)+'% changed');
   }
  }
  await context.close();
 }
}finally{await browser.close();}
report.total=report.cases.length;report.passed=report.cases.filter(c=>c.pass).length;
report.failed=report.total-report.passed;report.captureErrors=report.cases.filter(c=>c.error).length;
await writeFile(DIR+'/report.json',JSON.stringify(report,null,2)+'\n');
const table=['# Visual parity report','','Reference: '+report.reference,'','| Preset | View | Size | Pixels changed | Status |','|---|---|---|---:|---|',...report.cases.map(c=>'| '+c.preset+' | '+c.view+' | '+c.size+' | '+(c.changedFraction==null?'N/A':(c.changedFraction*100).toFixed(2)+'%')+' | '+(c.pass?'PASS':c.error?'CAPTURE ERROR':'DIFF')+' |')].join('\n');
await writeFile(DIR+'/report.md',table+'\n');
assert.equal(report.total,68);
if(process.env.GITHUB_STEP_SUMMARY)await writeFile(process.env.GITHUB_STEP_SUMMARY,table+'\n');
if(process.env.REQUIRE_VISUAL_PARITY==='1')assert.equal(report.failed,0,'Visual parity is not satisfied');
