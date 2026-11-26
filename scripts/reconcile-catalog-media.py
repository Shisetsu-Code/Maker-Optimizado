#!/usr/bin/env python3
"""Fill missing thumbnails only through unambiguous same-title/provider matches."""
from pathlib import Path
import json
import re
import unicodedata
from collections import defaultdict

root=Path(__file__).resolve().parents[1]
directory=root/"public/catalogs"
report=json.loads((root/"docs/catalog-parity.json").read_text())
catalogs={p.stem:json.loads(p.read_text()) for p in directory.glob("*.json")}
def key(s):
    s=unicodedata.normalize("NFKD",s).lower()
    return "".join(c for c in s if c.isalnum() and not unicodedata.combining(c))
def provider(s):
    s=key(s)
    if s.startswith("slot"):s=s[4:]
    aliases={"pragmatic":"pragmaticplay","playngo":"playngo","playandgo":"playngo",
             "pgsoft":"pgsoft","pocketgamesoft":"pgsoft","pgsoftgaming":"pgsoft",
             "pragmaticplaylive":"pragmaticplaylive","pplivepro":"pragmaticplaylive",
             "3oaks":"3oaksgaming","ka":"kagaming","kag":"kagaming",
             "redrakegaming":"redrake","pushgaming":"push","spribegaming":"spribe"}
    return aliases.get(s,s)
def identity(item):
    return (provider(item["provider"]),key(item["title"]))
known=defaultdict(set)
for name,data in catalogs.items():
    if report["catalogs"][name]["method"]!="structured-exact":continue
    for item in data["items"]:
        if item["asset"]:known[identity(item)].add(item["asset"])
result={}
for name,data in catalogs.items():
    changes=[]
    for item in data["items"]:
        if item["asset"]:continue
        matches=known.get(identity(item),set())
        if len(matches)==1:
            item["asset"]=next(iter(matches));changes.append({"id":item["id"],"asset":item["asset"]})
    result[name]=changes
    report["catalogs"][name]["cross_collection_matches"]=len(changes)
    report["catalogs"][name]["without_local_image"]=sum(i["asset"] is None for i in data["items"])
    (directory/(name+".json")).write_text(json.dumps(data,ensure_ascii=False,separators=(",",":"))+"\n")
(root/"docs/catalog-parity.json").write_text(json.dumps(report,ensure_ascii=False,indent=2)+"\n")
(root/"docs/catalog-media-reconciliation.json").write_text(json.dumps(result,ensure_ascii=False,separators=(",",":"))+"\n")
print(json.dumps({k:{"recovered":len(v),"missing":report["catalogs"][k]["without_local_image"]} for k,v in result.items()},indent=2))
