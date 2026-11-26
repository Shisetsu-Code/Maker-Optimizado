import {normalizeContentMeta} from './maker-wasm.mjs';
export const SECTION_NAMES=Object.freeze({hero:'Portada',discover:'Búsqueda y categorías',games:'Juegos destacados',lounge:'Casino en vivo'});
export const DEFAULT_CONTENT=Object.freeze({title:'La suerte tiene\nun nuevo lugar.',description:'Grandes clásicos, nuevas emociones.\nElegí tu juego y hacé tuya la noche.',button:'Explorar juegos ↗',image:'',sections:Object.keys(SECTION_NAMES),hidden:[]});
export function normalizeContent(value={}){
    if(!value||typeof value!=='object')value={};
    const meta=normalizeContentMeta({
        title:typeof value.title==='string'?value.title.slice(0,200):null,
        description:typeof value.description==='string'?value.description.slice(0,440):null,
        button:typeof value.button==='string'?value.button.slice(0,90):null,
        sections:Array.isArray(value.sections)?value.sections.filter(k=>typeof k==='string'&&Object.hasOwn(SECTION_NAMES,k)):[],
        hidden:Array.isArray(value.hidden)?value.hidden.filter(k=>typeof k==='string'&&Object.hasOwn(SECTION_NAMES,k)):[]
    });
    const image=typeof value.image==='string'&&value.image.length<=800000&&/^data:image\/(?:jpeg|png|webp);base64,[A-Za-z0-9+/=]+$/.test(value.image)?value.image:'';
    return {...meta,image};
}
