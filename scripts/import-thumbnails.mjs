// Offline, reviewable import of user-authorized graphics; never fetch remote URLs.
import {readFile,readdir,realpath,stat,mkdir,mkdtemp,writeFile,rename,rm} from 'node:fs/promises';
import {resolve,relative,isAbsolute,join} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {createHash,randomUUID} from 'node:crypto';
import {inventory,planBindings} from './thumbnail-manifest.mjs';

const root=fileURLToPath(new URL('..',import.meta.url));
const limit=8*1024*1024;
const digest=(data,algorithm='sha256')=>createHash(algorithm).update(data).digest('hex');
async function json(path,max=8*1024*1024){
 const info=await stat(path);
 if(!info.isFile()||info.size>max)throw Error('JSON inválido o demasiado grande: '+path);
 return JSON.parse(await readFile(path,'utf8'));
}
async function catalogues(){
 const result=new Map(),dir=join(root,'public/catalogs');
 for(const name of (await readdir(dir)).sort()){
  if(!/^[a-z0-9-]+\.json$/.test(name))continue;
  const value=await json(join(dir,name));
  if(value.preset!==name.slice(0,-5)||!Array.isArray(value.items))throw Error('Catálogo inválido.');
  result.set(value.preset,value);
 }
 return result;
}
export async function authorizedFile(directory,entry){
 if(isAbsolute(entry.file)||entry.file.includes('\\')||entry.file.includes('\0')||
   entry.file.split('/').some(part=>!part||part==='.'||part==='..'))
  throw Error('Ruta de origen no permitida.');
 const path=await realpath(resolve(directory,entry.file));
 const rel=relative(directory,path);
 if(!rel||isAbsolute(rel)||rel==='..'||rel.startsWith('..'+(process.platform==='win32'?'\\':'/')))
  throw Error('El archivo no pertenece al directorio autorizado.');
 const info=await stat(path);
 if(!info.isFile()||info.size>limit)throw Error('Imagen de origen demasiado grande.');
 const data=await readFile(path);
 if(data.length>limit||digest(data)!==entry.sha256)throw Error('SHA-256 de origen incorrecto.');
 return data;
}
function mime(data){
 if(data.subarray(0,8).equals(Buffer.from([137,80,78,71,13,10,26,10])))return 'image/png';
 if(data[0]===255&&data[1]===216)return 'image/jpeg';
 if(data.subarray(0,4).toString('ascii')==='RIFF'&&data.subarray(8,12).toString('ascii')==='WEBP')return 'image/webp';
 throw Error('Formato gráfico no admitido.');
}
export async function encodeThumbnail(page,studio,data){
 const type=mime(data);
 studio.check_upload(new Uint8Array(data),type);
 const encoded=await page.evaluate(async({base64,type})=>{
  const bytes=Uint8Array.from(atob(base64),c=>c.charCodeAt(0));
  const bitmap=await createImageBitmap(new Blob([bytes],{type}));
  try{
   const ratio=Math.min(1,640/Math.max(bitmap.width,bitmap.height));
   const canvas=document.createElement('canvas');
   canvas.width=Math.max(1,Math.round(bitmap.width*ratio));
   canvas.height=Math.max(1,Math.round(bitmap.height*ratio));
   canvas.getContext('2d').drawImage(bitmap,0,0,canvas.width,canvas.height);
   const result=canvas.toDataURL('image/webp',.85);
   if(!result.startsWith('data:image/webp;base64,'))throw Error('No se pudo codificar WebP.');
   return result.split(',')[1];
  }finally{bitmap.close();}
 },{base64:data.toString('base64'),type});
 const output=Buffer.from(encoded,'base64');
 studio.check_upload(new Uint8Array(output),'image/webp');
 return output;
}
async function atomicJson(path,value,indent){
 const temp=path+'.'+randomUUID()+'.tmp';
 try{await writeFile(temp,JSON.stringify(value,null,indent)+'\n',{flag:'wx'});await rename(temp,path);}
 finally{await rm(temp,{force:true});}
}
async function main(args){
 const catalog=await catalogues();
 if(args.length===1&&args[0]==='--inventory'){
  console.log(JSON.stringify(inventory(catalog),null,2));return;
 }
 const manifestAt=args.indexOf('--manifest'),sourceAt=args.indexOf('--source'),apply=args.includes('--apply');
 if(manifestAt<0||sourceAt<0||!args[manifestAt+1]||!args[sourceAt+1]||args.length!==(apply?5:4))
  throw Error('Uso: --inventory | --manifest archivo.json --source directorio [--apply]');
 const source=await realpath(args[sourceAt+1]),manifest=await json(args[manifestAt+1]);
 const plan=planBindings(catalog,manifest);
 if(!plan.length){console.log('No hay entradas para importar.');return;}
 const {default:init,Studio}=await import('../public/wasm/maker_studio.js');
 await init({module_or_path:await readFile(join(root,'public/wasm/maker_studio_bg.wasm'))});
 const {chromium}=await import('playwright');
 const studio=new Studio();
 await mkdir(join(root,'work'),{recursive:true});
 const staging=await mkdtemp(join(root,'work/thumbnail-stage-'));
 const objects=new Map();
 let browser;
 try{
  browser=await chromium.launch({headless:true});
  const context=await browser.newContext({offline:true});
  await context.route('**/*',route=>route.abort());
  const page=await context.newPage();
  const mediaRoot=join(root,'public/media');
  const index=await json(join(mediaRoot,'index.json'));
  const report=await json(join(root,'docs/catalog-parity.json'));
  const importedPath=join(root,'docs/imported-thumbnails.json');
  const imported=await json(importedPath);
  const collections=new Map();
  for(const collection of index.collections)collections.set(collection.id,new Set(await json(join(mediaRoot,'collections',collection.id+'.json'))));
  const deployed=collections.get('all');
  const bySource=new Map();
  let changed=0,totalNewBytes=0;
  const changedPresets=new Set();
  for(const entry of plan){
   let id=bySource.get(entry.sha256);
   if(!id){
    const data=await authorizedFile(source,entry);
    const output=await encodeThumbnail(page,studio,data);
    id=createHash('sha1').update('blob '+output.length+'\0').update(output).digest('hex')+'.webp';
    if(deployed.has(id)){
     const existing=await readFile(join(mediaRoot,'objects',id));
     if(!existing.equals(output))throw Error('Conflicto de objeto gráfico existente.');
    }else if(!objects.has(id)){
     totalNewBytes+=output.length;
     if(totalNewBytes>512*1024*1024||deployed.size+objects.size>=19500)throw Error('Presupuesto de biblioteca excedido.');
     await writeFile(join(staging,id),output,{flag:'wx'});
     objects.set(id,{bytes:output.length,sha256:digest(output),source_sha256:entry.sha256});
    }
    bySource.set(entry.sha256,id);
   }else{
    // A reused digest must still refer to an authorized, matching input.
    await authorizedFile(source,entry);
   }
   for(const target of entry.targets){
    target.item.asset=id;collections.get(target.preset).add(id);changed++;
    changedPresets.add(target.preset);
    const row=report.catalogs[target.preset];row.authorized_import_matches=(row.authorized_import_matches??0)+1;
   }
  }
  for(const id of objects.keys())deployed.add(id);
  for(const [id,metadata] of objects)imported.objects[id]=metadata;
  index.unique=deployed.size;
  for(const collection of index.collections)collection.count=collections.get(collection.id).size;
  for(const [preset,data] of catalog)report.catalogs[preset].without_local_image=data.items.filter(item=>item.asset===null).length;
  console.log(JSON.stringify({dryRun:!apply,bindings:changed,newObjects:objects.size,newBytes:totalNewBytes,remaining:[...catalog.values()].reduce((n,c)=>n+c.items.filter(i=>i.asset===null).length,0)},null,2));
  if(!apply)return;
  // All input validation and decoding finished before touching the served tree.
  // Individual writes are atomic; use a clean branch and review the Git diff.
  for(const id of objects.keys())await writeFile(join(mediaRoot,'objects',id),await readFile(join(staging,id)),{flag:'wx'});
  for(const preset of changedPresets)await atomicJson(join(root,'public/catalogs',preset+'.json'),catalog.get(preset));
  for(const preset of changedPresets)await atomicJson(join(mediaRoot,'collections',preset+'.json'),[...collections.get(preset)]);
  if(objects.size)await atomicJson(join(mediaRoot,'collections/all.json'),[...deployed].sort());
  await atomicJson(join(mediaRoot,'index.json'),index);
  await atomicJson(importedPath,imported,2);
  await atomicJson(join(root,'docs/catalog-parity.json'),report,2);
 }finally{
  if(browser)await browser.close();
  studio.free();
  await rm(staging,{recursive:true,force:true});
 }
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href)
 main(process.argv.slice(2)).catch(error=>{console.error(error.message??String(error));process.exitCode=1;});
