# Requisitos de paridad de Maker

Objetivo: sustituir el motor, no reducir las capacidades. La revisión de referencia es
el Maker original fijado en 9bc456f6ed1b4344a0cf4fb2e84ca4791061ba10,
conservado en el historial y en legacy/pre-rebuild. El código operativo nuevo no
ejecuta los bundles anteriores. Las imágenes autorizadas se preservan byte a byte.

## Puertas de aceptación

La CI ordinaria verifica el motor Rust, WASM, contrato HTTP, biblioteca gráfica,
búsqueda y filtros del catálogo, páginas de 48 registros, edición sparse,
restauración, persistencia, exportación y aislamiento de plantillas.

No se considera demostrada la paridad por el mero hecho de que compile o de que
las 17 plantillas existan. También son necesarios:
- Comparar el número y contenido de registros de catálogo contra cada fuente.
- Comparar cada pantalla y tamaño con capturas de referencia, con revisión de
  diferencias visuales; no sustituir arte por ilustraciones genéricas.
- Validar los controles de marca, fondos, tipografía, campos y bloques y sus
  efectos en lobby/panel y escritorio/móvil.
- Validar las interacciones locales originales, incluidos los flujos del panel.
- Importar archivos anteriores mediante una migración explícita, informando
  cualquier regla no representable en el modelo declarativo.

El informe generado catalog-parity.json distingue las 12 fuentes estructuradas
de la extracción de literales de las cinco plantillas históricas. Los nombres
de proveedores y juegos son datos de presentación; no se reutilizan protocolos
ni endpoints de juego, autenticación o pagos. Los registros sin imagen local
se mantienen y se contabilizan; no se descargan recursos externos.

## Estado de esta fase

Catálogo Rust, filtros, destacados, favoritos, recientes, carrusel y restauración
individual/de pantalla implementados; pendientes de verificar sobre los datos
normalizados. No hay apuestas reales ni cuentas reales. Las interacciones
complejas del panel, las opciones de estilo avanzadas, la migración de JSON
antiguo y la comparación visual exacta continúan pendientes. No se certifica
paridad 1:1 hasta superar todas las puertas de aceptación.

## Apariencia recuperada

La nueva estructura admite el alcance global de fuente, escala 85–120%, peso, esquinas y fondo (mostrar/ocultar, ajuste, posición y oscurecimiento), además de color de botones/brillos por plantilla. Los estilos por componente incluyen tamaño 8–110 px, peso, alineación, bordes, dimensiones y ajuste/posición de imágenes. Se restauran las fuentes locales Barlow, Archivo y Oswald con sus avisos SIL OFL. La edición de fragmentos arbitrarios del DOM y los fondos independientes por subelemento todavía requieren correspondencia adicional.

## Migración anterior

Se recuperan los ajustes globales, colores y nombre de marca, portada y orden de secciones Resort, y reglas con correspondencia semántica única. Los selectores arbitrarios de DOM, logos descargados como archivo separado, perillas sin equivalente y estilos de fragmentos no se aplican por aproximación: el importador descarga un informe de los elementos pendientes. El JSON original no se modifica. Las miniaturas ausentes solo se reconcilian cuando una colección autorizada contiene una coincidencia inequívoca de título y proveedor.
