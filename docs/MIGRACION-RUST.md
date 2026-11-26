# Migración a Rust

## Cambios
- API Worker en Rust con rutas y métodos explícitos, 404 para endpoints desconocidos, 405 + Allow, límite streaming de 4096 bytes, respuestas no-store y URLs de lanzamiento del mismo origen. El endpoint /api/monitoreo se mantiene como simulación para el panel histórico.
- Rust nativo y navegador WASM comparten reglas de contenido y preferencias. Se mantiene la compatibilidad de JSON; la truncación UTF-16 no corta un carácter suplementario. La validación de imágenes se mantiene en JS para evitar duplicación de memoria y copias Base64.
- Se conserva el editor original y sus plantillas; no se pretende afirmar que sus bundles capturados fueron reconstruidos.
- Guardado IndexedDB con compare-and-swap sobre la revisión; un conflicto conserva el diseño en memoria y pide exportar/recargar.
- Límite de reglas por pantalla informado en la interfaz, en vez de descartar una edición nueva.
- Cabeceras estáticas y Worker, CSP de conexión local y restricciones de formularios/marcos. Revisar licencias de los recursos originales antes de publicar.
- CI: pruebas Rust, JS, HTTP, navegador y dry-run, sin deploy ni credenciales Cloudflare.
- El inventario de duplicados no sustituye las URLs de recursos. Cloudflare Static Assets ya deduplica por contenido; migrar todo a R2 sin necesidad añade complejidad.

## Pendiente de validación
Las pruebas históricas del repositorio fuente no prueban este build. Revisar los resultados de la nueva CI antes de publicar. No se presume una mejora porcentual de latencia sin benchmark. La identificación de elementos heredados sigue dependiendo del DOM: se recomienda migrar gradualmente las plantillas a componentes con identificadores estables. La documentación antigua de publicación y pruebas corresponde a la implementación anterior.

## Importación completada
El historial contiene el commit de importación de la revisión de origen. Se retiró de CI la escritura automática de archivos; las comprobaciones ordinarias usan permisos de solo lectura. El script de importación queda disponible para reproducir la base, sin alterar el repositorio original. La CI conserva el Cargo.lock resuelto como artefacto para revisión.
