import {chromium} from 'playwright';
import {mkdir,writeFile} from 'node:fs/promises';

const original='http://127.0.0.1:8788';
const candidate='http://127.0.0.1:8787';
const reference={
  ember:'vista-socio-lobby',cobalt:'vista-azul',flare:'vista-blaze',coal:'vista-brasa',ivory:'resort',
  violet:'bateupix',saffron:'cobraya',midnight:'lunabet',amethyst:'multisala',jade:'pinaplay',
  rose:'celuapuestas40',garnet:'bet30',brass:'casinotower30',aurum:'fenarys',verdant:'fortunasjk',
  crimson:'ravenbet',onyx:'luckrex',
};
const out='work/visual';
await mkdir(out,{recursive:true});
const browser=await chromium.launch();
const results=[];
try{
  const measure=await browser.newPage();
  for(const [name,width,height] of [['desktop',1440,900],['mobile',393,798]]){
    const context=await browser.newContext({viewport:{width,height},deviceScaleFactor:1,reducedMotion:'reduce',locale:'es-AR',timezoneId:'UTC'});
    await context.route('**/*',route=>{
      const url=new URL(route.request().url());
      return [original,candidate].includes(url.origin)||['data:','blob:'].includes(url.protocol)?route.continue():route.abort();
    });
    const old=await context.newPage(),editor=await context.newPage();
    await editor.goto(candidate);
    await editor.locator('html[data-ready=true]').waitFor();
    await editor.locator('#mode').click();
    for(const [preset,slug] of Object.entries(reference)){
      await editor.locator('#preset').selectOption(preset,{force:true});
      for(const view of ['lobby','panel']){
        const id=[preset,view,name].join('-');
        const refPath='/muestra/'+(view==='panel'?'vista-socio-panel':slug)+'.html';
        const ref=await old.goto(original+refPath);
        if(!ref?.ok())throw Error('Referencia inaccesible: '+refPath);
        await old.waitForLoadState('networkidle',{timeout:8000}).catch(()=>{});
        await editor.locator('#'+view).evaluate(node=>node.click());
        const surface=editor.frameLocator('#preview');
        await surface.locator(view==='lobby'?'#catalog[data-catalog-ready=true]':'#panel-title').waitFor();
        await surface.locator('body[data-preset="'+preset+'"] #surface.'+view).waitFor();
        const frame=editor.locator('#preview');
        await frame.evaluate((node,{width,height})=>{
          if(node.parentElement!==document.body)document.body.append(node);
          node.style.cssText='position:fixed!important;inset:0!important;z-index:99999!important;border:none!important;border-radius:0!important;transform:none!important;width:'+width+'px!important;height:'+height+'px!important;max-width:none!important;max-height:none!important';
        },{width,height});
        // Reparenting an iframe reloads srcdoc. Only resize once per page.
        await surface.locator(view==='lobby'?'#catalog[data-catalog-ready=true]':'#panel-title').waitFor();
        for(const target of [old.locator('body'),surface.locator('body')]){
          await target.evaluate(async body=>{
            await body.ownerDocument.fonts.ready;
            await Promise.all([...body.querySelectorAll('img')].filter(img=>{
              const rect=img.getBoundingClientRect();return rect.width&&rect.height&&rect.top<body.ownerDocument.defaultView.innerHeight&&rect.bottom>0;
            }).map(img=>img.decode().catch(()=>{})));
          });
        }
        const before=await old.screenshot({animations:'disabled',path:out+'/'+id+'-original.png'});
        const after=await frame.screenshot({animations:'disabled',path:out+'/'+id+'-maker.png'});
        const difference=await measure.evaluate(async ([a,b])=>{
          const decode=async str=>{const img=new Image();img.src='data:image/png;base64,'+str;await img.decode();return img;};
          const [left,right]=await Promise.all([decode(a),decode(b)]);
          if(left.width!==right.width||left.height!==right.height)throw Error('Dimensiones diferentes');
          const canvas=document.createElement('canvas');canvas.width=left.width;canvas.height=left.height;
          const ctx=canvas.getContext('2d',{willReadFrequently:true});
          const pixels=img=>{ctx.drawImage(img,0,0);return ctx.getImageData(0,0,canvas.width,canvas.height);};
          const first=pixels(left),second=pixels(right);let changed=0;
          for(let i=0;i<first.data.length;i+=4){
            const delta=first.data[i]!==second.data[i]||first.data[i+1]!==second.data[i+1]||first.data[i+2]!==second.data[i+2];
            if(delta)changed++;
            first.data[i]=delta?255:0;first.data[i+1]=0;first.data[i+2]=delta?100:0;
          }
          ctx.putImageData(first,0,0);
          return {changed,pixels:canvas.width*canvas.height,png:canvas.toDataURL().split(',')[1]};
        },[before.toString('base64'),after.toString('base64')]);
        await writeFile(out+'/'+id+'-diff.png',Buffer.from(difference.png,'base64'));
        results.push({id,refPath,changed:difference.changed,pixels:difference.pixels});
        await writeFile(out+'/comparison.json',JSON.stringify(results,null,2));
      }
    }
    await context.close();
  }
}finally{await browser.close();}
if(results.length!==68||results.some(r=>r.changed))process.exitCode=1;
