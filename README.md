# Maker Studio

Constructor visual local-first para interfaces de entretenimiento de demostración. Motor declarativo y renderizador HTML en Rust, WebAssembly en navegador y Cloudflare Workers. Sin frameworks frontend ni servicios de terceros necesarios para editar.

## Capacidades

- 17 presets propios basados en las familias de color y distribución de la edición anterior.
- Un renderizador, componentes tipados con identificadores permanentes, lobby y panel separados.
- Edición de tema, selección visual, capas, arrastrar para ordenar dentro del grupo, añadir/eliminar/reordenar componentes y tarjetas.
- Imágenes raster propias, estilos acotados, historial, importación/exportación JSON y exportación HTML autónoma (vista estática; no integra motores de juego).
- IndexedDB con guardado transaccional y rechazo de sobrescrituras concurrentes.
- API Rust versionada: salud, presets, validación acotada y previsualización estática.
- Workers Static Assets sirve el frontend sin invocar el Worker para cada archivo.

## Desarrollo

Requiere Rust 1.100.0 (fijado en rust-toolchain.toml), Node 24 y npm.

En el primer checkout, si faltan los lockfiles, generarlos con `cargo generate-lockfile` y `npm install --package-lock-only --ignore-scripts`. Las revisiones posteriores deben conservar los archivos de bloqueo.

```sh
cargo install worker-build --version 0.9.0 --locked
cargo install wasm-bindgen-cli --version 0.2.130 --locked
npm ci
npm test
npm run dev
```

Abrir http://127.0.0.1:8787. Con el servidor activo: `npm run test:runtime` y `npm run test:browser` (instalar Chromium con `npx playwright install chromium`).

Publicación: `npx wrangler deploy --dry-run`, después `npx wrangler login` y `npm run deploy`. No incluye identificadores de cuenta, credenciales ni publicación automática.

## Alcance

Esta es una reconstrucción independiente. No reproduce píxel a píxel cada diseño anterior: conserva paletas, jerarquías visuales y familias de distribución, usa ilustraciones CSS originales y evita marcas/recursos sin licencia comprobada. Los archivos de proyecto antiguos no son compatibles automáticamente con el esquema nuevo. No hay apuestas, pagos, cuentas reales ni sesiones de terceros. El editor funciona localmente; D1/R2 se reservan para una eventual publicación con autenticación y almacenamiento remoto. La API de validación limita el cuerpo a 256 KiB; imágenes y proyectos grandes se validan localmente.

Ver [arquitectura](docs/ARCHITECTURE.md) y [procedencia](docs/PROVENANCE.md). La edición anterior está preservada en el historial de Git y la rama principal mientras esta reconstrucción se verifica.
