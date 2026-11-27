// Only exact, explicitly authorized bindings. No provider discovery or network.
export function inventory(catalogues){
 const groups=new Map();
 for(const [preset,data] of catalogues){
  for(const item of data.items){
   if(item.asset!==null)continue;
   const key=JSON.stringify([item.provider,item.title]);
   if(!groups.has(key))groups.set(key,{provider:item.provider,title:item.title,targets:[]});
   groups.get(key).targets.push({preset,id:item.id});
  }
 }
 return {version:1,entries:[...groups.values()].sort((a,b)=>a.provider.localeCompare(b.provider,'en')||a.title.localeCompare(b.title,'en'))};
}
export function planBindings(catalogues,manifest){
 if(manifest?.version!==1||!Array.isArray(manifest.entries)||manifest.entries.length>10000)
  throw Error('Manifiesto inválido.');
 const index=new Map([...catalogues].map(([preset,data])=>[preset,new Map(data.items.map(item=>[item.id,item]))]));
 const claimed=new Set(),plan=[];
 for(const entry of manifest.entries){
  if(!entry||typeof entry.provider!=='string'||typeof entry.title!=='string'||
    typeof entry.file!=='string'||entry.file.length>500||
    !/^[a-f0-9]{64}$/.test(entry.sha256??'')||!Array.isArray(entry.targets)||
    !entry.targets.length||entry.targets.length>100)
   throw Error('Entrada de miniatura inválida.');
  const targets=[];
  for(const target of entry.targets){
   const item=index.get(target?.preset)?.get(target?.id);
   if(!item||item.provider!==entry.provider||item.title!==entry.title)
    throw Error('Identidad de juego no coincide con el catálogo.');
   if(item.asset!==null)throw Error('No se sobrescriben miniaturas existentes: '+target.id);
   const key=JSON.stringify([target.preset,target.id]);
   if(claimed.has(key))throw Error('Un juego tiene más de una asignación: '+target.id);
   claimed.add(key);targets.push({preset:target.preset,item});
  }
  plan.push({file:entry.file,sha256:entry.sha256,targets});
 }
 return plan;
}
