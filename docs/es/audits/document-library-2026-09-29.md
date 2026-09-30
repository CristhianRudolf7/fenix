# Validación de la biblioteca — 29 de septiembre de 2026

[English](../../en/audits/document-library-2026-09-29.md)

Se cargaron doce archivos de fuentes públicas: PDF nativos, Word, Excel, PowerPoint, Markdown, TXT, CSV, JSON y una imagen de prueba OCR convertida a PNG, JPEG y PDF escaneado. Se verificaron las 114 páginas del RFC 2616 y los 151 registros del CSV Iris. El [informe verificable](../../research/document-library-validation-2026-09-29.json) conserva las fuentes, hashes, resultados y limitaciones. Los documentos permanecen identificados como “Library audit” en la biblioteca del usuario.

Funcionaron la descarga del original, la fragmentación, la paginación, la búsqueda textual, los duplicados, las versiones, las citas de versiones anteriores, los filtros, los cambios de título, la desactivación, la recuperación de archivos dañados y la eliminación. La vista web mostró un PDF real en escritorio y móvil sin errores de JavaScript.

La primera clave de OpenAI devolvió saldo agotado. La segunda clave distinta funcionó y quedó configurada exclusivamente para la biblioteca. Se probaron embeddings reales de 1.536 dimensiones, cinco consultas híbridas y la reutilización de diez vectores sin cambios al subir una nueva versión. También se recuperó texto y polígonos OCR en PNG, JPEG y PDF escaneado.

Se encontró y corrigió en el código un fallo que rompía citas al reprocesar documentos parciales. Ahora se conservan los identificadores si no cambian el texto, la ubicación y la posición del fragmento dentro de la misma versión. **Esta corrección aún no se ha desplegado.** Pasaron nueve pruebas de biblioteca, una de integración HTTP y tres de aislamiento, incluyendo regresiones de citas y recuperación después de reabrir el almacenamiento.

La primera extracción PNG falló y funcionó al reintentar. No se confirmó la causa inicial; no se redujo el aislamiento. Una prueba diagnóstica sin el punto de entrada de compatibilidad de producción no era equivalente y se excluyó de las conclusiones. También se corrigió una comprobación que buscaba una palabra inexistente en el Word original.

Es una auditoría funcional: no se probaron mil páginas, carga masiva ni la exactitud de respuestas conversacionales completas. Se mantienen las limitaciones de OCR, tablas, diseño de Office y fórmulas sin recalcular. No se desplegó código ni se reiniciaron servicios.
