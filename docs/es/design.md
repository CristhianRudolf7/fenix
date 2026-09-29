# Design y resultados visuales

[English](../en/design.md)

Design es el motor visual local de la conversación continua. Ya no existe una página Design: las propuestas de diseño se eligen en el chat y los resultados terminados se abren en el panel derecho del chat. En escritorio ese panel colapsa el menú lateral hasta cerrarlo; hasta 900 px ocupa la pantalla, con opción Volver y tecla Escape. Cerrar la vista no cancela tareas del agente.

## Catálogo inicial

Incluye **72 ejemplos propios editables**: 20 interfaces web, 10 diagramas Mermaid, 10 presentaciones, 8 gráficas, 8 documentos, 8 hojas de cálculo, 6 imágenes SVG y 2 vídeos WebM cortos y sin sonido. Los datos son ilustrativos. No depende de Open Design ni incorpora sus plantillas.

- **Propuestas:** el agente presenta plantillas del catálogo como un carrusel de portadas reales dentro de su respuesta; cada una se abre en una vista modal centrada.
- **Favoritos:** la estrella de cada tarjeta se guarda por usuario y el agente ve los favoritos al buscar plantillas.
- **Resultados:** los resultados guardados aparecen sobre el cuadro de mensaje y se abren en el panel derecho, con descarga y versiones.
- **Identidad visual:** el nombre, logo, colores y tipografía guardados siguen disponibles para el agente mediante `design_templates`. El editor de identidad y la importación de archivos de la antigua página Design se retiraron con ella; los datos y las APIs no cambian.

Los cambios por conversación son la vía principal para modificar un resultado. Los diagramas se consultan en modo de solo lectura y se modifican por chat.

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

`design_templates` busca plantillas e informa su tema de color, composición, los favoritos y si hay una identidad visual guardada; `design_present` muestra entre una y ocho propuestas ordenadas como carrusel en el chat. `design_list` y `design_read` recuperan resultados; `design_publish` publica o actualiza un resultado, y `design_import` incorpora un archivo del espacio permitido. `diagram_edit` aplica cambios atómicos a nodos, conexiones y grupos con comprobación de revisión. Se descubren mediante `tool_search`.

El motor fija el proyecto desde la tarea y evita lecturas o cambios entre proyectos. El modelo de conversación recibe metadatos recientes de resultados y opciones para comunicar su estado. Las revisiones se guardan en SQLite por usuario y las actualizaciones exigen la versión actual; una edición atrasada no sobrescribe otra. Los archivos quedan en `<data-dir>/design-assets`, con IDs generados y acceso a través del usuario autenticado.

Límites de esta versión: archivos de hasta 64 MiB, contenido estructurado de 1 MB, HTML de 500 KB y Mermaid de 128 KB. Las miniaturas se toman de las vistas reales. El catálogo se reproduce con `scripts/build-design-catalog.py`; la guía en inglés detalla contratos, pruebas y regeneración de capturas.

## Segunda edición de las plantillas originales

Se renovaron las 64 plantillas: identidades web distintas, ocho temas de presentaciones,
documentos editoriales, composiciones para datos y diagramas, cuatro gráficos vectoriales
y dos piezas animadas. Las referencias fueron las galerías visuales de [Pitch](https://pitch.com/)
y [Canva](https://www.canva.com/templates/), sin copiar sus plantillas, recursos ni código.
Las ilustraciones SVG/CSS son locales y no requieren fuentes o imágenes externas.

Los nuevos estilos se aplican solo a
contenido con `theme`; los diseños anteriores sin tema mantienen su presentación.
Las copias guardadas por el usuario no se sobrescriben. El contenido sigue siendo editable
y las miniaturas se capturan del visor real en un lienzo de 960×600.

Generación: `python3 scripts/build-design-catalog.py` para el catálogo y
`node scripts/build-design-motion.mjs` para las piezas animadas. Las composiciones están
en `scripts/design_template_styles.py`. Tras compilar el servidor, la prueba de catálogo
de Playwright permite renovar las miniaturas y comprobar todas las plantillas.
Después de regenerarlas, vuelve a compilar el frontend y el servidor para incluirlas
en el carrusel de propuestas. `tests/design-creation.spec.js` comprueba las portadas
servidas y guarda capturas para la revisión visual en escritorio y móvil.
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

Para una nueva página web o presentación, el agente primero aprovecha lo que ya sabe:
la petición, la conversación, la memoria, las plantillas favoritas y la identidad visual
guardada. Si todavía falta una preferencia visual importante (propósito o público, paleta
de colores, estilo o tono), hace una sola pregunta breve sobre lo que falta y espera la
respuesta. No vuelve a preguntar lo que ya se indicó, no inventa respuestas y omite la
pregunta si le pides que decida, das un encargo completo o quieres un diseño propio.

Con esas preferencias, busca con `design_templates` y llama a `design_present`. El chat
muestra las propuestas dentro de esa respuesta como un carrusel horizontal y compacto de
portadas reales, numeradas para que también puedas responder por texto («la segunda»).
Los botones anterior/siguiente aparecen cuando hay más tarjetas; cada tarjeta se puede
enfocar con el teclado y tiene una estrella de favorito. Al pulsarla se abre la vista
modal centrada existente (pantalla completa en móvil, Escape o clic fuera para cerrar).
El panel derecho nunca se usa para elegir propuestas; queda para resultados terminados,
archivos y fuentes de la biblioteca.

**Elegir este diseño** envía un mensaje breve con la elección (`design_choice`). El
servidor valida la plantilla y añade su enlace, de modo que el agente continúa con los
requisitos ya expresados, carga la plantilla, adapta el contenido conservando la
composición y publica el resultado real. Si no se puede enviar en ese momento, la
elección queda adjunta sobre el cuadro de mensaje y viaja con el siguiente mensaje.

Los enlaces antiguos siguen funcionando: `/design` lleva al chat y
`/design?template=<id>` abre la vista previa de esa plantilla sobre el chat, tanto al
cargar la página como al pulsar el enlace guardado en la conversación. Los errores de
carga se muestran en la vista previa y un favorito que no se guarda vuelve a su estado
anterior con un aviso. Las pruebas y el contrato de API se detallan en la guía en inglés.
