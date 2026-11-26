// Browser adapter for the Rust demonstration state. No network or storage.
let active=null;
function paint(){
  if(!active)return;
  const {doc,studio}=active;
  const root=doc.querySelector('#surface.panel'),workspace=doc.querySelector('#panel-workspace');
  if(!root||!workspace)return;
  const meta=JSON.parse(studio.panel_metadata());
  const nav=root.querySelector('.panel-navigation');
  if(nav)nav.outerHTML=studio.panel_navigation();
  workspace.innerHTML=studio.panel_content();
  root.dataset.panelSection=meta.section;root.dataset.panelReady='true';
}
export function attachPanel(doc,studio,editing){
  active={doc,studio,editing};paint();
}
function apply(action,notice){
  try{active.studio.panel_action(JSON.stringify(action));paint();}
  catch(error){notice('Panel de muestra',String(error));}
}
export function panelClick(doc,target,notice){
  if(active?.doc!==doc)return false;
  const b=target.closest?.('button');if(!b)return false;
  if(b.dataset.panelSection){apply({type:'navigate',section:b.dataset.panelSection},notice);return true;}
  if(active.editing())return false;
  if(b.dataset.panelRow){apply({type:'select',id:b.dataset.panelRow},notice);return true;}
  if(b.dataset.panelToggle){apply({type:'toggle',id:b.dataset.panelToggle},notice);return true;}
  if(b.dataset.panelReview){apply({type:'review',id:b.dataset.panelReview},notice);return true;}
  if(b.dataset.panelSaveNote){
    const id=b.dataset.panelSaveNote,field=doc.querySelector('[data-panel-note="'+CSS.escape(id)+'"]');
    apply({type:'note',id,text:field?.value??''},notice);return true;
  }
  if(b.hasAttribute('data-panel-maintenance')){
    if(confirm('¿Cambiar la vista local de mantenimiento?'))apply({type:'maintenance',enabled:b.dataset.panelMaintenance==='true'},notice);
    return true;
  }
  if(b.hasAttribute('data-panel-reset')){
    if(confirm('¿Restablecer los datos ficticios de la sesión?'))apply({type:'reset'},notice);
    return true;
  }
  return false;
}
export function panelInput(doc,target,notice){
  if(active?.doc!==doc||active.editing())return false;
  if(target.matches?.('[data-panel-search]')){
    const value=target.value,position=target.selectionStart;
    apply({type:'search',query:value},notice);
    const input=doc.querySelector('[data-panel-search]');input?.focus();if(input&&position!=null)input.setSelectionRange(position,position);
    return true;
  }
  if(target.matches?.('[data-panel-filter]')){apply({type:'filter',filter:target.value},notice);return true;}
  return false;
}
export function previewMaintenance(doc,studio,editing){
  if(editing||!JSON.parse(studio.panel_metadata()).maintenance)return false;
  const root=doc.querySelector('#surface.lobby');if(!root)return false;
  // Fixed copy, no user data or executable markup.
  root.replaceChildren();
  const section=doc.createElement('section');section.className='maintenance-preview';
  const h=doc.createElement('h1');h.textContent='Estamos en mantenimiento';
  const p=doc.createElement('p');p.textContent='Vista de demostración local. El sitio real no fue modificado.';
  section.append(h,p);root.append(section);return true;
}
