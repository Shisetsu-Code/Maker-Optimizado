import {normalizeElements} from './element-settings.mjs';
let database;
function open(){
    if(!database)database=new Promise((resolve,reject)=>{
        const request=indexedDB.open('maker-content-editor',1);
        request.onupgradeneeded=()=>request.result.createObjectStore('settings');
        request.onsuccess=()=>{const db=request.result;db.onversionchange=()=>{db.close();database=undefined;};resolve(db);};
        request.onerror=()=>reject(request.error);
    }).catch(error=>{database=undefined;throw error;});
    return database;
}
export async function loadElementSnapshot(){
    const db=await open();
    return new Promise((resolve,reject)=>{
        const tx=db.transaction('settings','readonly'),store=tx.objectStore('settings'),state=store.get('elements'),version=store.get('revision');
        tx.oncomplete=()=>resolve({state:normalizeElements(state.result),version:version.result??null});
        tx.onerror=()=>reject(tx.error||Error('No se pudo leer.'));
        tx.onabort=()=>reject(tx.error||Error('Lectura cancelada.'));
    });
}
export async function loadElements(){return (await loadElementSnapshot()).state;}
export async function saveElementSnapshot(value,expectedVersion){
    const state=normalizeElements(value),db=await open(),version=crypto.randomUUID();
    return new Promise((resolve,reject)=>{
        const tx=db.transaction('settings','readwrite'),store=tx.objectStore('settings'),current=store.get('revision');
        let conflict=false;
        current.onsuccess=()=>{
            if((current.result??null)!==expectedVersion){conflict=true;tx.abort();return;}
            store.put(state,'elements');store.put(version,'revision');
        };
        tx.oncomplete=()=>resolve({state,version});
        tx.onerror=()=>reject(tx.error||Error('No se pudo guardar.'));
        tx.onabort=()=>reject(conflict?Object.assign(Error('El diseño cambió en otra pestaña. Exportá tus cambios y recargá.'),{name:'ConflictError'}):tx.error||Error('No se pudo guardar.'));
    });
}
export async function saveElements(value){
    const snapshot=await loadElementSnapshot();
    return (await saveElementSnapshot(value,snapshot.version)).state;
}
