# Búsqueda web propia y lectura de fuentes

[English — complete reference](../en/web-search.md)

Fénix incorpora `web_search` y `web_read` como herramientas del agente. Basta con
pedir la búsqueda en la conversación habitual. No hay una pantalla adicional,
proveedor de búsqueda, suscripción ni API key para esta funcionalidad. El modelo
de la conversación sigue utilizando su proveedor configurado.

La búsqueda consulta **el índice público de esta instalación**, que crece poco a
poco. Acepta cualquier tema, pero eso no significa que ya tenga información sobre
todos los temas. Una instalación nueva parte de cero páginas. Un resultado vacío
no demuestra que no exista información en internet.

## Funcionamiento

1. Un trabajador Rust rastrea páginas HTTP en segundo plano, independientemente
   de las tareas y planes del agente. Conserva su cola de URLs en disco.
2. Descubre enlaces públicos, incluso de otros dominios, y sitemaps XML del mismo
   origen. Extrae texto principal, elimina navegación habitual y scripts y guarda
   contenido acotado en un índice SQLite FTS5 separado.
3. `web_search` busca palabras con soporte Unicode, ordena por BM25 y devuelve
   títulos, URLs, fragmentos, fechas de indexación y cobertura. Primero busca
   todos los términos; si faltan resultados, amplía con términos individuales.
4. Por defecto también lee hasta tres resultados. Reutiliza una caché reciente o
   vuelve a consultar la página. `web_read` permite leer una URL conocida aunque
   todavía no figure en el índice.

Las URLs iniciales de Wikipedia, gob.pe, NASA, ONU, OMS, Banco Mundial,
Smithsonian, BBC Mundo, arXiv, MDN y el libro de Rust son **puntos de partida, no
una lista cerrada**. Sus enlaces pueden descubrir otros dominios. La cobertura
inicial depende de esas fuentes y de sus políticas y disponibilidad. No se
consultan buscadores externos ni se generan embeddings.

## Uso por el agente

Sólo es obligatorio indicar qué buscar:

```json
{"query": "fotosíntesis en plantas"}
```

`limit` permite 1–10 resultados, con 5 por defecto. `read_pages` permite 0–3
lecturas, con 3 por defecto. Con `read_pages: 0`, esa consulta sólo usa el índice;
el rastreador independiente puede seguir trabajando. La consulta admite hasta
400 caracteres y debe contener letras o números.

Para abrir una fuente conocida:

```json
{"url": "https://doc.rust-lang.org/book/ch01-01-installation.html", "query": "installation"}
```

En `web_read`, `query` es opcional y orienta el fragmento. La URL debe ser HTTP(S)
pública, sin credenciales y con puerto 80 o 443.

Cada resultado identifica su fuente y diferencia lectura de red (`network`),
caché (`cache`), comprobación condicional (`revalidated`), fallo (`failed`),
lectura compartida en curso (`shared`) y lectura no solicitada (`not_requested`).
Los fragmentos del índice tienen hasta
350 caracteres; la segunda fase devuelve hasta 2.400 por página, y una lectura
directa hasta 6.000. Las lecturas incluyen hash, idioma, fecha y truncamiento.

El agente debe citar las URLs, comunicar cobertura limitada y distinguir la
evidencia guardada de una comprobación reciente. `retrieved_at` indica cuándo se
consultó o comprobó la página, **no cuándo se publicó**. Si falla una lectura,
queda el fragmento antiguo con una advertencia. Las instrucciones que aparezcan
en una web son datos externos y no autorizan acciones ni cambian las reglas del
agente. No debe inventar resultados ni consultar repetidamente un índice vacío.

## Diez búsquedas simultáneas

`max_concurrent_searches` vale **10** por defecto y admite valores entre 1 y 10.
Las búsquedas y las lecturas directas comparten ese límite entre todos los agentes.
La undécima espera un cupo hasta 30 segundos. `queue_wait_ms` identifica esa espera,
separada del presupuesto posterior de 35 segundos para leer fuentes. Cancelar una
llamada libera sus cupos. Los archivos de configuración anteriores adoptan diez
si no contienen el nuevo campo.

Puede haber hasta diez descargas HTTP activas, incluido el rastreador. Los dominios
distintos avanzan en paralelo; cada origen conserva una sola solicitud activa y
sus pausas. Esperar a un sitio no ocupa un cupo global de descarga. El rastreador
no empieza trabajo nuevo mientras haya consultas activas o esperando; una descarga
que ya empezó puede terminar.

Cada búsqueda puede leer sus tres primeros resultados en paralelo, conservando
su orden. Si varios agentes piden la misma URL mientras se está leyendo, comparten
la descarga y el resultado, incluso con `cache_minutes=0`. Esto también evita
repetir solicitudes simultáneas de robots.txt. No añade una caché persistente:
los resultados compartidos se liberan cuando terminan sus lectores. Si se cancela
el lector que inició la operación, un lector pendiente puede continuarla.

Cuando una respuesta del modelo contiene únicamente llamadas `web_search` y
`web_read`, el ejecutor las lanza juntas. Registra cada resultado al terminar y
conserva el orden al enviarlos al modelo. Las respuestas que mezclan herramientas
web con otras herramientas mantienen su ejecución ordenada. Este límite no cambia
la cantidad de subagentes del planificador.

La [medición de concurrencia y memoria](web-search-concurrency.md) compara diez
temas y contiene las órdenes para reproducir la prueba.

## Configuración y consumo

Al iniciar normalmente Fénix se inicia el rastreador, incluso antes de configurar
el modelo. No consume llamadas de inferencia. Al apagar el servicio se cancela
la descarga pendiente; al reiniciar continúa desde su cola persistida. Esto no
reanuda tareas ni efectos del agente.

Dentro de `--data-dir`, por defecto `.fenix/`, se crean:

- `web-search.json`, editable con el servicio detenido. Reinicia para aplicar.
- `web/index.sqlite3`, con el índice público, URLs, políticas robots y presupuestos.

La base pública no lee conversaciones, memoria privada, archivos de proyectos ni
credenciales. Una URL enviada a una web puede revelar información al destinatario;
se rechazan parámetros habituales de credenciales, sin intentar clasificar todos
los posibles datos sensibles de una URL. No se envían cookies de sesión,
cabeceras de autorización ni referer, y no se heredan proxies del entorno.

| Parámetro | Valor inicial |
| --- | --- |
| `enabled` | `true` |
| `max_concurrent_searches` | 10 búsquedas/lecturas y hasta 10 descargas activas |
| `max_pages` | 5.000 páginas almacenadas |
| `max_frontier` | 20.000 URLs conocidas |
| `max_urls_per_origin` | 200 URLs por origen |
| `max_body_bytes` | 524.288 bytes descomprimidos por página |
| `max_text_chars` | 24.000 caracteres almacenados por página |
| `request_interval_ms` | 2.000 ms entre solicitudes al mismo origen |
| `requests_per_day` | 2.000 intentos HTTP por día UTC, incluidos robots y redirecciones |
| `refresh_hours` | 72 horas para actualizar páginas rastreadas |
| `cache_minutes` | 15 minutos de caché; 0 obliga a comprobar la fuente |

Hay hasta diez descargas activas y un solo análisis HTML simultáneo, compartidos
entre agente, hijos y rastreador. Varias respuestas acotadas pueden esperar su análisis. El índice queda en disco y la caché de páginas de
SQLite tiene un objetivo de 2 MiB. Eso **no es un límite de RAM de todo Fénix**:
también consumen memoria TLS, el analizador y las demás funciones del programa.
No se abre Chromium ni se carga un modelo local para buscar.

Pon `"enabled": false` y reinicia para desactivar ambas herramientas y el
rastreo. Edita `seeds` para cambiar los puntos de partida; las URLs ya descubiertas
permanecen. Con `seeds: []`, las lecturas directas pueden alimentar un índice
inicialmente vacío. Aumentar los límites mejora el potencial de cobertura a costa
de disco, tráfico y tiempo. Al llegar al máximo no se sustituyen automáticamente
las páginas ni las URLs existentes. Bajar un límite tampoco borra datos previos.

`GET /api/web/status` muestra páginas y orígenes indexados, URLs conocidas y
pendientes, fecha de indexación, intentos del día, límite y estado del rastreador.
Conserva la protección de Host/Origin de localhost. Es una consulta de estado;
las búsquedas las realiza el agente. También expone el límite y las cantidades
`active_operations`, `queued_operations` y `active_downloads`.

Para integraciones Rust, `Runtime::open` no realiza tráfico de red y
`Runtime::start()` inicia los trabajadores del agente. El ejecutable añade
`Runtime::start_web_search()` para iniciar explícitamente el rastreo.

## Límites de lectura

Se valida el DNS y cada redirección para impedir acceso a direcciones locales,
privadas o reservadas. Se respeta robots.txt como `FenixBot`, con caché de 24 horas
y denegación temporal ante fallos; también se respetan noindex/noarchive/nofollow.
Se acotan descargas, enlaces, estructura HTML, tiempos y reintentos. Una lectura
completa tiene 30 segundos y la segunda fase de búsqueda comparte 35 segundos, incluidas las pausas por origen. La lectura directa tiene
35 segundos y la espera inicial de cupo tiene su propio límite de 30 segundos.
El presupuesto diario persiste al reiniciar; al agotarse sigue disponible la
búsqueda local y la lectura de contenido que aún esté en caché.

Esta fase lee HTML/XHTML, texto y sitemaps XML. No ejecuta JavaScript, calcula
visibilidad mediante CSS externo, extrae PDFs, inicia sesión, resuelve CAPTCHA,
rellena formularios, reserva vuelos ni prueba diseños renderizados. Navegación
interactiva, capturas y cambio de tamaño de pantalla quedan para otra fase.

## Verificación

Véase la [prueba real de diez temas](web-search-benchmark.md), con tiempos, RAM,
cobertura, concurrencia y fallos observados.

```sh
cargo test --locked -p fenix-web
cargo test --locked -p fenix-agent -p fenix-server --test web_search
```

Las pruebas usan servidores HTTP y un modelo simulados. Cubren descubrimiento,
búsqueda, lectura, persistencia, caché, robots, límites, redirecciones privadas,
cancelación e integración. No necesitan claves ni sitios externos. Para una
comprobación HTTP real y voluntaria, con índice temporal y sin modelo:

```sh
cargo run --locked -p fenix-web --example read_search -- \
  https://doc.rust-lang.org/book/ch01-01-installation.html installation
```

La [referencia en inglés](../en/web-search.md) detalla rangos de configuración,
contratos de salida y decisiones de implementación.
