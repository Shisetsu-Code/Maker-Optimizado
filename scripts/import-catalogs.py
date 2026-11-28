#!/usr/bin/env python3
"""Project the old demo catalogue data into the new schema.

Only local public presentation records are read. No bundle is executed, no
provider protocol, login data or endpoint is copied into the runtime.
"""
import hashlib
import json
import pathlib
import re
import subprocess
import urllib.parse
from collections import defaultdict

ROOT=pathlib.Path(__file__).resolve().parents[1]
SOURCE="8a8b09cfdda55d8233b2d61c4d878bb112f4f591"
MEDIA=json.loads((ROOT/"docs/asset-provenance.json").read_text())["files"]
ART=json.loads((ROOT/"assets/preset-artwork.json").read_text())
MAP={path:info["id"] for path,info in MEDIA.items() if info["served"]}
SOURCES={"ember":"clasica","cobalt":"azul","flare":"blaze","coal":"brasa","ivory":"resort",
         "violet":"bateupix","saffron":"cobraya","midnight":"lunabet","amethyst":"multisala",
         "jade":"pinaplay","rose":"celuapuestas40","garnet":"bet30","brass":"casinotower30",
         "aurum":"fenarys","verdant":"fortunasjk","crimson":"ravenbet","onyx":"luckrex"}
def show(path):
    return subprocess.check_output(["git","-C",str(ROOT),"show",SOURCE+":"+path],stderr=subprocess.DEVNULL).decode("utf-8")
def clean(value,limit=120):
    return "".join(c for c in str(value or "") if c.isprintable()).strip()[:limit]
def hid(*parts):
    return hashlib.sha256("\0".join(map(str,parts)).encode()).hexdigest()[:24]
def category(value):
    value=clean(value,80).lower()
    if "live" in value or "vivo" in value:return "En vivo"
    if "crash" in value or "instant" in value:return "Crash" if "crash" in value else "Instantáneos"
    if "sport" in value or "deport" in value:return "Deportes"
    if "tower" in value:return "Tower"
    if "new" in value or "nuevo" in value:return "Nuevos"
    if "slot" in value or not value:return "Slots"
    return value.capitalize()
def resolve(group,path,vendor="",code=""):
    path=str(path or "")
    candidates=[]
    if path.startswith(("http://","https://")):
        u=urllib.parse.urlsplit(path)
        candidates.append("public/assets/external/"+u.netloc+urllib.parse.unquote(u.path))
    elif path.startswith("/"):
        candidates += ["public"+path,"public/muestra"+path]
    elif path:
        candidates += ["public/muestra/"+group+"/"+path,"public/muestra/"+path,"public/"+path]
    if vendor and code:
        candidates += [f"public/muestra/{group}/assets/covers/{vendor}-{code}.webp",
                       f"public/games-cartas/{vendor}/{code}.webp",
                       f"public/games-cartas/{vendor}/{code.lower()}.webp"]
    for candidate in candidates:
        if candidate in MAP:return MAP[candidate]
    # Existing rewritten CDN paths are frequently namespaced by a captured host.
    return None
def row(title,provider,cat,path="",vendor="",code="",group=""):
    return {"title":clean(title) or "Juego de muestra","provider":clean(provider) or "Sin proveedor",
            "category":category(cat),"asset":resolve(group,path,vendor,code)}
def structured(preset,group):
    source=json.loads(show(f"public/muestra/{group}/catalog.json"))
    items=[];ids=defaultdict(list)
    for index,game in enumerate(source["games"]):
        old=game.get("id",str(index))
        item=row(game.get("name"),game.get("providerName") or game.get("provider"),
                 game.get("category"),game.get("image"),game.get("provider"),game.get("code"),group)
        item["id"]="item-"+hid(preset,old,index)
        ids[str(old)].append(item["id"])
        items.append(item)
    groups=[]
    for index,g in enumerate(source.get("groups",[])):
        members=list(dict.fromkeys(new for old in g.get("games",[]) for new in ids.get(str(old),[])))
        groups.append({"id":"group-"+hid(preset,index,g.get("name")),"title":clean(g.get("name")) or "Colección","items":members})
    banners=[]
    mobile=source.get("bannerMobile",[])
    for i,p in enumerate(source.get("banners",[])):
        desktop=resolve(group,p)
        if desktop:banners.append({"desktop":desktop,"mobile":resolve(group,mobile[i]) if i<len(mobile) else None})
    return items,groups,banners,len(source["games"]),"structured-exact"
def literal_rows(group):
    if group=="resort":
        src=show("public/muestra/resort.js").split("const $=",1)[0]
        result=[]
        for obj in re.findall(r"\{id:'[^']+',[^{}]*\}",src):
            fields=dict(re.findall(r"(\w+):'([^']*)'",obj))
            if "name" not in fields:continue
            path=fields.get("localImage") or "/games-cartas/"+fields.get("image","")
            result.append(row(fields["name"],fields.get("provider"),fields.get("cat"),path,
                              fields.get("vendor"),fields.get("code"),""))
        return result,"literal-resort"
    if group=="blaze":
        src=show("public/muestra/blaze.js")
        result=[]
        pattern=r"D\(\x60([^\x60]+)\x60,\x60([^\x60]+)\x60,\x60([^\x60]+)\x60,\x60([^\x60]+)\x60,\x60([^\x60]+)\x60\)"
        for _,key,title,provider,path in re.findall(pattern,src):
            if "|" not in key:continue
            vendor,code=key.split("|",1)
            result.append(row(title,provider,"live" if "live" in provider.lower() else "slots",
                              path,vendor,code,group))
        return result,"literal-blaze"
    if group=="brasa":
        src=show("public/muestra/brasa.js")
        providers=dict(re.findall(r"\{code:\x60([^\x60]+)\x60,name:\x60([^\x60]+)\x60,type:\d+\}",src))
        pattern=r"\{diseno:\x60[^\x60]+\x60,disenoNombre:\x60[^\x60]+\x60,vendorCode:\x60([^\x60]+)\x60,gameCode:\x60([^\x60]+)\x60,nombre:\x60([^\x60]+)\x60,tipo:(\d+),"
        result=[]
        for vendor,code,title,kind in re.findall(pattern,src):
            result.append(row(title,providers.get(vendor,vendor),"live" if kind=="2" else "slots",
                              "",vendor,code,group))
        return result,"literal-brasa"
    src=show("public/muestra/"+("socio-lobby" if group=="clasica" else group)+".js")
    result=[]
    pattern=r"\{v:\x60([^\x60]+)\x60,n:\x60([^\x60]+)\x60,t:(\d+),p:\x60([^\x60]*)\x60,g:\[(.*?)\]\}"
    triple=r"\[\x60([^\x60]*)\x60,\x60([^\x60]*)\x60,\x60([^\x60]*)\x60\]"
    for vendor,provider,kind,prefix,entries in re.findall(pattern,src,re.S):
        for code,title,image in re.findall(triple,entries):
            path=image if image.startswith(("/","http")) else prefix+image
            result.append(row(title,provider,"crash" if kind=="9" else "slots",path,vendor,code,group))
    prefix=re.search(r"prefijoBanner:\x60([^\x60]+)\x60,juegos:\[(.*?)\]",src,re.S)
    if prefix:
        for code,title,image in re.findall(r"\{c:\x60([^\x60]+)\x60,n:\x60([^\x60]+)\x60,b:\x60([^\x60]+)\x60\}",prefix[2]):
            result.append(row(title,"Pragmatic Play Live","live",prefix[1]+image,"PP_LIVE_PRO",code,group))
    return result,"literal-provider-groups"
def legacy(preset,group,known):
    items,mode=literal_rows(group)
    # Where the historic bundle does not expose a supported literal record
    # format, retain the known local gallery rather than inventing records.
    if not items:
        items=[]
        for path,asset in MAP.items():
            if not path.startswith("public/games-cartas/"):continue
            if asset not in known:continue
            title,provider,cat=known[asset]
            items.append({"title":title,"provider":provider,"category":cat,"asset":asset})
        mode="local-gallery-fallback"
    for index,item in enumerate(items):item["id"]="item-"+hid(preset,index,item["title"],item["provider"])
    banners=[]
    if ART[preset].get("hero"):banners.append({"desktop":ART[preset]["hero"],"mobile":None})
    return items,[],banners,len(items),mode
def main():
    out=ROOT/"public/catalogs";out.mkdir(parents=True,exist_ok=True)
    normalized={};known={};report={}
    for preset,group in SOURCES.items():
        if group in ("clasica","azul","blaze","brasa","resort"):continue
        normalized[preset]=structured(preset,group)
        for item in normalized[preset][0]:
            if item["asset"]:known[item["asset"]]=(item["title"],item["provider"],item["category"])
    for preset,group in SOURCES.items():
        if preset not in normalized:normalized[preset]=legacy(preset,group,known)
    # The classic landing page features these games separately from its provider list.
    ember,collections,*_=normalized["ember"]
    names=["Rich Wilde and the Book of Dead","Aviator","Reactoonz","Rise of Olympus","Fire Joker","Legacy of Dead","Fortune Tiger","Fortune Ox"]
    members=[]
    for name in names:
        item=next((game for game in ember if game["title"]==name),None)
        if item is None:
            item=dict(next(game for game in normalized["flare"][0] if game["title"]==name))
            item.pop("seed",None)
            ember.append(item)
        members.append(item["id"])
    collections.append({"id":"original-recommendations","title":"Recomendados","items":members})
    for preset,(items,groups,banners,source_count,mode) in normalized.items():
        if not items:raise RuntimeError("Empty catalogue: "+preset)
        seed_count=0
        for index,seed in enumerate(ART[preset].get("cards",[])):
            matches=[i for i in items if i["asset"]==seed["asset"] and "seed" not in i]
            if not matches:continue
            item=next((i for i in matches if i["title"]==seed.get("title")),matches[0])
            item["seed"]=f"game-{index}"
            seed["title"]=item["title"]
            seed["provider"]=item["provider"]
            seed_count+=1
        if not banners and ART[preset].get("hero"):banners=[{"desktop":ART[preset]["hero"],"mobile":None}]
        data={"version":1,"preset":preset,"items":items,"groups":groups,"banners":banners}
        (out/(preset+".json")).write_text(json.dumps(data,ensure_ascii=False,separators=(",",":"))+"\n")
        report[preset]={"source_records":source_count,"normalized_records":len(items),"method":mode,
                        "without_local_image":sum(i["asset"] is None for i in items),"groups":len(groups),
                        "banners":len(banners),"mapped_editable_seeds":seed_count}
    (ROOT/"assets/preset-artwork.json").write_text(json.dumps(ART,ensure_ascii=False,sort_keys=True,separators=(",",":"))+"\n")
    (ROOT/"docs/catalog-parity.json").write_text(json.dumps({"source_commit":SOURCE,"catalogs":report},ensure_ascii=False,indent=2)+"\n")
    print(json.dumps(report,ensure_ascii=False,indent=2))
if __name__=="__main__":main()
