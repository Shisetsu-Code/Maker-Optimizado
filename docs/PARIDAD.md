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

## Auditoría visual reproducible

`visual-parity.yml` compara las 17 plantillas en lobby y panel, a 1440×900 y 393×852 (68 casos). El original se sirve de forma aislada y de solo lectura desde su commit fijo. El navegador bloquea solicitudes externas y registra sus dominios. Se guardan captura anterior, nueva, diferencias y reporte JSON/Markdown. La métrica cuenta píxeles cuya diferencia máxima de canal RGB supera 16; el umbral inicial de aceptación es 1% de píxeles distintos. Es una métrica diagnóstica, no prueba por sí sola equivalencia funcional. El modo estricto falla si alguna captura excede ese umbral o no se pudo capturar. No se ocultan diferencias ni se actualizan referencias para que el resultado apruebe artificialmente. El panel histórico era compartido; se compara para cada combinación de tema. La referencia directa no aplica las personalizaciones del editor anterior y bloquea recursos externos, por lo que el informe documenta esos límites.

## Panel administrativo compartido

Se recuperó la navegación de doce áreas con un único modelo de demostración Rust, compartido entre las plantillas. Se incluyen búsqueda y filtros sobre registros ficticios, notas y respuestas locales, cambio de estado de muestra, revisión visual, historial y vista de mantenimiento reversible. No se ejecutan operaciones reales ni se generan credenciales. El estado de interacción es de sesión, separado del proyecto editable; no modifica un sitio externo. Las pantallas detalladas, el editor fino de cada subelemento y las demás interacciones del panel original todavía requieren equivalencia específica; esta implementación no supone paridad total.

## Correcciones de consistencia

La exportación congela el proyecto y los filtros de catálogo al pulsar el botón,
carga el catálogo correspondiente aunque la vista todavía esté cargando y emplea
una instancia WASM independiente. Cambiar de plantilla durante la exportación no
modifica el archivo. En las vistas administrativas los filtros eliminan selecciones
ocultas, no admiten estados ajenos a la sección y Movimientos informa cuando no
hay coincidencias. La navegación y los registros siguen siendo exclusivamente
locales y ficticios.

Las miniaturas originales pendientes siguen contabilizadas (3.065 referencias
en Ember y 3.065 en Cobalt; 3.062 pares distintos de proveedor y título exactos).
La CI verifica todos los registros, las referencias gráficas locales y el número
de faltantes por catálogo. Las tarjetas sin imagen conservan su representación
local sin solicitudes externas. No se inventan asociaciones.

La auditoría histórica 45854061871 comparó 68 casos; ninguno alcanzó el umbral
del 1% (17,91%–99,98% de píxeles diferentes). Corresponde a la revisión
1adff1f37711b225a6ab067ca10cdca53524ae4a, anterior a las correcciones del panel.
El éxito del workflow en modo diagnóstico no implica paridad. La revisión
visual debe repetirse después de cambios en el renderizador, los componentes
o los estilos; no se certifica equivalencia 1:1.

## Geometría y navegación de referencia

Los perfiles neutrales de las familias con portada completa recuperan las
proporciones de banner, tarjetas y columnas de las hojas de estilo anteriores.
Las portadas sin texto añadido usan únicamente el arte local; los textos de las
portadas editoriales se conservan como contenido editable. Se restauran una
navegación inferior móvil, el menú lateral de la familia Violet, accesos a
categorías y navegación de la portada hacia el catálogo. Las tarjetas creadas
por el usuario participan también en favoritos y recientes. Cambios de
geometría se aplican por perfil si el usuario conserva la distribución base;
no se sobrescribe un proyecto guardado para imponer nuevos textos. Esto
recupera funciones y geometría, pero todavía requiere comparación visual.

## Miniaturas de una fuente autorizada

Se incorpora un importador offline con inventario de faltantes agrupado por
proveedor/título e IDs explícitos por plantilla. Verifica SHA-256 de la fuente,
rutas confinadas, dimensiones, identidad exacta y ausencia de imagen anterior;
recodifica y registra los nuevos recursos, deduplicados por contenido. No
incluye un contrato de API desconocido ni completa los faltantes con imágenes
de otros juegos. El inventario de 6.130 referencias permanece hasta recibir
material autorizado.

## Composiciones de catálogo recuperadas

Los doce perfiles de catálogo utilizan una composición propia en Rust y hojas
de estilo de geometría normalizadas, con el mismo modelo editable, IDs estables,
búsqueda, filtros, favoritos e historial. No se ejecuta ningún bundle anterior.
La portada, imágenes de experiencias, salas, menús, carrusel, tarjetas, grupos y
navegación móvil utilizan recursos verificados de la biblioteca. Los controles
adicionales se mantienen accesibles bajo «Más filtros y recientes». La
composición común permanece disponible si se cambia la distribución de base.
Los diseños guardados no se modifican automáticamente.

La auditoría exige que el catálogo de referencia esté inicializado y registra
dimensiones del viewport y posibles imágenes fallidas. El umbral de 1% y las
68 capturas permanecen intactos. No se declara equivalencia por la existencia
de esta implementación.

La composición editorial Ivory se reconstruyó con su estructura original de
dos carteles, portada, navegación, búsqueda, selección destacada, grilla y
sección final. Los carteles se resuelven por los IDs locales del catálogo,
no mediante imágenes de pantalla. El diseño sigue siendo editable y se
mantienen las operaciones de demostración del motor. El nombre de marca
predeterminado es neutral y la prueba configura el mismo nombre en la
referencia. La espera de la referencia descarta las pantallas de carga sin
recortar ni enmascarar los píxeles comparados.

### Revisión de diferencias residuales (27 de noviembre)

La revisión 0d68e30 obtuvo 15/68 vistas dentro de la tolerancia; aprobó también
la CI funcional y la auditoría de dependencias. Se corrigen el orden editorial
de las colecciones Jade, el orden y los iconos de categorías Rose, el ancho
intrínseco de las salas Amethyst, los estilos de cubierta Verdant/Crimson y el
ajuste de portadas responsive. La referencia y el límite del 1% se conservan.

### Panel compartido

Las referencias del panel de escritorio son idénticas para las 17 plantillas.
Se reconstruye una única composición responsive: lateral de 272 px, barra
móvil, cabecera, tarjetas de monitoreo y resumen de ejemplo, gráfica SVG
generada desde una serie sintética, y navegación a las doce áreas locales.
Los números de monitoreo y saldos son exclusivamente ficticios, sin operaciones
financieras ni autenticación. Las preferencias de marca y los componentes
editables conservan su modelo Rust y sus IDs. Los estilos base del panel no
dependen del perfil del lobby; los colores personalizados explícitamente
siguen aplicándose. La tipografía Figtree original se recupera localmente
con su licencia OFL. Se vuelve a medir la composición sin relajar el umbral.

### Familias clásicas
Se reemplaza la composición genérica de Ember, Cobalt, Flare y Coal por renderizadores nativos específicos con las geometrías de referencia: lateral, portadas, tarjetas de experiencia, recomendaciones y cartera de demostración. Se preservan los componentes editables y la navegación local. Las imágenes provienen exclusivamente de la biblioteca verificada. Se recuperan diez referencias de miniaturas faltantes (cinco títulos exactos de Pragmatic Play repetidos en ambos catálogos) a partir de la tira original de recomendaciones Cobalt; no se rellenan las coincidencias ambiguas. Las filas de recomendaciones se generan con Rust y mantienen el orden explícito, también en exportación autónoma. Estas composiciones requieren nueva medición visual; no se certifica equivalencia 1:1 todavía.
