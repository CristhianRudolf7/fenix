# Design y resultados visuales

[English](../en/design.md)

Design es un espacio visual conectado a la conversación continua. En escritorio la vista aparece a la derecha y colapsa el menú lateral hasta cerrarla; hasta 900 px ocupa la pantalla, con opción Volver y tecla Escape. Cerrar la vista no cancela tareas del agente.

## Catálogo inicial

Incluye **72 ejemplos propios editables**: 20 interfaces web, 10 diagramas Mermaid, 10 presentaciones, 8 gráficas, 8 documentos, 8 hojas de cálculo, 6 imágenes SVG y 2 vídeos WebM cortos y sin sonido. Los datos son ilustrativos. No depende de Open Design ni incorpora sus plantillas.

- **Plantillas:** búsqueda, filtros, paginación y previsualización.
- **Favoritos:** selección persistente por usuario.
- **Mis diseños:** resultados guardados, filtro por proyecto e importación.
- **Identidad visual:** nombre, logo, colores y familia tipográfica. Los colores y la tipografía se aplican explícitamente a webs y presentaciones. El logo queda como recurso privado y aparece en la vista de identidad; las plantillas no lo insertan automáticamente.

Usar una plantilla crea una copia en el proyecto elegido o en el ámbito general. El panel permite inspeccionar contenido, guardar cambios, descargar, revisar versiones y restaurar una versión creando otra nueva. Continuar en el chat prepara un borrador con el ID del diseño y su proyecto; no envía el mensaje automáticamente. Los diagramas se consultan en modo de solo lectura y se modifican/restauran por chat. El editor de contenido para otros formatos usa JSON, mientras que los cambios por conversación siguen siendo la opción principal para usuarios no técnicos.

## Qué puede visualizar

- **Interfaces web:** HTML aislado. Scripts, formularios y recursos remotos desactivados; no representa una aplicación con backend funcionando. Los recursos de esta versión deben estar embebidos.
- **Diagramas:** Mermaid o grafos estructurados, con zoom, desplazamiento y pantalla completa; edición exclusiva mediante el agente. El renderizador se carga solo al necesitarlo.
- **Presentaciones:** diapositivas con título, cuerpo y notas, con navegación.
- **Datos:** barras, líneas y sectores con tabla de valores; admite barras negativas.
- **Documentos:** texto o PDF mediante el visor del navegador.
- **Hojas de cálculo:** tablas con filtro, ordenación y páginas de 50 filas, sin motor de fórmulas.
- **Imágenes:** SVG, PNG, JPEG, WebP y GIF.
- **Vídeo:** MP4/WebM con controles nativos y soporte de rangos; depende de los códecs del navegador.

Se importan HTML, Mermaid, TXT/MD, CSV y JSON estructurado. DOCX/XLSX/PPTX se muestran como **contenido extraído**, sin reproducir el formato Office original, imágenes internas ni recalcular fórmulas. Una extracción incompleta se identifica. Las herramientas nativas de documentos siguen disponibles para crear y editar esos archivos. Presentaciones y gráficas estructuradas se descargan como JSON; no se promete conversión automática a PDF o PPTX.

## Soporte del agente y persistencia

`design_templates` busca plantillas; `design_present` muestra entre una y ocho opciones ordenadas en el chat. `design_list` y `design_read` recuperan resultados; `design_publish` publica o actualiza un resultado, y `design_import` incorpora un archivo del espacio permitido. `diagram_edit` aplica cambios atómicos a nodos, conexiones y grupos con comprobación de revisión. Se descubren mediante `tool_search`.

El motor fija el proyecto desde la tarea y evita lecturas o cambios entre proyectos. El modelo de conversación recibe metadatos recientes de resultados y opciones para comunicar su estado. Las revisiones se guardan en SQLite por usuario y las actualizaciones exigen la versión actual; una edición atrasada no sobrescribe otra. Los archivos quedan en `<data-dir>/design-assets`, con IDs generados y acceso a través del usuario autenticado.

Límites de esta versión: archivos de hasta 64 MiB, contenido estructurado de 1 MB, HTML de 500 KB y Mermaid de 128 KB. Las miniaturas se toman de las vistas reales. El catálogo se reproduce con `scripts/build-design-catalog.py`; la guía en inglés detalla contratos, pruebas y regeneración de capturas.

## Segunda edición de las plantillas originales

Se renovaron las 64 plantillas: identidades web distintas, ocho temas de presentaciones,
documentos editoriales, composiciones para datos y diagramas, cuatro gráficos vectoriales
y dos piezas animadas. Las referencias fueron las galerías visuales de [Pitch](https://pitch.com/)
y [Canva](https://www.canva.com/templates/), sin copiar sus plantillas, recursos ni código.
Las ilustraciones SVG/CSS son locales y no requieren fuentes o imágenes externas.

La galería, navegación y controles no cambian. Los nuevos estilos se aplican solo a
contenido con `theme`; los diseños anteriores sin tema mantienen su presentación.
Las copias guardadas por el usuario no se sobrescriben. El contenido sigue siendo editable
y las miniaturas se capturan del visor real en un lienzo de 960×600.

Generación: `python3 scripts/build-design-catalog.py` para el catálogo y
`node scripts/build-design-motion.mjs` para las piezas animadas. Las composiciones están
en `scripts/design_template_styles.py`. Tras compilar el servidor, la prueba de catálogo
de Playwright permite renovar las miniaturas y comprobar todas las plantillas.
Después de regenerarlas, vuelve a compilar el frontend y el servidor para incluirlas
en la galería distribuida. La prueba `Template gallery` comprueba las portadas servidas
y guarda capturas para la revisión visual en escritorio y móvil.
Para renovar solo ciertos formatos, define `DESIGN_TEMPLATE_KINDS` con los tipos
separados por comas; sin esa variable se comprueban las 72 plantillas.

## Tercera colección: composiciones distintas y gráficos sobre blanco

Las miniaturas de gráficas muestran el gráfico sin los controles de la tabla; los datos
editables siguen disponibles en la vista del diseño.

El catálogo contiene **72 plantillas originales locales**: 20 webs, 10 presentaciones,
10 diagramas, 8 gráficos, 8 documentos, 8 hojas de cálculo, 6 imágenes SVG y 2 animaciones.
Las ocho nuevas son cuatro webs (arquitectura, investigación orbital, festival de sonido
y viajes), dos presentaciones (sistemas y diario cultural) y dos carteles.

`composition` selecciona distribuciones, tipografías e ilustraciones diferentes;
`theme` sigue controlando la paleta. Las diez presentaciones tienen portadas distintas.
Los documentos y las hojas de cálculo también se componen según su función.
Los gráficos y diagramas usan fondo blanco tanto en el visor como en las miniaturas.
Los gráficos se dibujan a partir de sus datos editables, sin añadir métricas ficticias.
Las copias previamente guardadas no se sobrescriben.

Se examinaron diez referencias de Open Design y los repositorios de Slidev y Presenton;
la tabla de [referencias y procedencia](../en/design.md#reference-research-and-asset-provenance)
detalla los ejemplos y criterios. No se copiaron sus plantillas, código, marcas, imágenes
ni fuentes. Las composiciones y las ilustraciones CSS/SVG se escribieron aquí y no
requieren proveedores, APIs ni recursos externos. Las bibliotecas existentes del visor
conservan sus licencias.

Las fuentes nuevas están en `scripts/design_collection.py` y
`apps/web/src/design/template-compositions.{tsx,css}`. El generador, las comprobaciones
en navegador y las miniaturas siguen el procedimiento anterior.


[Diagramas editados por el agente: navegación, almacenamiento y revisiones](diagrams.md).

## Elegir un diseño desde el chat

Las peticiones de creación visual abren **Elegir diseño** en el panel derecho existente
antes de enviarse al agente. Se reconoce la categoría en peticiones habituales en español
e inglés. El botón **Elegir diseño** del cuadro de mensaje permite abrir el catálogo
manualmente. Las correcciones de diseños existentes, las consultas explicativas y las
peticiones explícitas “sin plantilla / desde cero” siguen su flujo normal.

Se muestran las miniaturas reales de Design y todas las páginas de la categoría, con
búsqueda y cambio de categoría. Puedes ajustar tu descripción, seleccionar una plantilla
y pulsar **Generar con esta plantilla**, o continuar con **Generar sin plantilla**. Cerrar
el panel conserva la descripción en el cuadro del chat durante la misma sesión de página.
En móvil se aprovecha el panel a pantalla completa existente.

La selección conserva las fuentes adjuntas y la referencia a la tarea respondida. El
servidor valida el identificador y transmite la preferencia al agente, que debe adaptar
el contenido conservando la composición y generar el resultado en el proyecto resuelto.
El selector no crea una copia vacía ni envía nada al modelo antes de continuar. Los
resultados publicados se abren con el mecanismo habitual del espacio de trabajo.

Los errores de carga o envío conservan la descripción; si el catálogo falla, aún puedes
crear sin plantilla. Las pruebas y el contrato de API se detallan en la guía en inglés.
