#!/usr/bin/env python3
"""One-time, reproducible import of user-authorized graphic assets.

No captured code is imported. Files are byte-preserved; their Git blob hash
becomes the stable address. Original paths remain in the provenance manifest.
"""
import collections
import hashlib
import json
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
REF = "8a8b09cfdda55d8233b2d61c4d878bb112f4f591"
MEDIA = ROOT / "public/media"
OBJECTS = MEDIA / "objects"
ARCHIVE = ROOT / "assets/archive"
COLLECTIONS = MEDIA / "collections"
EXTENSION = re.compile(r"\.(png|jpe?g|webp|avif|gif|svg)(?:\.[a-f0-9]{12})?$", re.I)
SOURCES = {
    "ember": "rushybet", "cobalt": "azul", "flare": "blaze", "coal": "brasa",
    "ivory": "rushybet", "violet": "bateupix", "saffron": "cobraya",
    "midnight": "lunabet", "amethyst": "multisala", "jade": "pinaplay",
    "rose": "celuapuestas40", "garnet": "bet30", "brass": "casinotower30",
    "aurum": "fenarys", "verdant": "fortunasjk", "crimson": "ravenbet",
    "onyx": "luckrex",
}
NAMES = {"ember": "Ember Classic", "cobalt": "Cobalt", "flare": "Flare",
         "coal": "Coal", "ivory": "Ivory Resort", "violet": "Violet",
         "saffron": "Saffron", "midnight": "Midnight", "amethyst": "Amethyst Rooms",
         "jade": "Jade", "rose": "Rose", "garnet": "Garnet", "brass": "Brass",
         "aurum": "Aurum", "verdant": "Verdant", "crimson": "Crimson", "onyx": "Onyx Gold"}
HERO = {
    "ember": "public/rushybet/v2/hero-dragon.webp",
    "cobalt": "public/muestra/azul/assets/hero-bg.webp",
    "flare": "public/muestra/rushybet/v2/pop-slots.webp",
    "coal": "public/muestra/rushybet/v3/cut-ruleta.webp",
    "ivory": "public/games-cartas/slot-playngo/bookofdead.webp",
}

def git(*args):
    return subprocess.check_output(["git", "-C", str(ROOT), *args])

def dump(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, sort_keys=True,
                               separators=(",", ":")) + "\n", encoding="utf-8")

def signature(data):
    if data.startswith(b"\x89PNG\r\n\x1a\n"):
        return "png"
    if data.startswith(b"\xff\xd8\xff"):
        return "jpg"
    if data[:4] == b"RIFF" and data[8:12] == b"WEBP":
        return "webp"
    if data[:6] in (b"GIF87a", b"GIF89a"):
        return "gif"
    if data[4:8] == b"ftyp" and any(x in data[8:48] for x in (b"avif", b"avis")):
        return "avif"
    return None

def read_blob(proc, oid):
    proc.stdin.write(oid.encode() + b"\n")
    proc.stdin.flush()
    meta = proc.stdout.readline().split()
    if len(meta) != 3 or meta[1] != b"blob":
        raise RuntimeError(f"Missing blob {oid}")
    size = int(meta[2])
    data = proc.stdout.read(size)
    if len(data) != size or proc.stdout.read(1) != b"\n":
        raise RuntimeError(f"Truncated blob {oid}")
    if hashlib.sha1(f"blob {size}\0".encode() + data).hexdigest() != oid:
        raise RuntimeError(f"Blob integrity error {oid}")
    return data

def main():
    if git("rev-parse", REF).decode().strip() != REF:
        raise RuntimeError("Incorrect source revision")
    for d in (OBJECTS, COLLECTIONS, ARCHIVE):
        d.mkdir(parents=True, exist_ok=True)
    entries = []
    for row in git("ls-tree", "-r", "-z", REF, "--", "public").split(b"\0"):
        if not row:
            continue
        metadata, name = row.split(b"\t", 1)
        mode, kind, oid = metadata.decode().split()
        path = name.decode("utf-8")
        if kind == "blob" and mode == "100644" and EXTENSION.search(path):
            entries.append((path, oid))
    if not entries:
        raise RuntimeError("No graphics found")
    groups = collections.defaultdict(list)
    mapped, provenance, by_hash = {}, {}, {}
    deployed_bytes = archived_bytes = 0
    with subprocess.Popen(["git", "-C", str(ROOT), "cat-file", "--batch"],
                          stdin=subprocess.PIPE, stdout=subprocess.PIPE) as proc:
        for path, oid in entries:
            if oid not in by_hash:
                data = read_blob(proc, oid)
                ext = signature(data)
                if ext and len(data) <= 25*1024*1024:
                    key = f"{oid}.{ext}"
                    (OBJECTS / key).write_bytes(data)
                    deployed_bytes += len(data)
                    by_hash[oid] = (key, len(data), True)
                else:
                    # Preserve vectors and non-raster/mislabeled content outside the serving tree.
                    ext = "svg" if path.lower().endswith(".svg") else "bin"
                    key = f"{oid}.{ext}"
                    (ARCHIVE / key).write_bytes(data)
                    archived_bytes += len(data)
                    by_hash[oid] = (key, len(data), False)
            key, size, deployed = by_hash[oid]
            provenance[path] = {"id": key, "git_blob": oid, "bytes": size, "served": deployed}
            if deployed:
                mapped[path] = key
                parts = path.split("/")
                group = parts[2] if parts[1] == "muestra" else parts[1]
                groups[group].append(key)
        proc.stdin.close()
        if proc.wait() != 0:
            raise RuntimeError("cat-file failed")

    def resolve(group, path):
        if not isinstance(path, str):
            return None
        if path.startswith("/"):
            return mapped.get("public" + path)
        return mapped.get(f"public/muestra/{group}/{path}")

    shared = groups["games-cartas"] + groups["games-zitro"]
    defaults, collections_info = {}, []
    all_ids = sorted({key for key, _, served in by_hash.values() if served})
    def collection(cid, label, ids):
        unique = list(dict.fromkeys(ids))
        dump(COLLECTIONS / f"{cid}.json", unique)
        collections_info.append({"id": cid, "name": label, "count": len(unique)})
    collection("all", "Todos los recursos", all_ids)
    cinematic = [key for path, key in mapped.items() if path.startswith(("public/rushybet/", "public/muestra/rushybet/"))]
    collection("cinematic", "Composiciones cinematográficas", cinematic)
    for preset, group in SOURCES.items():
        catalog = {}
        try:
            catalog = json.loads(git("show", f"{REF}:public/muestra/{group}/catalog.json"))
        except subprocess.CalledProcessError:
            pass
        banners = [resolve(group, p) for p in catalog.get("banners", []) + catalog.get("bannerMobile", [])]
        artwork = [resolve(group, p) for p in catalog.get("artwork", {}).values()]
        prioritized = [p for p in [mapped.get(HERO.get(preset, "")), *banners, *artwork] if p]
        if not prioritized:
            prioritized = cinematic[:1]
        cards, live = [], []
        for game in catalog.get("games", []):
            key = resolve(group, game.get("image"))
            title = game.get("name", "")
            if not key or not isinstance(title, str):
                continue
            title = "".join(c for c in title if c.isprintable())[:120]
            target = live if "live" in str(game.get("category", "")).lower() else cards
            if len(target) < (3 if target is live else 12):
                target.append({"asset": key, "title": title})
        # Original common catalog artwork for the five historical layouts.
        if not cards:
            for n, key in enumerate(list(dict.fromkeys(shared))[:12]):
                cards.append({"asset": key, "title": ""})
        if not live:
            for key in list(dict.fromkeys([k for k in groups[group] if k not in prioritized] + shared))[:3]:
                live.append({"asset": key, "title": ""})
        defaults[preset] = {"hero": prioritized[0] if prioritized else None,
                            "cards": cards, "live": live,
                            "categories": [p for p in artwork if p][:5]}
        collection(preset, NAMES[preset], prioritized + groups[group] + shared)
    dump(ROOT / "assets/preset-artwork.json", defaults)
    dump(MEDIA / "index.json", {"version": 1, "unique": len(all_ids),
                              "collections": collections_info})
    dump(ROOT / "docs/asset-provenance.json", {"source_commit": REF,
         "authorization": "User confirmed ownership and reuse of graphic resources",
         "files": provenance, "source_paths": len(entries), "unique_raster": len(all_ids),
         "raster_bytes": deployed_bytes, "archived_unique": len(by_hash)-len(all_ids),
         "archived_bytes": archived_bytes})
    print(json.dumps({"source_paths":len(entries), "unique_raster":len(all_ids),
                      "raster_bytes":deployed_bytes, "archived_unique":len(by_hash)-len(all_ids),
                      "archived_bytes":archived_bytes}, indent=2))
    if len(all_ids) >= 20000:
        raise RuntimeError("Static Assets file count exceeds configured limit")
    if any(not v["hero"] for v in defaults.values()):
        raise RuntimeError("At least one preset lacks a default hero")

if __name__ == "__main__":
    main()
