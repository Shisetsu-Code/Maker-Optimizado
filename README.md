# Maker Optimizado — Rust + Cloudflare Workers

Migración independiente de [Maker](https://github.com/Shisetsu-Code/Maker), fijada al commit `9bc456f6ed1b4344a0cf4fb2e84ca4791061ba10`. El proyecto es un editor visual de demostración, sin apuestas reales, pagos, cuentas ni conexiones a motores de terceros.

## Arquitectura

- Rust/WebAssembly: Worker completo, contrato HTTP y normalización portable de diseño/contenido; build de navegador separado y sin dependencias de DOM.
- JavaScript: DOM, IndexedDB, CSS dinámico, imágenes y adaptadores de las 17 plantillas existentes. Los datos raster no cruzan la frontera WASM.
- Workers Static Assets: archivos estáticos servidos directamente por Cloudflare, con ETag y caché nativa. Rust solo interviene en API, raíz y alias heredado.
- CLI Rust: inventario SHA-256, bytes únicos/redundantes, límites de archivos y tamaño. No altera URLs ni elimina recursos; los duplicados se reportan.
- Persistencia local: revisión transaccional para impedir sobrescrituras entre pestañas.

## Preparación

La primera ejecución de CI copia las plantillas y recursos de la revisión pública fija; el script se puede ejecutar localmente si aún faltan. El repositorio original no se modifica.

```sh
bash scripts/import-upstream.sh
rustup target add wasm32-unknown-unknown
cargo install worker-build --version 0.8.7 --locked
cargo install wasm-bindgen-cli --version 0.2.130 --locked
npm ci
cargo test --lib
npm test
npm run check:assets
npm run dev
```

Abrir http://127.0.0.1:8787. En otra terminal: `npm run test:runtime`.

Publicación: `npx wrangler deploy --dry-run`, luego `npx wrangler login` y `npm run deploy`. El nombre del Worker es `maker-optimizado`, independiente del original. No se incluye account_id ni credenciales. La CI no publica.

## Cloudflare

No se agrega D1, KV, R2 ni Durable Objects al flujo obligatorio: los diseños son privados/locales y no hay un requisito de sincronización o colaboración. Workers Static Assets ya sirve el catálogo con caché y direccionamiento por contenido. Para proyectos remotos se propone D1 para metadatos/autorización, R2 para imágenes por hash y control de concurrencia mediante versión; no implementados en esta revisión.

Ver [migración y límites](docs/MIGRACION-RUST.md). La documentación original se conserva como referencia histórica.
