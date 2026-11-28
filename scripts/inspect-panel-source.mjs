import {chromium} from 'playwright';
import {writeFile} from 'node:fs/promises';
const b=await chromium.launch({channel:'chrome',headless:true});
try{
 const page=await b.newPage({viewport:{width:1440,height:900},locale:'es-AR',timezoneId:'UTC'});
 await page.route('**/*',route=>new URL(route.request().url()).hostname==='127.0.0.1'?route.continue():route.abort());
 await page.clock.setFixedTime(new Date('2026-11-26T15:00:00Z'));
 await page.goto('http://127.0.0.1:8788/muestra/vista-socio-panel.html');
 await page.getByRole('heading',{name:'Jugadores',exact:true}).waitFor();
 await page.evaluate(()=>document.fonts.ready);
 const value=await page.evaluate(()=>{
   const root=document.querySelector('#root').cloneNode(true);
   for(const node of root.querySelectorAll('script,iframe,form,object,embed,style,link'))node.remove();
   for(const el of root.querySelectorAll('*')){
     for(const attr of [...el.attributes]){
       if(attr.name.startsWith('on')||attr.name==='srcdoc'||attr.name==='action'||attr.name==='formaction')el.removeAttribute(attr.name);
     }
   }
   return {markup:root.innerHTML,styles:[...document.styleSheets].map(s=>({href:s.href,css:s.href?null:s.ownerNode.textContent})), root:document.documentElement.className,body:document.body.className,text:document.body.innerText};
 });
 await writeFile(process.argv[2]||'panel-source.json',JSON.stringify(value));
}finally{await b.close();}
