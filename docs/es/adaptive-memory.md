# Memoria adaptable

Fénix conserva recuerdos y conocimiento en SQLite, con migración automática al esquema 13. Los IDs y ámbitos anteriores se conservan. El Markdown exportado es una referencia de hasta 200 recuerdos activos, no otra fuente de verdad. El historial de versiones comienza con la migración: no recupera sobrescrituras anteriores.

## Qué guarda

Cada recuerdo incluye contenido atómico, tipo, estado, procedencia, proyecto/tarea, entidad opcional y vigencia. Los tipos son hecho, preferencia, decisión, restricción, compromiso y supuesto. Los candidatos e inferencias no entran como hechos confirmados al contexto. Las correcciones explícitas pueden sustituir recuerdos conservando su historial.

Retirar un recuerdo bloquea su recaptura exacta en ese ámbito. No equivale a borrar conversaciones, revisiones o backups; el historial puede consultarse deliberadamente. La semejanza entre textos no demuestra contradicción ni autoriza a fusionarlos.

## Entidades y relaciones por proyecto

Las entidades tienen nombre, alias y tipo extensible. Cada proyecto define esquemas de relación versionados, con tipos de origen y destino. Un investigador puede usar artículos, hipótesis y experimentos; un programador, módulos e incidencias. La estructura se adapta mediante las herramientas y el skill del agente, sin elegir una profesión al registrarse ni inferirla como cierta.

Las relaciones confirmadas requieren un esquema activo y evidencia vigente del mismo ámbito. Al modificar un esquema se deben revalidar sus relaciones. El recorrido usa SQLite y se limita a tres saltos; devuelve rutas y fuentes. No requiere un modelo ni embeddings y no indexa todo el repositorio de código.

## Recuperación y contexto

`memory_search` busca por palabras, títulos e ID, con filtros temporales opcionales `since` y `as_of`. La consulta histórica usa las revisiones registradas en aquella fecha. Los filtros de ámbito y vigencia se aplican antes del límite de resultados.

`memory_context` permite consultar tarjetas, vecinos, candidatos, pares para revisión, historial, cambios incrementales y paquetes de contexto. `knowledge_manage` administra entidades, esquemas y relaciones. `memory_manage` conserva las acciones de guardar, corregir y retirar, ahora con metadatos.

El motor prepara contexto para el principal y los subagentes y lo recupera después de compactar. Prioriza restricciones, preferencias, decisiones y recuerdos fijados, luego coincidencias de consulta y tarjetas. El presupuesto predeterminado es de 3.000 tokens estimados mediante bytes/3; se informa cuántas entradas no caben y se conservan sus originales. Es un límite de contexto, no de la tarea.

Los cambios de hechos, entidades, relaciones y esquemas tienen cursor secuencial. Solo se avanza después de entregarlos; si falla la entrega se repite el cursor. Esto no activa un heartbeat ni crea recordatorios.

## Automatización y costos

La captura automática está activa por defecto y puede desactivarse en la web. Un filtro determinista selecciona declaraciones potencialmente útiles; el modelo configurado extrae hasta seis candidatos con citas exactas. La cola es durable, independiente del turno y permite tres intentos. Repetir la extracción de un mensaje no cuenta como nueva evidencia.

Una preferencia con el mismo contenido normalizado puede confirmarse tras tres mensajes fuente distintos. Otras formulaciones no se cuentan automáticamente como repeticiones. Los demás candidatos requieren revisión. El filtro y el modelo pueden omitir información: siguen disponibles los guardados explícitos.

Los checkpoints conservan los originales antes del procesamiento. Se reutilizan mensajes originales del usuario para extraer memoria; no se atribuyen al usuario resúmenes del modelo o salidas de herramientas. Antes de compactar conversación también se encolan fuentes de tareas anteriores. No se realiza extracción semántica indiscriminada de todas las salidas.

El mantenimiento revisa caducidad y propone pares con solapamiento léxico. Permite clasificarlos como complementarios, duplicados, conflictivos o descartados, sin fusionar ni borrar automáticamente. Las tareas de memoria pendientes se recuperan al reiniciar; las acciones interrumpidas del agente no se reanudan por ello.

## Embeddings opcionales

Permanecen desactivados hasta configurar endpoint y modelo compatibles con `/embeddings` mediante POST `/api/memory/settings`. Se acepta HTTPS o HTTP local. Las claves no se devuelven y están asociadas al endpoint.

Se indexan hasta ocho recuerdos confirmados por ciclo. Cambiar contenido o modelo invalida el vector previo. Con `semantic:true`, la búsqueda combina rankings léxicos y semánticos mediante RRF; si el servicio falla, devuelve resultados léxicos e informa la degradación. El índice inicial puede estar incompleto. El contexto habitual no hace llamadas de embeddings. Las consultas históricas son léxicas; no hay reranker obligatorio ni segunda llamada para sintetizar.

## Interfaz y validación

La web permite revisar tipo, estado, citas, historial, caducidad y tarjetas, y activar/desactivar captura. La conversación y POST `/api/memory/knowledge` permiten editar la estructura del conocimiento. Los detalles del contrato y configuración están también en [la documentación en inglés](../en/adaptive-memory.md).

Las pruebas cubren aislamiento, migración, retiro, promoción por fuentes distintas, revisiones temporales, grafos, colas, evidencia, contexto y proveedores simulados. No sustituyen un benchmark longitudinal con conversaciones reales.

## Recuperación con evidencia y revisión selectiva

`memory_search` devuelve ahora `{results, diagnostics, warnings, reviews}`. Cada resultado incluye el recuerdo, las ramas de recuperación, coincidencias, rutas de relaciones y señales de vigencia. La lista administrativa `/api/memory` conserva su formato.

La búsqueda combina texto, embeddings opcionales y relaciones con anclas de nombre/alias explícito. Recorre dos saltos por defecto (máximo tres), hasta cuatro anclas y 40 aristas por ancla; informa los recortes y respeta el ámbito. `graph:false` desactiva las relaciones. Las consultas históricas no mezclan el grafo ni los vectores actuales. ID/contenido exactos permiten señalar que un registro existe; un título o vector similar nunca basta para fusionar hechos.

`recency:auto|on|off` ajusta relevancia, no veracidad. En consultas de actualidad, compromisos, supuestos y hechos usan vidas medias iniciales de 14, 30 y 90 días, con un aumento máximo del 15 %. Preferencias, restricciones y decisiones no decaen. La edad parte de `valid_from` o la creación, no de una actualización de mantenimiento. Los recuerdos antiguos se señalan para revisar; no se borran por antigüedad.

Las relaciones quedan ligadas a la revisión exacta del recuerdo que las respalda. Si cambia, se ocultan hasta su revalidación explícita. La migración 13 establece una revisión inicial para las relaciones existentes; no demuestra retrospectivamente que fueran correctas.

### Contradicciones y preguntas

La detección incremental propone pares por coincidencias léxicas, título, tipo y entidad en el mismo ámbito. Puede perder paráfrasis o afirmaciones muy cortas. Intervalos explícitos que no se superponen generan una propuesta de cambio temporal sin usar el modelo. El resto puede pasar a revisión automática.

La opción `automatic_review`, activada por defecto y configurable en la página de memoria, revisa como máximo un par por ciclo con el modelo ya configurado. Hay hasta dos intentos por pareja de revisiones, una reserva de 60 segundos, un timeout de 45 segundos y hasta 1.400 tokens de salida. Solo se envían los dos hechos y extractos acotados de sus fuentes. Esta revisión consume tokens; la búsqueda normal y el contexto no añaden llamadas al modelo.

El resultado puede ser duplicado, complementario, contradictorio, cambio temporal o ambiguo. Debe citar fragmentos exactos de ambos hechos y proponer una pregunta cuando sea ambiguo. Nunca sobrescribe ni confirma hechos. Las propuestas relevantes aparecen en búsqueda/contexto para que el agente contraste las fuentes y pregunte al usuario. Cambiar un hecho invalida su evaluación anterior. Los intentos persisten tras reiniciar.

`memory_context reviews` permite inspeccionar ambos hechos y su propuesta; `knowledge_manage review_resolve` registra la decisión con `first_revision` y `second_revision` del par revisado, para rechazar decisiones sobre versiones anteriores. Esto es independiente de corregir o retirar un recuerdo. La consulta devuelve hasta 100 pares. Las etiquetas no garantizan la verdad.

### Mantenimiento y evaluación

`memory_context health` informa, por ámbito, índices léxicos incompletos, embeddings pendientes, entidades huérfanas, relaciones con evidencia/esquema desactualizados, revisiones agotadas y recuerdos que conviene revalidar. La muestra temporal cubre hasta 500 registros y devuelve hasta 50 IDs, indicando su cobertura. El trabajador repara hasta 50 entradas del índice por ciclo y alterna los ámbitos que diagnostica. No elimina recuerdos, relaciones ni historial.

`cargo run -p fenix-storage --example memory_benchmark` compara ambas búsquedas con 26 consultas de desarrollo, en español e inglés, y una base temporal aislada. Mide precisión, cobertura, posición, filtraciones de ámbito, latencia y tokens estimados del resultado. No utiliza credenciales ni hace llamadas a modelos; no evalúa la calidad del revisor ni demuestra el acierto en tareas reales. Las rutas y explicaciones pueden aumentar el contexto, aunque mejore la recuperación.

[Evaluación de desarrollo registrada](../benchmarks/memory-retrieval-2026-09-21.json): cobertura@5 92.3% → 100.0%, nDCG@5 0.710 → 0.769, sin resultados prohibidos. El tamaño estimado de los resultados aumentó de 158 a 393 tokens de media. La latencia corresponde a una sola pasada local, primero la base léxica; no demuestra mayor velocidad. Es un conjunto de desarrollo, no una validación independiente.
