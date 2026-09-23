# Skills escalables y Fénix Documents

[English](../en/skills-and-documents.md)

La lectura de PDF e imágenes escaneadas usa `document_extract` y la skill
`pdf-workflows`. Consulta [OCR en CPU](pdf-ocr.md) para instalación y límites.
La edición de PDFs y el visor del panel derecho se describen en las [herramientas PDF](pdf-toolkit.md).

## Catálogo de skills

El maestro y los subagentes reciben hasta dos sugerencias pertinentes a su objetivo. El catálogo completo deja de insertarse en su contexto. `skill_search` devuelve hasta ocho candidatos; `skill_list` permite explorar páginas de veinte entradas; `skill_load` carga una skill habilitada o una referencia UTF-8 mediante `resource`, relativa a la carpeta de esa skill.

El índice léxico incremental se conserva en el SQLite de cada usuario y en caché de memoria. Comprueba ruta, inode, tamaño y tiempos de modificación/cambio para actualizar solo archivos modificados; registra SHA-256 del contenido después de redactar secretos. Elimina entradas borradas y comprueba el estado habilitado al consultar. Todavía recorre metadatos y puntuaciones del catálogo: no es una búsqueda de coste constante.

La puntuación considera nombre, descripción, etiquetas, alias y cuerpo de `SKILL.md`. Elimina resultados de contenido idéntico, permite resultados vacíos y no utiliza embeddings ni otro modelo. Los alias bilingües ayudan a buscar en español e inglés; no es traducción semántica automática. Las referencias solo se cargan cuando se necesitan y no se indexan automáticamente.

Admite frontmatter YAML con descripciones multilínea, listas y `metadata`. Una skill malformada no bloquea las demás; `skill_list` muestra el error. No pueden cargarse skills desactivadas ni referencias fuera de su carpeta. Se mantienen la ubicación configurada y los archivos personalizados. Las nuevas skills incluidas se instalan al iniciar el runtime.

## Motor propio de documentos

`fenix-documents` está escrito en Rust desde cero sobre bibliotecas generales de ZIP, XML y serialización. No integra Microsoft 365, Office CLI ni un proveedor de documentos. Es un motor para el agente, **no una suite ofimática completa ni un editor visual web**.

Incluye tres skills propias: `documents-word`, `documents-spreadsheets` y `documents-presentations`. Presentan estas herramientas bajo demanda:

- `document_create`: crea DOCX, XLSX y PPTX; nunca sobrescribe.
- `document_read`: lee texto/celdas por páginas y devuelve partes internas y el hash del archivo.
- `document_edit`: sustituye una coincidencia única dentro de un párrafo de DOCX/PPTX, incluso si abarca varios fragmentos con formato; o modifica una celda XLSX por dirección A1. Exige el hash de la lectura anterior.

La creación admite párrafos, títulos, negrita y tablas en Word; hojas, celdas tipadas y fórmulas explícitas en Excel; diapositivas 16:9 con títulos y viñetas en PowerPoint. Las ediciones preservan las partes no modificadas. Se utilizan las aprobaciones, el aislamiento por proyecto, los registros de acciones y la coordinación de escrituras existentes. La escritura es mediante archivo temporal y sustitución atómica, con comprobaciones de hash; no hay restauración de versiones ni bloqueo de aplicaciones externas.

## Límites actuales

Las fórmulas se guardan pero no se calculan. Al editar celdas se limpian los resultados de fórmulas almacenados y se solicita recalcular al abrir con una aplicación compatible. Se rechazan libros con fórmulas compartidas, matriciales o de tablas de datos antes de editarlos.

Todavía no hay gráficos/tablas dinámicas, control de cambios, inserción de imágenes, maquetación avanzada, motor de fórmulas, exportación PDF ni renderizado visual. La lectura de texto no certifica el aspecto de las páginas. No se admiten formatos binarios antiguos, archivos cifrados ni partes XML UTF-16. Los paquetes comprimidos y descomprimidos se limitan a 64 MiB y 4.096 partes; también aplica el límite existente de argumentos de herramientas. El motor nunca ejecuta macros ni consulta enlaces externos.

Las pruebas cubren creación/edición, lectura paginada, hashes obsoletos y confinamiento de rutas. Las [pruebas de la biblioteca real](skill-library.md) verifican estructura, búsqueda bilingüe, desactivación y recursos bajo demanda. Esto comprueba recuperación determinista, no el acierto del modelo en todas las tareas. Las comprobaciones con LibreOffice/lectores Python son validación independiente, no dependencias del producto.

## Verificación local (2026-09-19)


Los documentos generados se leyeron con `python-docx`, `openpyxl` y `python-pptx`, comprobando tablas, hojas, fórmulas y texto/estructura de diapositivas. LibreOffice Writer abrió un DOCX; no se comprobó el renderizado de XLSX/PPTX en Calc/Impress porque no están instalados. Estas verificaciones no necesitaron llamadas a proveedores de modelos.

También pasaron las 24 pruebas de integración del runtime, incluida la aprobación y el registro de documentos, y las siete pruebas del módulo de documentos. Clippy y la comprobación de compilación del servidor pasaron.
