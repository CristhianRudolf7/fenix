# Auditoría de documentación en directo — 20 de septiembre de 2026

**La búsqueda actual todavía no cumple el flujo solicitado.** Puede descargar documentación pública cuando conoce la URL, pero depende de un índice local y puede responder desde caché.

Se ejecutaron diez búsquedas reales con `discover: true`, como hace el agente por defecto, más un control con caché desactivada y dos pruebas controladas. Cada caso utilizó una carpeta temporal independiente, sin rastreador en segundo plano ni modelo de lenguaje. Las URLs exactas fueron entradas suministradas para comparar; no las encontró el motor por sí mismo. Se probó el código actual del proyecto, sin actualizar la instancia en ejecución ni modificar la lógica del buscador. Las carpetas temporales fueron eliminadas.

| Prueba | Resultado |
| --- | --- |
| React, Next.js y Tokio sin dominio ni URL | Las tres devolvieron resultados ajenos al tema: gob.pe o NASA. |
| Solo el dominio oficial de React | Leyó la portada y devolvió cero resultados. |
| Solo el dominio oficial de Next.js | Encontró una publicación relevante de Next.js 16 y una página de conferencia, no la referencia de la API. |
| Solo el dominio oficial de Context7 | Devolvió páginas de guías y contacto sin el endpoint solicitado en los fragmentos. |
| URL exacta de React useEffect | Falló por superar el límite de descarga. |
| URL exacta de la referencia de proxy de Next.js | La referencia superó el límite; encontró una guía introductoria más pequeña. |
| URLs exactas de Tokio y de la guía API de Context7 | Sí devolvió documentación pertinente. |

## Problemas comprobados

- **Localización de fuentes:** sin pistas usa unas pocas páginas iniciales y coincidencias de palabras. Puede rellenar resultados con páginas irrelevantes. Elegir el dominio ayuda, pero no siempre llega a la documentación necesaria.
- **Actualidad:** repetir las lecturas de Tokio y Context7 devolvió caché y realizó cero peticiones nuevas. En una prueba cambiamos el contenido del servidor y el lector siguió devolviendo el anterior hasta caducar la caché de 15 minutos.
- **Almacenamiento:** poner `cache_minutes: 0` sí revalidó Tokio mediante HTTP en cada lectura, pero siguió escribiendo el índice SQLite local. No existe una opción por consulta para trabajar sin persistir documentos.
- **Tamaño:** las páginas exactas de React y Next.js excedieron el límite predeterminado de 512 KiB. Reintentar sin cambiar la estrategia no resolvió el problema.
- **Ejemplos de código:** el extractor HTML quitó los saltos de línea e indentación de un ejemplo Python dentro de `<pre><code>`.
- **Fechas:** `days: 30` eliminó documentación de Tokio recién consultada porque no declaraba fecha de publicación. Este filtro no sirve para expresar que queremos documentación vigente.
- **Versiones:** no hay resolución de biblioteca/versión a partir del proyecto ni verificación de compatibilidad de la documentación encontrada.
- **JavaScript y fragmentos:** la herramienta HTTP no renderiza ni activa automáticamente Chromium. Además, limita el texto y elige fragmentos por coincidencias sencillas, sin comprender secciones de documentación.

El recorrido completo tardó 41,77 segundos, incluidas repeticiones y control sin caché, con tres casos independientes a la vez. El pico de RAM del proceso de auditoría fue 35,07 MiB, excluyendo compilación, navegador y modelo. Es una muestra pequeña, no una medida de carga de producción ni una prueba de calidad por sí sola.

## Qué falta

Una modalidad de consulta en directo que localice fuentes oficiales y versiones, consulte o revalide cada petición, procese temporalmente sin guardar documentos, conserve ejemplos y secciones y descarte resultados poco relacionados. Conviene aprovechar versiones oficiales en Markdown cuando existan y recurrir al navegador cuando haga falta renderizar. No basta con desactivar la caché.

Las dos pruebas controladas pasaron porque reprodujeron las limitaciones actuales; no indican que la funcionalidad solicitada ya exista.

[Resultados medidos](../benchmarks/live-documentation-search-2026-09-20.json) · [Detalle técnico y reproducción en inglés](../en/live-documentation-search-audit.md)
