# Búsqueda web propia y lectura de fuentes

[English — complete reference](../en/web-search.md)

Fénix incorpora `web_search` y `web_read` como herramientas del agente. Basta con
pedir la búsqueda en la conversación habitual o usar **Navegador → Buscar o introducir URL**.
Nuestra API `POST /api/web/search` consulta el mismo índice. No hay proveedor de
búsqueda, suscripción ni API key para esta funcionalidad. El modelo
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
   todos los términos útiles; las coincidencias parciales requieren un mínimo
   del 60% de los términos del tema y todos cuando hay hasta tres, con proximidad
   entre coincidencias. El nombre del sitio no sustituye al tema.
4. Por defecto también lee hasta tres resultados. Con `fresh: true` revalida por
   HTTP; `fresh: false` permite una caché reciente. `web_read` permite leer una URL conocida aunque
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
lecturas, con 3 por defecto. Con `discover: false`, `read_pages: 0` y sin `source_urls`, esa consulta sólo usa el índice;
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
350 caracteres; la segunda fase devuelve hasta 6.000 por página, y una lectura
directa hasta 6.000. Las lecturas incluyen hash, idioma, fecha y truncamiento.

El agente debe citar las URLs, comunicar cobertura limitada y distinguir la
evidencia guardada de una comprobación reciente. `retrieved_at` indica cuándo se
consultó o comprobó la página, **no cuándo se publicó**. Si falla una lectura,
queda el fragmento antiguo con una advertencia. Las instrucciones que aparezcan
en una web son datos externos y no autorizan acciones ni cambian las reglas del
agente. No debe inventar resultados ni consultar repetidamente un índice vacío.

## Fuentes, fechas y navegador propio

- `domains`: hasta ocho dominios y sus subdominios reales, filtrados en nuestro índice.
- `days`: últimos 1–365 días según metadatos explícitos de publicación. Excluye fechas desconocidas, futuras o antiguas; no usa la fecha de rastreo.
- `max_per_domain`: máximo por dominio/host, 2 por defecto, para diversificar. Los hashes eliminan copias exactas.
- `source_urls`: hasta cinco URLs públicas originales conocidas para leer/indexar antes de buscar. `source_reads` informa aciertos, enlaces descubiertos y fallos. No adivina URLs ni consulta buscadores externos.

La API web usa `read_pages: 0` por defecto; la herramienta del agente conserva 3.
Se puede buscar sin abrir Chromium ni elegir perfil. Abrir un resultado utiliza
el perfil seleccionado. El navegador es nuestro controlador Rust y nuestra interfaz,
con Chromium local como motor de renderizado; no es un servicio alojado externo.
Las sesiones personales no se incorporan al índice público.

Los feeds RSS/Atom descubren enlaces que el rastreador procesará después; sus
fragmentos no se presentan como artículos leídos. Las palabras comunes en español
e inglés pueden descartarse de forma determinista. Se informa cuando se amplía la
coincidencia de todos los términos a términos individuales. Se revisan hasta 1.000
candidatos por pasada y se informa si ese recorte puede omitir resultados. Los
filtros se vuelven a comprobar tras redirecciones o cambios de fecha. Los dominios sin
rastrear, bloqueados o sin fechas verificables seguirán siendo huecos de cobertura.

No existen adaptadores, claves ni endpoints configurables de proveedores de búsqueda.
Se rechazan URLs conocidas de servicios de búsqueda antes de descargarlas. Esa lista
es una protección adicional, no una clasificación exhaustiva de internet. El agente
tampoco debe usar el navegador o comandos para delegar consultas en otro buscador.

## Diez búsquedas simultáneas

`max_concurrent_searches` vale **10** por defecto y admite valores entre 1 y 10.
Las búsquedas y las lecturas directas comparten ese límite entre todos los agentes.
La undécima espera un cupo hasta 30 segundos. `queue_wait_ms` identifica esa espera,
separada del presupuesto posterior: 100 segundos con descubrimiento o fuentes explícitas; 35 para lecturas de resultados del índice. Cancelar una
llamada libera sus cupos. Los archivos de configuración anteriores adoptan diez
si no contienen el nuevo campo.

Puede haber hasta diez descargas HTTP activas, incluido el rastreador. Los dominios
distintos avanzan en paralelo; cada origen conserva una sola solicitud activa y
sus pausas. Esperar a un sitio no ocupa un cupo global de descarga. El rastreador
no empieza trabajo nuevo mientras haya consultas activas o esperando; una descarga
que ya empezó puede terminar. Además, `foreground_reserve_percent` reserva de
forma aproximada el 20 % del presupuesto diario para lecturas solicitadas por el
usuario/agente (configurable entre 0 y 90 %). El rastreador deja de iniciar trabajo
al llegar a su parte; robots y redirecciones ya en curso pueden gastar algo más.
Las lecturas explícitas conservan el límite diario total. El estado muestra ambos
presupuestos. Con cupos mínimos se permite al menos un intento de rastreo.

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
| `max_urls_per_origin` | 2.000 URLs por origen |
| `max_body_bytes` | 2.097.152 bytes descomprimidos por página |
| `max_text_chars` | 64.000 caracteres almacenados por página |
| `request_interval_ms` | 2.000 ms entre solicitudes al mismo origen |
| `requests_per_day` | 2.000 intentos HTTP por día UTC, incluidos robots y redirecciones |
| `refresh_hours` | 72 horas para actualizar páginas rastreadas |
| `cache_minutes` | 15 minutos con `fresh: false`; la lectura actualizada del agente revalida por HTTP |

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
completa tiene 100 segundos, incluidas las pausas por origen; cada solicitud HTTP
conserva un límite de 12 segundos después de esperar. El descubrimiento, las fuentes
explícitas y la lectura de resultados comparten 100 segundos. Sin descubrimiento
ni fuentes explícitas, las lecturas de resultados conservan 35 segundos. La lectura directa tiene
100 segundos y la espera inicial de cupo tiene su propio límite de 30 segundos.
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

[Verificación actual del buscador propio](independent-search-audit.md).

## Descubrimiento por consulta e investigación reciente

El agente activa `discover: true` por defecto. La biblioteca Rust y el endpoint HTTP
conservan `false` por compatibilidad. Se leen hasta cuatro páginas iniciales y hasta 16
enlaces observados pertinentes, con un máximo de ocho por origen. Indica dominios o
páginas iniciales conocidas para mejorar la cobertura; sin ellos se usa una selección
limitada de semillas configuradas. No se inventan URLs ni se usan proveedores externos.

Los títulos de enlaces y feeds ayudan a encontrar originales; no se presentan como
artículos verificados. `discovery.sources` muestra los intentos y sus fallos;
`unvisited_domains` identifica dominios omitidos por el límite. El índice conserva
las etiquetas en disco mediante una migración aditiva. Las fechas de Hacker News se
asocian al enlace solicitado, sin rejuvenecer historias por comentarios recientes.

`web-research` 1.5.0 es nuestra adaptación propia de Last 30 Days: consultas precisas,
fuentes pertinentes, fechas verificadas, lectura de originales y límites explícitos.
Las versiones incluidas sin personalización se actualizan; se conservan las del usuario.
La cobertura sigue siendo limitada y Reddit puede denegar el acceso. Consulta la
[auditoría de implementación](research-discovery-audit.md).

## Recuperación actualizada y pertinencia

`fresh: true` es el valor predeterminado de búsqueda y de `web_read` en el agente:
consulta o revalida la fuente por HTTP. `fresh: false` permite usar la caché de
15 minutos. Una página ya comprobada durante la misma búsqueda puede reutilizarse.
El método Rust `read()` conserva compatibilidad; `read_with_fresh()` ofrece control
explícito. Consultar el índice sin descubrimiento, fuentes ni lecturas sigue sin
hacer peticiones HTTP.

Se reconocen algunos nombres de fuentes y bibliotecas para empezar en sus sitios
oficiales; este catálogo inicial no cubre todos los temas ni inventa URLs de
artículos. Se exploran hasta cuatro niveles, 16 enlaces en total y ocho por origen,
incluyendo mapas del sitio anunciados en robots.txt. Se extraen hasta 2.000 enlaces
por documento dentro de los límites de la cola. Las coincidencias parciales deben
cubrir al menos el 60% de los términos del tema, y todos cuando hay hasta tres,
con coincidencias cercanas. El nombre del sitio no sustituye al tema. Esto mide pertinencia léxica, no veracidad.

Se conservan bloques e indentación del código y se seleccionan pasajes con más
términos distintos de la consulta. Tras revalidar se actualizan los fragmentos y se
descartan páginas que dejaron de coincidir. Continúan los límites de descarga,
memoria, tiempo y diez operaciones simultáneas. Las configuraciones existentes
conservan sus límites explícitos. El texto acotado del índice se guarda en disco;
no es un modo sin persistencia ni una copia completa de la documentación.

Se registra el origen de los enlaces para que búsquedas simultáneas no desvíen su
exploración hacia enlaces ajenos. `discovery.stop_reason` explica la parada por
evidencia léxica suficiente o por límites de enlaces/tiempo. Es una heurística de
recursos, no una garantía de veracidad. Las lecturas de fuentes incluyen fecha de
consulta y procedencia de red, caché o descarga compartida. El skill puede formular
consultas breves en el idioma de la fuente; el motor no traduce por sí mismo.

Los nombres de secciones que apuntan a una misma URL se acumulan al quitar sus
fragmentos, en lugar de perderse por deduplicación. Los enlaces observados también
pueden consultarse si la cola de rastreo está llena, sin ampliar sus límites:
se conserva una tabla de procedencia acotada a cuatro veces el tamaño de la cola.
La selección prioriza términos completos en las rutas y permite al menos dos
oleadas, si hay enlaces y tiempo, antes de detenerse por coincidencia léxica.
Así una página de introducción puede conducir a la referencia específica.

Consulta la [auditoría de recuperación](search-retrieval-improvements.md) para
las mediciones y las limitaciones observadas.

## Optimización de latencia

Las consultas normales buscan primero páginas pertinentes en el índice y
comprueban hasta tres directamente, con un máximo de 15 segundos para esa fase
dentro del presupuesto total. Si siguen siendo pertinentes, devuelven
`discovery.stop_reason: "indexed_sources"`. No se repite el recorrido de la web.
Si cambian o fallan, se activa descubrimiento con navegación actualizada. Las
consultas con fechas o términos explícitos de actualidad, como `latest`, `news`,
`hoy` o `recientes`, siguen buscando enlaces nuevos.

Las páginas de navegación pueden usar caché como directorio; las fuentes finales
se revalidan con `fresh: true`. Un fallo no se reintenta dentro de la misma llamada
ni se presenta como verificación exitosa. Compartir dominio solo desempata enlaces;
no convierte un enlace ajeno al tema en candidato.

Hasta 16 enlaces observados por llamada pueden priorizarse para el rastreador
existente. La cola adicional tiene un máximo de 256 URLs y respeta límites y
reintentos. `background_queued` indica trabajo priorizado y `coverage.priority_urls`
su cantidad pendiente; no significa que ya se haya leído. No se crea un rastreador
independiente por búsqueda. Las lecturas recientes del agente también aplazan el
próximo rastreo para evitar descargas duplicadas.

Se reutilizan conexiones con un máximo de 32 clientes, uno por origen, y una conexión
inactiva por cliente. La conexión inactiva expira a los 30 segundos; los clientes
sin uso durante 60 segundos se retiran al acceder al conjunto. Cada petición vuelve
a validar DNS; una dirección privada se rechaza y un cambio de direcciones renueva
el cliente. También se redujeron asignaciones temporales y consultas SQL repetidas.

Consulta la [comparación de tiempos](search-latency-optimization.md). Las búsquedas
nuevas siguen necesitando descubrimiento; las conocidas dependen de la respuesta
del sitio. Una consulta sin red es un caso distinto y se informa como tal.
