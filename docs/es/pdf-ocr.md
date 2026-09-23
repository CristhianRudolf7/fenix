# Extracción local de PDF e imágenes

[English — configuración técnica y despliegue](../en/pdf-ocr.md)

Para edición, unión de páginas y visualización en el panel, consulta las [herramientas PDF](pdf-toolkit.md).

Fénix incorpora `document_extract`, herramienta de OCR local en CPU, y actualiza
su skill propia `pdf-workflows`. PDFium lee texto existente y renderiza páginas;
PaddleOCR reconoce los escaneos. No requiere GPU ni una API OCR de pago.

Descubre la herramienta con `tool_search` y utiliza, por ejemplo:

```json
{"path":"factura.pdf","mode":"auto","start_page":1,"page_count":5,"dpi":200}
```

`command_status` consulta progreso; `command_output` recupera texto omitido en
la vista previa; `command_cancel` cancela. Continúa usando `next_page` y
`expected_sha256` para no mezclar versiones. Un resumen completo y código de
salida 0 solo cubren las páginas seleccionadas; no garantizan exactitud del OCR.

El modo automático lee texto nativo de páginas legibles sin imágenes y aplica OCR
a páginas escaneadas o mixtas. Esta decisión conservadora incluye logotipos.
`native` omite OCR y marca contenido no extraído; `ocr` fuerza reconocimiento.
Los bloques mantienen página, origen y posiciones para OCR. Los errores y límites
se indican expresamente. Las puntuaciones no equivalen a probabilidades exactas.

El procesamiento ocurre en el servicio aislado, sin red, con un único archivo de
solo lectura. No tiene acceso a credenciales ni archivos de otros usuarios. El
texto extraído se trata como contenido, nunca como permisos o instrucciones.

Para activarlo, el operador debe construir `packaging/linux/Dockerfile.ocr`,
instalar el agente y servicio actualizados, y seleccionar la imagen por su ID
inmutable. Dependencias y modelos están fijados; los modelos se verifican por
SHA-256 y se descargan al construir, no durante la lectura. Los pasos completos
están en la versión inglesa. Modificar el repositorio no migra producción.
Sin servicio o imagen no existe un fallback que analice PDFs en el controlador.

Límites: 64 MiB, 2.000 páginas totales, lotes de hasta 20, 100–300 DPI,
12 millones de píxeles por imagen y hasta 300 segundos por lote. Hereda cuotas,
cancelación y registros privados. Un reinicio interrumpe el proceso; no hay
una nueva cola persistente ni caché de extracción. No reconstruye tablas,
garantiza columnas ni genera PDFs seleccionables. Escaneos girados y escritura
manual requieren revisión; PDFs cifrados o dañados producen error.

oneDNN está desactivado por una incompatibilidad observada en la prueba real;
se usa la ruta normal de CPU. Consulta la [evidencia de validación](../research/pdf-ocr-validation-2026-09-20.json)
para distinguir las pruebas ejecutadas del despliegue pendiente.
