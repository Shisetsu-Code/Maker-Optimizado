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
    await context.clock.setFixedTime(new Date('2026-11-26T15:00:00Z'));
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
        if(refPath.includes('vista-socio-'))await old.locator('#root > *').first().waitFor();
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
        if(preset==='ember'&&view==='panel'){
          const structure=await old.locator('body').evaluate(body=>[...body.querySelectorAll('header,nav,aside,main,h1,h2,h3,button,dt,dd,[role=tab]')].map(el=>{
            const r=el.getBoundingClientRect();
            return {tag:el.localName,text:(el.innerText??'').trim().replace(/\\s+/g,' ').slice(0,90),box:[r.x,r.y,r.width,r.height].map(Math.round)};
          }).filter(({box:[x,y,w,h]})=>w>0&&h>0&&x<innerWidth&&y<innerHeight&&x+w>0&&y+h>0));
          console.log('Panel original '+name+' '+JSON.stringify(structure));
          await writeFile(out+'/panel-'+name+'-structure.json',JSON.stringify(structure,null,2));
        }
        if((view==='panel'&&preset==='ember')||(view==='lobby'&&['violet','ivory','garnet','rose'].includes(preset))){
          const inspect=async(target,legacy)=>target.evaluate((body,{legacy,view})=>{
            const doc=body.ownerDocument,rows={};
            const selectors=view==='panel'?['.marco','.lateral','.lat-marca','.lat-float','.lat-cargar','.lat-nav','.lat-item','.hdr','.main','.cab-pantalla','.ceja','h1','.bajada','.mo','.mo-bloques','.mo-caja','.mo-tit','.mo-cifra-n','.mo-grafico','.mo-med','.numeros','.nav']:
              ['header','main','.'+(legacy?'bp':'lobby')+'-brand','.'+(legacy?'bp':'lobby')+'-banner','.'+(legacy?'bp':'lobby')+'-grid','.'+(legacy?'bp':'lobby')+'-game','.ce-home','.hero','.resort-hero','.quick-links','.provider-strip'];
            for(const sel of selectors){
              const el=doc.querySelector(sel);if(!el)continue;
              const win=doc.defaultView,style=win.getComputedStyle(el),rect=el.getBoundingClientRect();
              rows[sel.replace(/bp-/g,'lobby-')]={box:[rect.x,rect.y,rect.width,rect.height].map(x=>+x.toFixed(2)),font:style.fontFamily,size:style.fontSize,weight:style.fontWeight,line:style.lineHeight,color:style.color,background:style.backgroundColor,text:el.innerText?.trim().slice(0,60)};
            }
            return rows;
          },{legacy,view});
          const originalRows=await inspect(old.locator('body'),true),makerRows=await inspect(surface.locator('body'),false);
          const deviations={};
          for(const [key,orig] of Object.entries(originalRows)){
            const rebuilt=makerRows[key];
            const changed=Object.fromEntries(Object.entries(orig).filter(([name,value])=>JSON.stringify(value)!==JSON.stringify(rebuilt?.[name])).map(([name,value])=>[name,[value,rebuilt?.[name]]]));
            if(Object.keys(changed).length)deviations[key]=changed;
          }
          await writeFile(out+'/'+id+'-layout.json',JSON.stringify(deviations,null,2));
          console.log('LAYOUT '+id+' '+JSON.stringify(deviations));
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
        console.log(id+': '+difference.changed+'/'+difference.pixels+' píxeles diferentes');
      }
    }
    await context.close();
  }
}finally{await browser.close();}
const exact=results.filter(r=>r.changed===0).length;
const cards=results.map(({id,changed,pixels})=>`<section><h2>${id} · ${(100*changed/pixels).toFixed(2)} % diferente</h2><div>${['original','maker','diff'].map(kind=>`<figure><a href="${id}-${kind}.png"><img loading="lazy" src="${id}-${kind}.png" alt="${kind}"></a><figcaption>${kind}</figcaption></figure>`).join('')}</div></section>`).join('');
await writeFile(out+'/index.html',`<!doctype html><html lang="es"><meta charset="utf-8"><title>Comparación de Maker</title><style>body{font:16px system-ui;margin:2rem;background:#16191c;color:#eee}section{margin:2rem 0}section div{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:1rem}figure{margin:0}img{max-width:100%}</style><h1>Maker original y reconstrucción</h1><p>${exact} de ${results.length} vistas idénticas.</p>${cards}</html>`);
console.log('Coincidencias exactas: '+exact+'/'+results.length);
if(results.length!==68||exact!==68)process.exitCode=1;
