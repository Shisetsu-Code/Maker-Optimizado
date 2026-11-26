const DB='maker-studio';
let opening;
function open(){
  if(!opening)opening=new Promise((resolve,reject)=>{
    const request=indexedDB.open(DB,1);
    request.onupgradeneeded=()=>request.result.createObjectStore('projects');
    request.onsuccess=()=>{const db=request.result;db.onversionchange=()=>{db.close();opening=undefined;};resolve(db);};
    request.onerror=()=>reject(request.error||Error('No se pudo abrir el almacenamiento.'));
  }).catch(error=>{opening=undefined;throw error;});
  return opening;
}
export async function load(){
  const db=await open();
  return new Promise((resolve,reject)=>{
    const tx=db.transaction('projects','readonly'),request=tx.objectStore('projects').get('current');
    tx.oncomplete=()=>resolve(request.result??{revision:null,json:null});
    tx.onabort=()=>reject(tx.error||Error('Lectura cancelada.'));
    tx.onerror=()=>reject(tx.error||Error('No se pudo leer.'));
  });
}
export async function save(json,expected){
  const db=await open();
  return new Promise((resolve,reject)=>{
    const tx=db.transaction('projects','readwrite'),store=tx.objectStore('projects'),request=store.get('current');
    let conflict=false;const revision=crypto.randomUUID();
    request.onsuccess=()=>{
      if((request.result?.revision??null)!==expected){conflict=true;tx.abort();return;}
      store.put({revision,json},'current');
    };
    tx.oncomplete=()=>resolve(revision);
    tx.onabort=()=>reject(conflict?Object.assign(Error('Otra pestaña modificó el proyecto. Exportá tus cambios y recargá.'),{name:'ConflictError'}):tx.error||Error('Guardado cancelado.'));
    tx.onerror=()=>reject(tx.error||Error('No se pudo guardar.'));
  });
}
