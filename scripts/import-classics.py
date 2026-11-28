"""Compile editable classic headers and covers from the original presentation. Usage: python import-classics.py snapshots original/public output.json"""
import base64
import gzip
import hashlib
import json
import re
import sys
from pathlib import Path
from urllib.parse import unquote, urlsplit
from bs4 import BeautifulSoup, Tag

snap, public, output = map(Path, sys.argv[1:])
def asset(url):
    path = unquote(urlsplit(url.strip()).path)
    for base in ([public] if path.startswith('/') else [public / 'muestra', public]):
        file = (base / path.lstrip('/')).resolve()
        if not file.is_relative_to(public.resolve()) or not file.is_file(): continue
        data = file.read_bytes()
        ext = 'webp' if data[:4]==b'RIFF' else 'png' if data.startswith(b'\x89PNG') else 'jpg' if data.startswith(b'\xff\xd8') else 'gif' if data[:3]==b'GIF' else ''
        if ext: return '/media/objects/'+hashlib.sha1(f'blob {len(data)}\0'.encode()+data).hexdigest()+'.'+ext
    return ''
def css(text):
    def replace(m):
        url=m[1].strip(' "\'')
        if url.startswith('data:image/'):return m[0]
        if url.startswith('#'):return m[0]
        image=asset(url)
        return 'url("'+image+'")' if image else 'none'
    return re.sub(r'url\(([^)]+)\)',replace,text)
def children(n):return n.find_all(recursive=False)
def sanitize(n):
    for t in [n,*n.find_all(True)]:
        for k in list(t.attrs):
            if k.startswith('on') or k in ['srcdoc','href','srcset','formaction','action','target','data-screen-label']:del t[k]
        if t.has_attr('style'):t['style']=css(t['style'])
        if t.name=='image-slot':
            t.name='img'; t['style']=t.get('style','')+';object-fit:cover;max-width:100%';t['alt']=t.get('placeholder','')
        if t.has_attr('src'):
            v=t['src'];t['src']=v if v.startswith('data:image/') else asset(v)
            if not t['src']:del t['src']
        if t.name=='button':t['type']='button';t['data-demo']='true'
def wrapper(s, nodes, parent=None):
    box=s.new_tag('div')
    if parent:
        box.attrs={k:v for k,v in parent.attrs.items() if k in ('style','class')}
    for n in nodes:box.append(n.extract())
    return box
def compile_one(p, mobile, d):
    s=BeautifulSoup(d['markup'],'html.parser')
    app=s.select_one('.sc-host > div')
    main=s.select_one('main')
    if p=='ember':
        desk=s.select_one('.lb-main'); mob=s.select_one('.lb-mob > div')
        if mobile:
            m=children(mob);h=wrapper(s,[m[0],s.select_one('.lb-tabbar')]);hero=wrapper(s,m[1:6],mob)
        else:
            m=children(desk);h=wrapper(s,[s.select_one('.lb-side'),m[0]]);hero=wrapper(s,m[1:3])
        for n in [h,hero]:n['class']=['lb','mb-tema','original-ember']
    elif p=='cobalt':
        if mobile:
            parent=children(children(app)[0])[1];rows=children(parent)
            h=wrapper(s,[s.header,s.nav]);hero=wrapper(s,rows[:4])
        else:
            h=wrapper(s,[s.header,s.aside]);hero=wrapper(s,children(main)[:2],main)
        for n in [h,hero]:
            n['class']=['original-cobalt'];n['style']=app.get('style','')+';'+n.get('style','')
    elif p=='flare':
        kids=children(main);promo=s.select_one('.bgpass');ads=children(promo)
        h=wrapper(s,([children(app)[0]] if not mobile else [s.nav])+[kids[0]])
        hero=wrapper(s,[kids[1],wrapper(s,ads[:3],promo)])
        kids[1]['data-fixture-only']=''
    else:
        kids=children(app);h=wrapper(s,[kids[0],kids[1]]+([kids[3]] if mobile else []))
        belt=s.select_one('#inicio');parent=belt.parent
        if mobile:children(parent)[0]['data-fixture-only']=''
        children(belt)[0]['data-fixture-only']=''
        hero=wrapper(s,([children(parent)[0]] if mobile else [])+[belt],parent)
    foot=s.footer if p!='coal' else s.select_one('#apuestas').find_next_sibling()
    if not foot:raise ValueError('missing original footer')
    if p=='ember':
        for t in foot.find_all(string=True):
            if '5.915' in t and '41' in t:
                span=s.new_tag('span');span['data-classic-summary']='';t.replace_with(span)
    # Keep the original shelf geometry; the account catalog supplies every card at runtime.
    if p=='ember':
        if mobile:
            title,row=m[6:8]
            shelf=wrapper(s,[title,row],mob)
        else:
            section=desk.select_one('section.lb-desk')
            title,row=children(section)[:2]
            shelf=wrapper(s,[title,row],section)
        shelf['class']=['mb-tema','original-ember','native-shelf']
    elif p=='cobalt':
        section=s.select_one('#m-rec' if mobile else '#recomendados')
        title,row=children(section)
        shelf=section if mobile else wrapper(s,[section.parent],main)
        if not mobile:shelf['class']=['original-cobalt']
    elif p=='flare':
        shelf=promo.select_one('section')
        title,row=children(shelf)
    elif mobile:
        shelf=children(next(n for n in children(parent) if n.get_text(strip=True).startswith('Populares')))[0]
        title,row=children(shelf)
    else:
        shelf=s.new_tag('div')
        row=s.new_tag('div')
        shelf.append(row)
        title=None
    cards=children(row)
    if cards:
        item=cards[0]
        row['data-item-style']=item.get('style','') if p!='cobalt' or not mobile else 'flex:0 0 128px;height:168px;border-radius:12px'
        row['data-item-limit']=str(len(cards))
    row.clear()
    row['data-native-cards']=''
    if title:
        for el in title.find_all(string=True):
            if el.strip().isdigit():el.parent['data-native-count']=''
        for el in [title,*title.select('button')]:
            if el.name=='button':el['data-shelf-action']=''
    for node in [h,hero,foot,shelf]:sanitize(node)
    for img in h.select('img'):
        if 'logo' in str(img.get('class','')).lower() or img.get('alt','').lower()=='rushybet':img['data-brand-logo']=''
    values={}
    def bind(key,default):
        values[key]=str(default)
        return '{{'+key+'}}'
    for mode,root in [('header',h),('hero',hero),('footer',foot)]:
        btns=[b for b in root.select('button') if b.get_text(strip=True)]
        if btns:
            b=btns[-1];leaf=next((t for t in reversed(b.find_all(string=True)) if t.strip()),None)
            if leaf:leaf.replace_with(bind(mode+'_button',leaf))
        # Replace text nodes in place so edits retain the original type and layout.
        leaves=[t for t in root.find_all(string=True) if t.strip() and '{{' not in t and t.parent.name not in ('svg','path','style')]
        if leaves:
            title=next((t for t in leaves if len(t.strip())>4),leaves[0])
            title.replace_with(bind(mode+'_title',title))
            detail=next((t for t in leaves if t.parent and t is not title and len(t.strip())>5),None)
            if detail:detail.replace_with(bind(mode+'_text',detail))
        picture=root.select_one('img[data-brand-logo]') if mode=='header' else root.select_one('img[src]')
        if picture:
            fallback=picture.get('src') or '/media/objects/c7d60c43ba450618f022de96049c769a2cf81e87.webp'
            picture['src']=bind(mode+'_src',fallback)
            if mode=='header':
                span=s.new_tag('span',attrs={'class':'brand-art','style':'display:contents'})
                picture.wrap(span)
                span.insert_after('{{brand_text}}')
        elif mode=='hero':root.append('{{extra_image}}')
    for root in [h,hero,foot]:
        for element in root.select('[id]'):
            ident=element['id']
            # Prefix duplicate SVG gradient, mask and clip identifiers together with their references.
            if element.name.lower() in ('lineargradient','radialgradient','mask','clippath','filter','pattern'):
                scoped=('m-' if mobile else 'd-')+ident
                for tagged in root.find_all(True):
                    for key,value in list(tagged.attrs.items()):
                        if isinstance(value,str) and 'url(#'+ident+')' in value:tagged[key]=value.replace('url(#'+ident+')','url(#'+scoped+')')
                element['id']=scoped
    sheet=[]
    for entry in d['styles']:
        href=entry['href'] or '';v=entry['css'] or ''
        if href.endswith('/piel-lobby.css'):v=(public/'muestra/piel-lobby.css').read_text(encoding='utf8')
        if not v or 'data-maker-select' in v:continue
        sheet.append(css(re.sub(r'/\*[\s\S]*?\*/','',v)))
    return dict(shelf=str(shelf),header=str(h),hero=str(hero),footer=str(foot),values=values,css='\n'.join(dict.fromkeys(sheet)))
out={}
for preset in ['ember','cobalt','flare','coal']:
    variants={}
    for width,name in [(1440,'desktop'),(393,'mobile')]:
        filename=snap/f'{preset}-{width}-source.json.gz'
        data=json.loads(gzip.decompress(filename.read_bytes()))
        variants[name]=compile_one(preset,name=='mobile',data)
    out[preset]=variants
output.parent.mkdir(parents=True,exist_ok=True)
output.write_text(json.dumps(out,ensure_ascii=False,separators=(',',':')),encoding='utf8')
