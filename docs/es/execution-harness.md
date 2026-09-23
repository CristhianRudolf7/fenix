# Ejecución interactiva y herramientas bajo demanda

[English](../en/execution-harness.md)

El motor Rust incorpora procesos interactivos, servicios persistentes, registros
incrementales redactados, reutilización de subagentes y descubrimiento de
herramientas. Funcionan tanto desde la web como desde Telegram, manteniendo la
misma conversación.

## Terminal y procesos persistentes

`command_run` recibe `command`, `cwd` y opcionalmente `tty`, `persistent` y
`timeout_seconds`. Los procesos con pipes aceptan entrada estándar. `tty: true`
crea una terminal PTY real mediante la biblioteca Rust `pty-process`, inicialmente
de 24 filas y 100 columnas. `command_input` envía texto, permite cambiar
`rows`/`columns` y admite `eof`: cierra la entrada de un pipe o envía Ctrl-D a la
terminal. También pueden enviarse caracteres de control como Ctrl-C. Cada entrada
respeta la aprobación textual exacta de comandos; una escritura parcial no se
reintenta automáticamente.

Los comandos normales deben terminar o cancelarse antes de finalizar una tarea.
Con `persistent: true`, una sesión o servidor sobrevive al final satisfactorio del
turno. `command_list` permite encontrarlo y el maestro puede consultarlo, enviarle
entrada o cancelarlo desde turnos posteriores del ámbito autorizado. Los
subagentes solo controlan sus propios procesos.

Estos servicios terminan al cancelarse, vencer su plazo, fallar/cancelarse su tarea
activa o apagarse Fénix. No se reinician automáticamente; sus registros guardados
siguen disponibles por ID. Los comandos normales conservan el bloqueo de efectos;
las sesiones persistentes lo liberan al arrancar y pueden coincidir con cambios
posteriores. No constituyen un sandbox ni un worktree aislado. Se admiten hasta
16 procesos simultáneos. La limpieza detiene el grupo original y los miembros de
la sesión PTY; un programa que escape deliberadamente de ella queda fuera de esa
garantía de aislamiento.

Los subagentes que ejecutan tareas en Git ahora reciben [worktrees aislados](parallel-worktrees.md).
Sus comandos de archivos tienen bloqueos independientes. El motor detiene sus
procesos antes de integrar; los servicios compartidos y las carpetas sin Git
conservan el comportamiento anterior.

## Salidas incrementales

`command_status` entrega únicamente novedades por defecto, con cursor por tarea
lectora. `cursor` permite volver a una posición y `next_cursor` indica la siguiente.
`wait_ms` espera hasta 30 segundos y `limit` limita la respuesta a 12.000 bytes.
Si hay demasiada salida, se conserva el principio y el final y se informa cuántos
bytes se omitieron. Evita consultar repetidamente si no hay novedades.

`command_output` recupera el registro con `offset`/`limit` en bytes; devuelve
`next_offset` y `has_more` sin mover el cursor incremental. Los registros se
redactan antes de guardarse en archivos privados de `DATA_DIR/processes`,
preservando fragmentos de secretos entre lecturas. Los tokens incompletos
excesivamente grandes se sustituyen y se marca el registro como incompleto; los
bytes UTF-8 inválidos se reemplazan. stdout/stderr se combinan según su llegada.
El límite de 64 MiB por registro o un fallo de escritura detiene el proceso con un
error explícito. Por ahora los registros permanecen hasta que el propietario
elimine los datos; no permiten restaurar archivos ni deshacer acciones.

Los turnos posteriores del principal pueden recuperar logs de comandos en primer
plano dentro de su ámbito autorizado, sin obtener control de esos procesos.
`log_complete` y `output_error` indican si la captura está completa; un proceso
interrumpido queda marcado como incompleto tras reiniciar.

## Límites configurables

En **Configuración → Avanzado → Límites de ejecución**:

| Ajuste | Predeterminado |
| --- | ---: |
| Pasos del modelo por tarea | 128 |
| Tiempo activo del maestro | 4 horas |
| Tiempo por intento de subagente | 1 hora |
| Tiempo por comando normal | 30 minutos |
| Duración de sesión persistente | 24 horas |
| Máximo de tokens de salida | 16.384 |

Se permiten hasta siete días, 4.096 pasos y 131.072 tokens de salida configurados.
El proveedor debe admitir el tamaño elegido; el espacio de contexto estimado
limita además cada respuesta y reserva 1.024 tokens de margen. Esperar respuestas
humanas no consume tiempo activo. Los pasos no se reinician al replanificar,
compactar o enviar seguimientos. Se conservan la detección de repeticiones y los
tres intentos para fallos transitorios. No se añade recuperación automática tras
reinicios ni presupuesto monetario.

## Seguimiento de subagentes

`subagent_send` recibe `subagent_id` y `message`. Solo el maestro activo puede
utilizarlo. Una cola SQLite conserva orden y procedencia y deduplica mensajes
idénticos del mismo remitente. El mensaje y el contexto actualizado se guardan
en una transacción al llegar a un límite seguro entre herramientas. Si llega
durante una inferencia, se omiten las acciones restantes que hayan quedado
desactualizadas; los efectos completados se conservan. Una pregunta pendiente al
usuario sigue esperando su respuesta o cancelación.

Un trabajador independiente completado o bloqueado puede continuar con el mismo
ID, propósito, ámbito, contexto y contadores. Un trabajador fallido necesita
intentos disponibles. No se reabren trabajadores cancelados, interrumpidos por
reinicio o sin intentos restantes. Cada trabajador admite hasta 100 seguimientos;
una nueva finalización emite otro informe y conserva los anteriores. El maestro
permanece disponible.

Los revisores de planes no reciben instrucciones laterales. Para cambiar un hito
terminado se utiliza la revisión del plan. Los hitos en curso pueden recibir
aclaraciones que mantengan su objetivo. Sigue existiendo un solo nivel de
delegación.

## Herramientas bajo demanda

El contexto inicial contiene herramientas esenciales y una lista compacta de las
otras herramientas permitidas. `tool_search` busca por propósito, carga `names`
exactos o muestra un catálogo breve. Utiliza BM25 local y términos en español e
inglés, sin embeddings ni llamadas al modelo. Carga hasta ocho esquemas por
consulta para la siguiente inferencia y conserva hasta 24 esquemas no esenciales.
Las herramientas retiradas siguen siendo descubribles.

Buscar herramientas nunca amplía permisos de rol o proyecto: cada ejecución se
valida nuevamente. Una llamada a una herramienta conocida y permitida puede
validarse aunque el modelo recuerde su nombre sin buscarla. Los skills mantienen
su carga independiente mediante `skill_load`. Esta actualización no incorpora MCP
ni un motor JavaScript para componer herramientas.
