#!/usr/bin/env bash
set -euo pipefail
# Import one immutable public revision. Never push or write to the source.
source_sha=9bc456f6ed1b4344a0cf4fb2e84ca4791061ba10
cd "$(dirname "$0")/.."
if [[ -f public/muestra/socio.js && -f package-lock.json ]]; then
  echo "Upstream files already present."
  exit 0
fi
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
git init -q "$tmp/source"
git -C "$tmp/source" fetch --depth=1 --no-tags https://github.com/Shisetsu-Code/Maker.git "$source_sha"
test "$(git -C "$tmp/source" rev-parse FETCH_HEAD)" = "$source_sha"
mkdir "$tmp/export"
git -C "$tmp/source" archive "$source_sha" public docs outputs scripts tests package-lock.json README.md | tar -x -C "$tmp/export"
mv "$tmp/export/README.md" "$tmp/export/docs/ORIGINAL-README.md"
# JS demo API unit tests belong to the replaced backend; Rust and HTTP tests supersede them.
rm "$tmp/export/tests/demo-api.test.mjs"
# Preserve all edited files in this repository (including the Rust/WASM adapters).
cp -an "$tmp/export/." .
# Allow Chromium in Linux CI while retaining Edge as the local default.
for f in tests/*-browser*.cjs; do
  sed -i "s/channel:[[:space:]]*'msedge'/channel:process.env.PLAYWRIGHT_CHANNEL||'msedge'/g" "$f"
done
test -f public/muestra/socio.js
test -f package-lock.json
echo "Imported $source_sha with original static paths."
