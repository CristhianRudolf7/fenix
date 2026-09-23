# Auditoría de búsqueda comunitaria — 2026-09-19

[English — full report](../en/community-search-audit.md)

**El skill adaptado está instalado y activo; todavía no existe una búsqueda completa de Reddit/Hacker News.** Se verificó directamente el motor, sin llamar a modelos, modificar el agente ni desplegar cambios.

`web-research` 1.2.0 y su referencia están instalados. Toma del skill original la selección de fuentes, fechas, procedencia y contraste; no instala sus conectores ni servicios externos.

En producción había **0 páginas de Hacker News, 0 de Reddit y 2 de GitHub**. La configuración existente no incluye las nuevas semillas comunitarias. Las consultas a HN/Reddit devolvieron cero; Rust en GitHub devolvió dos resultados.

En una instancia temporal se probó la lectura e indexación, separada del índice de producción:

- **Hacker News:** RSS leído y 30 enlaces descubiertos. Con solicitudes espaciadas, portada y tres hilos leídos; los tres hilos pudieron recuperarse por sus títulos. Eso verifica recuperación básica, no precisión general.
- **Fechas:** los tres hilos quedaron sin fecha de publicación reconocida. El filtro de 30 días devolvió cero.
- **Tiempos:** la primera lectura consecutiva de portada agotó 30 s. HN exige 30 s entre peticiones y el motor incluye esa espera en su timeout de 30 s. Al separar las lecturas 31 s funcionaron; el lote tardó 130,545 s incluyendo esperas.
- **Reddit:** tanto r/rust como el RSS de r/all fueron denegados por robots.txt. No se eludió la restricción.
- **GitHub:** el repositorio rust-lang/rust se leyó, indexó y recuperó con Rust.

Tras la ingestión temporal: SQLite en HN dio 0 resultados, Rust 1 y Show HN 2. Las seis consultas locales no hicieron solicitudes HTTP adicionales. El índice de producción no recibió estos datos de prueba.

Pendientes identificados: ingestión comunitaria en instalaciones existentes, separación de espera y timeout, y extracción de fechas de comunidades. El skill por sí solo no resuelve cobertura ni restricciones de acceso.

Evidencias: [producción](../benchmarks/community-search-production-2026-09-19.json), [primera pasada](../benchmarks/community-search-2026-09-19.json), [pasada espaciada](../benchmarks/community-search-paced-2026-09-19.json). Se conservan URLs, consultas, estados, hashes y tiempos, sin reproducir comentarios completos.
