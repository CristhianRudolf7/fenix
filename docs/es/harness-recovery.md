# Skills de procedimiento y recuperación basada en evidencia

[English](../en/harness-recovery.md)

Fénix incluye catorce skills manuales: las cinco anteriores y nueve familias para repositorios, reparación con pruebas, dependencias, servicios, paquetes, SQL, procesamiento de texto, Git e implementación de software. Son procedimientos generales, no respuestas del benchmark. La instalación conserva las modificaciones del usuario.

`skill_search(query, limit=3)` busca por nombre, descripción y contenido completo, dando más peso a los dos primeros. Devuelve metadatos. `skill_load` carga el contenido y hasta ocho herramientas autorizadas declaradas en el campo opcional `required_tools`. Los skills anteriores siguen funcionando. Cargar un skill no concede permisos.

Si el modelo solicita una herramienta autorizada cuyo esquema todavía no se expuso, recibe el esquema y `executed=false`; debe enviar otra llamada válida. Los errores de argumentos muestran campos necesarios, recibidos y un ejemplo de formato.

## Procesos

`command_run` admite `read_paths` para entradas existentes. Un proceso Rust separado aplica Landlock ABI 3 y seccomp antes de ejecutar el comando: lectura de entradas y bibliotecas estándar, sin escritura, red ni modificación de metadatos. Si Linux no puede aplicar las restricciones, falla explícitamente; nunca cambia silenciosamente a ejecución normal. Algunos programas necesitan cachés o temporales y no son compatibles con este modo.

Los archivos de entrada permanecen registrados mientras vive el proceso. Las escrituras y ediciones independientes pueden continuar; los conflictos esperan. Los parches múltiples y efectos desconocidos siguen siendo conservadores. Leer directorios amplios reduce la concurrencia. Son lecturas en vivo, no snapshots, y otros procesos ajenos a Fénix quedan fuera de este control. El modo restringido no admite TTY ni servicios persistentes. Los servicios normales conservan `persistent=true` explícito y sus resultados describen cuándo se eliminan sus descendientes.

## Contexto y recuperación

Antes de compactar se guarda el original redactado. Primero se intenta reducir salidas antiguas exitosas mediante referencias recuperables, conservando parejas de llamadas/resultados, mensajes recientes, roles y contrato. Esta ruta no llama al modelo para resumir. Si no cabe, se conserva la ruta de resumen y respaldo existente. Los resúmenes generados tienen rol de asistente y no constituyen instrucciones del usuario ni evidencia de ejecución. La pérdida semántica no queda eliminada por completo.

La repetición sin progreso ofrece diagnóstico limitado: procesos bloqueantes, esquemas, checkpoints o pruebas concretas. Se mantienen tres recuperaciones por causa y ocho totales. Las respuestas truncadas nunca se ejecutan. Se distingue heurísticamente entre argumentos grandes y agotamiento sin un payload grande; no es una medición directa de tokens de razonamiento.

Se reutilizan planificación editable, subagentes y revisores existentes. Las instrucciones reservan revisión independiente para incertidumbre material que las comprobaciones locales no resuelvan, sin cambiar modelos ni habilitar delegación anidada.

## Evaluación

`experiment.py`, en `scripts/benchmarks/harbor`, ejecuta lotes de cinco tareas con el evaluador nativo, hashes del binario/configuración, calificación oficial y costes del proxy. Requiere proxy activo y rutas explícitas del dataset, binario, configuración y Harbor. `--summarize --root RUTA` actualiza resultados sin lanzar tareas.

Se comparan binarios congelados de main y candidata con la misma configuración. Las tareas inspeccionadas sirven para desarrollo; la confirmación utiliza diez nuevas en dos lotes, con dos intentos por versión. Se separan fallos de infraestructura y se mantienen desconocidos los costes ausentes. La rama experimental se publica para revisión sin integración automática.

Los trabajadores liberan el bloqueo de lectura del modelo antes de esperar configuración o inferencia. Esto evita que un trabajador inactivo bloquee un cambio de modelo. La web descarta respuestas de vistas anteriores después de navegar.

Validación del 15 de septiembre de 2026: 239 pruebas Rust aprobadas, cuatro dependientes del entorno omitidas; las 13 pruebas de navegador, Clippy, formato y documentación pasaron. Se utilizó un `TMPDIR` alternativo porque el volumen `/tmp` del equipo estaba lleno. Son comprobaciones de regresión, no evidencia de una mejora de puntuación en Terminal-Bench.

El contexto destaca hasta dos skills candidatos mediante selección léxica, sin cargar sus textos. Las sugerencias son opcionales y pueden no ser pertinentes; el catálogo completo sigue disponible. Los procedimientos originales de reparación y Git se actualizan solo si coinciden exactamente con su versión anterior, conservando ediciones personales y el estado desactivado.
