# Skills de procedimiento y recuperación basada en evidencia

[English](../en/harness-recovery.md)

Fénix incluye diecisiete skills manuales: las cinco anteriores y nueve familias para repositorios, reparación con pruebas, dependencias, servicios, paquetes, SQL, procesamiento de texto, Git e implementación de software, además de tres para Word, hojas de cálculo y presentaciones. Son procedimientos generales, no respuestas del benchmark. La instalación conserva las modificaciones del usuario.

`skill_search(query, limit=3)` busca por nombre, descripción y contenido completo, dando más peso a los dos primeros. Devuelve metadatos. `skill_load` carga el contenido y hasta ocho herramientas autorizadas declaradas en el campo opcional `required_tools`. Los skills anteriores siguen funcionando. Cargar un skill no concede permisos.

Si el modelo solicita una herramienta autorizada cuyo esquema todavía no se expuso, recibe el esquema y `executed=false`; debe enviar otra llamada válida. Los errores de argumentos muestran campos necesarios, recibidos y un ejemplo de formato.

## Procesos

`command_run` admite `read_paths` para entradas existentes. Un proceso Rust separado aplica Landlock ABI 3 y seccomp antes de ejecutar el comando: lectura de entradas y bibliotecas estándar, sin escritura, red ni modificación de metadatos. Si Linux no puede aplicar las restricciones, falla explícitamente; nunca cambia silenciosamente a ejecución normal. Algunos programas necesitan cachés o temporales y no son compatibles con este modo.

Los archivos de entrada permanecen registrados mientras vive el proceso. Las escrituras y ediciones independientes pueden continuar; los conflictos esperan. Los parches múltiples y efectos desconocidos siguen siendo conservadores. Leer directorios amplios reduce la concurrencia. Son lecturas en vivo, no snapshots, y otros procesos ajenos a Fénix quedan fuera de este control. El modo restringido no admite TTY ni servicios persistentes. Los servicios normales conservan `persistent=true` explícito y sus resultados describen cuándo se eliminan sus descendientes.

## Contexto y recuperación

Antes de compactar se guarda el original redactado. Primero se intenta reducir salidas antiguas exitosas mediante referencias recuperables, conservando parejas de llamadas/resultados, mensajes recientes, roles y contrato. Esta ruta no llama al modelo para resumir. Si no cabe, se conserva la ruta de resumen y respaldo existente. Los resúmenes generados tienen rol de asistente y no constituyen instrucciones del usuario ni evidencia de ejecución. La pérdida semántica no queda eliminada por completo.

La repetición sin progreso ofrece diagnóstico limitado: procesos bloqueantes, esquemas, checkpoints o pruebas concretas. Se mantienen tres recuperaciones por causa y ocho totales. Las respuestas truncadas nunca se ejecutan. Se distingue heurísticamente entre argumentos grandes y agotamiento sin un payload grande; no es una medición directa de tokens de razonamiento.

Se reutilizan planificación editable, subagentes y revisores existentes. Las instrucciones reservan revisión independiente para incertidumbre material que las comprobaciones locales no resuelvan, sin cambiar modelos ni habilitar delegación anidada.

## Resumen operativo e intentos fallidos

Antes de consultar al modelo, el motor prepara un resumen acotado desde el plan y
contrato existentes: versión, hitos activos, restricciones, progreso, bloqueos,
cambios del usuario y notas opcionales sobre el siguiente paso y las decisiones.
Las tareas simples sin plan, notas operativas ni fallos no reciben este bloque.
Se inserta una sola vez en cada solicitud, sin acumular copias en el historial.
Permanece idéntico si el estado no cambia y se reconstruye después de compactar.
Es información de referencia con rol de asistente, no autorización del usuario.

El contenido estructurado ocupa como máximo 12 KB y prioriza cuatro hitos,
tres fallos propios y hasta dos registros históricos permitidos. La compactación
reserva espacio para él. Consume tokens normales de entrada cuando está presente,
pero no necesita otra llamada al modelo, embeddings ni un resumidor en segundo
plano. No se ha medido un ahorro de facturación.

Los errores de herramientas, comandos con salida distinta de cero y resultados
fallidos o agotados se registran en el contrato de la tarea. Se guardan hasta ocho
acciones distintas, con extractos redactados, llamadas de origen y número de
observaciones. Las repeticiones se agrupan; los originales siguen en el registro.
Un proceso en marcha, un esquema cargado o una aprobación denegada no se confunden
con una ejecución fallida. Un éxito posterior de la herramienta tampoco demuestra
que la tarea completa esté resuelta.

Con `task_contract annotate`, el agente puede guardar `next_step`, `decision` y
`failed_approach`, una fuente y la condición opcional `retry_when`. Las causas que
proponga siguen siendo hipótesis, no hechos certificados. Las notas sobre el
siguiente paso y decisiones dejan de mostrarse cuando cambia la versión del plan
o las instrucciones/respuestas del usuario. Sus originales siguen disponibles.
Solo se mantiene una nota actual de siguiente paso; no se exige anotar cada acción.

Un nuevo trabajador puede recuperar fallos de sus predecesores o generaciones
anteriores dentro del ámbito ya autorizado, nunca de ramas independientes u otros
proyectos. Se reutilizan SQLite, los contratos, los registros y los checkpoints;
no hay un plan paralelo ni reanudación automática de efectos tras reinicios.

## Evaluación

`experiment.py`, en `scripts/benchmarks/harbor`, ejecuta lotes de cinco tareas con el evaluador nativo, hashes del binario/configuración, calificación oficial y costes del proxy. Requiere proxy activo y rutas explícitas del dataset, binario, configuración y Harbor. `--summarize --root RUTA` actualiza resultados sin lanzar tareas.

Se comparan binarios congelados de main y candidata con la misma configuración. Las tareas inspeccionadas sirven para desarrollo; la confirmación utiliza diez nuevas en dos lotes, con dos intentos por versión. Se separan fallos de infraestructura y se mantienen desconocidos los costes ausentes. La rama experimental se publica para revisión sin integración automática.

Los trabajadores liberan el bloqueo de lectura del modelo antes de esperar configuración o inferencia. Esto evita que un trabajador inactivo bloquee un cambio de modelo. La web descarta respuestas de vistas anteriores después de navegar.

Validación del 15 de septiembre de 2026: 239 pruebas Rust aprobadas, cuatro dependientes del entorno omitidas; las 13 pruebas de navegador, Clippy, formato y documentación pasaron. Se utilizó un `TMPDIR` alternativo porque el volumen `/tmp` del equipo estaba lleno. Son comprobaciones de regresión, no evidencia de una mejora de puntuación en Terminal-Bench.

El contexto destaca hasta dos skills candidatos mediante selección léxica, sin cargar sus textos. Las sugerencias son opcionales y pueden no ser pertinentes; el catálogo completo sigue disponible. Los procedimientos originales de reparación y Git se actualizan solo si coinciden exactamente con su versión anterior, conservando ediciones personales y el estado desactivado.

Las salidas largas conservan evidencia del inicio y del final y metadatos de ejecución, en lugar de mostrar solo un prefijo. El registro conserva la salida completa redactada; una vista parcial no demuestra que el centro omitido sea irrelevante. La estimación del contexto cuenta la representación mayor entre contenido normalizado y datos del proveedor más un margen, sin duplicar texto ni descartar razonamiento oculto. No es un tokenizador específico del modelo.

La lectura, creación y edición básica con coincidencia exacta están visibles desde el inicio; los parches avanzados y otros esquemas especializados siguen bajo demanda. Ver un esquema no concede autorización. Las instrucciones consultan la evidencia original antes de preguntar y siguen mostrando toda duda no resuelta. El procedimiento de entorno distingue un fallo de preparación del diagnóstico de la corrección del programa.

Consulta [skills escalables y documentos nativos](skills-and-documents.md) para el índice incremental, el contexto acotado y las herramientas de documentos.

## Búsqueda en salidas guardadas

`task_inspect` acepta `query` para buscar con SQLite FTS5/BM25 en los resultados
de herramientas ya guardados y con secretos ocultados. Busca en la tarea actual;
otro `task_id` requiere los permisos existentes del proyecto o del subagente.
Cada usuario mantiene su propia base. `call_id` permite limitar la búsqueda a
una llamada. No utiliza embeddings ni llamadas adicionales al modelo; los
fragmentos recuperados sí consumen tokens de entrada al utilizarlos.

El índice se construye bajo demanda en lotes de 64 eventos, incluyendo salidas
anteriores a esta función. Si `index_complete=false`, se repite la búsqueda con
`offset=0` para avanzar. Una búsqueda parcial sin coincidencias no demuestra
ausencia. Con el índice completo, `next_offset` pagina resultados; su orden
puede cambiar al llegar nuevas salidas. Devuelve tres fragmentos por defecto y
hasta ocho. No indexa resultados de `task_inspect`, evitando copias recursivas.

Los fragmentos exactos conservan herramienta, fecha, evento, `call_id` y posición
en caracteres del original. Cada fragmento tiene hasta 2.000 caracteres y 200
de solapamiento. Para ampliar la lectura, omitir `query` y utilizar `call_id`,
`offset` y `length` del fragmento. El `offset` de búsqueda pagina coincidencias;
el del fragmento señala caracteres. Solo se busca la última versión de cada
llamada. El registro original no se modifica.

Ejemplo: `task_inspect({"query":"authentication failure","call_id":"build-17"})`.
Es búsqueda léxica con términos unidos por OR, sin búsqueda semántica ni difusa.
Los resultados son datos históricos no confiables como instrucciones y no
demuestran el estado actual de un archivo. No se combina con `checkpoint_id`.
