# Fénix — Hoja de ruta

> Memory implementation / Implementación de memoria: [English](docs/en/adaptive-memory.md) · [Español](docs/es/adaptive-memory.md). This document retains broader design goals; see the implementation contract for delivered behavior and limits. / Este documento conserva objetivos más amplios; consulta el contrato para conocer lo implementado y sus límites.

## Alcance del documento

Este documento conserva las capacidades futuras de Fénix. Se implementarán después de cerrar y validar la especificación de requisitos. Las ideas se mantienen como requisitos de evolución, propuestas de arquitectura y líneas de investigación; no representan funcionalidades disponibles actualmente.

El trabajo futuro debe construirse de forma incremental sobre el contrato, los identificadores, la memoria básica, las herramientas y los controles actuales. Cada ampliación necesita pruebas, límites, procedencia y una política clara de recuperación.

## Memoria avanzada y autonomía ampliada

Todo lo que sigue queda conservado como requisitos de evolución o investigación. Se mantiene la estructura operativa actual, pero se añaden procedencia, versiones, recuperación y evaluación donde corresponda.

### 2.1. Política de memoria automática, preferencias y procedencia

- Decidir explícitamente qué guardar, cuándo introducirlo en el contexto y cómo reducirlo conservando la fuente recuperable. Tratar la memoria como un ciclo `write → manage → read`, con memoria episódica (acontecimientos), semántica (hechos y relaciones), procedimental (formas de trabajar) y operativa (tareas, bloqueos y próximos pasos).
- La política común de admisión debe valorar utilidad futura y relación con el objetivo o perfil, novedad, repetición, saliencia, especificidad, confianza, estabilidad, frescura, verificabilidad, dependencia causal, sensibilidad, privacidad, riesgo, vigencia y costos de almacenamiento y recuperación. Se aplica a recuerdos, preferencias, tarjetas, recordatorios y extracción retrospectiva.
- Detectar patrones repetidos de tono, formato, modelos o herramientas elegidos, horarios, convenciones, flujos de trabajo y decisiones. Toda preferencia aprendida empieza en `candidate` y sólo se promociona tras repeticiones consistentes o confirmación explícita. Una instrucción aislada permanece en su sesión; una preferencia explícita nueva prevalece sobre la inferida.
- Conservar en cada memoria `memory_id`, tipo, contenido, ámbito, `source_event_ids`, `session_id`, `task_id`, confianza, `valid_from`, `valid_until` o TTL, estado, versión y `content_hash`. Añadir fecha de detección, ejemplos, número de confirmaciones, fecha de revisión y motivo de promoción cuando sea aprendida.
- Aplicar siempre aislamiento por usuario, proyecto, tarea o sesión. Las restricciones de privacidad, visibilidad y permisos se comprueban antes de leer, buscar, escribir, compartir con un subagente o enviar datos a un proveedor.
- Resolver la entidad y comprobar duplicados y contradicciones antes de escribir. Elegir entre `create`, `update`, `append`, `supersede`, `archive` y `reject`; mantener revisiones y relaciones `superseded`, `disputed` o `expired` sin sobrescribir silenciosamente la evidencia. No fusionar entidades sólo porque sus nombres se parecen.
- No promocionar automáticamente saludos, conversación casual, información transitoria, contenido redundante, salidas sin valor durable, secretos, credenciales ni datos sensibles o contradictorios. Los datos de alto impacto, privados o ambiguos requieren la autorización aplicable o quedan como candidatos; una puntuación de utilidad alta no basta.
- Separar hechos de inferencias. Guardar todos los supuestos operativos del agente en el contrato de la tarea, con su estado y procedencia; esto no los convierte en hechos del usuario ni autoriza incorporarlos automáticamente a tarjetas o memoria durable.
- El usuario debe poder consultar, corregir, rechazar o eliminar memorias y candidatos. Las correcciones, fallos de ejecución y cambios manuales deben alimentar la revisión de patrones. La promoción común sigue: detectar → acumular ejemplos → proponer → validar → activar con alcance limitado → medir → ampliar o retirar.
- Para inyectar memoria, usar tarjetas de entidades y hechos críticos al inicio, `context_pack` al iniciar o compactar, `delta` en heartbeats, `recall` para recuperación explícita y síntesis sólo cuando haga falta combinar fuentes. Ordenar por relevancia, vigencia, confianza, saliencia, causalidad y procedencia, y respetar el presupuesto de tokens.
- Cada inyección debe conservar IDs y enlaces a evidencia, presupuesto usado, motivo de selección, elementos omitidos y `has_more` cuando corresponda. Nunca inyectar todo lo recuperado ni truncar silenciosamente una lista que se presente como completa.

### 2.2. Compactación durable y rehidratación del contexto

- Antes de compactar se debe guardar de forma síncrona la ventana exacta que se va a reducir, después de escanear secretos y aplicar la política de privacidad. El segmento debe ser direccionado por contenido mediante SHA-256 o BLAKE3, conservar el `content_hash`, `session_id`, rango temporal y `checkpoint_id`, y quedar disponible aunque falle la extracción posterior.
- La captura exacta corresponde al segmento autorizado después de aplicar la política de secretos y privacidad. El hash identifica esos bytes y la normalización aplicada; conservar una fuente no significa guardar secretos sin control.
- El resumen debe utilizar la estructura operativa definida en la sección «Memoria básica y continuidad de contexto» de `REQUIREMENTS.md`, con progreso `done`, `in_progress` y `blocked`, e incluir supuestos, errores, reintentos y efectos ya ocurridos. No es la fuente de verdad: deben permanecer recuperables los mensajes y resultados necesarios para reconstruir las herramientas.
- La compactación debe rehidratar el contexto con el prompt estable, las instrucciones vigentes, el resumen, los mensajes recientes intactos, el estado operativo y un `context_pack` de memoria. Los enlaces al segmento original y a los hechos extraídos deben aparecer después de la frontera de compactación para permitir una recuperación bajo demanda.
- La extracción de hechos puede ejecutarse de forma asíncrona, pero no debe bloquear la respuesta ni ser la única copia. Si falla, el segmento se conserva para un *sweep* posterior. La ruta normal debe ser idempotente mediante hashes y deduplicación; los reintentos pueden ser *at-least-once* sin insertar hechos duplicados.
- La compactación debe distinguir tres garantías: los bytes del segmento son durables en el límite, los hechos quedan disponibles después de la extracción y los enlaces de rehidratación pueden publicarse en la siguiente frontera. Nunca se debe declarar que un dato fue preservado sólo porque el modelo lo incluyó en un resumen.
- Flujo operativo canónico para el guardado y la rehidratación de contexto:

  ```text
  al recibir un mensaje:
    recuperar entity y contexto esencial
    ejecutar el turno
    detectar candidatos de memoria en paralelo
    guardar hechos con procedencia y ámbito

  antes de compactar:
    capturar la ventana exacta
    escanear y redactar secretos
    calcular hash
    guardar checkpoint durable
    iniciar extracción asíncrona

  después de compactar:
    generar resumen estructurado
    recuperar context_pack
    insertar enlaces a checkpoints
    conservar mensajes recientes
    continuar con el mismo task_id y session_id

  en cada heartbeat:
    ejecutar delta con cursor
    entregar sólo cambios nuevos
    conservar los no entregados para el siguiente ciclo
  ```
- Los procesos estancados o interrumpidos y los pendientes deben sobrevivir a la compactación mediante su estado estructurado, no sólo mediante texto. La continuidad mantiene el mismo objetivo y los mismos `task_id` y `session_id`.

### 2.3. Búsqueda avanzada de memoria y recordatorios

La búsqueda debe cubrir hechos, preferencias, decisiones, compromisos, memoria curada y recordatorios. Un recordatorio conserva estado, fechas, entidad, ámbito, evidencia y siguiente acción; no se reduce a una página de texto.

#### Modos e índices de recuperación

| Modo | Capacidad |
| --- | --- |
| Exacto | Resolver `reminder_id`, `memory_id`, `task_id`, `event_id`, `session_id`, slug, título, alias y frases literales. |
| Léxico | BM25 o FTS (`tsvector`) sobre contenido, títulos, entidades, alias, etiquetas y procedencia. |
| Semántico | Embeddings densos para paráfrasis y conceptos, con umbral y evidencia del fragmento. |
| Temporal | Filtrar `due_at`, `snooze_until`, `valid_from`, `valid_until`, `created_at`, `updated_at`, `since`, `until` y consultas históricas `as_of`. |
| Estado | Distinguir `open`, `snoozed`, `blocked`, `triggered`, `completed`, `cancelled`, `expired` y `unknown`; próximas, vencidas, activadas e históricas. |
| Estructurado | Combinar tipo (`reminder`, `fact`, `preference`, `commitment`, `decision`), usuario, proyecto, entidad, prioridad, confianza, fuente, visibilidad y etiquetas. |
| Entidades y relaciones | Recorrer personas, proyectos, tareas y compromisos para resolver dependencias y preguntas como “¿qué debo a Ana?”. |
| Línea temporal | Ordenar primera mención, actividad, revisiones, cambios de estado y vencimiento para reconstruir la evolución. |
| Similitud | Detectar duplicados y sustituciones `supersedes`, conservando fuentes y versiones según la política de curación. |
| Cobertura y agregación | Resolver “todos”, “cuántos”, “qué falta” y “cuáles vencieron” mediante enumeración y filtros. Un `top-k` no es una lista exhaustiva. |

- Combinar recuperación léxica y semántica mediante RRF y, si el presupuesto lo permite, un reranker. Sin proveedor de embeddings, continuar con texto y filtros e informar la degradación; IDs y fechas se resuelven sin modelo.
- El router puede componer operaciones `exact`, `semantic`, `temporal`, `status`, `structured`, `entity`, `relationship` y `aggregate`. Resolver primero IDs y filtros, aplicar tiempo y estado en la base de datos, recuperar candidatos, ampliar relaciones, correlacionar, fusionar, rerankear, deduplicar y limitar.
- Aplicar usuario, proyecto, fuente, visibilidad y privacidad antes de recuperar, fusionar o enviar información a síntesis. No saltar esas restricciones mediante embeddings o grafo. Los expirados salen de la vista activa y permanecen en historia autorizada.
- Mantener sincronizados los índices léxicos, vectoriales, de fecha y estado, alias y adyacencias como vistas reconstruibles de la sección 2.5. Escrituras, ediciones, archivados y eliminaciones deben actualizar o invalidar las vistas afectadas.

#### Dónde buscar, profundidad, guardado y relaciones

La traza debe explicar cuatro decisiones: **dónde** (fuentes, proyectos, sesiones e índices autorizados), **qué tan profundo** (zoom, saltos, fragmentos y presupuesto), **qué guardar automáticamente** (temporal, candidato, memoria durable, tarea, recordatorio o conocimiento derivado, según 2.1) y **cómo se relacionan los resultados** (entidades, dependencias, versiones, procedencia, contradicciones y evidencia).

Dar énfasis a `tilt` para elegir la vista de la estructura de memoria: `tilt=keywords` para términos exactos, `tilt=embedding` para significado, `tilt=graph` para entidades, aristas, backlinks y comunidades, `tilt=temporal` para secuencias y vigencia, y `tilt=concept` para conceptos, atributos y relaciones abstractas. Poder combinar vistas y registrar por qué se eligieron.

#### Contrato de búsqueda, evidencia y evaluación

- Aceptar `query`, `scope`, `filters`, `since`, `until`, `as_of`, cursor, `limit`, `budget_tokens` y nivel de detalle.
- Devolver por resultado `id`, tipo, contenido o resumen, entidad, proyecto, estado, fechas, visibilidad, `source`/`evidence`, procedencia, puntuación por señal, confianza, `create_safety` y enlace al registro completo. Incluir `has_more`, descartes autorizados por estado o privacidad, degradación y explicación; si falta evidencia, indicar `no_evidence` o `unknown`.
- Las respuestas grandes siguen la política de artefactos de la sección 2.9 con `event_id` o `artifact_id`; retirarlas del contexto no elimina la evidencia necesaria para repetir la consulta.
- `recall` recupera hechos y fragmentos con presupuesto y procedencia; `think` combina varias fuentes cuando hace falta, cita evidencia, señala huecos y mantiene las inferencias separadas de los hechos guardados.
- Probar ID exacto, paráfrasis, frase literal, fechas y vencimientos, estados, relaciones, historia `as_of`, deduplicación y sustitución, cobertura exhaustiva, privacidad, expirados y caída del proveedor de embeddings. Ofrecer `search --explain` o equivalente para mostrar señales y rutas.

#### Clasificación determinista y selección de la vista de búsqueda

- Antes de recuperar memoria, Fénix debe clasificar la consulta con reglas deterministas, sin depender de una llamada adicional al modelo. La clasificación debe quedar registrada en `query_intent` y utilizarse para seleccionar el `tilt`, las fuentes, la profundidad del grafo, los filtros temporales y el presupuesto de recuperación.
- Las categorías iniciales son:
  - `entity`: preguntas sobre personas, proyectos, empresas u otras entidades. Ejemplo: «¿Quién trabaja en el proyecto X?». Priorizar nombres, títulos, alias, relaciones y ámbito del proyecto.
  - `temporal`: preguntas sobre un periodo o una fecha. Ejemplo: «¿Qué ocurrió la semana pasada?». Priorizar fechas, recencia, cronología y registros con `valid_from`/`valid_until`; no aplicar un boost de fuente que oculte conversaciones o registros diarios relevantes.
  - `event`: preguntas sobre un hecho o decisión concreta. Ejemplo: «¿Cuándo se aprobó esta decisión?». Priorizar eventos, marcas de tiempo, decisiones y su evidencia de origen.
  - `concept`: preguntas que piden significado, explicación o panorama. Ejemplo: «¿Qué significa esta arquitectura?». Priorizar búsqueda semántica, expansión de consulta controlada y documentos conceptuales.
  - `general`: preguntas comunes que no activan ninguna regla especializada. Usar la búsqueda híbrida estándar.
- Las reglas deben considerar verbos interrogativos, expresiones temporales, nombres propios, identificadores, patrones de decisión y términos conceptuales. Las consultas con señales mezcladas pueden recibir una intención principal y una secundaria, pero deben conservar la consulta original.
- Si hay empate o baja confianza, Fénix debe usar `general`, registrar las señales que produjeron el empate y permitir una segunda recuperación con otro `tilt` cuando la confianza CRAG sea débil. Una clasificación errónea debe degradar la calidad de la ordenación, no impedir que funcionen las ramas léxica y vectorial.
- La clasificación no sustituye los filtros estructurados: una consulta `entity` debe seguir limitada por `project_id`/`source_id`, y una consulta `temporal` debe usar índices o campos de fecha además de la búsqueda semántica. Cada resultado debe conservar `query_intent`, `intent_signals` y la política de enrutamiento para auditoría.

`query_intent` clasifica la pregunta (`entity`, `temporal`, `event`, `concept`, `general`); los modos `exact`, `status`, `aggregate`, etc. describen operaciones que el router puede combinar. No son dos taxonomías incompatibles. `tilt` expresa la vista o prioridad de recuperación, no un nuevo tipo de memoria.

#### Variantes de consulta y control de deriva

- Las variantes pueden mejorar la recuperación cuando el contenido usa sinónimos, otro idioma o una formulación distinta, pero no se deben asumir útiles para todas las consultas. Siempre se debe conservar la consulta original como primera búsqueda y comparar contra ella.
- Ejemplo de expansión para una consulta `entity`:

  ```text
  original:  personas que trabajan en recuperación
  variante:  quién trabaja en retrieval
  variante:  responsables de search ranking
  variante:  equipo de recuperación de información
  ```

- `quién trabaja en retrieval` conserva mejor la intención y añade el término inglés. `equipo de recuperación de información` puede recuperar la formulación técnica equivalente. `responsables de search ranking` es más estrecha y puede cambiar el significado de «todas las personas que trabajan» a «quienes son responsables»; debe tratarse como candidata de mayor riesgo de deriva semántica.
- Cada variante debe conservar el mismo `project_id`/`source_id`, entidad, permisos, intención principal y límites temporales de la consulta original. No se debe permitir que una variante cambie el ámbito y mezcle proyectos o usuarios.
- La expansión debe registrar `variant_id`, texto original, texto generado, intención, rama que produjo cada resultado y costo. Los resultados de todas las variantes se deben fusionar mediante RRF, deduplicar por página o hecho y conservar la procedencia de la variante que aportó cada evidencia.
- La utilidad se debe verificar con una prueba de ablación sobre el mismo corpus, conjunto de respuestas esperadas, semilla, filtros y límite: (a) sólo consulta original y (b) original más variantes. Comparar `Hit@1`, `Hit@3`, `Precision@k`, `Recall@k`, MRR o nDCG, tasa de falsos positivos, consultas negativas, duplicados, latencia, tokens y costo.
- Aceptar una variante o una política de expansión sólo si aumenta la recuperación de respuestas relevantes sin superar el umbral permitido de pérdida de precisión, falsos positivos, latencia o costo. La decisión debe hacerse por familia de consultas (`entity`, `temporal`, `event`, `concept`, `general`), no con un promedio global.
- La expansión debe activarse preferentemente para consultas conceptuales, paráfrasis, consultas largas o resultados con confianza CRAG débil. Debe omitirse para nombres exactos, identificadores, frases entrecomilladas y consultas que ya tienen una coincidencia fuerte.
- Nota histórica de la revisión de GBrain: se registró un límite de consulta original más dos alternativas, saneamiento de entradas y salidas, eliminación de duplicados y retorno a la consulta original ante fallos. También se anotó `recall_all@5` de 54,89 % con expansión frente a 93,19 % sin expansión en una medición de LongMemEval-S. La referencia era `docs/eval/SEARCH_MODE_METHODOLOGY.md`, línea 60, dentro del clon eliminado. Se conservan estos datos como antecedente pendiente de revalidar contra una revisión identificada; no son un benchmark de Fénix ni justifican una activación global.

#### Recuperación por ramas

- Ejecutar en paralelo las ramas independientes `BM25`, `vector`, `título` y `temporal`, con el mismo `project_id`/`source_id`, permisos, privacidad y filtros. La rama `grafo` puede empezar en paralelo si ya hay entidades ancla resueltas; si las anclas salen de los candidatos, el recorrido se ejecuta después. Conservar las cinco ramas sin ignorar esa dependencia.
- `BM25` recupera coincidencias léxicas y nombres exactos; `vector` recupera paráfrasis y significado semántico; `título` protege la identificación de páginas por su nombre; `grafo` sigue relaciones tipadas entre entidades; y `temporal` consulta fechas, cronologías, estados y ventanas de validez.
- Cada rama debe devolver candidatos con su propia puntuación, rango, `branch_id`, procedencia y evidencia. Una rama lenta o no disponible debe poder degradarse sin bloquear las demás, salvo un fallo real de la base de datos.
- Después de la recuperación paralela, Fénix debe fusionar las listas con RRF, recalcular señales comunes, aplicar el `tilt` derivado de `query_intent`, deduplicar, ordenar y hacer cumplir el presupuesto de tokens. La respuesta debe conservar qué ramas contribuyeron a cada resultado y cuáles no estaban disponibles.

#### Correlación y unión de resultados

- Los resultados de `BM25`, `vector`, `título`, `grafo` y `temporal` no se deben concatenar ni fusionar sólo porque tengan palabras parecidas. Fénix debe verificar primero si apuntan a la misma entidad, hecho, relación o versión de un documento.
- La correlación debe intentar, en este orden, `page_id`/`fact_id`/`entity_id` canónico, slug y alias resueltos, enlaces del grafo, hash o versión del contenido, coincidencia temporal y similitud semántica. La similitud semántica por sí sola no autoriza a unir dos hechos.
- Cada grupo correlacionado debe clasificarse como:
  - `same_evidence`: varias ramas encontraron la misma evidencia; se conserva una entrada con las señales de recuperación. Varias búsquedas sobre una misma fuente no constituyen corroboración independiente ni autorizan a multiplicar la confianza en el hecho.
  - `complementary`: las ramas encontraron fragmentos o páginas diferentes que describen el mismo hecho; se enlazan como evidencias complementarias sin borrar sus fuentes.
  - `conflicting`: las fuentes tienen valores, fechas, estados o relaciones incompatibles; se mantienen separadas, se registra el conflicto y se aplica la política de vigencia, procedencia y confianza o se solicita aclaración.
  - `unrelated`: los resultados no están relacionados; permanecen como candidatos independientes y no comparten score ni contexto.
- La unión debe conservar `correlation_id`, `merge_decision`, `branch_ids`, evidencias originales, fechas de validez y procedencia. RRF puede combinar las puntuaciones de las ramas, pero no debe convertir dos resultados distintos en un solo hecho ni contar dos veces el mismo chunk.
- La deduplicación se ejecuta después de correlacionar y antes de inyectar contexto. El modelo debe recibir una evidencia agrupada y trazable, no una lista de fragmentos repetidos. Si la correlación es incierta, Fénix debe conservar las alternativas y marcar la incertidumbre.

#### Entidades ancla, vecinos y cadenas

- Después de obtener los candidatos iniciales, Fénix debe identificar entidades ancla y buscar vecinos directos y cadenas de relaciones que conecten la consulta con los resultados. Una cadena puede tener la forma `A → relación → B → relación → C`; los vecinos pueden ser páginas, hechos, tareas, proyectos o personas.
- El recorrido debe usar enlaces tipados, IDs canónicos, alias resueltos, entidades mencionadas en los chunks, backlinks, coocurrencias y relaciones temporales. Las conexiones inferidas a partir de texto o similitud deben etiquetarse como `inferred`; las conexiones declaradas en un enlace o hecho estructurado como `explicit`.
- Usar un recorrido acotado, como BFS o beam search, con `walk_depth`, `max_neighbors`, `max_paths` y un presupuesto de tokens. Evitar ciclos, explosión de vecinos, sesgo hacia páginas con muchos enlaces y saltos fuera de `project_id`/`source_id` o de los permisos del lector.
- Puntuar cada cadena por longitud, tipo y confianza de los enlaces, coincidencia con la intención, recencia, procedencia y acuerdo entre ramas. Las rutas cortas y con enlaces explícitos deben superar a las rutas largas o sólo semánticas; una ruta no debe considerarse prueba suficiente si carece de evidencia en sus nodos o aristas.
- Conservar para cada ruta `path_id`, `anchor_id`, nodos, aristas, profundidad, `path_score`, tipo de inferencia, fechas de validez y fuentes. La respuesta debe mostrar la cadena que relaciona los resultados y no sólo el nodo final.
- La expansión de vecinos sirve para descubrir contexto relacionado, pero no reemplaza la búsqueda exacta ni la validación de hechos. Si no existe una cadena confiable, Fénix debe devolver los candidatos originales y declarar que la relación no está confirmada.

#### Flujo canónico de consulta relacional

- Para una consulta que pueda requerir entidades, relaciones o vecinos, Fénix debe seguir este orden:

  ```text
  1. clasificar la consulta;
  2. recuperar candidatos con BM25, vector, título y tiempo;
  3. identificar entidades ancla;
  4. recorrer vecinos y cadenas con profundidad limitada;
  5. verificar que los nodos pertenezcan al mismo ámbito;
  6. correlacionar los resultados;
  7. separar evidencia explícita, complementaria y conflictiva;
  8. fusionar puntuaciones;
  9. entregar la ruta y sus fuentes al modelo.
  ```

- El agente debe conservar la consulta original, los candidatos de cada rama, las entidades ancla, las rutas descartadas, la decisión de correlación, las puntuaciones y la procedencia. La síntesis del modelo sólo puede utilizar las rutas y fuentes que hayan superado los filtros de ámbito y evidencia.

### 2.4. Grafo de conocimiento adaptado a cada usuario

- Cada usuario debe tener un esquema de grafo contextual, aislado por `user_id` y, cuando corresponda, por `project_id`/`source_id`. El vocabulario base puede compartirse, pero sus significados, alias, campos obligatorios y reglas de inferencia deben poder adaptarse al dominio, idioma, preferencias y forma de trabajo de ese usuario.
- El conjunto inicial de nodos debe incluir:

  ```text
  personas
  empresas
  proyectos
  reuniones
  documentos
  hechos
  tareas
  conceptos
  ```

- El conjunto inicial de aristas debe incluir:

  ```text
  works_at
  attended
  invested_in
  founded
  advises
  depends_on
  mentions
  related_to
  ```

- Las aristas son verbos semánticos, no simples etiquetas. Cada una debe declarar dirección, arista inversa si existe, tipos de nodo permitidos, propiedades, sinónimos y condiciones de validez. Por ejemplo, `works_at` puede tener como inversa `employs`, y `depends_on` como inversa `required_by`.
- El esquema de un usuario debe poder añadir relaciones propias, por ejemplo `prefers`, `owns`, `blocked_by`, `uses`, `referred_by` o `learning_from`, sin modificar el vocabulario global ni mezclar relaciones de otros usuarios. Las extensiones deben estar versionadas y validadas por el schema pack activo.
- Cada arista debe conservar como mínimo `edge_id`, `from_node_id`, `to_node_id`, `edge_type`, dirección, `user_id`, `project_id`/`source_id`, procedencia, contexto textual, `explicit|inferred`, confianza, `valid_from`, `valid_until`, versión, `content_hash` y `source_event_ids`.
- Una relación explícita creada por un enlace, un campo estructurado o una confirmación del usuario debe tener prioridad sobre una relación inferida. Las inferencias deben poder proponerse, revisarse, rechazarse y expirar sin borrar la evidencia original.
- El agente debe seleccionar el vocabulario y las relaciones activas según el perfil del usuario, el proyecto actual, la intención de la consulta y el idioma. Una misma palabra puede mapear a aristas distintas en dos dominios, por lo que la resolución siempre debe incluir el contexto y no sólo el texto del nombre.
- La búsqueda debe respetar el ámbito en cada salto: no se deben seguir aristas privadas, expiradas o de otro proyecto aunque conecten semánticamente con el resultado. Los nodos y aristas de un usuario nunca se deben reutilizar como memoria de otro sin una política explícita de compartición.
- Las pruebas del grafo deben cubrir el vocabulario base, aristas personalizadas, direcciones inversas, alias, relaciones ambiguas, conflictos, expiración, aislamiento entre usuarios y reconstrucción de una ruta con su procedencia. El modo de explicación debe mostrar qué definición contextual de la arista se utilizó.

### 2.5. Hashes, vectorización dinámica y wiki

#### Integridad y versionado de contenido

- Calcular un hash criptográfico, SHA-256 o BLAKE3, para cada archivo, artefacto y versión relevante antes de indexarlo o modificarlo. Conservar `file_id`, ruta lógica, tamaño, MIME, fechas, `content_hash`, hash del padre cuando corresponda, versión, `created_at` y `updated_at`.
- Identificar los bytes o contenido canónico al que corresponde el hash y registrar normalizaciones de codificación, saltos de línea, metadatos o compresión. El hash de contenido no es un hash de ruta. Una modificación crea una nueva versión y mantiene la evidencia necesaria para auditoría y restauración.
- Usar los hashes para integridad, duplicados exactos, indexación incremental y correspondencia entre una respuesta y la versión consultada, también tras herramientas, compactación o transferencias. Un hash no anonimiza información: aplicar privacidad y redacción antes de almacenar o enviar datos.

#### Decisión de vectorizar y ciclo de vida de embeddings

- Conservar primero la fuente, hash, búsqueda exacta o léxica y metadatos. Decidir dinámicamente por archivo, página o fragmento qué vectorizar; no generar embeddings de todo el repositorio por defecto.
- Notas, decisiones, preferencias y documentos pueden necesitar búsqueda semántica. Para código, combinar texto, rutas, símbolos, AST y referencias; el embedding complementa la búsqueda exacta de nombres, firmas y errores. Tablas, configuraciones, logs, IDs, fechas y estados deben consultarse primero de forma estructurada. Imágenes, audio, vídeo y binarios requieren un modelo o índice específico y una necesidad real.
- Evaluar estabilidad, frecuencia de lectura y actualización, tamaño, idioma, calidad del texto, sensibilidad, relaciones, consultas fallidas, disponibilidad de modelo, beneficio semántico, costo, latencia y retención. No enviar contenido privado, secretos o datos con restricciones de residencia a un proveedor sin la autorización aplicable.
- Ante paráfrasis recurrentes o falta de evidencia, permitir generación diferida `lazy/on-demand` y una nueva búsqueda. Mantener disponibles los resultados léxicos y mostrar indexación pendiente si se agota el tiempo.
- Utilizar estados: **candidato** (hash, FTS y metadatos), **bajo demanda**, **activo**, **frío/archivado** (fuente conservada y embedding congelado o eliminado) y **obsoleto** (contenido, configuración o permisos cambiados).
- Reevaluar periódicamente y ante cambios de uso; priorizar documentos nuevos, alta demanda, páginas enlazadas a consultas fallidas y fragmentos que causen errores. Aplicar presupuesto por proyecto de tiempo, llamadas, tokens, almacenamiento y energía.
- Registrar `embedding_required`, `decision_reason`, `demand_score`, `semantic_gain`, `cost_estimate`, `privacy_class`, `last_queried_at`, `next_review_at`, modalidad, proveedor, modelo y versión, dimensiones, idioma, estrategia de fragmentación, fecha de generación, `source_content_hash` y política de retención. Explicar qué consulta o métrica provocó cada decisión.
- Crear fragmentos estables con `chunk_id`, líneas u offsets, hash, `file_id`, página o revisión y enlace a la fuente. Ajustar tamaño y solapamiento al contenido; usar página completa para orientación y fragmentos para evidencia cuando resulte útil.
- Un embedding es válido sólo para su fuente y configuración. Cambios en hash, modelo, dimensiones, idioma, fragmentación, privacidad o permisos deben invalidarlo; regenerar únicamente lo afectado y mantener búsqueda léxica mientras se valida.
- Registrar quién creó, actualizó, invalidó o eliminó hashes e índices, con herramienta, versión de fuente y resultados dependientes. Al eliminar o retirar acceso a una fuente, invalidar sus índices léxicos, vectoriales y de grafo, sujeto a la política explícita de conservación de evidencia.
- Comparar ruta exacta/léxica/estructurada, vectorización completa y vectorización selectiva mediante precisión, recall, nDCG, idiomas, cobertura, resultados sin evidencia, duplicados, latencia y costo. Los datos de uso ajustan umbrales; una preferencia puntual no crea por sí sola una regla permanente.

#### Memoria con estructura de wiki

- Evaluar una wiki personal y de proyectos con `page_id` estable, título canónico, alias, resumen, contenido, tipo, propietario, permisos, etiquetas, entidades, enlaces entrantes y salientes, citas, validez, estado y revisiones enlazadas.
- La fuente de verdad son los registros canónicos y sus revisiones. FTS, embeddings, índices temporales y adyacencias son vistas reconstruibles. Los hechos, decisiones, preferencias, tareas y recordatorios enlazados conservan su propio estado y procedencia.
- Navegar por títulos, backlinks, entidades, relaciones y versiones `as_of` con la política de recuperación de la sección 2.3. Los encabezados, anclas y fragmentos deben conservar referencias estables.
- Cada cambio activa indexación incremental: comparar hashes, recalcular fragmentos afectados, mantener la revisión anterior mientras se valida la nueva y retirar entradas derivadas sin fuente autorizada.
- La wiki es una vista del conocimiento. Contratos, presupuestos, pendientes, bloqueos, cron, heartbeats y auditoría siguen en registros operativos estructurados con permisos y retención propios. Un resumen o inferencia no se transforma automáticamente en un hecho.

### 2.6. Edición estructural de Markdown con Tree-sitter

- Toda modificación automática de un archivo Markdown debe pasar primero por Tree-sitter con la gramática correspondiente, por ejemplo `tree-sitter-markdown`. El árbol sintáctico permite identificar encabezados, secciones, párrafos, listas, tablas, enlaces, citas, bloques de código, HTML, front matter y sus rangos exactos de bytes o líneas.
- Antes de editar, el contrato de la tarea debe definir qué nodos o secciones están autorizados y cuáles están protegidos. La autorización debe expresarse mediante el `file_id`, el hash de la versión de entrada, la ruta del nodo, el tipo de nodo, su rango y la operación permitida (`insert`, `update`, `move`, `delete` o sólo lectura). Una instrucción para añadir una nota no autoriza cambiar encabezados, enlaces, código, front matter ni otras secciones.
- El agente debe resolver el objetivo en el árbol, no mediante reemplazos globales o expresiones regulares. Si hay varios encabezados con el mismo texto, debe usar la ruta completa, el índice de la sección, el `node_id` o una ancla estable. Si no puede identificar una única región, debe detenerse y marcar la operación como ambigua antes de escribir.
- Debe existir una lista de protección por archivo. Como valores iniciales se deben proteger los bloques de código, comandos, front matter, HTML, enlaces y referencias no relacionados, tablas, secciones marcadas como inmutables, contenido generado y cualquier nodo fuera del alcance del contrato. Las excepciones deben quedar registradas de forma explícita.
- La edición debe producir el parche mínimo posible sobre los rangos autorizados y conservar el estilo local: niveles de encabezado, listas, indentación, saltos de línea, formato de enlaces y texto fuera del rango. Tree-sitter se usa para delimitar y verificar la estructura; una capa de reglas decide si el cambio semántico está permitido y cómo mantener el formato.
- Después de aplicar el parche se debe volver a analizar el archivo y comprobar que no existan errores de parseo. También se deben comparar los hashes de los nodos protegidos, los rangos no autorizados y los enlaces; cualquier cambio fuera de la región aprobada debe invalidar el parche y restaurar la versión anterior.
- La validación debe comprobar que la nueva sección está bajo el encabezado correcto, que no se insertó dentro de un bloque de código o una cita, que los enlaces y referencias siguen resolviendo y que no se alteró el front matter ni la estructura de la wiki. Para documentos con un esquema propio se deben validar además campos obligatorios, IDs, etiquetas, estados y relaciones.
- Cada operación debe guardar en la auditoría el hash anterior y posterior, el `task_id`, la versión del contrato, la ruta y tipo de cada nodo modificado, rangos originales y nuevos, parche aplicado, reglas que lo autorizaron, resultado del parseo y hashes de las regiones protegidas. La reversión debe poder reaplicar el parche inverso sobre la misma versión.
- Tree-sitter no demuestra por sí solo que el contenido sea correcto: sólo aporta la estructura sintáctica. La autorización, privacidad, alcance del proyecto, contrato de la tarea y validación de enlaces o datos deben permanecer en una política independiente. Si la gramática no puede analizar un archivo o el cambio requiere reinterpretar contenido ambiguo, se debe conservar el archivo intacto y solicitar revisión.

### 2.7. Tarjetas rápidas de entidades

> ¿Cuál es la información mínima, útil, verificable y segura que Fénix debe guardar para poder generar una tarjeta rápida de una persona, empresa o proyecto, y qué herramientas o skills deben existir para crearla, actualizarla y recuperarla sin mezclar contextos?

#### Esquema, afirmaciones y presentación

- Una tarjeta es una vista compacta de una entidad canónica y enlaza a la evidencia completa: hechos, decisiones, tareas, conversaciones, documentos y revisiones.
- Compartir un esquema versionado con `entity_id`, `entity_type`, nombre canónico, alias, resumen, propietario, ámbito (`user`, `project`, `global`), permisos, etiquetas, estado, creación, actualización, validez, `source_event_ids`, confianza, `content_hash`, versión y entidades relacionadas.
- Cada afirmación importante tiene `claim_id`, texto o valor estructurado, fuente, fecha observada, `valid_from`, `valid_until` o `expires_at`, confianza, autor y estado (`active`, `superseded`, `disputed`, `unknown`). Corregir una afirmación no debe reescribir toda la tarjeta.
- Mostrar resumen, objetivo o rol, estado, relaciones, últimos cambios, tareas abiertas, bloqueos, próximos pasos y evidencia por separado. Indicar `last_verified_at`, campos desconocidos, disputas y posible obsolescencia; acceder a campos sensibles sólo cuando los permisos y la necesidad lo permitan.
- Usar Markdown con front matter estructurado o JSON validado como presentación. Aplicar Tree-sitter a la vista Markdown; el registro estructurado y sus hashes permiten reconstruirla.
- Seleccionar y promocionar datos con la política común de la sección 2.1. Las observaciones temporales y los supuestos no confirmados no deben rellenar campos como hechos.
- Resolver homónimos y alias con su ámbito. Si una persona o empresa aparece en varios proyectos, mostrar cuál se consulta y aclarar una ambigüedad que pueda mezclar contextos.

#### Campos iniciales por tipo

- **Persona:** nombre y alias, pronombres si fueron proporcionados, rol, organización, relación con el usuario, preferencias de comunicación explícitas, compromisos, temas de interés, última interacción, próximos seguimientos y restricciones de privacidad o consentimiento.
- **Empresa u organización:** nombre legal y nombres usados, dominio, sector, descripción, contactos vinculados, relación con el usuario, productos o servicios, proyectos asociados, contratos o decisiones relevantes, estado de la relación, ubicación si es necesaria y fuentes.
- **Proyecto:** `project_id`, nombre, propósito, propietario, participantes, repositorio o rutas autorizadas, estado, objetivos y criterios de éxito, hitos, decisiones, riesgos, bloqueos, tareas abiertas, presupuesto, dependencias, última actividad y próximos pasos.
- El esquema debe permitir tipos futuros como equipo, cliente, producto, lugar, documento o concepto. La tarjeta debe indicar explícitamente los campos que no se conocen; no se deben rellenar con inferencias presentadas como hechos.

#### Herramientas y skills de tarjetas

- `extract_memory_candidates`: extraer posibles hechos, preferencias, compromisos y entidades desde un turno o artefacto, con citas y confianza.
- `resolve_entity`: encontrar o crear una entidad canónica usando nombre, alias, dominio, proyecto y contexto permitido; devolver candidatos y pedir desambiguación cuando sea necesario.
- `get_entity_card`: recuperar la tarjeta rápida con filtros de ámbito, permisos, frescura y nivel de detalle.
- `search_entity_cards`: buscar tarjetas por texto exacto, búsqueda híbrida, filtros estructurados, fechas, estado, etiquetas o relaciones.
- `expand_entity`: recorrer relaciones autorizadas hacia personas, empresas, proyectos, tareas y documentos con límite de profundidad y presupuesto.
- `get_entity_timeline`: mostrar hechos, interacciones, revisiones, cambios de estado y decisiones en orden temporal.
- `get_open_actions`: recuperar compromisos, recordatorios, bloqueos y próximos pasos asociados a una entidad.
- `refresh_entity_card`: volver a consultar las fuentes, detectar datos obsoletos y proponer actualizaciones sin sobrescribir la evidencia.
- `curate_entity_memory`: deduplicar, consolidar y marcar afirmaciones obsoletas o contradictorias mediante propuestas versionadas y reversibles.
- `audit_entity_card`: mostrar quién creó o cambió cada campo, qué herramientas se utilizaron, hashes, fuentes, permisos y posibilidad de revertir.
- Estas capacidades pueden implementarse como skills internas con contratos claros. Cada skill debe recibir `entity_id`, `scope`, permisos, presupuesto y objetivo; no debe consultar automáticamente toda la memoria del usuario ni otros proyectos.

El router prefiere `get_entity_card` para preguntas directas, `get_open_actions` para pendientes, `get_entity_timeline` para historia y `expand_entity` o `search_entity_cards` para relaciones. La síntesis cita los `claim_id`; no se guarda como hecho automáticamente.

#### Evaluación de las tarjetas

- Probar creación desde texto explícito, actualización de una preferencia, resolución de alias, entidades homónimas, contradicciones, datos caducados, permisos, separación entre proyectos, recuperación por ID y búsqueda semántica. Medir completitud, exactitud, frescura, trazabilidad, tasa de fusiones incorrectas, latencia y costo.
- La pregunta de diseño queda abierta para definir el esquema mínimo definitivo, los umbrales de guardado automático, qué campos requieren confirmación y qué tarjetas deben tener una política de retención especial.

### 2.8. Curación, duplicados, contradicciones y hook nocturno

#### Ejecución del mantenimiento

- Ejecutar mantenimiento de baja prioridad en una ventana configurable por usuario, proyecto y zona horaria. Las correcciones urgentes, la revocación de permisos y las invalidaciones de índices también deben aplicarse inmediatamente.
- Consultar sesiones activas, tareas, cron, heartbeats, locks y procesos antes de actuar. No modificar recursos en edición, bajo recuperación, con contrato abierto o con una tarea bloqueada que dependa de ellos.
- Usar lotes con cursor, cola durable, checkpoint, lock con vencimiento y presupuestos independientes de tiempo, tokens, almacenamiento, modelos, concurrencia y mutaciones. Poder pausar, liberar recursos y continuar en otra ventana sin perder candidatos ni repetir cambios.

#### Extracción nocturna de sesiones anteriores

- Evaluar un hook que recorra desde la última revisión y detecte información importante aún no persistida: decisiones, preferencias, compromisos, cambios de proyecto, lecciones de errores, pendientes y procesos estancados.
- Conservar la evidencia original autorizada, el ámbito y la confianza; aplicar la política de admisión de la sección 2.1. Los supuestos van al registro operativo, no a memoria factual; excluir saludos, secretos, salidas sin valor durable y datos temporales sin TTL.
- Registrar `run_id`, sesiones examinadas, `memory_id` creados o actualizados, hashes, candidatos omitidos, costos y errores. Esta extracción complementa la del turno y recupera fallos asíncronos; no es la única copia de información importante.

#### Detección y consolidación de duplicados

- Buscar duplicados en segundo plano después de escribir, antes de promocionar memoria o durante mantenimiento, sin bloquear la respuesta.
- Filtrar primero por ámbito, permisos, vigencia y tipo. Comparar por etapas: `content_hash` → texto normalizado y similitud léxica → entidades, relaciones y fechas → embeddings para paráfrasis cuando hagan falta.
- Clasificar cada grupo como `exact_duplicate`, `near_duplicate`, `complementary`, `conflicting` o `unrelated`. Automatizar sólo duplicados exactos o casi duplicados de alta confianza; conservar alternativas y propuestas revisables para casos complementarios, conflictivos o inciertos.
- Elegir un registro canónico con `duplicate_of` o `supersedes`, conservando fuentes, versiones, hashes, fechas, procedencia y decisión de fusión. Repetir una curación no debe generar otro resumen o volver a fusionar el mismo grupo.
- Registrar `dedup_run_id`, cursor, candidatos comparados, método, umbrales, coincidencias, omisiones, costo, errores y estado. Antes de inyectar contexto, reutilizar las decisiones confirmadas. Marcar `dedup_pending` cuando falte curación; deduplicar provisionalmente sólo lo inequívoco y conservar evidencias distintas si aún no se sabe si son equivalentes.

#### Huérfanos, caducidad y contradicciones

- Localizar páginas sin conexión con raíces activas, sin enlaces entrantes o salientes, hechos sin entidad o fuente, tareas y recordatorios sin proyecto o referencia válida, índices sin fuente y registros con `valid_until`, `expires_at`, TTL o retención vencidos.
- Distinguir `orphan`, `expired`, `stale` y `overdue`. Guardar motivo, referencias rotas, fechas, hash, `curation_id` y propuesta: reparar enlaces, renovar vigencia, archivar o eliminar según política. Estar vencido no equivale a estar completado ni a carecer de valor histórico.
- Retirar elementos expirados de la vista activa y de `context_pack`; conservarlos en historia autorizada hasta que la retención permita purga.
- Investigar el ciclo completo de contradicciones: detección, almacenamiento de afirmaciones incompatibles, autoridad y vigencia, presentación al agente, compactación, consolidación, resolución y expiración. Nunca sobrescribir silenciosamente una afirmación ni perder la fuente que permite cuestionarla.

#### Propuestas, mutaciones y reversión

- Operaciones de bajo riesgo: detectar duplicados exactos, compactar índices derivados, retirar entradas sin fuente, recalcular enlaces y backlinks, invalidar embeddings, actualizar estadísticas, detectar recuerdos sin referencias y preparar resúmenes.
- Producir propuestas con IDs, diferencias, confianza, motivo, fuentes y efecto esperado para fusiones semánticas, alias, consolidación de revisiones, resúmenes antiguos, posible obsolescencia, sustitución de recordatorios o correcciones de relaciones.
- No borrar ni modificar automáticamente datos personales, preferencias, decisiones, contratos, pendientes, bloqueos, credenciales o auditoría sólo por parecer redundantes. La eliminación permanente necesita una política explícita o autorización; preferir archivo o revisión recuperable dentro de esa política.
- Antes de mutar, verificar hash y versión esperada, usar transacciones o escrituras atómicas, crear checkpoint y plan de reversión. Para Markdown, delimitar nodos con Tree-sitter y cancelar si cambian regiones protegidas.
- Cada candidato lleva `curation_id`, `run_id`, estado (`candidate`, `proposed`, `applied`, `rejected`, `rolled_back`, `blocked`), versión, hashes, criterio y resultado. Índices y cachés pueden reconstruirse; el registro canónico exige revisión versionada.
- Registrar inicio y fin, ventana y zona horaria, versión de reglas, modelo, candidatos, cambios, omisiones, errores, consumo, hashes, artefactos y bloqueos.
- Validar integridad, Markdown, enlaces, permisos, tiempo, referencias de tareas, reconstrucción de índices y ausencia de procesos activos afectados. Revertir sólo el lote que falle y dejarlo `blocked`.
- Medir duplicados, fusiones aceptadas o revertidas, falsos positivos, espacio ahorrado, costo, consultas que empeoran, contradicciones ocultas, falsos conflictos, tiempo de resolución y pérdida de procedencia. Comparar con el comportamiento observado de GBrain cuando se vuelva a disponer de una revisión verificable.

### 2.9. Retención avanzada de salidas y artefactos

La retención depende de la utilidad para la tarea, recuperación y auditoría, además del tiempo transcurrido. Los errores abiertos, verificaciones, decisiones, procesos, cambios y dependencias pendientes pueden obligar a conservar una salida.

| Clase | Política |
| --- | --- |
| `critical` | Conservar completa mientras una tarea, bloqueo o auditoría dependa de ella. |
| `active` | Mantener disponible durante el turno y mientras haya reintentos o verificaciones. |
| `reference` | Retirar del contexto y conservar como artefacto recuperable. |
| `transient` | Resumir o eliminar después de procesarla, si ya no aporta evidencia ni estado. |

- Registrar `retention_class`, `retention_reason`, `expires_at` o `review_at`, `artifact_id`, `content_hash` y la tarea o auditoría que la mantiene. Cada cambio de clase conserva valor anterior y nuevo, regla y evidencia.
- Evaluar como valores iniciales configurables 24 horas sin actividad para resumir una salida `active` y 30 días para retener un artefacto `reference`. No aplicar estos plazos a trabajo activo, bloqueado o investigado, ni convertirlos en purgas obligatorias sin medir su utilidad.
- Antes de resumir o eliminar, extraer conclusión, errores y códigos de salida, archivos y cambios, procesos iniciados o detenidos, IDs, decisiones, pruebas y próximos pasos. Conservar `event_id`, `tool_call_id`, fecha, comando, hash y enlace a la fuente.
- Guardar salidas grandes fuera del historial como artefactos inmutables. Enviar al modelo tamaño, estado, últimas líneas o extracto útil, advertencias y ruta; permitir recuperar mediante `session_search` o una herramienta equivalente.
- Antes de compactar, retirar transitorios antiguos, resumir referencias y mantener disponible la evidencia de pendientes, bloqueos, verificaciones, checkpoints y efectos externos. No romper la relación entre llamada y resultado.
- No eliminar la única prueba de una ejecución, los datos necesarios para revertir, contenido bajo disputa o salidas de un proceso aún activo. Conservar el artefacto aunque salga del prompt.
- Redactar credenciales, tokens, cookies y datos personales según política antes de almacenar o resumir. Toda purga, truncamiento, resumen o restauración deja motivo, regla, tamaños anterior y posterior, hash original y ubicación del artefacto.
- Medir consultas a salidas antiguas, errores por resúmenes incompletos, almacenamiento, tokens ahorrados y tiempo de recuperación; ajustar con esos datos los plazos y umbrales.

### 2.10. Autonomía prolongada, presupuestos y supervisión

Application-level maximum token budgets are deferred. Initial onboarding must not request a token quota. Future work may add cumulative per-task, daily, or project quotas and their administration. Existing provider context/output constraints and usage reporting remain distinct.

Los presupuestos máximos de tokens de la aplicación quedan aplazados. La bienvenida inicial no debe pedir una cuota de tokens. Más adelante se podrán añadir cuotas acumuladas por tarea, día o proyecto y su administración. Los límites técnicos de contexto/salida del proveedor y el registro de consumo son conceptos distintos.

- Ejecutar tareas durante horas o días con progreso persistido, dependencias explícitas, supervisión y recuperación. La vida de la tarea debe ser independiente de la sesión de chat o llamada.
- Ampliar el presupuesto básico por tarea a tiempo, tokens, costo, llamadas al modelo y herramientas, reintentos, tamaño de contexto y efectos externos. Reservar presupuestos para delegaciones, mantenimiento y validación, con límites globales y por proyecto.
- Registrar consumo y progreso. Ante un límite, resumir el estado y detener, pausar o bloquear con una razón verificable; no reiniciar el presupuesto por compactar o reintentar.
- Detectar repeticiones sin progreso con firmas normalizadas de herramientas, argumentos y resultados. Advertir, bloquear el siguiente ciclo si persiste y terminar cuando se repita una condición crítica; reforzar la comprobación después de compactaciones y reintentos.
- Separar los heartbeats de revisión del asistente de los latidos técnicos de procesos o trabajadores. Registrar procesos estancados, interrumpidos, activos y terminados para decidir una recuperación.
- Ampliar el planificador con disparadores por eventos, prioridades, capacidad disponible, historia de ejecuciones y dependencias. Cron y heartbeat siguen fuera del razonamiento del modelo y de los adaptadores de canal; pueden utilizar contexto ligero o sesiones aisladas.
- Coordinar la ejecución mediante la cola durable de la sección 2.11, los contratos de la sección 2.13 y la auditoría de la sección 2.12.

### 2.11. Cola durable, deduplicación de tareas y recuperación

- Fénix debe crear una cola de tareas persistida en la base de datos. La cola debe ser la fuente de verdad del trabajo pendiente y no depender de la memoria del proceso, de una sesión de chat abierta ni de una cola sólo en RAM. Telegram puede iniciar o consultar una tarea, pero la tarea debe continuar siendo recuperable aunque se cierre la sesión, se reinicie el agente o se caiga el trabajador.
- Cada elemento de la cola debe conservar, como mínimo, `job_id`, `task_id`, `run_id` o `attempt_id`, `parent_job_id`, `session_id`, `project_id`, tipo de tarea, referencia al payload, prioridad, estado, número de intentos, `available_at`, `lease_until`, identificador del trabajador, último heartbeat, fecha límite, `idempotency_key`, cursor, `checkpoint_id`, presupuesto, resultado o `artifact_id`, error normalizado y marcas de tiempo de creación, actualización y finalización. Los datos grandes deben almacenarse como artefactos direccionados por hash y la fila debe conservar su referencia.
- El ciclo de vida debe ser explícito: `queued`, `waiting`, `claimed`, `running`, `waiting_approval`, `waiting_external`, `blocked`, `succeeded`, `failed`, `cancelled`, `dead` o `unknown`. Una tarea con dependencias puede permanecer en `waiting`; una tarea agotada debe pasar a `dead` o `blocked` con la razón y la evidencia, nunca desaparecer silenciosamente.
- La toma de trabajo debe ser atómica. El trabajador debe reclamar una tarea con una transacción y un *lease* con vencimiento (por ejemplo, `SELECT ... FOR UPDATE SKIP LOCKED` o el mecanismo equivalente), registrar el evento de reclamación y renovar el *lease* mediante heartbeats. Si el *lease* expira, un recuperador debe volver a poner la tarea en `queued` o marcarla `unknown` según el tipo de efecto y el último checkpoint.
- La cola debe persistir la intención antes de ejecutar, el progreso y los checkpoints durante la ejecución, y el resultado antes de marcar la tarea como terminal. Tras un reinicio, Fénix debe escanear tareas `claimed` o `running` con *lease* vencido, comprobar procesos y hashes, y reanudar desde el último cursor o checkpoint, reintentar con retroceso exponencial o pedir revisión humana. No se debe repetir un efecto externo no verificable; se debe consultar su idempotency key o marcarlo `unknown`.
- Debe soportar reintentos acotados, `max_attempts`, presupuesto de tiempo, tokens, costo, herramientas y concurrencia, además de cancelación, pausa, prioridad y *backoff*. Cada reintento crea un nuevo `run_id` o `attempt_id`, conserva el anterior y registra `retry_of`; no debe duplicar tareas por volver a insertar el mismo `idempotency_key`.
- Se debe evitar la duplicación de tareas en tres puntos: al recibir la solicitud, al insertar en la cola y al reclamarla para ejecución. Fénix debe construir una huella determinista a partir de la intención normalizada, el ámbito (`user_id`, `project_id` y recursos), el payload o su hash, la ventana temporal y la versión de la tarea; una restricción única en la base de datos sobre esa huella o `idempotency_key` debe rechazar inserciones repetidas incluso si llegan concurrentemente. Si ya existe una tarea equivalente, la nueva solicitud debe enlazarse a la original y devolver su `job_id`, esperar su resultado o fusionar sólo las partes compatibles, según una política explícita. Los reintentos del mismo trabajo deben reutilizar la clave de idempotencia, mientras que una nueva ejecución solicitada de forma intencional debe crear otro `run_id` y registrar por qué no es un duplicado.
- Las tareas padre e hijas deben formar un DAG auditable. El padre debe poder esperar, continuar o fallar según el estado de sus hijos; los trabajos bloqueados deben conservar qué dependencia falta. La cola también debe servir para subagentes, cron, heartbeats, compactación, curación nocturna, actualización de índices y recuperación de huérfanos, cada uno con su ámbito de usuario, proyecto y permisos.
- Cada transición debe generar un evento append-only con `event_id`, estado anterior y nuevo, actor, timestamps, motivo, `task_id`, `job_id`, hashes y referencias a herramientas, artefactos y checkpoints. La interfaz debe devolver un identificador de seguimiento y permitir `inspect`, reanudar, cancelar, reintentar o restaurar una tarea sin perder la auditoría.

### 2.12. Identificadores, auditoría completa e historial restaurable

- Cada sesión, tarea, ejecución y turno debe tener un identificador único, estable y correlacionable. Como mínimo se deben utilizar `session_id` para la conversación persistente, `task_id` para el objetivo lógico, `run_id` para una ejecución concreta de ese objetivo, `turn_id` para cada intercambio con el usuario, `iteration_id` para cada llamada al modelo dentro del turno, `tool_call_id` para cada llamada de herramienta, `event_id` para cada registro del historial y `checkpoint_id` para cada punto de restauración.
- Los identificadores deben ser generados por Fénix antes de ejecutar trabajo, no por el modelo ni por la herramienta. Deben ser únicos entre procesos y reinicios, preferiblemente UUIDv7 o un formato equivalente que conserve orden temporal. Cada registro debe incluir `parent_id` o los vínculos equivalentes para reconstruir la cadena `session → task → run → turn → iteration → tool_call` y relacionar una ejecución hija con su agente padre.
- Un reintento no debe sobrescribir la ejecución anterior. Debe conservar el mismo `task_id`, crear un nuevo `run_id` o `attempt_id` y enlazarlo mediante `retry_of`. Una delegación debe crear su propio `task_id` y conservar `parent_task_id`, `parent_run_id` y el motivo de la delegación.

#### Registro inmutable de eventos

- La auditoría debe ser un registro append-only. Cada cambio de estado se añade como un evento nuevo y no se edita destructivamente. El registro debe conservar, como mínimo, `event_id`, identificadores de correlación, tipo de evento, estado anterior y nuevo, timestamp del servidor, actor (`user`, `model`, `tool`, `scheduler` o `system`), versión del agente, proveedor y modelo utilizados.
- Antes de ejecutar una herramienta se debe persistir un evento de intención (`tool_call_requested`) con el nombre exacto, versión de la herramienta, esquema y argumentos normalizados, hash de los argumentos, motivo u objetivo, política de riesgo, aprobación requerida y aprobación recibida. Si el registro no se puede guardar, la herramienta no debe ejecutarse.
- Durante la ejecución se deben guardar `tool_call_started`, latidos para operaciones largas, cambios de progreso, solicitudes de aprobación, cancelación, timeout y el evento terminal (`tool_call_succeeded`, `tool_call_failed` o `tool_call_cancelled`). Cada evento debe referir al mismo `tool_call_id` y tener una secuencia monotónica dentro de la ejecución.
- El resultado debe registrar estado, código de salida, respuesta estructurada, hash del resultado, duración, consumo de tokens o costo cuando corresponda, proceso utilizado, directorio de trabajo, backend (local, Docker, SSH, etc.), archivos o artefactos producidos y error normalizado. Las salidas grandes deben guardarse como artefactos con ruta, tamaño, hash y política de retención; el evento puede contener sólo una vista reducida.
- También se deben registrar el mensaje original del usuario, la respuesta del modelo, los `tool_calls` emitidos, el prompt o versión de contexto utilizada, las compactaciones, los cambios de proveedor, los reintentos, los límites alcanzados, las verificaciones y la decisión final. Debe ser posible responder: qué pidió el usuario, qué decidió el modelo, qué se ejecutó, con qué permisos, qué cambió y por qué terminó.
- Los eventos deben encadenarse mediante `previous_event_hash` y un hash del contenido canónico. Esto permite detectar modificaciones posteriores. La auditoría debe poder exportarse por `session_id`, `task_id`, `run_id` o intervalo temporal sin depender del texto generado por el modelo.

#### Estado, recuperación y reanudación

- El estado actual debe derivarse de los eventos o mantenerse en una proyección reconstruible. No se debe considerar que una tarea terminó sólo porque existe una respuesta textual; debe existir un estado explícito como `queued`, `running`, `waiting_approval`, `waiting_external`, `blocked`, `succeeded`, `failed`, `cancelled` o `unknown`.
- La persistencia debe seguir este orden: registrar la intención, crear el checkpoint si hay una mutación local, ejecutar el efecto, registrar el resultado, actualizar la proyección y sólo entonces permitir que el modelo continúe. Las escrituras de la intención y del checkpoint deben ser atómicas desde el punto de vista de la cola de trabajo.
- Al iniciar Fénix, se deben buscar ejecuciones que quedaron en `running` sin evento terminal. El recuperador debe revisar el último heartbeat, verificar si el proceso sigue vivo, comparar los hashes del estado y decidir entre reanudar, reintentar con una clave de idempotencia, cancelar o marcar la tarea como `unknown` para revisión humana. Nunca se debe repetir automáticamente una operación externa cuyo resultado no pueda comprobarse.
- Cada herramienta debe declarar si es `idempotent`, `reversible_local`, `checkpointed`, `compensatable` o `irreversible`. Para operaciones idempotentes se puede reintentar usando `idempotency_key = task_id + tool_call_id`; para operaciones reversibles se restaura un checkpoint; para efectos externos se debe ejecutar una acción compensatoria explícita; para operaciones irreversibles se requiere aprobación y registro de esa aprobación.
- El agente debe poder reconstruir un “estado anterior” en dos niveles: el estado lógico de la conversación y tareas mediante el event log, y el estado físico de los archivos mediante checkpoints o snapshots. Un log por sí solo no deshace un correo enviado, un mensaje de Telegram, una solicitud HTTP o una modificación remota.

#### Inspect, checkpoints, restauración por niveles e historial tipo Git

Limitación registrada durante la revisión de OpenClaw: cancelar una ejecución no revierte automáticamente los efectos ya realizados. Fénix debe distinguir siempre recuperación lógica, restauración de archivos y compensación de acciones externas.

- `inspect` debe ser la operación principal para volver de forma segura al estado anterior: debe mostrar la línea de tiempo de eventos, el `checkpoint_id`, la versión, los hashes, los archivos o artefactos afectados, las herramientas ejecutadas, el estado lógico de la tarea y la acción de restauración o compensación disponible. `inspect` no debe mutar nada; primero permite comparar el estado actual con uno anterior y después solicitar una restauración explícita.
- Fénix debe ofrecer una funcionalidad `restore` o `rollback` por niveles: (1) tarea completa, para restaurar todos los cambios locales reversibles asociados a un `task_id`; (2) turno, para restaurar sólo los efectos de un `turn_id`; (3) llamada de herramienta, para revertir el alcance producido por un `tool_call_id`; y (4) archivo, para recuperar una versión concreta mediante su `checkpoint_id`, hash o snapshot. Cada nivel debe mostrar una vista previa y declarar exactamente qué eventos, archivos, artefactos y dependencias serán afectados.
- Antes de `write_file`, `patch`, migraciones, comandos destructivos o cualquier operación que modifique un repositorio, se debe crear un `checkpoint_id` y guardar el commit o snapshot, el alcance, el hash inicial y los archivos incluidos. Se debe limitar la frecuencia para evitar checkpoints redundantes, pero nunca omitirlo cuando la política de la herramienta lo exija.
- Antes de restaurar, Fénix debe comprobar permisos, hashes, versiones, tareas dependientes y ediciones posteriores. Por defecto debe preservar cambios humanos posteriores mediante un ledger de cambios; si detecta un conflicto, debe detenerse y marcarlo para revisión. Una restauración completa puede reemplazar todo el alcance sólo con una orden explícita y una confirmación adecuada al riesgo.
- La restauración debe ser idempotente, admitir modo de simulación (`dry_run`) y crear un nuevo checkpoint antes de mutar. Cada operación debe registrar el nivel, el objetivo, el estado previo y posterior, los cambios aplicados u omitidos, el motivo, el actor y los hashes. Los efectos externos —por ejemplo, un mensaje enviado— no se revierten automáticamente: requieren una acción compensatoria específica, también auditada.
- Fénix debe ofrecer una vista de historial navegable, similar a Git, para consultar la evolución de una tarea, turno, llamada de herramienta, archivo o proyecto. Cada punto debe mostrar su identificador, padre, fecha, autor, mensaje o motivo, estado, diff o resumen del cambio, archivos y artefactos afectados, hashes, checkpoint asociado y relación con reintentos o restauraciones. Debe permitir recorrer una línea temporal, comparar dos versiones y localizar qué cambio introdujo un dato, sin asumir que el historial visible sustituye al registro append-only.
- Cada restauración debe generar sus propios eventos (`restore_requested`, `restore_started`, `restore_succeeded` o `restore_failed`) y enlazarse con el `checkpoint_id` y el `tool_call_id` que produjo el cambio. Restaurar no debe borrar la auditoría del cambio original.

#### Esquema de evento sugerido

```json
{
  "event_id": "01J...",
  "session_id": "01J...",
  "task_id": "01J...",
  "run_id": "01J...",
  "turn_id": "01J...",
  "iteration_id": "01J...",
  "tool_call_id": "call_...",
  "parent_id": "01J...",
  "type": "tool_call_succeeded",
  "status": "succeeded",
  "actor": "tool",
  "tool": {
    "name": "patch",
    "version": "1.0",
    "arguments_hash": "sha256:...",
    "idempotency_key": "01J...:call_..."
  },
  "checkpoint_id": "01J...",
  "started_at": "2026-09-06T12:00:00Z",
  "finished_at": "2026-09-06T12:00:04Z",
  "result": {
    "summary": "Archivo actualizado",
    "exit_code": 0,
    "artifacts": []
  },
  "previous_event_hash": "sha256:...",
  "content_hash": "sha256:..."
}
```

- El esquema real debe separar metadatos de ejecución, contenido visible y datos sensibles. Tokens, claves, cookies y secretos nunca deben entrar en el registro sin redacción o cifrado. La auditoría debe conservar suficiente información para reproducir y verificar la operación, pero debe aplicar retención y controles de acceso por usuario, sesión y proyecto.
- La interfaz de Telegram debe mostrar el `task_id` o un identificador corto cuando sea útil, y permitir consultar el progreso, cancelar, reanudar, revisar las herramientas utilizadas y solicitar una restauración. La lógica de auditoría debe vivir en el núcleo, de modo que Telegram sea sólo otro canal de presentación.

### 2.13. Contrato versionado, supuestos y validación de cumplimiento

- Cada tarea debe tener un contrato explícito que describa qué resultado final pidió el usuario y qué significa cumplirlo. El contrato debe conservar el texto original, una interpretación estructurada, el alcance, las restricciones, los entregables esperados, los criterios de aceptación, las condiciones de no cumplimiento y los límites que el agente no puede cruzar.
- El contrato debe separar requisitos obligatorios, preferencias, supuestos y preguntas abiertas. Los requisitos explícitos del usuario tienen prioridad; los supuestos inferidos deben identificarse como tales. Comunicar todas las dudas detectadas, incluso pequeñas, según la política confirmada de la sección 2.24; no ocultarlas por considerarlas de bajo impacto.
- Fénix debe guardar todos los supuestos que el agente haya inferido, aunque parezcan pequeños o provisionales. Cada supuesto debe tener `assumption_id`, texto, evidencia o señales de origen, `source_event_ids`, confianza, alcance (`session`, `task`, `project` o `user`), impacto potencial, fecha de creación, versión del contrato, estado (`inferred`, `pending_confirmation`, `confirmed`, `rejected`, `superseded` o `expired`) y, cuando corresponda, fecha de revisión. Los supuestos no deben presentarse como hechos ni mezclarse con requisitos explícitos.
- Antes de ejecutar una acción que necesite resolver una duda, Fénix debe preguntar y dejar pendiente esa parte hasta recibir respuesta. Puede continuar el trabajo independiente; el bajo riesgo no autoriza a resolver silenciosamente una duda mediante un supuesto. Los supuestos deben viajar con el contexto y poder corregirse, invalidarse o sustituirse sin borrar su historial.
- Al comenzar una tarea, Fénix debe transformar el pedido en una lista de condiciones comprobables. Cada condición debe tener un identificador, una descripción, un método de validación, el resultado esperado, la evidencia requerida y un estado (`pending`, `passed`, `failed`, `waived` o `blocked`). Si el pedido es ambiguo y no existe una interpretación segura, el agente debe preguntar antes de ejecutar efectos importantes.
- El contrato debe acompañar a todos los turnos, delegaciones, reintentos y compactaciones mediante `task_id`, `run_id` y una versión del contrato. Si el usuario cambia el objetivo, se debe crear una nueva versión enlazada a la anterior; no se debe sobrescribir el contrato original ni mezclar criterios de aceptación de versiones distintas.

#### Plan de pruebas y evidencia

- Antes de declarar una tarea terminada, el agente debe decidir qué validación corresponde: inspección de archivos, comparación con un esquema, ejecución de tests unitarios o de integración, lint, compilación, prueba manual, consulta a una API, revisión de estado o confirmación del usuario. La validación debe ser proporcional al riesgo y al tipo de entregable.
- Los tests deterministas y los comandos de verificación deben ejecutarse antes de confiar en la afirmación textual del modelo. Para cambios de código, el contrato debe indicar como mínimo los comandos, directorios, variables necesarias, resultado esperado y qué hacer si una prueba falla. Para tareas no ejecutables, debe definirse una evidencia alternativa y quién debe confirmarla.
- Cada validación debe registrarse como un evento relacionado con el criterio de aceptación: comando o procedimiento, versión del entorno, duración, código de salida, salida relevante, artefactos, hash del estado probado y resultado. Las salidas extensas deben seguir la política de retención de herramientas y conservarse como artefactos recuperables.
- Una prueba pasada no demuestra por sí sola que todos los requisitos se cumplieron. El agente debe producir una matriz requisito → evidencia y marcar como `unknown` o `blocked` cualquier condición que no haya podido verificar. No debe presentar como completada una tarea con criterios obligatorios pendientes.
- Si una prueba falla, el error debe volver al bucle de trabajo con su `criterion_id`, sin perder el intento anterior. Después de varios fallos sin cambio de estado, la tarea debe detenerse y marcarse como bloqueada, explicando qué impide cumplir el contrato y qué intervención se necesita.

#### Cierre del contrato

- El cierre debe comprobar los criterios obligatorios, revisar restricciones y verificar que los entregables existan en la ubicación esperada. También debe confirmar que no queden procesos relevantes en ejecución, cambios sin guardar, efectos externos sin registrar ni tareas pendientes ocultas.
- La respuesta final debe indicar qué requisitos se cumplieron, qué evidencia los valida, qué quedó pendiente, qué supuestos se usaron y qué limitaciones permanecen. El texto del modelo nunca debe sustituir a la evidencia registrada.
- El usuario debe poder solicitar una revisión del contrato, ver la matriz de validación, repetir una prueba, aceptar explícitamente una condición no verificada (`waived`) o reabrir una tarea cerrada. Reabrirla debe crear un nuevo `run_id` y conservar la auditoría del cierre anterior.

### 2.14. Subagentes con contexto específico

**Current baseline (0.2.0):** asynchronous delegation, one engine-enforced level,
up to four concurrent children, isolated persisted transcripts, optional scoped
context, three total attempts for retryable failures, complete shared skill metadata
and on-demand instruction loading are in the current requirements. The Tasks view
shows results and textual request IDs; restart recovery remains manual. The points
below describe extensions such as richer contracts, project isolation, checkpoints,
configurable budgets and durable recovery, rather than requirements to add fields
to every current handoff. See `REQUIREMENTS.md`, section 1.15.

**Base actual (0.2.0):** la delegación asíncrona de un nivel, cuatro hijos simultáneos,
contextos persistidos independientes, contexto opcional, tres intentos totales ante
fallos reintentables, catálogo global completo y carga de instrucciones bajo demanda
pasan a los requisitos actuales. Tareas muestra informes y pendientes por ID; los
reinicios no reanudan el trabajo. Las propuestas siguientes amplían contratos,
aislación de proyectos, checkpoints, presupuestos y recuperación; no exigen más
campos en cada delegación actual. Ver `REQUIREMENTS.md`, sección 1.15.

- Fénix debe poder crear subagentes para tareas concretas sin entregarles automáticamente toda la conversación principal. Cada subagente debe recibir un `task_id` propio, `parent_task_id`, `parent_run_id`, objetivo preciso, criterios de aceptación, restricciones, archivos o recursos autorizados y el contexto mínimo necesario para trabajar.
- El contexto debe construirse mediante una lista explícita de *context packages*: objetivo, hechos relevantes, decisiones aplicables, entradas, salidas esperadas, instrucciones del proyecto, dependencias y límites. El subagente no debe inferir que puede modificar otros proyectos, consultar toda la memoria del usuario o ejecutar herramientas fuera de su alcance.
- El subagente debe tener un presupuesto independiente de iteraciones, tiempo, tokens, costo, herramientas y reintentos. El agente padre debe definir si puede crear otros subagentes, qué herramientas puede utilizar, si puede modificar archivos y qué operaciones requieren aprobación. Una tarea secundaria no debe consumir indefinidamente el presupuesto de la tarea principal.
- Las instrucciones generales (`SOUL.md`, reglas de seguridad y políticas del sistema) deben mantenerse, pero el contexto de trabajo debe estar limitado al proyecto o tarea asignada. La memoria específica del proyecto puede compartirse con permisos; la memoria privada de otros proyectos y el historial irrelevante deben quedar aislados.
- Antes de delegar se debe registrar la intención, el objetivo, el contexto entregado, los permisos, el presupuesto y la versión de las instrucciones. El subagente debe registrar sus llamadas de modelo, herramientas, checkpoints, artefactos, errores, bloqueos y consumo con los mismos identificadores de auditoría que el agente principal.
- El resultado que vuelve al padre debe ser un informe estructurado, no sólo texto libre. Debe incluir estado (`succeeded`, `failed`, `blocked` o `unknown`), resumen, criterios verificados, evidencia y artefactos con sus hashes, cambios realizados, supuestos, riesgos, tareas pendientes y recomendaciones. Las salidas extensas deben quedar fuera del contexto y ser recuperables por `event_id` o `artifact_id`.
- El padre debe validar el informe contra el contrato original. No debe aceptar que el subagente declare terminada una tarea sin evidencia ni incorporar automáticamente sus conclusiones a la memoria global. Los cambios de archivos deben pasar por revisión, pruebas y una política de integración o *merge* antes de modificar el proyecto principal.
- Las tareas paralelas deben aislar directorios de trabajo, ramas, sesiones o locks cuando puedan editar los mismos recursos. Si dos subagentes necesitan el mismo archivo, se debe imponer una barrera de coordinación, serializar la edición o crear un conflicto explícito para revisión humana.
- Un subagente que se bloquee debe informar qué necesita y detenerse. El padre puede aportar contexto adicional, ajustar el presupuesto, reasignar la tarea o marcarla como bloqueada, pero no debe ocultar el bloqueo con una respuesta genérica. Si el proceso se interrumpe, su estado debe poder recuperarse mediante la auditoría, los checkpoints y los artefactos persistidos.
- La comunicación entre padre e hijo debe tratarse como eventos auditables. Cada mensaje debe incluir remitente, destinatario, `task_id`, finalidad, datos enviados y fecha. No se deben copiar secretos ni contexto de otras tareas en una delegación; el filtrado debe ocurrir antes de crear el prompt del subagente.
- La planificación y delegación largas deben ser asíncronas y mantener disponible al maestro según la sección 2.24. Una espera interna por una dependencia no debe bloquear el receptor del chat. Debe existir una cola de finalización durable, comprobación técnica de actividad, timeout, cancelación y una política clara para procesos que sobrevivan al reinicio; esto no exige activar el heartbeat conversacional aplazado.

### 2.15. Evolución de proyectos desde una sola conversación

**Implemented baseline / Base incorporada al alcance actual:** the project registry, general/project/task context separation, explicit aliases and reply references, routing before retrieval, scoped lexical memory/history, fixed execution workspaces and one-level delegation now belong to [REQUIREMENTS.md, section 1.16](REQUIREMENTS.md#116-projects-and-general-conversation--proyectos-y-conversación-general). See [the project guide](docs/en/projects.md).

El registro de proyectos, la conversación general, el contexto por proyecto/tarea, los alias y referencias de respuesta, el enrutamiento previo a recuperar información, la memoria e historial léxicos por ámbito, la carpeta fija por ejecución y la delegación de un nivel forman parte del alcance actual. La [guía de proyectos](docs/es/projects.md) explica sus límites. Se conserva una sola conversación; no se exigen chats separados.

La pregunta original se mantiene para evaluar la evolución: ¿cómo gestionar trabajos paralelos desde un solo canal sin mezclar instrucciones, memoria, herramientas, archivos, presupuestos o resultados? La base actual no resuelve por sí sola todas las variantes de lenguaje ni demuestra aislamiento del sistema operativo.

- Evaluar la arquitectura `brain → source` de GBrain para páginas, entidades, índices relacionales y permisos avanzados propios de cada proyecto. Conservar `project_id` como ámbito explícito; cualquier `source_id` adicional deberá mantener esa relación.
- Comparar recuperación léxica con embeddings, grafos y contextos por episodios; medir errores de enrutamiento, cruces entre ámbitos, continuidad, preguntas innecesarias, tareas completadas, tokens, latencia y RAM con conversaciones reales y proyectos similares.
- Añadir prioridades, dependencias durables, presupuestos por proyecto/tarea, permisos por recurso, skills específicas de proyecto y coordinación entre instalaciones cuando exista una política definida. No convertir un enlace entre proyectos en autorización implícita de lectura o escritura.
- Investigar recuperación tras reinicios y migración asistida de historiales previos a proyectos, con procedencia y revisión. La migración actual conserva el historial antiguo como general y no reanuda tareas interrumpidas.
- Ampliar los cambios de contexto y las referencias ambiguas según los fallos medidos. Mantener el contexto general y una sola conversación también en futuras interfaces de voz.

### 2.16. Aprendizaje y ciclo de vida de skills

Las preferencias y la promoción de memoria se rigen por la sección 2.1. Aquí se conserva la evolución de secuencias repetidas hacia capacidades reutilizables.

- Cuando Fénix detecte que una secuencia de trabajo se repite —por ejemplo, buscar información, transformar archivos, ejecutar validaciones y generar un informe— debe proponer la creación de un *skill* predefinido. El skill debe describir objetivo, entradas, pasos, herramientas permitidas, condiciones de éxito, límites, permisos necesarios, salidas esperadas y casos en los que debe detenerse y pedir confirmación.

- La generación automática de un skill debe pasar por un borrador y una validación. Primero se deben extraer los pasos comunes de ejecuciones anteriores; después se debe comprobar que los pasos sean reproducibles, que no dependan accidentalmente de datos concretos y que las herramientas tengan políticas de riesgo compatibles. El skill no debe activarse globalmente sólo porque una tarea se haya repetido una vez.

- Los skills aprendidos deben tener versión, alcance, historial de ejecuciones y relación con las tareas que los originaron. La ejecución debe registrar qué versión se utilizó y permitir volver a una versión anterior, deshabilitar el skill o convertirlo en un skill específico de un proyecto.

- Cada skill debe conservar metadatos operativos explícitos: `skill_id`, nombre, `status` (`draft`, `active`, `disabled`, `deprecated` o `blocked`), `version`, alcance, propietario, `created_from`, tareas de origen, permisos, herramientas, presupuesto, dependencias, esquema de entradas y salidas, criterios de aceptación, `content_hash`, `created_at`, `updated_at`, `last_used_at` y motivo de activación o desactivación. Al cambiar una versión se debe enlazar la anterior mediante `supersedes`; deshabilitarlo no debe borrar su historial ni sus resultados.

- Aplicar la progresión común de detección, acumulación de ejemplos, propuesta, validación, activación limitada y evaluación. Los fallos, correcciones y cambios manuales deben poder desactivar, revisar o retirar una versión; la medición del skill que guarda contexto se detalla en la sección 2.20.

### 2.17. Autoconfiguración conversacional y versiones de configuración

- Fénix debe poder configurarse desde la misma interfaz conversacional, sin exigir que el usuario edite archivos manualmente para las opciones habituales. El usuario debería poder solicitar cambios de identidad, tono, modelo, límites de presupuesto, herramientas habilitadas, proyecto activo, horarios, recordatorios, memoria y políticas de aprobación.
- Cada cambio de configuración debe convertirse en una operación explícita, validada y auditable. El agente debe mostrar qué valor cambiará, su alcance (usuario, sesión, proyecto o instalación), el valor anterior y el nuevo valor, y pedir confirmación cuando afecte permisos, costos, seguridad o efectos externos.
- La configuración debe estar separada por ámbitos y tener precedencia definida: valores predeterminados, instalación, usuario, proyecto, tarea y turno. Una configuración de un proyecto no puede modificar silenciosamente otro proyecto ni alterar el prompt de un turno ya iniciado.
- Antes de aplicar cambios importantes se debe crear una versión o snapshot de configuración. Debe ser posible consultar el historial, comparar versiones, deshacer el último cambio y restaurar una configuración anterior. Los secretos deben guardarse fuera del historial visible y nunca deben repetirse en las respuestas.
- Las órdenes conversacionales deben distinguir entre una propuesta y una aplicación efectiva. Por ejemplo, “usa un modelo más barato en este proyecto” puede generar un plan de cambio; sólo después de la confirmación se actualiza la configuración y se registra el evento correspondiente.

### 2.18. Recordatorios proactivos e intenciones persistentes

- Fénix debe poder crear automáticamente un candidato de recordatorio a partir de la conversación, aunque el usuario no diga explícitamente "recuérdame". Debe detectar fechas, compromisos, tareas pendientes, dependencias, revisiones periódicas y procesos estancados.
- El candidato debe guardar: identificador, acción o texto, tipo de disparador (hora o evento), evidencia y referencia al mensaje de origen, confianza, ámbito, fecha de expiración, cooldown, límite de activaciones, estado y última activación.
- La creación automática debe pasar por deduplicación, validación de fechas y alcance, y un umbral de confianza. Un recordatorio ambiguo puede quedar como candidato revisable; uno confirmado se persiste y se activa sólo cuando llega su hora o aparece su evento.
- Para una hora exacta se debe usar `cron`; para un evento se puede usar una intención persistente con palabras o condiciones de activación. El recordatorio debe poder inspeccionarse, editarse y cancelarse, y cada activación debe quedar registrada.
- La revisión anterior de OpenClaw registró las *standing intents* como memoria prospectiva persistida en SQLite, con coincidencia determinista, expiración, cooldown y límite de activaciones. Normalmente se crean mediante la herramienta de intención; Fénix debe añadir la extracción automática descrita arriba sin perder esos controles. Referencia: [Standing intents de OpenClaw](https://docs.openclaw.ai/concepts/standing-intents).

### 2.19. Voz en tiempo real, administración avanzada y arnés de programación

Estas ampliaciones proceden de la visión del proyecto discutida en la conversación y se sitúan aquí para mantener básico el alcance inicial.

- Conversar por voz en tiempo real desde la interfaz propia, con audio progresivo, transcripción, detección de turnos e interrupciones. Más adelante, evaluar llamadas telefónicas.
- Mantener independiente la conexión de voz del flujo de tareas: cambiar de pantalla o cerrar la llamada no debe borrar ni detener trabajo durable. Interrumpir la reproducción de voz no implica cancelar una tarea.
- Administrar proyectos, tareas, permisos, modelos, presupuestos, memoria, skills, programaciones y auditoría; ofrecer historial, comparación de cambios y restauración por los niveles definidos en la sección 2.12.
- Conservar como propuesta tecnológica de la conversación React + TypeScript + Vite para la interfaz, WebRTC y un cliente de LiveKit para voz, y Tauri para una aplicación instalada posterior. El núcleo permanece en Rust. Validar compatibilidad de audio y RAM con el uso real antes de cerrar esas decisiones.
- Desarrollar el arnés de programación que pueda usar el asistente: navegación de repositorios, herramientas de edición y ejecución, validaciones, integración de resultados y, cuando haga falta, automatización del navegador. Aplicar contratos, aislamiento, presupuestos y supervisión de subagentes.
- Evaluar modelos de bajo costo por éxito de tarea y gasto total, no sólo por precio por token. Terminal-Bench y SWE-bench son objetivos de evaluación a concretar; registrar versiones, entornos y límites. La indexación del código por símbolos o grafo se conecta con la investigación de memoria y vectorización selectiva.

### 2.20. Benchmarks y preguntas de investigación

Las capacidades avanzadas son hipótesis que deben compararse con el alcance actual como referencia. Identificar corpus, consultas, respuestas esperadas, configuración, versiones, semillas y hashes; medir regresiones, costo y calidad bajo el mismo presupuesto.

#### Arquitectura de memoria y siete verbos

- Investigar si una memoria basada en grafos, o una arquitectura híbrida, sería mejor que una memoria lineal basada únicamente en búsqueda vectorial. La comparación debe considerar recuperación de relaciones, precisión, actualización de hechos, procedencia, costo y complejidad operativa.
- Verificar mediante benchmarks si la interfaz de siete verbos aporta una mejora real frente a una interfaz más simple. La evaluación debe medir, como mínimo, recuperación de memoria, precisión, recall, nDCG, cumplimiento de tareas, pérdida de contexto tras compactaciones, latencia, costo, uso de herramientas, detección de bucles y tasa de respuestas sin evidencia. Se deben comparar ablations: sin los verbos de navegación, con cada verbo por separado y con el conjunto completo, usando un corpus y consultas reproducibles.

- La interfaz de siete verbos inspirada en GBrain queda como hipótesis de diseño. Conservar la distinción entre navegación, recuperación y síntesis, y el énfasis solicitado en `tilt` para escoger una vista de la estructura de memoria. Comparar `recall` barato con `think` para síntesis y verificar el alcance exacto del inventario de verbos contra una versión identificada.
- Evaluar `inspect` como consulta de historia y previsualización; la operación que cambia el estado es `restore` o `rollback`. Medir también la capacidad de explicar y reconstruir el estado anterior.

#### Guardado automático y compactación

- Mejorar con benchmarks reproducibles el skill de guardado de contexto y cada versión de sus reglas de saliencia, confianza, deduplicación, ámbito e inyección. Registrar corpus, configuración, resultados y hashes del skill.
- Incluir preferencias repetidas, decisiones, compromisos, datos temporales, instrucciones aisladas, correcciones, ambigüedades, salidas de herramientas y sesiones que atraviesan compactaciones.
- Comparar con una versión base y mediante ablaciones: sin memoria automática, guardar sólo hechos salientes frente a todas las afirmaciones, inyección por mensaje frente a fronteras de contexto, resumen abstractivo frente a extractivo, tasas de compresión y recuperación del segmento original.
- Medir éxito de tarea, precisión y recall de guardado y recuperación, falsos recuerdos, omisiones críticas, fidelidad, contradicciones, contaminación o filtraciones entre proyectos, recuperación de bloqueos, duplicados, reconstrucción de evidencia, tokens, latencia y costo.
- Promocionar cambios sólo si mejoran la utilidad y el cumplimiento sin empeorar de forma inaceptable los falsos guardados, filtraciones o costos. Mantener los resultados para detectar regresiones.

#### Recuperación, aislamiento, curación y costo

- Usar las pruebas por modo, variantes y familia de consultas definidas en la sección 2.3. Una mejora promedio no debe ocultar regresiones en consultas exactas, negativas, temporales o exhaustivas.
- Comparar contexto único frente a contexto por proyecto con una capa global mínima; medir enrutamiento, contaminación o filtraciones, tokens, continuidad y calidad.
- Contrastar las políticas de hashes y embeddings de la sección 2.5, las tarjetas de la sección 2.7, la curación de la sección 2.8 y la retención de la sección 2.9.
- Mantener abiertas las preguntas sobre qué guardar de cada entidad, cómo tratar contradicciones, qué umbrales justifican escritura automática y cuándo es necesaria confirmación.
- Medir consumo de RAM en reposo, durante una conversación de voz y con tareas o agentes concurrentes. Registrar contexto retenido, procesos, herramientas pesadas y cachés; no atribuir toda la memoria al lenguaje o framework.

### 2.21. Referencias de arXiv y evaluación de Grok Bot

Se conservan las referencias y descripciones anotadas anteriormente. Antes de usarlas como evidencia de una decisión, verificar el documento, su versión, metodología y resultados; este trabajo de organización no reproduce esos experimentos.

#### Bibliografía inicial de memoria y contexto

- [MemGPT: Towards LLMs as Operating Systems](https://arxiv.org/abs/2310.08560): contexto virtual jerárquico, movimiento entre niveles de memoria e interrupciones para gestionar una ventana limitada.
- [RECOMP: Improving Retrieval-Augmented LMs with Compression and Selective Augmentation](https://arxiv.org/abs/2310.04408): compresión extractiva o abstractiva de documentos recuperados y posibilidad de no inyectar nada cuando no aporta información.
- [LLMLingua](https://arxiv.org/abs/2310.05736) y [LLMLingua-2](https://arxiv.org/abs/2403.12968): compresión con presupuesto y métodos orientados a preservar la fidelidad semántica.
- [A-MEM: Agentic Memory for LLM Agents](https://arxiv.org/abs/2502.12110): notas estructuradas, enlaces dinámicos y evolución de las relaciones entre recuerdos.
- [Agentic Memory (AgeMem)](https://arxiv.org/abs/2601.01885): operaciones unificadas de memoria a corto y largo plazo para crear, recuperar, actualizar, resumir o descartar información.
- [Useful Memories Become Faulty When Continuously Updated by LLMs](https://arxiv.org/abs/2605.12978): evidencia de que la consolidación continua puede degradar la memoria; recomienda conservar episodios crudos y controlar explícitamente la consolidación.
- [Remember When It Matters](https://arxiv.org/abs/2607.08716): intervención selectiva de memoria en tareas largas, en lugar de inyectar recuerdos en cada turno.

#### Funcionalidades de Grok Bot por investigar

- Se debe investigar y documentar qué funcionalidades ofrece actualmente Grok Bot para evaluar cuáles podrían mejorar Fénix. La investigación debe usar fuentes oficiales y registrar la fecha de consulta, la versión o modalidad del producto y las restricciones conocidas.
- Comparar, como mínimo: conversación multimodal, búsqueda y uso de información reciente, análisis de imágenes y documentos, generación o edición de imágenes, manejo de archivos, ejecución de código, automatizaciones, memoria y personalización, creación de asistentes o agentes, integración con herramientas, control de privacidad, límites de uso y disponibilidad por plan o región.
- Para cada funcionalidad se debe indicar si es nativa de Grok, si depende de una integración externa, qué problema resuelve, qué riesgos introduce y qué componente de Fénix tendría que implementarla. No se debe copiar una función sin comprobar su comportamiento, licencia, costos y requisitos de seguridad.
- El resultado debe convertirse en una matriz de priorización para decidir qué incorporar al roadmap de Fénix, qué dejar como experimento y qué descartar por complejidad, dependencia de una plataforma externa o falta de utilidad para el núcleo del asistente.

### 2.22. Web search and browsing / Búsqueda web y navegación

Independent search and public HTTP reading are implemented in `fenix-web`: a
background crawler, SQLite FTS5 corpus and `web_search`/`web_read` tools. The user
selected an independent index without external search providers. Configurable
seeds start discovery across domains; coverage is bounded and starts empty.
See [the English guide](docs/en/web-search.md) and [requirements section 1.18](REQUIREMENTS.md#118-independent-web-search--búsqueda-web-propia).

- Measure crawl coverage, relevance, freshness and RSS on representative corpora;
  improve frontier rotation and source diversity beyond the initial fixed caps.
- Keep search discovery, HTTP reading and interactive browsing separate. Browser
  rendering does not supply a worldwide search index.
- Add browser interaction when tasks require JavaScript, authenticated sessions,
  forms or purchases, with controlled state and verification before commitment.
- Add rendered-site testing, screenshots and viewport/device-size controls for
  responsive design. Measure browser memory and bound browser concurrency.
- Do not introduce paid search APIs or metasearch upstreams as an implicit fallback;
  that would change the selected independence requirement.

La búsqueda propia y la lectura HTTP pública ya están incorporadas en
`fenix-web`, con rastreador en segundo plano, SQLite FTS5 y herramientas para el
agente. Se eligió un índice independiente sin proveedores externos de búsqueda.
Las semillas inician el descubrimiento entre dominios; la cobertura es acotada y
parte de cero. Véase [la guía en español](docs/es/web-search.md).

- Medir cobertura, relevancia, frescura y RAM con corpus representativos; mejorar
  la rotación de URLs y la diversidad de fuentes al alcanzar los límites iniciales.
- Mantener separadas búsqueda, lectura HTTP y navegación interactiva.
- Incorporar interacción con JavaScript, sesiones y formularios cuando se active
  esa fase, con comprobaciones antes de acciones como confirmar una reserva.
- Añadir pruebas de sitios renderizados, capturas y cambios de tamaño de pantalla
  para diseño responsivo, midiendo consumo y limitando concurrencia.
- No introducir APIs de búsqueda ni metabuscadores como alternativa automática;
  cambiarían el requisito de independencia elegido.

### 2.23. Heartbeat — Future versions / Versiones futuras

Heartbeat is explicitly excluded from the initial release. References to heartbeat elsewhere in this roadmap describe future capabilities, not a dependency of initial cron or reminders.

- Periodically review an explicit checklist, for example `HEARTBEAT.md`, and known task states. Surface only items needing attention, such as relevant blocks or pending approvals.
- Configure cadence, active hours, and budgets. Avoid overlapping reviews or interference with an active conversation task.
- Suppress duplicate alerts and model calls when there is no configured work. Resolve simple state checks in code and use the model only when interpretation is useful.
- Preserve this distinction: cron runs scheduled actions, assistant heartbeat reviews pending matters, and worker liveness checks detect whether a process is alive.
- Do not treat heartbeat as authorization to execute arbitrary actions or resume interrupted work. Combine it with future recovery only under an explicit recovery policy.

El heartbeat queda expresamente fuera de la entrega inicial. Sus referencias en otras secciones del roadmap describen capacidades futuras; cron y recordatorios iniciales no dependen de él.

- Revisar periódicamente una lista explícita, por ejemplo `HEARTBEAT.md`, y los estados de tareas conocidas. Comunicar sólo asuntos que necesiten atención, como bloqueos o aprobaciones pendientes relevantes.
- Configurar cadencia, horario activo y presupuesto. Evitar revisiones solapadas o interferencias con una tarea activa de la conversación.
- Evitar avisos repetidos y llamadas al modelo sin trabajo configurado. Resolver comprobaciones simples con código y usar el modelo cuando haga falta interpretar información.
- Distinguir cron para acciones programadas, heartbeat del asistente para revisar pendientes y latidos técnicos para comprobar si un proceso sigue vivo.
- El heartbeat no autoriza acciones arbitrarias ni reanudar tareas interrumpidas. Integrarlo con recuperación futura sólo cuando exista una política explícita.

Reference / Referencia: [OpenClaw heartbeat](https://docs.openclaw.ai/gateway/heartbeat).

### 2.24. Editable asynchronous planning and pub/sub / Planificación asíncrona editable y pub/sub

Implemented in the current runtime; retained here as the design rationale and acceptance criteria. See [English implementation details](docs/en/planning.md) and `REQUIREMENTS.md` section 1.17.

- **Surface every detected doubt.** Ask the owner textually as soon as a doubt is identified, even if it seems minor. Do not suppress questions based on impact or silently choose an interpretation. Consult existing explicit answers to avoid asking resolved questions again. Track each open question by ID and task; it is neither an approval nor resolved merely because time passes. Hold work that depends on its answer while continuing independent work and conversation.
- **Use pub/sub for asynchronous communication.** Decouple message intake, planning, workers and channel delivery. The master must remain able to handle messages while a planner is running, a worker is executing or a question is awaiting an answer. Publishing an event must not hold the chat open until its subscriber finishes. Maintain serialized or version-checked updates to shared task state despite concurrent processing.
- **Deliver to the correct channel.** Record `event_id`, `session_id`, optional `project_id`, `task_id`, `plan_id`, `plan_version`, correlation/reply IDs and the delivery destination. Preserve originating web/Telegram channel and conversation references; changing foreground project must not redirect a pending question or completion. A reply from another linked channel can resolve the same request without creating a separate conversation. Change delivery destination only on explicit instruction; publish confirmed questions, progress and completion through channel subscribers.
- **Accept edits at any time.** While drafting or executing a plan, the owner can add, remove, reorder or change pending work, or request replanning, using normal text. Route these as control messages without waiting for the ordinary task queue. Associate the edit with the correct task; ask immediately if that association or the requested change is unclear. Acknowledge receipt promptly, and distinguish a received change from one already applied.
- **Version and reconcile plan changes.** Store the instruction, previous/new plan version, affected nodes and reason for the revision. Recompute dependencies and notify affected workers while preserving still-valid progress. Check the active version before dispatching new work or applying worker results; reconcile late reports instead of letting them overwrite a newer plan. Superseded work must not silently resume after an old clarification is answered.
- **Handle actions already in flight.** Stop dispatching superseded work and request cooperative cancellation of affected actions where supported. Report any action that cannot stop immediately. Editing the plan does not undo completed effects: preserve their evidence and separately handle any needed correction or compensation. Never claim a change is effective before the execution state supports it.
- **Proposed delivery mechanism.** Persist events and outbound messages before dispatch, track acknowledgements per subscriber/destination, and retry only unacknowledged deliveries with idempotent consumption and bounded backoff. Reconnect using a cursor and retain undelivered questions/results. Do not equate an in-memory broadcast with durable delivery or promise exactly-once external messages when transport acknowledgement is uncertain. The implementation uses SQLite events/outbox, cursor-based subscribers and Tokio wakeups.
- **Acceptance scenarios.** Receive a question while planning; answer it while independent work continues; edit a plan halfway through execution; revise it while an earlier model call is still running; reconcile simultaneous edits and late results; reconnect without losing pending messages; preserve the channel and project after topic changes; surface minor doubts without repeatedly asking questions already answered. Retain one visible conversation, one delegation level and on-demand skill loading.

Implementado en el runtime actual; se conserva aquí como criterio de diseño. Véanse [los detalles en español](docs/es/planning.md) y `REQUIREMENTS.md` 1.17. Sustituye la regla anterior que hacía esperar las correcciones de tareas activas detrás del trabajo ordinario.

- **Comunicar toda duda detectada.** Preguntar por texto al identificarla, aunque sea pequeña; no ocultarla por su impacto ni escoger una interpretación silenciosamente. Consultar respuestas explícitas previas para no repetir preguntas resueltas. Vincular cada duda a un ID y tarea; no equivale a aprobación ni se resuelve por esperar. Dejar pendiente el trabajo que necesite esa respuesta y continuar el independiente y la conversación.
- **Mensajería pub/sub asíncrona.** Desacoplar recepción, planificación, trabajadores y entrega al canal. El maestro debe atender mensajes mientras se planifica, ejecuta o espera una respuesta. Publicar un evento no debe bloquear el chat hasta que termine el suscriptor. Serializar o comprobar versiones al actualizar el estado compartido.
- **Canal de entrega correcto.** Conservar IDs de evento, sesión, proyecto cuando aplique, tarea, plan, versión, correlación y respuesta, además del destino web/Telegram y sus referencias. Cambiar de tema no redirige preguntas ni resultados pendientes. Una respuesta desde otro canal vinculado puede resolver la misma solicitud sin abrir otra conversación. Cambiar el destino de entrega sólo por instrucción explícita; los suscriptores entregan dudas, progreso y finalización.
- **Editar en cualquier momento.** Permitir añadir, eliminar, reordenar o cambiar trabajo pendiente y pedir replanificación por texto, incluso mientras se genera el plan o a mitad de ejecución. Estos mensajes de control no esperan la cola ordinaria. Preguntar si no está claro a qué tarea se refieren o qué cambio solicitan. Acusar recibo y distinguirlo de la aplicación efectiva.
- **Versionar y conciliar.** Registrar instrucción, versiones anterior/nueva, nodos afectados y motivo. Recalcular dependencias, avisar a los trabajadores y conservar avances vigentes. Comprobar versión antes de despachar trabajo o incorporar resultados; conciliar informes tardíos sin sobrescribir el plan nuevo. Responder a una duda antigua no debe reactivar trabajo sustituido.
- **Acciones en curso.** No despachar pasos sustituidos; solicitar cancelación cooperativa cuando exista y comunicar lo que no pueda detenerse inmediatamente. Editar el plan no deshace efectos ya realizados: conservar evidencia y gestionar correcciones o compensaciones aparte. No declarar un cambio aplicado antes de que el estado de ejecución lo permita.
- **Propuesta de entrega durable.** Guardar eventos y mensajes salientes antes de enviarlos, registrar acuses por suscriptor/destino y reintentar entregas pendientes con consumo idempotente y espera acotada. Usar cursor al reconectar y conservar dudas/resultados sin entregar. Un broadcast en memoria no garantiza durabilidad; no prometer entrega externa exactamente una vez ante acuses inciertos. La implementación utiliza eventos y outbox de SQLite, suscriptores con cursor y avisos de Tokio.
- **Casos de aceptación.** Preguntar durante la planificación; responder mientras sigue trabajo independiente; editar a mitad de ejecución o con una llamada al modelo pendiente; conciliar cambios simultáneos e informes tardíos; reconectar sin perder pendientes; preservar canal/proyecto al cambiar de tema; comunicar dudas pequeñas sin repetir las ya resueltas. Conservar una conversación visible, un nivel de delegación y skills bajo demanda.

### Evaluation of adaptive plan review / Evaluación de revisión adaptativa

- The initial review workflow is implemented in [plan reviews](docs/en/plan-reviews.md).
  Evaluate it against no additional review, periodic review and specialist panels on
  coding, office and everyday tasks. Measure successful outcomes, invented requirements,
  harmful corrections, total tokens/time, reviewer overhead and user question burden.
  Calibrate the current deterministic triggers before adding learned routing.
- El flujo inicial está implementado en [revisión de planes](docs/es/plan-reviews.md).
  Compararlo con ausencia de revisión adicional, revisión periódica y paneles de
  especialistas en tareas de programación, oficina y cotidianas. Medir resultados,
  requisitos inventados, correcciones perjudiciales, tokens, tiempo y preguntas al usuario.
  Calibrar los disparadores actuales antes de añadir selección aprendida.

### Streaming across providers / Streaming entre proveedores

- Extend the implemented OpenAI Responses preview path to Chat Completions, Anthropic, Gemini/Vertex and Bedrock, preserving each protocol's tool arguments, terminal events, usage and signed reasoning metadata.
- Ampliar el streaming implementado de OpenAI Responses a Chat Completions, Anthropic, Gemini/Vertex y Bedrock, conservando argumentos, eventos terminales, consumo y metadatos firmados de cada protocolo.
- Additional model-generated task verification contracts remain deferred; ordinary tool error handling stays in the runtime.
- Los contratos adicionales de verificación generados por el modelo quedan aplazados; el manejo normal de errores permanece en el motor.


## Coding harness status / Estado del arnés de programación

Regex/glob search, exact-context multi-file patches, repository AGENTS.md discovery,
configurable LSP diagnostics and active-task context compaction are now implemented;
see [the current coding contract](docs/en/coding-harness.md). Future work remains:
repository graphs, automatic server installation, filesystem rollback, crash resume,
model-specific tokenizers and measured context-budget tuning. Context checkpoints
preserve originals but do not restore filesystem effects.

Ya están implementados búsqueda regex/glob, parches de contexto exacto, carga de
AGENTS.md, LSP configurable y compactación activa. Grafos, instalación automática
de servidores, restauración de archivos y reanudación tras reinicios siguen como
trabajo futuro. Ver [el contrato actual](docs/es/coding-harness.md).

## Execution harness update / Actualización del motor de ejecución

Implemented: interactive PTY/stdin, explicitly persistent services, cursor-based redacted logs, configurable execution limits, same-context subagent follow-ups and deterministic on-demand tool discovery. See [English](docs/en/execution-harness.md).

Implementado: PTY/entrada interactiva, servicios persistentes explícitos, registros redactados con cursor, límites configurables, seguimientos de subagentes con el mismo contexto y descubrimiento determinista de herramientas bajo demanda. Ver [Español](docs/es/execution-harness.md).

Automatic crash recovery, isolated parallel worktrees, MCP and programmatic tool composition remain separate future work. La recuperación automática tras reinicios, worktrees paralelos aislados, MCP y composición programática de herramientas siguen siendo trabajo futuro separado.


### Local browser implementation update / Actualización del navegador local

The baseline local Chromium controller is implemented in Rust, with nine agent tools, an owner browser panel, persistent profiles, a separate encrypted password vault, responsive viewports and a soft RSS budget. See [English](docs/en/browser.md) / [Español](docs/es/browser.md). Future browser work includes model vision, comprehensive cross-process frame support, cross-engine/physical-device testing, hard OS memory ceilings and broader site compatibility.

La base de navegación local ya está implementada. Quedan como ampliaciones la visión del modelo, la cobertura completa de frames entre procesos, pruebas en otros motores y dispositivos, límites estrictos de memoria del sistema operativo y compatibilidad ampliada con sitios.

### Harness experiments / Experimentos del arnés

Curated skills, actionable discovery and read-only execution are implemented; evaluate quality with paired native Terminal-Bench cohorts before claiming gains. Keep experimental changes in a separate branch and review their evidence. Semantic skill retrieval, automatic skill promotion and broader sandbox compatibility remain future work contingent on observed needs and regression results.

Las mejoras de skills y ejecución requieren comparación experimental para afirmar avances. La búsqueda semántica de skills, su promoción automática y mayor compatibilidad del entorno restringido quedan pendientes de evidencia y validación.
