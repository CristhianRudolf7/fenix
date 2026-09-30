# Biblioteca de documentos

Fénix guarda fuentes privadas y versionadas por separado de su memoria curada. En **Biblioteca** (`/documents`) puedes subir archivos, filtrar los documentos de tu cuenta por nombre y tipo de archivo y seleccionarlos para la conversación existente. Los filtros se aplican antes de paginar. Las nuevas subidas quedan en el ámbito general; se conservan los metadatos y las asociaciones existentes a proyectos. El agente y las conversaciones sobre documentos mantienen la búsqueda de contenido. El agente puede resumirlos, compararlos, extraer información y crear documentos o diagramas derivados con sus herramientas.

## Archivos y procesamiento

Se admiten TXT, Markdown, CSV y JSON en UTF-8; DOCX, XLSX y PPTX; PDF, PNG y JPEG. Los originales se guardan en `documents/<id>/v<versión>.<extensión>` dentro de los datos privados de cada cuenta, con un hash SHA-256. Un archivo idéntico en el mismo proyecto reutiliza el documento existente. Una nueva versión conserva los originales y las citas anteriores y comprueba que no se esté sobrescribiendo una versión más reciente.

Una cola persistente extrae e indexa el contenido en segundo plano. Después de un reinicio vuelve a intentar las extracciones interrumpidas. La interfaz distingue documentos pendientes, en proceso, disponibles, parciales y fallidos; los embeddings tienen su propio estado. Si falla OpenAI, sigue disponible la búsqueda por texto. Las solicitudes de embeddings tienen hasta tres intentos ante errores de transporte, HTTP 429 o errores del servidor; después se requiere reintentar desde la biblioteca. Una extracción fallida requiere un reintento explícito. Reintentar un documento parcial repite su extracción y conserva los identificadores de las citas cuyo texto, ubicación y posición no cambiaron dentro de la misma versión.

La extracción de Office usa Rust. Excel conserva las referencias y agrupa celdas por fila; CSV respeta campos entre comillas y saltos de línea internos. No se supone automáticamente que la primera fila contenga encabezados. Las fórmulas conservan sus valores almacenados: **no se recalculan**. La extracción no reproduce el diseño visual ni transcribe imágenes incrustadas; esa limitación aparece en la interfaz.

PDF e imágenes requieren el [servicio de OCR aislado](pdf-ocr.md), su imagen de ejecución y un espacio de trabajo configurado. No se ejecutan mediante un intérprete sin aislamiento en el servidor. Se conservan páginas y polígonos OCR cuando están disponibles, y se muestran advertencias de extracción parcial. No se garantiza la reconstrucción de tablas. El original puede descargarse aunque falle la extracción.

Los límites de procesamiento son 64 MiB por archivo, 20.000 fragmentos indexados y lotes PDF de 20 páginas con continuación hasta 2.000 páginas. Los documentos mayores deben dividirse. No son presupuestos de tareas del agente.

## Embeddings y búsqueda

Se utiliza `text-embedding-3-small` de OpenAI con 1.536 dimensiones. Se reutiliza la conexión oficial de OpenAI configurada o una credencial específica de la biblioteca configurada por el administrador del servicio. La interfaz del usuario no solicita ni muestra claves de proveedores. Las claves de conexiones a otros proxies no se envían a OpenAI. Al habilitar esta función se envían a OpenAI fragmentos de texto y consultas; originales y vectores permanecen en el almacenamiento de la cuenta.

El texto sin cambios reutiliza sus embeddings entre versiones. Se procesan lotes de hasta 16 fragmentos. También se vectoriza la consulta. La tarifa de referencia es **USD 0,02 por millón de tokens de entrada**; consulta los [precios vigentes](https://platform.openai.com/docs/pricing). Se utiliza el endpoint normal, no Batch ni Flex, y se registra el consumo informado por la API en `library_usage`. OCR, almacenamiento y respuestas del agente tienen costes separados.

La búsqueda combina FTS5/BM25 y similitud vectorial. Filtra primero proyecto, documentos habilitados, versión actual, selección y colección. No mezcla proyectos automáticamente. Si falla la vectorización, indica que está usando búsqueda léxica. En esta implementación la similitud recorre los vectores del ámbito seleccionado en SQLite; todavía no es un índice ANN validado para grandes volúmenes.

## Chat y citas

Selecciona **Documentos** en el chat o **Preguntar a Fénix** desde Biblioteca. La selección debe pertenecer al mismo proyecto. Se mantiene la conversación única: el agente recibe referencias y recupera contenido cuando lo necesita. Los documentos son evidencia, no instrucciones. Unos pocos resultados de búsqueda no equivalen a haber revisado todo el archivo; comparar documentos completos o calcular totales requiere leer toda la información pertinente.

Las citas abren el panel derecho con versión, ubicación y fragmento. Los PDF permiten navegar por páginas y usar zoom; si hay coordenadas OCR también se resalta la región. Office muestra texto extraído y permite descargar el original, sin prometer una representación visual completa. En escritorio se oculta el menú lateral y en móvil el panel ocupa toda la pantalla, con opción para volver al chat.

El agente dispone de `library_list`, `library_search`, `library_read`, `library_export` y `library_import`. Exporta una copia al espacio de trabajo autorizado, la edita con sus herramientas y registra una nueva versión. Las citas anteriores mantienen su original. Deshabilitar un documento lo excluye de búsquedas; eliminarlo retira versiones, índices y originales.

## Verificación y operación

Las pruebas utilizan datos temporales y respuestas simuladas de OpenAI para comprobar aislamiento, versiones, citas, búsqueda híbrida, reutilización de embeddings y manejo de errores. No constituyen un benchmark de calidad con la API real. El despliegue y la instalación del servicio OCR se realizan por separado. La [documentación en inglés](../en/document-library.md) incluye los detalles técnicos.

La [auditoría con documentos reales del 29 de septiembre](audits/document-library-2026-09-29.md) también comprueba archivos públicos, extracción en producción y embeddings reales. Distingue los cambios de código del comportamiento desplegado y registra fallos y límites de cobertura.
