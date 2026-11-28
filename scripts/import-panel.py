"""Compile trusted presentation fragments from the pinned Maker demo panel.

Usage: python scripts/import-panel.py PANEL_SNAPSHOT.json ORIGINAL_REPO DESTINATION
Requires beautifulsoup4. The reference contains local demonstration values only.
"""
import json
import pathlib
import re
import subprocess
import sys
from bs4 import BeautifulSoup, NavigableString

snapshot, original, out = map(pathlib.Path, sys.argv[1:])
ref = "9bc456f6ed1b4344a0cf4fb2e84ca4791061ba10"
def source(path):
    return subprocess.check_output(["git", "-C", str(original), "show", ref + ":" + path]).decode()
data = json.loads(snapshot.read_text(encoding="utf8"))
doc = BeautifulSoup('<html class="h-full antialiased"><body class="min-h-full flex flex-col"><main class="text-zinc-100"><div id="root">'+data["markup"]+'</div></main></body></html>', "html.parser")
parts = {}
def fragment(key, element, bind=True):
    if bind:
        element["data-bind"]="attrs"
        element["style"]=element.get("style","")+";{{style}}"
    html = str(element).replace('data-bind="attrs"', "{{attrs}}")
    parts[key] = html
def field(element, selector, name):
    node = element.select_one(selector)
    node.clear()
    node.append("{{"+name+"}}")
def extra(node):
    node.append(NavigableString("{{extra}}"))

# Only fixed, local presentation data enters the renderer.
for el in doc.find_all(True):
    if el.name in ("script","iframe","object","embed","form","link","style","foreignobject"):
        raise ValueError("unexpected element "+el.name)
    for attr in el.attrs:
        if attr.lower().startswith("on") or attr in ("srcdoc","action","formaction"):
            raise ValueError("unexpected executable attribute")
    if el.has_attr("href"):
        del el["href"]
        el["role"]="button"
    if el.name=="button" or el.get("role")=="button":
        el["data-demo"]="true"
    if el.name=="img":
        if el["src"]!="./rushybet/rushybet-logo.webp":
            raise ValueError("unknown panel image")
        el["src"]="/media/objects/c7d60c43ba450618f022de96049c769a2cf81e87.webp"

# Keep only rules whose selectors participate in this home screen, including
# rules hidden by desktop/mobile media queries. Interactive pseudo-classes
# are matched on their base selectors and preserved in the output.
def blocks(css):
    tokens=re.finditer(r'''"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|[{};]''',css)
    start=0
    depth=0
    for token in tokens:
        char=token.group()
        if char=="{" and not depth:
            head=css[start:token.start()].strip()
            body=token.end()
        if char=="{":
            depth+=1
        elif char=="}":
            depth-=1
            if depth==0:
                yield head,css[body:token.start()]
                start=token.end()
        elif char==";" and depth==0:
            start=token.end()
def matches(selector):
    base=re.sub(r':(?:hover|active|focus-visible|focus-within|focus)(?![\w-])','',selector)
    base=re.sub(r'::?[\w-]+(?:\([^)]*\))?$',lambda m:'' if m[0].startswith('::') or m[0] in (':before',':after') else m[0],base)
    try: return bool(base and doc.select_one(base))
    except Exception: return False
def css_for(css):
    retained=[]
    for head,body in blocks(re.sub(r'/\*[\s\S]*?\*/','',css)):
        if head.startswith(('@media','@supports','@layer','@container')):
            body=css_for(body)
            if not body:continue
        elif head.startswith('@keyframes'):
            continue
        elif head.startswith('@') or not matches(head):
            continue
        body=re.sub(r'(?<![\w-])(?:animation(?:-[\w-]+)?):[^;}]+;?', '',body)
        # These are background decorations from unrelated panel screens.
        body=re.sub(r'(?:^|(?<=;))[^;{}]*url\([^;{}]+;?', '',body)
        retained.append(head+'{'+body.strip()+'}')
    return '\n'.join(retained)
styles = []
for path in ["app.css","panel.css"]:
    styles.append(css_for(source("public/muestra/"+path)))
defaults=next(style["css"] for style in data["styles"] if style["css"] and "--dm-p:" in style["css"])
styles.extend([css_for(defaults),css_for(source("public/muestra/piel-panel.css"))])
css='\n'.join(styles).replace('.v4 ','.cash-panel ').replace('.v4{','.cash-panel{').replace('.v4.', '.cash-panel.')
css=re.sub(r'\.v4(?=[,:\\[])','.cash-panel',css)
if 'url(' in css or '@import' in css:raise ValueError("unexpected remote style resource")
out.mkdir(exist_ok=True,parents=True)
css=re.sub(r'(?<![-\w])font-size:\s*(\d+(?:\.\d+)?)px',r'font-size:calc(\1px * var(--text-scale,1))',css)
css=re.sub(r'(?<![-\w])font-weight:\s*([1-9]00)\b',r'font-weight:var(--global-weight,\1)',css)
(out/'panel.css').write_text(css+'\n',encoding='utf8')
side=doc.select_one('aside.lateral')
mobile=doc.select_one('header.hdr')
for node in [side,mobile]:
    field(node,'.hdr-quien b','title')
    field(node,'.hdr-quien em','text')
    logo=node.select_one('img.logo')
    logo.replace_with(NavigableString('{{logo}}'))
field(side,'.lat-cargar span','button')
fragment('header',side)
fragment('mobile_header',mobile)
fragment('mobile_nav',doc.select_one('nav.nav'),False)
title=doc.select_one('.cab-pantalla')
field(title,'h1','title')
field(title,'.bajada','text')
field(title,'.ceja span','eyebrow')
extra(title)
fragment('panel-title',title)
metrics=doc.select_one('.mo')
field(metrics,'.rotulo','title')
for index,tile in enumerate(metrics.select('.mo-bloques > section')):
    field(tile,'h2','title')
    extra(tile)
    fragment('metric-'+str(index),tile)
field(metrics,'.mo-bloques','children')
extra(metrics)
fragment('metrics',metrics)
totals=doc.select_one('.numeros')
for index,tile in enumerate(totals.select(':scope > div')):
    field(tile,'.numero-rot','title')
    field(tile,'.numero-valor','amount')
    for detail in tile.select('.numero-sub'): detail.string="{{button}}"
    extra(tile)
    fragment('total-'+str(index),tile)
totals.clear()
totals.append('{{children}}')
fragment('totals',totals)
activity=doc.select_one('.sigue').parent
field(activity,'.rotulo','title')
for index,tile in enumerate(activity.select('.sigue-fila')):
    field(tile,'.sigue-txt','title')
    field(tile,'button span','button')
    extra(tile)
    fragment('activity-'+str(index),tile)
field(activity,'.sigue','children')
extra(activity)
fragment('activity',activity)
foot=doc.select('main.main > section')[-1]
field(foot,'.rotulo','title')
field(foot,'p','text')
field(foot,'button span','button')
extra(foot)
fragment('footer',foot)
(out/'panel-parts.json').write_text(json.dumps(parts,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
