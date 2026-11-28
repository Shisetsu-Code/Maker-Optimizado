"""Build Ivory presentation slots from the original Resort page and stylesheet."""
from bs4 import BeautifulSoup
from pathlib import Path
import json, sys
src, out=map(Path,sys.argv[1:])
s=BeautifulSoup((src/'resort.html').read_text(encoding='utf8'),'html.parser')
def field(selector,key): s.select_one(selector).string='{{'+key+'}}'
for b in s.select('[data-cat]'):
    v=b.attrs.pop('data-cat')
    if v in ('todos','favoritos'):b['data-catalog-mode']={'todos':'all','favoritos':'favorites'}[v]
    else:b['data-resort-category']=v
for b in s.select('[data-scroll]'):
    b['data-catalog-mode']='all';del b['data-scroll']
for b in s.select('[data-wallet]'):
    b['data-demo']='true';del b['data-wallet']
for b in s.select('button'):b['type']='button'
field('.brand','ivory_brand');field('header .desktop button','title');field('header .wallet small','text');field('header .wallet strong','button')
field('.hero h1','lines_title');field('.hero-copy p','lines_text');field('.hero-copy .primary','button')
s.select_one('.poster.one')['src']='{{src}}'
s.select_one('.poster.two')['src']='/media/objects/5838f407e17892f42a6210429cce32b0a304ef8a.webp'
field('#discover h2','title');s.select_one('#search')['data-search']='true'
field('.games-section h2','title');s.select_one('#count')['data-catalog-count']='true'
s.select_one('#games')['class'].append('items');field('#games','seeds')
s.select_one('#empty')['data-catalog-empty']='true'
spot=s.select_one('#spotlight');spot['data-resort-featured']='';spot.img.attrs.pop('src')
b=s.new_tag('button',type='button');b['data-catalog-more']='';b.string='Ver más';s.select_one('.games-section').append(b)
field('.lounge h2','title');field('.lounge p','text');field('.lounge button','button')
field('footer span','title');field('footer span:last-child','text')
data=dict(defaults=dict(catalog_title='Los favoritos de la casa',footer_title='✦ Casino & Club',footer_text='18+ · DEMO · Sin apuestas reales'),styles=[],header=str(s.header),hero=str(s.select_one('.hero')),categories=str(s.select_one('#discover')),catalog=str(s.select_one('.games-section')),live=str(s.select_one('.lounge')),footer=str(s.footer),mobile=str(s.select_one('.mobile')))
out.mkdir(exist_ok=True,parents=True)
(out/'resort.json').write_text(json.dumps(data,ensure_ascii=False),encoding='utf8')
style=(src/'resort.css').read_text(encoding='utf8')
style=style[style.index(':root'):].replace('--dm-','--skin-')
(out/'resort.css').write_text(style,encoding='utf8')
