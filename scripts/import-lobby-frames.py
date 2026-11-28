"""Compile original HTML/CSS presentation slots. No original JavaScript is shipped."""
import json
import pathlib
import re
import subprocess
import sys
from bs4 import BeautifulSoup, NavigableString
original, out = map(pathlib.Path,sys.argv[1:])
out.mkdir(parents=True,exist_ok=True)
ref="9bc456f6ed1b4344a0cf4fb2e84ca4791061ba10"
names=dict(violet="bateupix",saffron="cobraya",midnight="lunabet",amethyst="multisala",jade="pinaplay",rose="celuapuestas40",garnet="bet30",brass="casinotower30",aurum="fenarys",verdant="fortunasjk",crimson="ravenbet",onyx="luckrex")
def get(path):
    return subprocess.check_output(["git","-C",str(original),"show",ref+":public/muestra/"+path]).decode()
def words(s):
    s=s.replace("bp-","lobby-").replace("--dm-","--skin-")
    for preset,slug in names.items():
        s=s.replace("[data-catalog="+slug+"]","[data-preset="+preset+"]")
    return s
def html(n): return words(str(n)) if n else ""
def slot(n,key):
    n.clear()
    n.append(NavigableString("{{"+key+"}}"))
def optional(n,key):
    if n:slot(n,key)
css={}
frames={}
for preset,slug in names.items():
    s=BeautifulSoup(get(slug+".html"),"html.parser")
    styles=[e["href"] for e in s.select('link[rel=stylesheet]')]
    for name in styles:
        if name in css: continue
        entry=words(get(name))
        if re.search(r'url\(|@import',entry,re.I):raise ValueError("non-local stylesheet in "+name)
        css[name]=entry.replace('.lobby-game h2','.lobby-game h3').replace('.lobby-banner>img','.lobby-banner img[data-banner-art]')
    for node in s.select("script,link,iframe,dialog,style,form,object,embed"):node.decompose()
    for el in s.find_all(True):
        if any(k.lower().startswith("on") for k in el.attrs): raise ValueError("event attribute")
        if el.get("src") or (el.get("href") and not el["href"].startswith("#")):raise ValueError("unexpected resource")
        if el.has_attr('data-filter'):
            old=el.attrs.pop('data-filter')
            mode={'todos':'all','favoritos':'favorites','top':'top'}.get(old)
            if mode:el['data-catalog-mode']=mode
            else:el['data-demo']='true'
        if el.has_attr('data-info'):
            del el['data-info']
            el['data-demo']='true'
        if el.name=='button':el['type']='button'
    body=s.body;main=body.main
    header=body.header
    header.select_one('.bp-menu')['data-catalog-menu']='true'
    header.select_one('.bp-menu')['aria-expanded']='false'
    slot(header.select_one('.bp-brand'),'brand')
    slot(header.select_one('nav button'),'title')
    left,right=header.select('.bp-access button')[:2]
    slot(left,'text');slot(right,'button')
    sidebar=body.select_one('.bp-sidebar')
    mobile=body.select_one('.bp-mobile')
    defaults={'catalog_title':main.select_one('#bp-title').get_text(strip=True),
              'footer_title':main.footer.select_one('strong').get_text(strip=True),'footer_text':main.footer.select_one('p').get_text(strip=True)}
    result={'defaults':defaults,'styles':styles,'header':html(body.select_one('.bp-demo'))+html(header)+html(sidebar),'mobile':html(mobile)}
    hero=main.find(recursive=False)
    dots=main.select_one('#bp-dots')
    if hero.select_one('#bp-banner'):
        img=hero.select_one('#bp-banner')
        img['src']='{{src}}';img['data-banner-art']='true'
        src=hero.select_one('source')
        if src:src['data-banner-mobile']='true'
        hero.select_one('#bp-prev')['data-banner-step']='-1'
        hero.select_one('#bp-next')['data-banner-step']='1'
        dots['data-banner-dots']='true'
        hero.append(NavigableString('{{caption}}'))
        result['hero']=html(hero)+html(dots)
    elif preset=='rose':
        slot(hero,'children')
        result['hero']=html(hero)
    else:
        img=hero.select_one('[data-artwork]')
        img['src']='{{src}}'
        slot(hero.select_one('h1'),'title')
        if preset=='aurum':
            slot(hero.select_one('p:not(.md-eyebrow)') or hero.select_one('p'),'text')
        else:slot(hero.select_one('h2'),'text')
        slot(hero.select_one('button'),'button')
        result['hero']=html(hero)
    quick=main.select_one('.quick-links')
    experience=main.select_one('.ms-rooms,.md-experiences,.bf-experiences')
    if experience:
        grid=experience.select_one('.ms-room-grid') or experience
        slot(grid,'children')
        if preset=='amethyst':
            optional(experience.select_one('h2'),'title');optional(experience.select_one('p'),'text')
        result['categories']=html(experience)+html(quick)
    elif quick:
        slot(quick,'children')
        result['categories']=html(quick)
    else:result['categories']='{{children}}'
    menu=main.select_one('#ce-categories,#bf-categories,#modern-groups,#pp-collections')
    if menu:menu['data-collections']='true'
    strip=main.select_one('.provider-strip')
    if strip:strip.select_one('#bp-providers')['data-providers']='true'
    filters=main.select_one('.bp-discover')
    search=filters.select_one('input')
    search['data-search']='true';search['placeholder']='{{placeholder}}'
    tabs=filters.select_one('.bp-tabs')
    if not tabs.select_one('[data-catalog-mode=recent]'):
        btn=s.new_tag('button',type='button');btn['data-catalog-mode']='recent';btn['hidden']='';btn.string='Recientes';tabs.append(btn)
    group=main.select_one('#bp-games').parent
    group.select_one('#bp-games')['class'].append('items')
    group.select_one('#bp-count')['data-catalog-count']='true'
    group.select_one('#bp-empty')['data-catalog-empty']='true'
    group.select_one('#bp-more')['data-catalog-more']='true'
    slot(group.select_one('#bp-title'),'title')
    slot(group.select_one('#bp-games'),'seeds')
    result['catalog']=html(main.select_one('.ce-heading'))+html(menu)+html(strip)+html(filters)+html(group)
    if preset=='rose':
        prefix=html(main.select_one('.ce-heading'))+html(menu)
        result['categories']=prefix+result['categories']
        result['catalog']=result['catalog'].removeprefix(prefix)
    foot=main.footer
    slot(foot.select_one('strong'),'title');slot(foot.select_one('p'),'text')
    result['footer']=html(foot)+'{{extra}}'
    result['hero']=result['hero'].replace('src="{{src}}"','{{image_src}}')
    frames[preset]=result
(out/'lobbies.json').write_text(json.dumps(frames,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
(out/'styles.json').write_text(json.dumps(css,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
