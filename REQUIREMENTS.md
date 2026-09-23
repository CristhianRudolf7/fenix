# Fénix — Especificación de requisitos

> Memory implementation / Implementación de memoria: [English](docs/en/adaptive-memory.md) · [Español](docs/es/adaptive-memory.md). This document retains broader design goals; see the implementation contract for delivered behavior and limits. / Este documento conserva objetivos más amplios; consulta el contrato para conocer lo implementado y sus límites.

## Current scope decisions / Decisiones de alcance actuales

- Maintain one continuous conversation for the owner. Telegram support ships initially, but its token and activation are optional. Do not expose separate chats, conversation creation, session switching, or resets as the normal workflow. Internal task IDs and context compaction must preserve that continuity.
- Keep Telegram text-only. The web Design workspace accepts supported visual file imports and shows task results in the continuous chat; general chat attachments and audio-file import remain outside this delivery. / Telegram conserva texto; Design admite archivos visuales y muestra resultados en el mismo chat.
- Independent web search and bounded public HTTP reading are in scope: a background crawler, a local disk index and `web_search`/`web_read` agent tools without external search providers. Local Chromium navigation, persistent profiles and responsive testing are also in scope; generic HTTP actions remain in the roadmap. See [the web search contract](docs/en/web-search.md).
- Use `America/Lima` initially. Automatic resumption of interrupted tasks and catch-up of missed scheduled executions remain future work; keeping history and schedules does not imply replaying their actions.
- Initially support Linux and one owner per installation. The project must be clonable and run locally, with its web interface bound to loopback by default; routine use must not require a terminal.
- On first visit without configuration, show a welcome/setup screen. Require at least one provider API key and a successful real inference request to the selected provider/model before completing setup, with provider-dependent selectors and fields covering all 17 integrated providers. Telegram setup can be skipped and completed later.
- The web must include chat and manage provider credentials, Telegram, explicit memory, skills, scheduled tasks, reminders, and their states. Web and optional Telegram use the same continuous conversation and queue.
- Use a warm visual identity: ivory canvas, white surfaces, charcoal text, terracotta actions and teal status accents, with legible typography and clear hierarchy. The first visit uses light mode regardless of the operating-system theme. Persist explicit theme and language selections across the login and application. See `docs/en/interface.md`.
- Planning, execution and conversation are asynchronous. Textual replies and plan edits bypass ordinary queued work; plans and deliveries are persisted and versioned. See section 1.17 and the planning documentation.
- La planificación, ejecución y conversación son asíncronas. Las respuestas y ediciones textuales de planes no esperan la cola ordinaria; los planes y entregas se guardan con versiones. Véase la sección 1.17.
- Store manually installed skills at `~/.fenix/skills/<skill-name>/SKILL.md`, shared under the current user home independently of runtime data, with `--skills-dir` as an explicit override. Keep personal runtime data out of Git.
- Command execution may use any working directory accessible to the Linux user running Fénix; do not restrict it to a project workspace or request approval merely because the directory is outside that workspace. Keep explicit working directories, OS permissions, action-specific approvals, timeouts, and process controls. A file-tool workspace is not a sandbox boundary for this broader command capability.
- The master catalog covers local tools, project management and `subagent_spawn`, `subagent_status` and `subagent_cancel`. Children use a restricted catalog without conversation history or delegation and require `subagent_finish`; the executor restricts inspection to their own ID and authorized predecessor/prior-generation plan evidence. Include five manual default skills, including `browser-navigation`. Merge explicit memory and reminders/scheduling into `memory-and-reminders`; retain `local-file-work`, `conversation-recap`, and `local-troubleshooting`.
- Current agent provider/model: OpenAI `gpt-5.6-luna` through Responses, with `reasoning.effort: "low"`, replacing the earlier Astra selection at the owner's request for initial development and verification. The request contract and Responses adapter implement this setting. No automatic model switching is required.
- Heartbeat is future work in `ROADMAP.md`, section 2.23. Cron and explicit reminders remain in scope. Provide an approval-request tool backed by mandatory runtime permission checks.
- Do not implement an application-level maximum token budget or ask for one in initial setup. Provider context/output limits, token usage reporting, timeouts, bounded retries, and basic loop prevention are distinct from that deferred budget feature.

## Alcance del documento

Este documento define el alcance actual de Fénix: un asistente personal funcional, con un único agente principal y subagentes en segundo plano, conversación por texto, herramientas autorizadas, memoria básica explícita y controles operativos. Estas son las capacidades que se deben construir y validar primero.

- Incluir soporte de Telegram desde el comienzo como único canal externo de mensajería, con token y activación opcionales. La instalación debe poder completarse y la web funcionar sin configurar Telegram. Mantener una única conversación continua del propietario, sólo por texto.
- Uso personal: un propietario autorizado por instalación. Cualquier persona debe poder clonar el proyecto y ejecutar su propia instalación local, con datos y credenciales independientes.
- Soportar inicialmente Linux. Incluir una web en localhost, vinculada por defecto sólo a loopback, con bienvenida, configuración de proveedores y Telegram, memoria, skills, programaciones y estados. El uso habitual no debe requerir terminal; no se requiere una plataforma SaaS ni multiusuario compartida.
- La búsqueda y lectura web propias, la navegación local con Chromium, los perfiles persistentes y las pruebas responsivas forman parte del alcance actual. La herramienta HTTP genérica queda para el roadmap. La conexión con modelos y el canal opcional de Telegram siguen dentro del alcance.
- El heartbeat y el resto de ampliaciones se conservan en [ROADMAP.md](ROADMAP.md). El alcance actual de voz se documenta en [Live](docs/es/live.md).
- Este documento describe requisitos y criterios de cierre; no implica que las funciones ya estén implementadas.
- Las referencias de OpenClaw, ZeroClaw y NullClaw sirven para definir el conjunto básico de capacidades y se mantienen como antecedentes de diseño.

## Asistente funcional con memoria básica

### 1.1. Ejecución local y proveedores de modelos

- Mantener el núcleo en Rust como decisión de la conversación. El servicio debe poder iniciarse, detenerse, cargar su configuración y funcionar independientemente de que la interfaz esté abierta.
- Ejecutar el servicio y la web localmente, sin necesitar un servidor público propio. La máquina y el servicio deben estar encendidos para atender Telegram y ejecutar programaciones; cerrar una pestaña no debe detener el servicio.
- Definir adaptadores para modelos remotos o locales compatibles con conversación y llamadas a herramientas. Permitir seleccionar proveedor y modelo, configurar credenciales y tratar errores de conexión, límites de uso y timeouts.
- Modelo predeterminado actual: OpenAI GPT-5.6 Luna, identificador `gpt-5.6-luna`, mediante Responses API y `reasoning.effort: "low"`. Esta selección sustituye la anterior de Astra por indicación del propietario para la implementación inicial. Conservar la selección manual; no cambiar automáticamente el modelo ni aumentar el razonamiento.
- Este valor no obliga a configurar OpenAI: se puede elegir cualquiera de los 17 proveedores y un modelo compatible con herramientas. No aplicar el identificador ni los parámetros específicos de Luna a otros proveedores.
- El contrato y adaptador Responses incluyen esfuerzo de razonamiento explícito. Las llamadas del agente a Luna omiten `temperature`, `top_p` y `top_logprobs`. El acceso real y varios recorridos con herramientas se comprobaron con una cuenta autorizada; ver [el registro de verificación](docs/es/verification.md).
- Mantener separados el núcleo, los proveedores, las herramientas, la persistencia y los canales, para poder ampliar cada uno sin reescribir el bucle del agente.

Referencia del modelo actual: [GPT-5.6 Luna](https://developers.openai.com/api/docs/models/gpt-5.6-luna), consultada el 2026-09-12.

### 1.2. Bucle del agente y cumplimiento básico de la tarea

- Seguir el ciclo: recibir mensaje → identificar sesión y objetivo → construir contexto → consultar modelo → validar y ejecutar herramientas → incorporar resultados → continuar o responder.
- Cada tarea debe conservar el pedido, el objetivo, las restricciones, el resultado esperado y los supuestos utilizados, identificados como tales. Una inferencia del agente no debe confundirse con una instrucción del usuario.
- Comprobar el resultado antes de declarar éxito: leer el archivo producido, comprobar un estado o ejecutar una prueba existente cuando corresponda. La validación debe ser proporcional a la tarea.
- Permitir consultar el estado y cancelar la ejecución. Cancelar detiene trabajo pendiente; no revierte por sí solo efectos que ya ocurrieron.

### 1.3. Conversación única y persistente

- Mantener una conversación continua, como con una persona, también al conectar Telegram más adelante. No ofrecer conversaciones separadas ni exigir crear, seleccionar o reiniciar sesiones para seguir hablando, cambiar de tema o iniciar una tarea.
- Aceptar únicamente mensajes de texto. No incorporar recepción ni procesamiento de documentos, archivos, imágenes, audio o vídeo en el alcance inicial.
- Guardar mensajes, respuestas y resultados de herramientas para continuar la misma conversación después de reiniciar el servicio. Los identificadores internos organizan el trabajo sin dividir la experiencia del usuario en nuevos chats.
- Mantener una conversación visible con turnos independientes asíncronos y subagentes en segundo plano. Despachar solicitudes ordinarias por llegada dentro de límites de concurrencia, priorizando respuestas y ediciones; no mezclar secuencias incompletas de herramientas entre turnos.
- Tramitar por texto las aprobaciones, rechazos, aclaraciones y cancelaciones; no usar botones para esas acciones. Cuando el agente espere una respuesta, vincular una respuesta clara a esa solicitud pendiente y continuarla sin esperar al final de la cola. Las peticiones independientes pueden atenderse mientras otra espera. Si el texto es ambiguo, pedir aclaración sin asumir autorización.
- Incluir chat en la web para usar el asistente sin Telegram. Debe mostrar y utilizar la misma conversación y cola, sin crear un segundo chat independiente; activar Telegram después no crea una conversación nueva.
- Conservar la conversación general y contextos específicos por proyecto y tarea dentro del mismo chat. Resolver el destino antes de recuperar información o ejecutar herramientas, capturando el espacio de trabajo por ejecución; ver la sección 1.16.

### 1.4. Instrucciones, identidad y perfil

| Archivo | Función inicial |
| --- | --- |
| `AGENTS.md` | Reglas operativas e instrucciones del agente. |
| `SOUL.md` | Personalidad, valores y estilo de comportamiento. |
| `IDENTITY.md` | Nombre, identidad y rol del asistente. |
| `USER.md` | Perfil y preferencias declaradas por el usuario. |
| `MEMORY.md` | Hechos y decisiones duraderos guardados de forma explícita o confirmada. |

Cargar estos archivos de forma controlada, con un límite de contexto y precedencia clara. Las herramientas y los documentos recuperados aportan datos; no pueden cambiar por sí solos las instrucciones del sistema. Las credenciales deben permanecer fuera de estos archivos y de las respuestas.

### 1.5. Memoria básica y continuidad de contexto

- Separar el historial de la sesión de la memoria duradera. Guardar, consultar, corregir y eliminar hechos de `MEMORY.md` o de un registro sencillo cuando el usuario lo pida o confirme.
- Guardar fecha y referencia al mensaje de origen cuando estén disponibles. Comprobar coincidencias exactas antes de añadir otra entrada y aplicar las correcciones explícitas del usuario.
- Recuperar por ID, título o palabras mediante búsqueda exacta o léxica sencilla. El alcance actual no requiere embeddings, grafos, tarjetas de entidades, wiki ni clasificación inteligente de consultas.
- Construir el contexto con las instrucciones estables, el perfil necesario, memoria explícita pertinente, un resumen del ámbito si hace falta y mensajes relacionados. Filtrar por proyecto antes de recuperar resultados; mantener la memoria general separada de las decisiones particulares.
- Cuando la conversación crezca, generar un resumen operativo básico y mantener el historial original recuperable. No sobrescribirlo con el resumen ni separar una llamada de herramienta del resultado que necesita para interpretarse.
- Mantener la misma conversación al compactar el contexto. Una conversación continua no implica enviar todo el historial al modelo en cada llamada ni disponer de memoria ilimitada.
- La extracción automática de hechos, la promoción de preferencias repetidas, la compactación con checkpoints y la inyección dinámica de memoria quedan para el trabajo futuro.

Usar esta estructura para el resumen operativo; los supuestos, errores, reintentos y efectos ya realizados deben aparecer en los apartados correspondientes:

```text
Objetivo
Restricciones y preferencias
Progreso
  Completado
  En curso
  Bloqueado
Decisiones clave
Archivos relevantes
Próximos pasos
Contexto crítico
```

Esta estructura también se reutiliza en la memoria operativa avanzada para mantener la continuidad entre etapas de desarrollo.

### 1.6. Herramientas básicas y extensibilidad

- Leer, listar, buscar, crear y editar archivos dentro del espacio autorizado.
- Ejecutar comandos desde cualquier directorio accesible para el usuario de Linux que ejecuta Fénix, sin restringirlos al espacio de trabajo del proyecto. Usar directorio de trabajo explícito, timeout, salida acotada y control del proceso.
- Incorporar búsqueda web con índice propio en disco, rastreo en segundo plano y lectura HTTP pública de fuentes mediante `web_search` y `web_read`, sin proveedores externos de búsqueda. La navegación interactiva y las acciones HTTP genéricas se conservan en el roadmap.
- Gestionar memoria explícita, consultar estado de tareas y crear o cancelar programaciones.
- Definir un registro extensible de herramientas con nombre, descripción, esquema de argumentos, permisos y resultado normalizado. Habilitar sólo las herramientas necesarias para el turno.
- Dejar preparados los límites de integración para nuevas herramientas o MCP. La navegación interactiva y el arnés de pruebas de sitios renderizados se incorporarán más adelante.

### 1.7. Permisos y aislamiento básico

- Restringir quién puede utilizar el asistente y qué herramientas y operaciones tiene habilitadas. Las herramientas específicas de archivos conservan su ámbito configurado; los comandos pueden operar en cualquier directorio accesible según los permisos del usuario de Linux.
- Aplicar las comprobaciones antes de ejecutar y los límites de proceso correspondientes. No imponer confinamiento de los comandos a una carpeta de proyecto ni solicitar autorización sólo por salir de ella. El espacio de las herramientas de archivos no constituye una frontera de aislamiento para esta capacidad más amplia de ejecutar comandos.
- Solicitar aprobación cuando una acción exceda la autorización vigente o lo exija su política; conservar esa autorización dentro de su alcance.
- Ofrecer una herramienta para solicitar aprobaciones textualmente en la conversación activa, con Telegram cuando esté configurado. Explicar la acción y su alcance; aceptar aprobación o rechazo por texto sin botones, vinculados a la solicitud concreta. El ejecutor debe comprobar los permisos aunque el modelo no solicite aprobación; el silencio y las respuestas ambiguas no autorizan la acción.
- Validar argumentos y tratar documentos, páginas y salidas de herramientas como contenido no confiable.
- Redactar secretos en registros y respuestas. Los sistemas avanzados de permisos entre proyectos y de compensación de efectos se desarrollarán más adelante.

### 1.8. Skills predefinidos y manuales

- Cargar skills instalados o redactados explícitamente, seleccionarlos por su descripción y utilizarlos cuando encajen con la tarea.
- Ubicación predeterminada: `~/.fenix/skills/<skill-name>/SKILL.md`, dentro del directorio personal real del usuario y compartida entre directorios de ejecución. Permitir configurar otra ruta explícita. Cada carpeta puede contener los recursos locales que referencie el skill.
- Mantener `.fenix/` fuera de Git para no publicar skills personales ni datos de ejecución. No instalar skills mediante adjuntos de Telegram ni incorporar aprendizaje automático.
- Cada skill debe explicar objetivo, entradas, pasos, herramientas permitidas, límites, salidas y condiciones de éxito.
- Permitir listar, habilitar y deshabilitar skills, conservando al menos nombre, descripción y versión utilizada.
- Distribuir plantillas predeterminadas versionadas en `resources/skills/` e instalarlas en `~/.fenix/skills/` durante la configuración inicial sin sobrescribir skills personales existentes. La memoria, los permisos y la persistencia siguen siendo responsabilidades del programa aunque se deshabilite un skill.

Approved default skills / Skills predeterminados aprobados:

| Skill | Purpose | Función |
| --- | --- | --- |
| `memory-and-reminders` | Explicit memory CRUD, duplicate checks and provenance; one-time reminders and recurring schedules, clarification of dates/times, and confirmation of the saved outcome. | Une memoria explícita y recordatorios: consultar, guardar, corregir y eliminar recuerdos; comprobar duplicados y procedencia; crear y gestionar recordatorios y programaciones, aclarar fechas/horas y confirmar el resultado guardado. |
| `local-file-work` | Inspect local files, apply authorized changes, and verify the result. | Inspeccionar archivos locales, aplicar cambios autorizados y verificar el resultado. |
| `conversation-recap` | Recover history and task states to summarize decisions, progress, and pending work with references. | Recuperar historial y estados para resumir decisiones, progreso y pendientes con referencias. |
| `local-troubleshooting` | Diagnose local errors using logs, commands, and state; verify fixes when authorized. | Diagnosticar errores locales con registros, comandos y estados; verificar correcciones cuando estén autorizadas. |

`memory-and-reminders` replaces the proposed `explicit-memory` and `reminders-and-schedules` defaults. It combines their procedures, not their storage: memory records and schedules remain separate structured resources. Load descriptions first and full instructions when relevant; no automatic skill learning or rewriting.

`memory-and-reminders` sustituye a los defaults propuestos `explicit-memory` y `reminders-and-schedules`. Une sus procedimientos, no su almacenamiento: los recuerdos y las programaciones siguen siendo recursos estructurados distintos. Cargar primero descripciones y luego instrucciones completas cuando sean pertinentes, sin aprendizaje ni reescritura automática de skills.

### 1.9. Cron y recordatorios explícitos

- El planificador funciona fuera del modelo y del adaptador de Telegram. Persiste las programaciones para que sobrevivan a un reinicio.
- Admitir ejecuciones de una sola vez, intervalos y expresiones cron, usando inicialmente la zona horaria de Perú, `America/Lima`. La configuración de otras zonas queda para el trabajo futuro.
- Permitir crear, listar, editar, pausar y cancelar tareas programadas o recordatorios pedidos por el usuario; conservar próxima ejecución, estado y último resultado.
- Aplicar timeout, reintentos acotados y protección contra solapamientos de la misma programación. La reanudación automática de tareas interrumpidas y la recuperación de ejecuciones vencidas durante una interrupción quedan fuera del alcance inicial.

Referencia de funcionamiento: [Automations/Cron de OpenClaw](https://docs.openclaw.ai/automation/cron-jobs).

### 1.10. Límites, errores y prevención básica de bucles

- Establecer límites operativos de tiempo, llamadas al modelo, herramientas y reintentos, y fijar concurrencia máxima. No incluir presupuesto máximo de tokens de la aplicación ni pedirlo en la bienvenida; se conserva como función futura en la sección 2.10 del roadmap.
- Conservar el registro de tokens disponibles y respetar los límites técnicos de contexto y salida del proveedor. Esos límites de una solicitud no equivalen a una cuota acumulada por tarea o día. Mostrar costo sólo cuando exista información verificable para calcularlo.
- Detectar repeticiones consecutivas de la misma herramienta con argumentos y resultados equivalentes. Al persistir la falta de progreso, detener el ciclo y explicar el bloqueo.
- Tratar fallos de proveedor, errores de herramientas, timeout y cancelación con estados explícitos. Reintentar sólo de forma acotada y cuando sea seguro.
- No volver a ejecutar automáticamente un efecto externo cuyo resultado sea desconocido.

Referencia: [Detección de bucles de OpenClaw](https://docs.openclaw.ai/tools/loop-detection).

### 1.11. Estado, registro mínimo y administración

- Generar `session_id`, `task_id`, `turn_id` y `tool_call_id` antes de procesar el trabajo correspondiente.
- Guardar herramienta, argumentos permitidos, resultado, error, código de salida, timestamps y consumo disponible. Mantener las salidas grandes fuera del contexto y ofrecer una referencia para recuperarlas.
- Mostrar estados como pendiente, ejecutando, bloqueado, completado, fallido, cancelado o interrumpido.
- Tras reiniciar, conservar historial y programaciones, e identificar las tareas que quedaron interrumpidas; el alcance actual no promete reanudar automáticamente cualquier efecto en curso.
- Administrar modelo, herramientas habilitadas, límites, skills, memoria explícita y horarios mediante opciones o comandos concretos. La autoconfiguración libre por lenguaje natural y el historial restaurable de configuración quedan para el trabajo futuro.

### 1.12. Bienvenida y administración web local

- Mostrar una pantalla de bienvenida y configuración inicial al abrir la web si no existe una configuración completa. La propia bienvenida debe ser accesible sin credenciales; el uso normal del agente requiere al menos una clave API de proveedor configurada.
- Ofrecer un desplegable con los 17 proveedores integrados, selección de modelo y campos dependientes del proveedor: clave API, endpoint cuando corresponda y ámbito de proyecto/organización cuando sea compatible. No exigir configurar todos los proveedores.
- Proponer OpenAI/GPT-5.6 Luna con razonamiento bajo al inicio y respetar otra selección explícita del usuario. No exigir una clave OpenAI si se seleccionó otro proveedor.
- Mantener la exigencia de clave indicada por el usuario en el flujo inicial, sin añadir silenciosamente un modo sin clave. El adaptador local de Ollama permite operar sin autenticación, pero eso no elimina por sí solo el requisito de bienvenida ni convierte un valor arbitrario en una credencial válida.
- Permitir omitir el token de Telegram y configurarlo más adelante. Su ausencia no debe bloquear la bienvenida, el guardado de la configuración ni la administración web.
- Guardar claves y tokens mediante el backend en almacenamiento local privado; mantenerlos fuera del historial, memoria, skills y repositorio. La web debe permitir reemplazarlos sin recuperar y mostrar el secreto completo guardado.
- Exigir una comprobación real de conexión antes de completar la bienvenida: realizar una solicitud mínima de inferencia al proveedor, endpoint y modelo seleccionados con la credencial introducida, y comprobar que produce una respuesta válida. Guardar una clave o consultar un listado de modelos no sustituye esta prueba. Si falla, mostrar el motivo sin revelar secretos y permitir corregir o reintentar; no marcar la instalación como lista. Esta prueba comprueba esa configuración, no los otros 16 proveedores ni todas las capacidades del agente.
- Administrar desde la web proveedores y credenciales, Telegram, memoria explícita con búsqueda léxica, skills manuales, tareas, recordatorios y programaciones. Mostrar los estados pertinentes: configurado/conectado, habilitado/deshabilitado y pendiente/ejecutando/bloqueado/completado/fallido según el recurso.
- No pedir presupuesto máximo de tokens. Usar desplegables y formularios para configuración, manteniendo las aprobaciones, rechazos, preguntas y cancelaciones del agente exclusivamente textuales.
- Incluir una interfaz de chat dentro de la web, conectada a la conversación única. La bienvenida, el chat y la administración deben compartir el mismo diseño visual.

### 1.13. Diseño visual y temas

- Usar una identidad cálida: fondo marfil, superficies blancas, texto carbón, acciones terracota y acentos verde petróleo, con tipografía legible y jerarquía clara. Véase `docs/es/interface.md`.
- Ofrecer modo claro y modo oscuro con un selector de tema accesible desde la interfaz.
- La primera visita debe usar modo claro, con fondo marfil y texto oscuro, incluso si el sistema operativo está en modo oscuro. El tema y el idioma elegidos se conservan en el login y en la aplicación.
- Guardar la elección explícita del usuario y recuperarla en visitas posteriores. Sin elección guardada, usar siempre modo claro; evitar un destello oscuro antes de aplicar el tema.
- Mantener legibilidad, contraste y estados de foco en ambos temas; comunicar errores y estados también mediante texto o iconos, sin depender únicamente del color.

### 1.14. Approved tool catalog / Catálogo de herramientas aprobado

The base catalog contains 17 tools; section 1.15 adds delegation controls for 20 master tools and 16 child tools. Names are internal; the owner interacts through text. Each tool has a typed argument schema, per-operation permissions, bounded output, and a normalized result. Register the catalog centrally and expose the tools relevant to the task. Logging, IDs, queue management, authorization enforcement, and channel delivery remain runtime responsibilities, not optional model calls.

El catálogo básico se amplía con delegación, proyectos, planificación y búsqueda web según las secciones siguientes. El principal y los hijos reciben las herramientas pertinentes a su función. Sus nombres son internos; el propietario interactúa por texto. Cada herramienta tiene un esquema de argumentos, permisos por operación, salida acotada y resultado normalizado. Registrar el catálogo de forma centralizada y exponer las herramientas pertinentes a la tarea. Registros, IDs, cola, comprobación de permisos y entrega al canal son responsabilidades del programa, no llamadas opcionales del modelo.

| # | Tool | Purpose | Función |
| --- | --- | --- | --- |
| 1 | `files_list` | List files and directories in the configured file scope. | Listar archivos y carpetas del ámbito configurado. |
| 2 | `file_read` | Read a file or bounded fragment. | Leer un archivo o fragmento acotado. |
| 3 | `files_search` | Search local paths/content and return located matches. | Buscar rutas o contenido local con ubicación de las coincidencias. |
| 4 | `file_write` | Create or replace a file when authorized. | Crear o sustituir un archivo cuando esté autorizado. |
| 5 | `file_edit` | Apply targeted changes after checking expected content. | Aplicar ediciones puntuales comprobando el contenido esperado. |
| 6 | `command_run` | Run a command in any OS-accessible directory with timeout and bounded output. | Ejecutar un comando desde cualquier directorio accesible para el usuario de Linux, con timeout y salida acotada. |
| 7 | `command_status` | Inspect a Fénix-managed command and retrieve available output. | Consultar el estado y salida disponible de un comando administrado por Fénix. |
| 8 | `command_cancel` | Stop a process started and managed by Fénix. | Detener un proceso iniciado y administrado por Fénix. |
| 9 | `memory_search` | Search explicit memory by words, title, or ID. | Buscar recuerdos explícitos por palabras, título o ID. |
| 10 | `memory_manage` | Read, add, correct, or delete explicit memory records. | Consultar, añadir, corregir o eliminar recuerdos explícitos. |
| 11 | `history_search` | Search conversation text with simple filters. | Buscar texto de la conversación con filtros sencillos. |
| 12 | `history_read` | Retrieve a bounded history segment with its context. | Recuperar un segmento acotado del historial con su contexto. |
| 13 | `skill_load` | Load an installed, enabled skill. | Cargar un skill instalado y habilitado. |
| 14 | `schedule_manage` | Create, read, edit, pause, resume a paused schedule, or cancel reminders and schedules. | Crear, consultar, editar, pausar, volver a habilitar o cancelar recordatorios y programaciones. |
| 15 | `task_inspect` | Inspect task state, errors, execution records, and paginated stored outputs. | Consultar estados, errores, registros y salidas guardadas por partes. |
| 16 | `ask_user` | Ask for missing information and track the pending response. | Pedir información que falta y registrar la respuesta pendiente. |
| 17 | `request_approval` | Request textual authorization for a specific action and scope. | Solicitar autorización textual para una acción y alcance concretos. |
| 18 | `web_search` | Search the local public index and optionally read the top sources. | Buscar en el índice público propio y leer las primeras fuentes. |
| 19 | `web_read` | Read a known public HTTP(S) source into a bounded passage. | Leer una fuente pública HTTP(S) y devolver un fragmento acotado. |

Resuming a paused schedule means enabling its future runs; it does not implement recovery of interrupted tasks or catch-up of missed runs. `task_inspect` is read-only and does not restore previous actions. Merging the memory/reminder skills does not merge or remove the underlying tools.

Volver a habilitar una programación pausada activa sus ejecuciones futuras; no implica recuperar tareas interrumpidas ni ejecuciones perdidas. `task_inspect` sólo consulta y no restaura acciones anteriores. Unir los skills de memoria y recordatorios no une ni elimina sus herramientas subyacentes.

### 1.15. Background subagents / Subagentes en segundo plano

- The master delegates asynchronously using a precise objective and expected output. Context and constraints are optional; do not require an “information needed” field or copy the entire conversation. The master remains available for queued requests while children work or wait for textual approval.
- Enforce one delegation level in the engine, with at most four concurrent child inference requests. Child tools cannot delegate or read conversation history; cross-task inspection is limited to authorized plan evidence. Children may search explicit memory when relevant; they do not receive it automatically.
- Give each child an isolated persisted transcript and its own task ID linked to the parent. Preserve context and completed tool results across at most three total attempts for retryable failures. Default to 128 model steps per task across retries/follow-ups and one hour per attempt, configurable in execution settings. Do not replay tool actions; finish early for non-retryable failure, blockage or cancellation. These are operational limits, not scientifically optimal constants.
- Require a structured `subagent_finish` report with outcome, evidence and pending work. Persist the report and return it to the master automatically; the master checks it against the expected output. Restarting interrupts unfinished children without automatic replay or recovery.
- Show purpose, expected output, state, child/parent IDs, attempts and results in the existing Tasks view. List simultaneous pending requests with their IDs. Approvals, rejection, answers and cancellation stay textual: `aprobar <request_id>`, `rechazar <request_id>`, `responder <request_id> <answer>` and `cancelar <task_id>`.
- The master and every child see all valid skill metadata, including disabled entries. Load enabled bodies only through `skill_load`; do not silently truncate the catalog. Store shared packages in `~/.fenix/skills`, allowing `--skills-dir`. Copy old `<data-dir>/skills` resources without overwriting destinations or following symbolic links, leaving originals intact. Bound migration resources and report limit failures.
- Serialize file, memory and schedule mutations and managed command lifetimes within a runtime. This is process-local coordination, not a Linux shell sandbox or coordination with separate Fénix instances. Block model/permission configuration changes while tasks are active.

- El principal delega de forma asíncrona con objetivo y resultado esperado; contexto y restricciones son opcionales, sin exigir “información necesaria” ni copiar toda la conversación. Sigue atendiendo solicitudes mientras los hijos trabajan o esperan aprobación.
- El motor impone un nivel de delegación y cuatro inferencias simultáneas de hijos. Cada hijo tiene ID, vínculo al padre y contexto persistido independiente. No puede delegar ni leer el historial principal; la inspección de otras tareas se limita a evidencia autorizada del plan; puede consultar memoria explícita pertinente mediante herramientas.
- Limitar a tres intentos totales ante fallos reintentables. Por defecto, permitir 128 pasos de modelo por tarea entre reintentos y seguimientos y una hora por intento, configurables desde Ajustes, conservando contexto y resultados sin repetir acciones. Detenerse antes ante fallo definitivo, bloqueo o cancelación. Son límites operativos, no constantes científicamente óptimas.
- Exigir informe estructurado con `subagent_finish`, guardar resultados y devolverlos al principal para comprobar la entrega. Reiniciar interrumpe los hijos inconclusos sin reanudar ni repetir acciones automáticamente.
- Mostrar propósito, entrega, estados, IDs, intentos, resultados y solicitudes pendientes en Tareas. Las respuestas y cancelaciones se escriben en la conversación usando los IDs correspondientes, sin botones de aprobación o rechazo.
- Principal e hijos ven todos los metadatos válidos de skills, incluidas las deshabilitadas; sólo `skill_load` carga instrucciones habilitadas. Usar `~/.fenix/skills` o `--skills-dir`, migrar recursos anteriores por copia sin sobrescribir ni seguir enlaces y conservar los originales.
- Serializar efectos dentro del programa, incluidos comandos hasta su finalización; las herramientas conservan los permisos del sistema y no constituyen un sandbox. No cambiar modelo o permisos mientras haya tareas activas.

### 1.16. Projects and general conversation / Proyectos y conversación general

- Keep one visible web/Telegram conversation with distinct general, project and task contexts. Provide a project registry with stable IDs, unique names/aliases, descriptions, versioned briefs and optional existing workspaces; include web create/edit/progress management.
- Resolve each request before project context or execution tools are available. Use deterministic explicit references where sufficient; use the configured model for ambiguous or compound language, with bounded catalog/recent-reference input. Account for routing token usage. Require a textual clarification for unresolved destinations; invalid routing must not execute effects.
- Capture reply references and foreground hints on message arrival. A resolved task binds its destination, readable scopes, workspace and context version. Later topic switches, child reports, approvals and project edits must not retarget an existing execution.
- Preserve ordered intents in mixed requests. The master handles simple state queries and general conversation; substantial project work can be delegated with an explicit destination and precise expected output. Cross-project coordination may consult authorized scopes but must delegate project mutations with a fixed destination. Children cannot delegate again.
- Apply scope filters before lexical memory/history retrieval and writes. Keep general memory, project facts, summaries and original evidence separate; retain relevant general conversational references without loading every project's transcript.
- Keep late child results and pending replies bound to their original task/project. A web Reply link or Telegram reply-to message anchors a request without creating another chat or granting approval by itself.
- Preserve existing database contents. Legacy messages and memories remain general; do not guess project ownership or automatically reclassify history during migration. No graph, embeddings, autonomous extraction or automatic task recovery is required.
- Validate routing and isolation with deterministic and mocked-provider tests, real HTTP/browser flows, general/project switches, multi-intent ordering, reply references and late background completion. Do not describe mocked tests as live-model benchmark results.

- Conservar una conversación visible en web/Telegram, con ámbitos general, por proyecto y por tarea. Registrar IDs, nombres/alias únicos, descripción, contexto versionado y carpeta opcional existente; administrar proyectos y progreso en la web.
- Resolver el destino antes de cargar su información o habilitar herramientas. Usar referencias explícitas con reglas y recurrir al modelo configurado para lenguaje compuesto o ambiguo, registrando su consumo. Pedir aclaración por texto si falta destino; un enrutamiento inválido no debe producir efectos.
- Capturar referencias de respuesta y foco al recibir el mensaje. Vincular destino, ámbitos consultables, carpeta y versión a cada ejecución. Cambios de tema, informes tardíos, aprobaciones o ediciones del proyecto no deben cambiar esa asignación.
- Conservar el orden de las peticiones. El maestro atiende conversación y consultas sencillas; puede delegar trabajo extenso con propósito y entrega precisa. La coordinación entre proyectos consulta ámbitos autorizados y delega cambios con destino fijo, sin un segundo nivel de subagentes.
- Aplicar filtros antes de recuperar o guardar memoria e historial. Separar información general, hechos particulares, resúmenes y evidencia original. Las referencias a tareas anteriores no requieren cargar todos los historiales de proyectos.
- Mantener resultados y respuestas pendientes asociados al origen. Responder a un mensaje desde web o Telegram no crea otro chat ni constituye aprobación por sí mismo.
- Conservar los datos previos como ámbito general sin deducir retrospectivamente su proyecto. Grafos, embeddings, extracción autónoma y recuperación automática siguen fuera del alcance actual. Documentar los límites de las pruebas simuladas y medir el comportamiento de modelos reales por separado.

Details / Detalles: [English](docs/en/projects.md), [Español](docs/es/projects.md).

Persistent project contracts / Contratos persistentes por proyecto:

- Keep stable, versioned behavior requirements separate from task plans and execution evidence. Workers propose sourced add/modify/remove deltas; only the scoped master consolidates or discards them. / Conservar requisitos versionados separados de los planes y evidencias. Los subagentes proponen cambios con procedencia; el maestro del proyecto consolida o descarta.
- Detect stale revisions and overlapping requirement IDs transactionally, including before integrating declared worktree changes. Load requirements on demand; do not require proposals for routine actions or claim semantic conflict detection. / Detectar revisiones obsoletas y solapamientos de IDs, también antes de integrar cambios declarados en worktrees. Consultar bajo demanda, sin propuestas obligatorias para acciones rutinarias ni prometer detección semántica.
- Implementation details / Detalles: [English](docs/en/project-contracts.md), [Español](docs/es/project-contracts.md).

### 1.17. Editable planning and asynchronous events / Planificación editable y eventos asíncronos

- Persist plans with an objective, explicit milestone outcomes, dependencies and versions. Dispatch ready milestones through one-level subagents with the complete on-demand skill catalog.
- Surface every detected unresolved doubt, including minor ones. Save question IDs and explicit answers; do not silently assume or re-ask resolved questions. Waiting releases execution capacity without consuming active execution time.
- Keep the conversation available during drafting, execution and questions. Accept textual edits at any point; version-check updates, preserve valid completed work, invalidate superseded generations and reconcile late results.
- Publish durable events and consume them using subscriber cursors. Preserve task/project/channel destinations; resume web delivery through SSE cursors and retry unacknowledged Telegram output with internal deduplication.
- Keep prior effects and evidence. A plan edit is not rollback; apply cancellation barriers before declaring the revision effective. Interrupted actions do not resume automatically after restart.

- Guardar planes con objetivo, resultados esperados por hito, dependencias y versiones. Ejecutar los hitos listos mediante subagentes de un nivel y catálogo de skills bajo demanda.
- Comunicar toda duda detectada, incluso menor. Guardar IDs y respuestas explícitas; no asumir respuestas ni repetir preguntas resueltas. La espera libera capacidad y no consume tiempo activo.
- Mantener disponible el chat durante elaboración, ejecución y preguntas. Aceptar cambios textuales en cualquier momento, comprobar versiones, conservar avances válidos e invalidar generaciones y resultados sustituidos.
- Usar eventos durables con cursores, destinos fijos por tarea/proyecto/canal, reconexión SSE y reintentos de Telegram con deduplicación interna.
- Conservar efectos y evidencia anteriores. Editar no revierte acciones; comprobar la detención antes de declarar aplicada una revisión. No reanudar efectos automáticamente al reiniciar.

Details / Detalles: [English](docs/en/planning.md), [Español](docs/es/planning.md).

### 1.18. Independent web search / Búsqueda web propia

- Provide query-only `web_search` to master and children, with optional result/read limits, and `web_read` for a known public URL. Use a background crawler, persistent URL frontier and separate SQLite FTS5 public corpus; no external search provider or search API key.
- Bootstrap from configurable seeds and follow permitted links across origins. Accept arbitrary topics while explicitly reporting bounded, initially empty coverage. Read source pages by HTTP, cache and conditionally refresh them, return URLs, excerpts, hashes and retrieval times, and distinguish failed reads from verified evidence.
- Bound network/parser concurrency, downloaded bytes, stored text, corpus/frontier size and daily attempts. Honor robots and indexing policies, validate public DNS and redirects, reject credentials and private addresses, and propagate shutdown/task cancellation. HTTP indexing does not execute JavaScript; interactive rendering uses the separate local browser.
- Ofrecer `web_search` y `web_read` al principal y a los hijos, con índice público separado en disco y rastreo en segundo plano. Las semillas son puntos de partida ampliables mediante enlaces, sin proveedores externos de búsqueda.
- Mostrar cobertura real, fuentes, caché y fallos de lectura; acotar memoria de trabajo, tráfico y almacenamiento. Respetar políticas de rastreo, comprobar DNS y redirecciones y tratar el contenido web como datos no confiables. La navegación interactiva usa el navegador local separado.

- Expose our authenticated `POST /api/web/search` and integrate it in the Browser address bar. Add domain/date filters, bounded source ingestion, duplicate removal and domain diversity; publication dates must come from explicit source metadata. Do not add external search services or metasearch fallback. / Exponer una API propia e integrarla en la barra del navegador, con filtros, fuentes conocidas, duplicados y diversidad, sin proveedores externos ni metabuscadores.

Details / Detalles: [English](docs/en/web-search.md), [Español](docs/es/web-search.md).

### Criterio de cierre del alcance actual

El asistente debe poder ejecutarse localmente en Linux para un propietario, ofrecer una web en localhost con bienvenida que requiera una clave API de proveedor y permita omitir Telegram, administrar la configuración y los recursos definidos, conservar una única conversación, encolar peticiones nuevas, solicitar aprobación y aclaraciones por texto, usar herramientas autorizadas y comprobar resultados. También debe recordar una preferencia explícita después de reiniciar, ejecutar una programación persistida, permitir inspeccionar lo ocurrido, corregir su memoria básica y detenerse ante cancelación, error o límites operativos. Al conectar Telegram debe mantener la misma conversación. La búsqueda web debe rastrear e indexar fuentes públicas en segundo plano y leer resultados con cobertura y evidencia explícitas. El alcance actual puede cerrarse sin presupuesto máximo de tokens de la aplicación, heartbeat, recepción de adjuntos, reanudación automática de tareas, componentes de memoria avanzada ni orquestación de más de un nivel de subagentes.

## Adaptive plan review / Revisión adaptativa de planes

- Review plans initially when they have at least three milestones, dependencies,
  explicit review reasons, assumptions or open questions. Use selective review for
  new contradictions and blocked execution; avoid per-tool LLM review.
- Keep confirmed requirements with sources, observations, assumptions and questions
  separate. Clarify broad requests without inventing scope or repeating answered doubts.
- Use the master's model for one read-only principal and at most two independent
  specialists. Keep one delegation level, on-demand skills and a responsive chat.
- Resolve objections using evidence or owner answers, not votes. Allow one master
  response and a final focused check after correction; unresolved findings block the
  affected scope. Owner edits remain available and invalidate obsolete reviews.
- Persist reports, resolutions, versions and provider-reported usage. Surface status
  and details in the web and notifications in the originating channel.
- Revisar inicialmente planes con al menos tres hitos, dependencias, motivos explícitos,
  suposiciones o preguntas. Revisar selectivamente contradicciones y bloqueos, evitando
  llamadas de revisión por cada herramienta. Separar requisitos con procedencia,
  observaciones, supuestos y dudas; aclarar instrucciones generales sin inventar alcance.
- Usar el modelo del maestro para un principal de solo lectura y hasta dos especialistas
  independientes. Mantener un nivel de delegación, skills bajo demanda y el chat disponible.
  Resolver con evidencia o respuestas del usuario, permitir una respuesta del maestro y
  una comprobación final; bloquear objeciones pendientes y admitir ediciones en cualquier
  momento. Guardar informes, versiones, resoluciones y consumo; mostrarlos en web y avisar
  al canal original. Detalles: [English](docs/en/plan-reviews.md) / [Español](docs/es/plan-reviews.md).

## Live chat updates / Actualizaciones del chat

- Show provisional streamed text for OpenAI Responses and runtime-generated tool activity for every provider. Other model adapters and Telegram keep complete-message delivery initially.
- Reconnect to the current preview without repeating inference, preserve task scope and textual approvals, and clear provisional output after completion, failure or cancellation.
- Keep progress bounded and redacted, outside model context and durable conversation history; do not ask the model to generate status messages.
- Mostrar texto progresivo con OpenAI Responses y actividad de herramientas para todos los proveedores. Los demás adaptadores y Telegram mantienen mensajes completos inicialmente.
- Reconectar sin repetir inferencias, mantener ámbitos y aprobaciones textuales y retirar la vista provisional al completar, fallar o cancelar.
- Mantener progreso acotado y redactado, fuera del contexto y del historial durable; los estados se generan en el motor.


## Repository-aware coding / Programación con contexto de repositorio

- Implement regex/glob search and exact-context multi-file patches with existing approval, project and effect controls.
- Load workspace and nested AGENTS.md guidance before mutations; do not inherit unrelated project rules.
- Provide explicitly configured, lazy stdio LSP queries and diagnostics, with bounded sessions and cancellation.
- Support file-scoped symbol outlines, exact/suffix name-path lookup, selective symbol bodies and optional permission-checked reference excerpts. Cache bounded symbol responses, refresh open buffers and allow independent workspace/worktree LSP sessions to query concurrently. Keep edits on the existing approved file-tool path. / Consultar símbolos por nombre y leer solo el cuerpo necesario, con referencias contextualizadas, caché acotada, actualización de documentos abiertos y consultas paralelas entre proyectos; conservar las herramientas de edición y aprobaciones actuales.
- Compact active master and worker contexts at complete tool boundaries, preserving durable originals, task/session IDs and recent context. Summaries must not replay effects or reset execution limits.
- Expose coding configuration in the local web and document capabilities and limits in [English](docs/en/coding-harness.md) and [Spanish](docs/es/coding-harness.md).

Búsqueda regex/glob, parches, instrucciones por repositorio, LSP configurable y compactación durante la tarea forman parte del alcance implementado. Los checkpoints permiten consultar el contexto original; no implican rollback ni reanudación automática tras reinicios.

## Execution harness update / Actualización del motor de ejecución

Implemented: interactive PTY/stdin, explicitly persistent services, cursor-based redacted logs, configurable execution limits, same-context subagent follow-ups and deterministic on-demand tool discovery. See [English](docs/en/execution-harness.md).

Implementado: PTY/entrada interactiva, servicios persistentes explícitos, registros redactados con cursor, límites configurables, seguimientos de subagentes con el mismo contexto y descubrimiento determinista de herramientas bajo demanda. Ver [Español](docs/es/execution-harness.md).

Parallel Git worktrees with serialized local integration are implemented: [English](docs/en/parallel-worktrees.md) / [Español](docs/es/parallel-worktrees.md). Automatic crash recovery, MCP and programmatic tool composition remain future work. / Los worktrees de Git y la integración local serializada están implementados; la recuperación automática tras reinicios, MCP y la composición programática de herramientas siguen pendientes.


## Local browser / Navegador local

The Rust runtime must control local Chromium with its sandbox enabled, without a hosted browser provider. Include manual owner controls, persistent personal profiles, explicit agent access, task-isolated test profiles, responsive viewports, bounded observations and screenshots, condition checks, encrypted owner-only password storage and process-memory monitoring. A skill guides tool use; actual actions execute in Rust. See [the browser contract](docs/en/browser.md) for guarantees and limits.

El navegador local incluye controles manuales, perfiles personales persistentes y pruebas temporales separadas. El propietario habilita el acceso del agente y guarda contraseñas en un almacén cifrado. Las sesiones pueden caducar según cada sitio. El presupuesto de RSS es periódico y flexible, no un límite estricto de RAM; las capturas todavía no son entrada visual del modelo. Véase [el contrato en español](docs/es/browser.md).

## Procedural coding and recovery / Procedimientos y recuperación

Derive a bounded, deterministic operational snapshot from existing plans/contracts for long work and relevant failure recovery. Persist deduplicated failed-tool observations and sourced agent hypotheses in the existing task contract; carry only authorized predecessor/prior-generation evidence to replacement workers. Reserve snapshot space during compaction, invalidate stale next-step notes after plan/owner changes, and avoid duplicate conversation entries or extra summary-model calls. / Generar un resumen operativo acotado desde los planes y contratos existentes; conservar fallos e hipótesis con procedencia, aislar la información entre tareas y actualizar el siguiente paso sin duplicar planes ni acumular mensajes.

Search saved, redacted tool outputs through `task_inspect(query)` using local lexical retrieval, exact source offsets and existing task permissions. Report incomplete indexing explicitly; retain the authoritative journal and exclude inspection results from the index. / Buscar salidas guardadas con búsqueda léxica local, fragmentos exactos y permisos por tarea; señalar cobertura incompleta, conservar los originales y evitar indexación recursiva.

Provide fourteen curated manual skills, lexical field-aware skill discovery, selective loading of authorized tool schemas, actionable argument errors, bounded failure-specific recovery and checkpoint-backed context reduction. Allow independent file writes alongside Linux-enforced read-only commands with explicit inputs; preserve ordinary command policies and conservative handling of unknown effects. Reuse editable plans, nonblocking supervision and one-level delegation. See [English](docs/en/harness-recovery.md) / [Español](docs/es/harness-recovery.md).

Incluir catorce skills manuales, selección léxica, esquemas bajo demanda, errores accionables y recuperación limitada según la causa. Conservar estado y procedencia al compactar. Permitir escrituras independientes durante lecturas restringidas por Linux sin ampliar permisos ni crear delegación anidada.

## Browser voice and Live / Voz del navegador y Live

- Build the web with Vite, React, TypeScript, Tailwind and pnpm; Rust/Axum serves the compiled assets without a Node server. / Compilar la web con Vite, React, TypeScript, Tailwind y pnpm; Rust/Axum sirve los archivos sin servidor Node.
- Keep existing administration, chat, project, memory, skill, scheduling and browser features. / Mantener las funciones existentes de administración, chat, proyectos, memoria, skills, programaciones y navegador.
- Dictation fills a reviewable draft. Live uses browser STT/TTS and a separately configured conversational model, defaulting to Nemotron. / El dictado llena un borrador revisable. Live usa STT/TTS del navegador y un modelo conversacional independiente, Nemotron por defecto.
- Group observable progress into natural updates; automatic narration cannot authorize actions. Explicit user requests reach the master's normal APIs with the original wording. / Agrupar el progreso observable en novedades naturales; la narración automática no autoriza acciones. Las instrucciones explícitas llegan a las APIs normales del maestro con el texto original.
- Preserve one shared conversation. Navigation and hanging up must not cancel master tasks, plans or subagents. / Conservar una conversación compartida. Navegar y colgar no deben cancelar tareas, planes ni subagentes.
- Unsupported recognition keeps text chat usable. Microphone permission, echo cancellation and interruption behavior need real-device verification. / Si el reconocimiento no está disponible, el chat escrito sigue funcionando. Los permisos, la cancelación de eco y las interrupciones requieren verificación en dispositivos reales.

## Design and visual results / Design y resultados visuales

- Provide Templates, Favorites, My designs and Brand identity, with an initial catalog of 64 original examples. / Ofrecer Plantillas, Favoritos, Mis diseños e Identidad visual, con 64 ejemplos propios.
- Display diagrams, web interfaces, presentations, charts, documents, tables, images and video in a desktop side panel and a mobile fullscreen view. / Mostrar los ocho formatos en un panel lateral o a pantalla completa en móvil.
- Let the agent discover templates, present ordered choices and publish or import project-scoped results; preserve the same conversation. / El agente puede ofrecer opciones y publicar resultados asociados a proyectos sin crear conversaciones separadas.
- Persist favorites, branding, artifacts and optimistic revisions per user. Preserve previous revisions when restoring. / Persistir preferencias, identidad, resultados y versiones por usuario; restaurar crea otra versión.
- Document actual format limitations, including Office content extraction and disabled scripts/network in HTML previews. / Documentar límites reales, incluida la extracción Office y el aislamiento de HTML.
- Implementation contract: [English](docs/en/design.md) · [Español](docs/es/design.md).

## Engine security boundaries / Protección en el motor

- Redact registered and recognizable secrets before provider requests, streamed previews and durable memory; keep API authentication separate and reject unsafe opaque replay without rewriting signatures. / Ocultar secretos antes de enviarlos al modelo, mostrarlos o guardarlos en memoria; separar autenticación y rechazar continuidad opaca insegura sin modificar firmas.
- Allow operator-owned destination lists per execution profile/account and exact host lists for the native web reader, checking redirects and preserving private-network denial. / Admitir destinos configurados por perfil/usuario y listas de hosts en el lector web, comprobando redirecciones y manteniendo el bloqueo de redes privadas.
- Screen protected instruction writes and loads; preserve suspicious research content as reference data and prevent its automatic promotion to confirmed memory or instruction context. / Revisar escrituras y cargas de instrucciones; conservar referencias sospechosas sin promocionarlas automáticamente a memoria confirmada o instrucciones.
- Configuration, enforcement limits and validation: [English](docs/en/security-boundaries.md) · [Español](docs/es/security-boundaries.md).
