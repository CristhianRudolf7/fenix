# Memoria adaptable

Fénix conserva recuerdos y conocimiento en SQLite, con migración automática al esquema 7. Los IDs y ámbitos anteriores se conservan. El Markdown exportado es una referencia de hasta 200 recuerdos activos, no otra fuente de verdad. El historial de versiones comienza con la migración: no recupera sobrescrituras anteriores.

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
