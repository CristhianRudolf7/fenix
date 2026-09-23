# Contexto, programas de herramientas, recuperación y aprendizaje

[English](../en/harness-continuity.md)

Estas mejoras reutilizan el aislamiento por usuario, los proyectos, los permisos y el historial SQLite. Son cambios locales; no implican despliegue ni una mejora de benchmark ya medida.

## Contexto

El motor utiliza los tokens de entrada reportados por el proveedor como referencia y estima lo añadido después. Una huella del contexto completo, modelo, endpoint y herramientas invalida la referencia cuando cambian. Sin consumo reportado utiliza la estimación conservadora anterior. Las instrucciones estables del maestro quedan separadas de datos variables; el estado operativo se añade al final de la solicitud. Esto favorece la caché cuando el proveedor la admite, sin garantizar un porcentaje de ahorro.

## Programas de herramientas

`tool_program` ejecuta pasos con referencias a resultados, condiciones y bucles sobre listas. Es un intérprete declarativo en Rust; no ejecuta Python o JavaScript arbitrario. Reutiliza las herramientas permitidas de workflows y sus permisos. Permite hasta 50 llamadas, detiene el flujo ante errores y no reintenta efectos previos. No admite delegación recursiva ni procesos persistentes/interactivos.

El modelo recibe el resultado seleccionado; los resultados intermedios completos quedan en el historial consultable con `task_inspect`. Las nuevas instrucciones de un repositorio pausan las escrituras hasta que el modelo las lea. No es una transacción: un error posterior no deshace pasos ya realizados. Véase el ejemplo JSON en la documentación inglesa.

## Recuperación tras reinicios

Antes de consultar al modelo se guarda un punto completo de contexto y resultados. Antes de ejecutar una nueva respuesta se invalida ese punto. Tras un reinicio pueden continuar las tareas encoladas nuevas y las interrumpidas en un punto seguro, conservando identificadores, ámbito, contadores y efectos previos. Se permiten como máximo tres recuperaciones por tarea.

Los procesos activos, escrituras inciertas, lotes incompletos, preguntas o aprobaciones pendientes y revisores independientes quedan para revisión. Los planes solo continúan automáticamente si estaban ejecutándose y todos sus trabajadores activos son recuperables. No se restauran procesos, descargas del navegador ni transacciones externas. No se vuelven a ejecutar las herramientas del contexto guardado.

## Historial privado de archivos

`file_history list/inspect/restore` permite consultar versiones y restaurar un archivo mediante `id`, o las modificaciones registradas de una tarea mediante `task_id`. Se guarda el contenido original antes de escrituras, ediciones, parches, documentos nativos y exportaciones de biblioteca realizadas con las herramientas administradas.

La implementación usa almacenamiento privado por hash y metadatos SQLite: **ofrece historial sin crear un repositorio Git** ni tocar el `.git` del usuario. La restauración verifica el proyecto, las rutas y el contenido actual; rechaza cambios posteriores del usuario y versiones incompletas. Comprueba todas las rutas antes de escribir y registra su propio historial para deshacer la restauración. Una restauración múltiple puede quedar parcial ante cancelación o error de disco; el historial informa lo ocurrido.

No captura automáticamente cambios de comandos arbitrarios, conectores o herramientas específicas de diseño. Para snapshots de todo el espacio existe `workspace_checkpoint` cuando el despliegue lo admite. Tampoco deshace correos enviados u otras acciones externas. Conserva bytes originales privados, con vistas previas redactadas. Los límites iniciales son 32 MiB por archivo y 512 MiB de contenido almacenado; no elimina versiones automáticamente.

## Aprendizaje y mantenimiento de skills

Las tareas del maestro completadas con suficiente actividad pueden proponer procedimientos reutilizables en segundo plano. Los saludos y tareas simples no generan revisiones. La selección inicial requiere tres resultados de herramientas y algún intento fallido, u ocho resultados sustantivos. Cada tarea se encola una sola vez.

El modelo configurado prepara un candidato y lo contrasta con la evidencia, la cobertura del skill original y ejemplos de regresión. Hay dos intentos como máximo y 90 segundos por intento; el consumo queda registrado. **La revisión semántica no equivale a ejecutar pruebas ni demuestra una mejora de calidad.**

Los candidatos aceptados **se activan automáticamente, sin aprobación del usuario**. Antes de proponerlos se busca una selección acotada de skills similares; la revisión comprueba si aportan algo nuevo y se rechazan procedimientos exactamente duplicados aunque cambie el nombre. No es una deduplicación semántica exhaustiva.

Solo se reescriben automáticamente skills previamente aprendidos, habilitados, no fijados y cuyo contenido siga intacto. Los manuales e incluidos de fábrica sirven como referencia protegida. Las actualizaciones conservan archivos auxiliares, referencias existentes, categoría y herramientas requeridas. La revisión semántica no garantiza por sí sola que todas las funciones se preserven.

Cada cambio guarda primero un registro durable con contenido anterior, nuevo y versión previa. Un reinicio permite reconciliar una aplicación interrumpida sin pisar cambios del usuario. `activate` reintenta un candidato validado bloqueado, sin aprobación. `rollback id` restaura la revisión anterior o desactiva un skill nuevo sin borrarlo; rechaza cambios posteriores y versiones más recientes. Las activaciones anteriores a este registro no tienen reversión mediante esta nueva acción.

El mantenimiento automático se ejecuta como máximo una vez al día cuando el maestro, los subagentes y los flujos están libres y no consulta al modelo. Marca como poco utilizados los skills aprendidos sin uso durante 30 días y los deshabilita a los 90 días. Protege los fijados, modificados manualmente y referenciados por nombre en tareas programadas o flujos, incluidos los pausados. Las referencias indirectas en lenguaje natural podrían no detectarse. `maintenance` muestra el estado; `pin`, `archive` y `unarchive` permiten gestionarlo. Reactivar inicia un nuevo período de gracia. No se borran archivos ni se fusionan paquetes automáticamente.

`list/inspect` muestra candidatos y evidencias; `reject` descarta propuestas inactivas. `configure enabled:false` detiene futuras revisiones, activaciones y mantenimiento automáticos. Los skills activos siguen instalados y pueden revertirse. Los nuevos skills permanecen privados, sin publicación automática en el marketplace.

## Alcance de las pruebas

Las regresiones verifican permisos, referencias y bucles, recuperación conservadora, restauración binaria, cambios posteriores, instrucciones de repositorios y revisiones simuladas de skills. No certifican ahorros, resultados de modelos reales ni recuperación de efectos externos arbitrarios.
