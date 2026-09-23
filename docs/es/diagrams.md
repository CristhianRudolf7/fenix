# Diagramas editados por el agente

Los diagramas se consultan desde el panel derecho: zoom, desplazamiento con ratón o gestos, ajuste a la vista, pantalla completa, descarga SVG y búsqueda de elementos estructurados. El usuario solicita todas las modificaciones y restauraciones por chat. Los otros formatos de Design conservan sus controles existentes.

## Creación y versiones

`design_publish` recibe un `content.graph` estructurado o `content.source` en Mermaid. Rust valida nodos, conexiones, grupos e identificadores y compila los grafos a Mermaid. El navegador renderiza localmente. Las plantillas existentes siguen funcionando.

`design_read` recupera contenido y revisión. `diagram_edit` modifica elementos por ID con comprobación de `expected_revision`; un lote inválido o desactualizado no publica cambios. Para sustituir todo el contenido se utiliza `design_publish`. La API de edición directa de la interfaz rechaza diagramas, y la vista de fuente es de solo lectura.

Cada usuario tiene `diagrams/<design-id>/` dentro de su directorio de datos. En cuentas aisladas corresponde a `<carpeta-del-usuario>/state/diagrams/`. Cada versión conserva `v000001.json` y `v000001.mmd`, etc. Las respuestas de las herramientas incluyen las rutas exactas. Los diagramas anteriores se archivan al consultarlos; los nuevos se archivan al guardar.

La base de datos determina qué revisiones están publicadas. Los archivos se escriben antes de confirmar la revisión; un fallo de escritura conserva la revisión anterior. Una interrupción puede dejar un archivo aún no publicado, que se podrá reemplazar al reintentar. Editar esos archivos directamente no cambia el diagrama publicado.

## Navegación y alcance

Arrastrar solo desplaza la vista. Los botones, el zoom, las referencias y la búsqueda no consumen tokens. Atajos: +/− para ampliar, flechas para desplazar y 0/Inicio para ajustar. Se mantiene el panel adaptable y el menú lateral colapsado.

Los grafos estructurados permiten leer detalles, referencias y conexiones de cada nodo. Mermaid libre conserva navegación visual y consulta de fuente. Durante una actualización se mantiene el último dibujo válido si el nuevo render falla, avisando qué revisión se está mostrando. Este respaldo vive en el visor: tras recargar una revisión inválida puede ser necesario seleccionar otra anterior.

La validación estructural no verifica hechos. Mermaid libre se valida sintácticamente en el navegador. No se añaden servicios externos de diagramas, Electron ni Python/Manim. Las animaciones avanzadas quedan para una capacidad independiente.

Consulta el [contrato del skill](../../resources/skills/diagram-authoring/references/diagrams.md).
