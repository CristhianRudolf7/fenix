# Auditoría de búsqueda y adaptación de Last 30 Days

Fecha: 2026-09-19. Se revisó el skill original y se amplió nuestra adaptación `web-research` a 1.3.0. El método aporta consultas precisas, selección de comunidades, fechas verificadas, lectura de originales y distinción entre opiniones, hechos y fuentes inaccesibles. No se importaron proveedores, scripts, credenciales ni cookies del proyecto original.

## Correcciones

- El agente activa descubrimiento limitado antes de consultar el índice. Visita hasta cuatro páginas iniciales o cinco fuentes explícitas y sigue hasta seis enlaces observados, en dos pasos y con máximo de dos por origen. Puede atravesar un índice de documentación sin ejecutar JavaScript.
- Las etiquetas de enlaces y títulos de feeds ayudan a encontrar originales; no se presentan como artículos leídos. Se conservan en SQLite mediante migración aditiva.
- Las fechas de Hacker News corresponden al enlace solicitado; comentarios recientes no convierten una publicación antigua en reciente.
- Se separó el tiempo de espera por sitio del límite de cada solicitud HTTP. La operación conserva un máximo total de 100 segundos y diez operaciones simultáneas.
- El agente distingue fuentes bloqueadas, fallidas o no visitadas. Si el contexto recorta los resultados, conserva un resumen y puede recuperar la evidencia completa antes de citarla.
- Se actualizaron las instrucciones y la selección del skill. Las versiones originales conocidas se actualizan; se conservan las personalizaciones del usuario.

## Prueba real

Se ejecutaron cuatro consultas simultáneas desde un índice vacío, sin navegador, modelo ni rastreador de fondo:

| Fuente y tema | Resultado | Tiempo |
| --- | --- | --- |
| Libro de Rust: ownership/borrowing | Índice de contenidos y capítulo original descubierto | 6,18 s |
| SQLite: transacciones | Índice de documentación encontrado; no equivale a un tutorial completo | 4,24 s |
| Hacker News: Show HN, últimos 30 días | Portada accesible; listado Show respondió HTTP 429; sin hilo reciente verificado | 32,28 s |
| Reddit: Rust, últimos 30 días | Lectura denegada por robots; no se leyeron hilos | 0,075 s |

Las cuatro terminaron en **32,28 segundos en conjunto**. Hubo 11 solicitudes HTTP. El proceso aislado de búsqueda alcanzó unos **19 MiB de RAM**; no es el consumo de todo Fénix ni del navegador. Las repeticiones sin red tardaron 0,80–1,84 ms y no hicieron solicitudes HTTP.

Un [control positivo adicional en Hacker News](../benchmarks/hacker-news-control-2026-09-19.json) buscó `open source`, tema observado en la portada. Descubrió y leyó un hilo real y lo devolvió con `days:30` y fecha de publicación `2026-09-19T10:46:58Z`. Tardó 32,54 segundos, con tres solicitudes HTTP. El pasaje estaba limitado a 2.400 caracteres: no demuestra haber leído todos los comentarios. Este acceso exitoso no elimina el fallo HTTP 429 registrado para el listado Show.

Las regresiones cubren descubrimiento desde cero, fechas, feeds, enlaces privados, límites, diez búsquedas paralelas, caché persistente, cancelación y esperas mayores de 30 segundos. Las pruebas del agente usan respuestas de modelo simuladas: verifican integración y entrega de evidencia, no la calidad de razonamiento de un modelo real.

La cobertura sigue limitada por las fuentes elegidas, los enlaces observados y las políticas de los sitios. No es un sustituto de un índice mundial y no garantiza buscar cualquier hilo de Reddit/Hacker News. Los fallos de acceso se informan sin eludirlos.

Consulta la [auditoría completa en inglés](../en/research-discovery-audit.md), el [reporte final](../benchmarks/research-discovery-2026-09-19.json) y el [reporte inicial](../benchmarks/research-discovery-initial-2026-09-19.json), que documenta los problemas encontrados antes de las correcciones.

Pasaron 43 pruebas del motor, cuatro de integración del agente, 14 de skills, cuatro
de resúmenes de herramientas y dos de HTTP. También pasaron la validación del skill,
Clippy y la compilación. El ejecutable actualizado está en `target/debug/fenix`;
reinícialo conservando los argumentos de tu instancia (`--data-dir` y, si lo usas,
`--skills-dir`).
