# Maker Studio

Constructor visual local-first para interfaces de entretenimiento de demostración. Motor declarativo y renderizador HTML en Rust, WebAssembly en navegador y Cloudflare Workers. Sin frameworks frontend ni servicios de terceros necesarios para editar.

## Capacidades

- 17 presets propios basados en las familias de color y distribución de la edición anterior.
- Un renderizador, componentes tipados con identificadores permanentes, lobby y panel separados.
- Edición de tema, selección visual, capas, arrastrar para ordenar dentro del grupo, añadir/eliminar/reordenar componentes y tarjetas.
- Biblioteca gráfica original deduplicada, imágenes raster propias, estilos acotados, historial, importación/exportación JSON y exportación HTML autónoma (vista estática; no integra motores de juego).
- IndexedDB con guardado transaccional y rechazo de sobrescrituras concurrentes.
- API Rust versionada: salud, presets, validación acotada y previsualización estática.
- Workers Static Assets sirve el frontend sin invocar el Worker para cada archivo.

## Desarrollo

Requiere Rust 1.100.0 (fijado en rust-toolchain.toml), Node 24 y npm.

Los lockfiles generados por la primera verificación se incorporaron al repositorio. Usá `cargo build --locked` y `npm ci`; no regeneres los lockfiles salvo para una actualización explícita de dependencias.

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

Esta es una reconstrucción independiente. No reproduce píxel a píxel cada diseño anterior: conserva paletas, jerarquías visuales y familias de distribución, reutiliza la biblioteca gráfica original autorizada por el propietario, con identificadores por contenido y sin bundles heredados. Los archivos de proyecto antiguos no son compatibles automáticamente con el esquema nuevo. No hay apuestas, pagos, cuentas reales ni sesiones de terceros. El editor funciona localmente; D1/R2 se reservan para una eventual publicación con autenticación y almacenamiento remoto. La API de validación limita el cuerpo a 256 KiB; imágenes y proyectos grandes se validan localmente.

Ver [arquitectura](docs/ARCHITECTURE.md) y [procedencia](docs/PROVENANCE.md). La edición anterior está preservada en el historial de Git y la rama principal mientras esta reconstrucción se verifica.

## Biblioteca gráfica

La importación reproducible se realiza una vez con `scripts/import-media.py` usando la revisión fijada. La biblioteca se sirve desde `public/media/objects` y se consulta por colecciones paginadas; no se cargan todas las imágenes al abrir el editor. Los recursos gráficos originales quedan disponibles sin recomprimir. `docs/asset-provenance.json` registra la correspondencia con sus rutas originales. El exportador HTML incorpora las imágenes seleccionadas como datos, con un límite de 24 MiB de recursos externos al proyecto para evitar agotamiento de memoria.

Verificación gráfica: `npm run test:media` comprueba la integridad de todos los archivos restaurados y sus referencias. Las pruebas E2E verifican portadas, tarjetas, biblioteca, logotipos y HTML autónomo sin red.
