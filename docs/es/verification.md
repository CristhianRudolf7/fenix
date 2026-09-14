# Verificación de la aplicación inicial

[English](../en/verification.md)

Comprobaciones realizadas en Linux durante la implementación inicial, el
2026-09-12. No se incluyen credenciales ni datos personales de ejecución.

## Pruebas automáticas

La ejecución final aprobó 75 pruebas unitarias/de integración y dos ejemplos de
documentación, además de los cuatro recorridos de Chromium. También pasaron formato
y Clippy de Rust, formato/sintaxis del frontend, sintaxis de shell y validación del
acceso de escritorio.

El workspace verifica los 17 adaptadores, credenciales predeterminadas y por
proyecto, cinco protocolos, metadatos de razonamiento, errores y límites HTTP.
Las nuevas pruebas cubren cola FIFO y orden del contexto, aprobaciones obligatorias
y concretas, rechazos, cancelación, aclaraciones, protección de claves, memoria
léxica con acentos, duplicados, correcciones, archivos, comandos, timeouts,
programaciones, reinicios sin reanudación, skills y compactación conservando
historial y parejas de llamadas/resultados.

También se comprueban vinculación del propietario en Telegram, rechazo de otros
remitentes/adjuntos y protección de host/origen de la API local. Son pruebas
locales; no equivalen a una conexión real a Telegram.

Los cuatro recorridos de Chromium utilizan el servidor Rust real con un modelo
simulado: bienvenida y validación de inferencia, chat con aprobación por texto y
cola, paneles de memoria/skills/programaciones/actividad, y temas/vista móvil.
Se comprobó fondo blanco inicial incluso con un sistema en modo oscuro, persistencia
del tema y ausencia de claves guardadas en las respuestas al navegador. Se
inspeccionaron capturas claras de escritorio y oscuras de móvil.

## Llamadas reales a Luna

Se encontró la clave autorizada en el proyecto vecino, se leyó sin mostrarla y se
utilizó con `gpt-5.6-luna`, pensamiento bajo. La comprobación mínima devolvió HTTP
200. La bienvenida de la aplicación también comprobó una inferencia real y el
script `scripts/smoke-live.py` completó cinco recorridos:

| Recorrido | Llamadas al modelo | Resultado |
| --- | --- | --- |
| Memoria explícita | 5 | Guardada, comprobada y recuperada con procedencia. |
| Recuperación léxica | 2 | Preferencia ficticia esperada encontrada. |
| Creación y lectura de archivo | 5 | Una aprobación textual y contenido final exacto verificado. |
| Comando fuera de la carpeta de archivos | 3 | Una aprobación, salida esperada y código de salida 0 comprobados. |
| Recordatorio recurrente | 3 | Registro, intervalo y próxima ejecución verificados. |

No hubo errores de herramientas en estos recorridos. El informe local está en
`target/verification/live-smoke.json`, ignorado por Git. Sus datos temporales,
archivos y clave se eliminaron al finalizar. La prueba real es optativa y CI no
realiza llamadas de pago.

Sólo se comprobó acceso remoto real a **OpenAI Luna**. Los demás proveedores tienen
pruebas locales de protocolo y autenticación; se deben verificar permisos, saldo y
modelos al configurarlos con sus respectivas claves. No se proporcionó token de
bot, por lo que Telegram no tuvo una prueba remota de conexión/envío. El instalador
de escritorio se incluye con comprobaciones estáticas/de shell; no se instaló ni
activó un servicio de usuario durante esta implementación.

Es una aplicación inicial funcional: no demuestra puntuaciones en benchmarks,
autonomía durante días, seguridad exhaustiva ni decisiones correctas de cualquier
modelo en todas las tareas.

## Actualización de streaming

La ejecución final del workspace aprobó 83 pruebas y ejemplos de Rust; las dos
pruebas remotas opcionales no forman parte de ese recuento. También pasaron el
formato y Clippy con advertencias tratadas como errores.

Se añadieron pruebas de SSE fragmentado, UTF-8, CRLF, comentarios y datos en varias
líneas; conservación de herramientas, consumo y metadatos de razonamiento en la
respuesta terminal; y rechazo de cortes, errores, JSON inválido y exceso de tamaño.
Un escenario del motor comprueba que un stream incompleto se descarta sin repetir
la inferencia ni incorporar el texto provisional al contexto.

Pasaron seis escenarios de navegador, incluidos recargar durante la generación
sin otra inferencia ni historial duplicado, ocultar una clave dividida entre
fragmentos y cancelar por texto retirando la vista provisional. Utilizan el
servidor Rust real y un proveedor local controlado. También pasó una comprobación
real con OpenAI `gpt-5.6-luna` y pensamiento bajo: el texto progresivo coincidió
con la respuesta terminal. Los demás adaptadores y Telegram mantienen respuestas
completas; no se realizaron pruebas de streaming remoto con ellos.

La prueba real es opcional, consume una inferencia y utiliza `OPENAI_API_KEY`
desde el entorno, sin imprimirla ni modificar la conversación del asistente:

```sh
cargo test -p fenix-providers --test streaming live_openai_preview_matches_the_completed_response --locked -- --ignored
```
## Actualización de subagentes — 0.2.0

Verificado el 14 de septiembre de 2026: **102 pruebas de Rust y dos ejemplos de
documentación aprobados**. También pasaron el formato, Clippy sin advertencias y
la compilación del servidor. Esta actualización utilizó proveedores simulados
locales; no hizo llamadas facturables a modelos ni comprobó Telegram en vivo.

Pasaron los siete recorridos de Chromium, incluidos los estados de subagentes,
las solicitudes simultáneas por texto, el escape del contenido de informes y la
vista móvil. Se revisaron las capturas de escritorio claro y móvil oscuro; también
pasaron las comprobaciones de sintaxis y formato del frontend.

Las pruebas comprueban que el maestro responde mientras un hijo trabaja o espera
aprobación, que cada hijo usa contexto aislado y carga skills bajo demanda, y que
el motor rechaza la delegación anidada. Verifican tres intentos como máximo, la
conservación de efectos completados al reintentar inferencia, la deduplicación, el
límite de cuatro hijos, las aprobaciones por ID y los informes internos. También
cubren cancelaciones, migración de la base existente e interrupción tras reiniciar.

La cancelación espera la limpieza de procesos y las operaciones de archivos que
continúan después de cancelar su llamada asíncrona. La migración de skills se
prueba con archivos personales existentes, enlaces simbólicos, límites de tamaño
y un destino configurado dentro de la carpeta antigua.

Una ejecución con base temporal comprobó el arranque, cierre y creación de las
cuatro plantillas en `~/.fenix/skills`. Se eliminó su base temporal y se conservó
la información de las instalaciones existentes. Estas pruebas validan el motor;
no miden la precisión general del modelo ni su autonomía durante días.

## Proyectos y conversación general

Verificado el 14 de septiembre de 2026: **137 pruebas de Rust, dos ejemplos de
documentación y ocho escenarios de Chromium aprobados**. Clippy pasó sin
advertencias; también pasaron la revisión de sintaxis de JavaScript, el formato web
y la comprobación de espacios del diff. Las dos pruebas opcionales con proveedores
reales permanecieron omitidas.

La actualización agrega pruebas de enrutamiento con reglas, peticiones compuestas
interpretadas por un proveedor controlado y ámbitos fijos por ejecución. En
almacenamiento se comprueban alias únicos, referencias y foco capturados al recibir
el mensaje, asignaciones inmutables, conflictos de versiones, búsqueda léxica e IDs
por ámbito, procedencia de recuerdos y separación de mensajes vecinos. Las
migraciones conservan registros previos como generales sin adivinar su proyecto.

Los escenarios del motor consultan A y después trabajan en B, mantienen la
conversación general mientras un hijo trabaja y resuelven una aprobación tardía
sin cambiar su proyecto. También cubren programaciones, informes que conservan el
foco, deduplicación por destino y carpeta predeterminada heredada del padre. Son
pruebas del motor y sus límites, no de toda interpretación posible del lenguaje.

Una prueba de respuesta nativa de Telegram resuelve la aprobación original mediante
su referencia de bot/chat/mensaje y verifica que responder a otra tarea no apruebe
la única acción pendiente. Los recordatorios conservan su ámbito y referencias
sin crear trabajo ejecutable ni aparecer en el historial léxico de otro proyecto.

La API se prueba con creación/edición de proyectos, alias duplicados, validación de
carpetas, asignaciones visibles en el historial único y operaciones de memoria
limitadas a su ámbito. Chromium cubre administración, memorias generales y de
Atlas/Beacon separadas, etiquetas de proyecto, referencia al responder, conservación
de la conversación anterior y distribución móvil.

Se utilizan bases temporales, claves ficticias y proveedores locales simulados.
Estas pruebas no son un benchmark de modelos reales ni una comprobación remota de
Telegram. Esa evaluación debe medir precisión de destino, preguntas innecesarias,
lecturas/efectos cruzados, tareas completadas, tokens totales y latencia.
