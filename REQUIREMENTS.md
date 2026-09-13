# Fénix — Especificación de requisitos

## Current scope decisions / Decisiones de alcance actuales

- Maintain one continuous conversation for the owner. Telegram support ships initially, but its token and activation are optional. Do not expose separate chats, conversation creation, session switching, or resets as the normal workflow. Internal task IDs and context compaction must preserve that continuity.
- Initially accept text only; document, file, image, audio, and video attachments are outside scope.
- Web search, page fetching, generic HTTP tools, and all browser functionality belong to `ROADMAP.md`, section 2.22. Model-provider transport and optional Telegram transport remain in scope.
- Use `America/Lima` initially. Automatic resumption of interrupted tasks and catch-up of missed scheduled executions remain future work; keeping history and schedules does not imply replaying their actions.
- Initially support Linux and one owner per installation. The project must be clonable and run locally, with its web interface bound to loopback by default; routine use must not require a terminal.
- On first visit without configuration, show a welcome/setup screen. Require at least one provider API key and a successful real inference request to the selected provider/model before completing setup, with provider-dependent selectors and fields covering all 17 integrated providers. Telegram setup can be skipped and completed later.
- The web must include chat and manage provider credentials, Telegram, explicit memory, skills, scheduled tasks, reminders, and their states. Web and optional Telegram use the same continuous conversation and queue.
- Use a minimal black-and-white visual design inspired by Vercel, with neutral grays and light/dark themes. The first visit must use light mode with a white background, regardless of the operating-system theme. Persist an explicit user theme selection for later visits.
- New conversational requests enter a FIFO queue, including corrections; do not steer or interrupt an active task automatically. Approvals, rejections, clarification questions, and cancellations are textual, without action buttons. A clear answer to a pending question must resolve that question rather than wait behind its blocked task. Ambiguous text must not grant permission.
- Store manually installed skills at `.fenix/skills/<skill-name>/SKILL.md`, relative to the installation root, with an optional configured skills directory. Keep personal runtime data out of Git.
- Command execution may use any working directory accessible to the Linux user running Fénix; do not restrict it to a project workspace or request approval merely because the directory is outside that workspace. Keep explicit working directories, OS permissions, action-specific approvals, timeouts, and process controls. A file-tool workspace is not a sandbox boundary for this broader command capability.
- The initial catalog is approved: 17 tools and four manual default skills. Merge explicit memory and reminders/scheduling into `memory-and-reminders`; retain `local-file-work`, `conversation-recap`, and `local-troubleshooting`.
- Current agent provider/model: OpenAI `gpt-5.6-luna` through Responses, with `reasoning.effort: "low"`, replacing the earlier Astra selection at the owner's request for initial development and verification. The request contract and Responses adapter implement this setting. No automatic model switching is required.
- Heartbeat is future work in `ROADMAP.md`, section 2.23. Cron and explicit reminders remain in scope. Provide an approval-request tool backed by mandatory runtime permission checks.
- Do not implement an application-level maximum token budget or ask for one in initial setup. Provider context/output limits, token usage reporting, timeouts, bounded retries, and basic loop prevention are distinct from that deferred budget feature.

## Alcance del documento

Este documento define el alcance actual de Fénix: un asistente personal funcional, con un único agente principal, conversación por texto, herramientas autorizadas, memoria básica explícita y controles operativos. Estas son las capacidades que se deben construir y validar primero.

- Incluir soporte de Telegram desde el comienzo como único canal externo de mensajería, con token y activación opcionales. La instalación debe poder completarse y la web funcionar sin configurar Telegram. Mantener una única conversación continua del propietario, sólo por texto.
- Uso personal: un propietario autorizado por instalación. Cualquier persona debe poder clonar el proyecto y ejecutar su propia instalación local, con datos y credenciales independientes.
- Soportar inicialmente Linux. Incluir una web en localhost, vinculada por defecto sólo a loopback, con bienvenida, configuración de proveedores y Telegram, memoria, skills, programaciones y estados. El uso habitual no debe requerir terminal; no se requiere una plataforma SaaS ni multiusuario compartida.
- Toda búsqueda en internet, lectura de páginas, herramienta HTTP genérica y funcionalidad de navegador queda para el trabajo futuro de la sección 2.22 de `ROADMAP.md`. La conexión con modelos y el canal opcional de Telegram siguen dentro del alcance.
- El heartbeat, la memoria avanzada, la autonomía prolongada, la voz en tiempo real, los subagentes y el resto de ampliaciones se conservan en [ROADMAP.md](ROADMAP.md).
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
- Mantener una sola ejecución activa. Encolar cada nuevo mensaje de conversación por orden de llegada (FIFO), incluso si contiene una corrección; procesarlo cuando termine la tarea activa, sin incorporarlo automáticamente a mitad de ejecución.
- Tramitar por texto las aprobaciones, rechazos, aclaraciones y cancelaciones; no usar botones para esas acciones. Cuando el agente espere una respuesta, vincular una respuesta clara a esa solicitud pendiente y continuarla sin esperar al final de la cola. Las nuevas peticiones independientes siguen en FIFO. Si el texto es ambiguo, pedir aclaración sin asumir autorización.
- Incluir chat en la web para usar el asistente sin Telegram. Debe mostrar y utilizar la misma conversación y cola, sin crear un segundo chat independiente; activar Telegram después no crea una conversación nueva.
- Conservar un espacio de trabajo explícito. El enrutamiento automático entre varios proyectos simultáneos pertenece al trabajo futuro y deberá conservar el mismo chat.

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
- Construir el contexto con las instrucciones estables, el perfil necesario, memoria explícita pertinente, un resumen si hace falta y los mensajes recientes.
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
- No habilitar herramientas de búsqueda web, descarga de páginas, HTTP genérico o navegación en el alcance inicial; se conservan en el roadmap.
- Gestionar memoria explícita, consultar estado de tareas y crear o cancelar programaciones.
- Definir un registro extensible de herramientas con nombre, descripción, esquema de argumentos, permisos y resultado normalizado. Habilitar sólo las herramientas necesarias para el turno.
- Dejar preparados los límites de integración para nuevas herramientas o MCP. Toda navegación web y el arnés de programación se incorporarán más adelante.

### 1.7. Permisos y aislamiento básico

- Restringir quién puede utilizar el asistente y qué herramientas y operaciones tiene habilitadas. Las herramientas específicas de archivos conservan su ámbito configurado; los comandos pueden operar en cualquier directorio accesible según los permisos del usuario de Linux.
- Aplicar las comprobaciones antes de ejecutar y los límites de proceso correspondientes. No imponer confinamiento de los comandos a una carpeta de proyecto ni solicitar autorización sólo por salir de ella. El espacio de las herramientas de archivos no constituye una frontera de aislamiento para esta capacidad más amplia de ejecutar comandos.
- Solicitar aprobación cuando una acción exceda la autorización vigente o lo exija su política; conservar esa autorización dentro de su alcance.
- Ofrecer una herramienta para solicitar aprobaciones textualmente en la conversación activa, con Telegram cuando esté configurado. Explicar la acción y su alcance; aceptar aprobación o rechazo por texto sin botones, vinculados a la solicitud concreta. El ejecutor debe comprobar los permisos aunque el modelo no solicite aprobación; el silencio y las respuestas ambiguas no autorizan la acción.
- Validar argumentos y tratar documentos, páginas y salidas de herramientas como contenido no confiable.
- Redactar secretos en registros y respuestas. Los sistemas avanzados de permisos entre proyectos y de compensación de efectos se desarrollarán más adelante.

### 1.8. Skills predefinidos y manuales

- Cargar skills instalados o redactados explícitamente, seleccionarlos por su descripción y utilizarlos cuando encajen con la tarea.
- Ubicación predeterminada: `.fenix/skills/<skill-name>/SKILL.md`, relativa a la raíz de la instalación, no al directorio de una tarea. Permitir configurar otra ruta explícita. Cada carpeta puede contener los recursos locales que referencie el skill.
- Mantener `.fenix/` fuera de Git para no publicar skills personales ni datos de ejecución. La ubicación elegida no implica que el cargador o la carpeta ya estén implementados. No instalar skills mediante adjuntos de Telegram ni incorporar aprendizaje automático.
- Cada skill debe explicar objetivo, entradas, pasos, herramientas permitidas, límites, salidas y condiciones de éxito.
- Permitir listar, habilitar y deshabilitar skills, conservando al menos nombre, descripción y versión utilizada.
- Distribuir plantillas predeterminadas versionadas en `resources/skills/` e instalarlas en `.fenix/skills/` durante la configuración inicial sin sobrescribir skills personales existentes. La memoria, los permisos y la persistencia siguen siendo responsabilidades del programa aunque se deshabilite un skill.

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

- Usar una estética minimalista inspirada en Vercel: blanco, negro y grises neutros, tipografía legible, bordes discretos y espaciado claro.
- Ofrecer modo claro y modo oscuro con un selector de tema accesible desde la interfaz.
- La primera visita debe usar modo claro, con fondo blanco y texto oscuro, incluso si el sistema operativo está en modo oscuro.
- Guardar la elección explícita del usuario y recuperarla en visitas posteriores. Sin elección guardada, usar siempre modo claro; evitar un destello oscuro antes de aplicar el tema.
- Mantener legibilidad, contraste y estados de foco en ambos temas; comunicar errores y estados también mediante texto o iconos, sin depender únicamente del color.

### 1.14. Approved tool catalog / Catálogo de herramientas aprobado

The initial catalog contains 17 tools. Names are internal; the owner interacts through text. Each tool has a typed argument schema, per-operation permissions, bounded output, and a normalized result. Register the catalog centrally and expose the tools relevant to the task. Logging, IDs, queue management, authorization enforcement, and channel delivery remain runtime responsibilities, not optional model calls.

El catálogo inicial contiene 17 herramientas. Sus nombres son internos; el propietario interactúa por texto. Cada herramienta tiene un esquema de argumentos, permisos por operación, salida acotada y resultado normalizado. Registrar el catálogo de forma centralizada y exponer las herramientas pertinentes a la tarea. Registros, IDs, cola, comprobación de permisos y entrega al canal son responsabilidades del programa, no llamadas opcionales del modelo.

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

Resuming a paused schedule means enabling its future runs; it does not implement recovery of interrupted tasks or catch-up of missed runs. `task_inspect` is read-only and does not restore previous actions. Merging the memory/reminder skills does not merge or remove the underlying tools.

Volver a habilitar una programación pausada activa sus ejecuciones futuras; no implica recuperar tareas interrumpidas ni ejecuciones perdidas. `task_inspect` sólo consulta y no restaura acciones anteriores. Unir los skills de memoria y recordatorios no une ni elimina sus herramientas subyacentes.

### Criterio de cierre del alcance actual

El asistente debe poder ejecutarse localmente en Linux para un propietario, ofrecer una web en localhost con bienvenida que requiera una clave API de proveedor y permita omitir Telegram, administrar la configuración y los recursos definidos, conservar una única conversación, encolar peticiones nuevas, solicitar aprobación y aclaraciones por texto, usar herramientas autorizadas y comprobar resultados. También debe recordar una preferencia explícita después de reiniciar, ejecutar una programación persistida, permitir inspeccionar lo ocurrido, corregir su memoria básica y detenerse ante cancelación, error o límites operativos. Al conectar Telegram debe mantener la misma conversación. El alcance actual puede cerrarse sin presupuesto máximo de tokens de la aplicación, heartbeat, búsqueda web, navegador, recepción de adjuntos, reanudación automática de tareas, componentes de memoria avanzada ni orquestación de subagentes.

## Live chat updates / Actualizaciones del chat

- Show provisional streamed text for OpenAI Responses and runtime-generated tool activity for every provider. Other model adapters and Telegram keep complete-message delivery initially.
- Reconnect to the current preview without repeating inference, preserve FIFO and textual approvals, and clear provisional output after completion, failure or cancellation.
- Keep progress bounded and redacted, outside model context and durable conversation history; do not ask the model to generate status messages.
- Mostrar texto progresivo con OpenAI Responses y actividad de herramientas para todos los proveedores. Los demás adaptadores y Telegram mantienen mensajes completos inicialmente.
- Reconectar sin repetir inferencias, mantener FIFO y aprobaciones textuales y retirar la vista provisional al completar, fallar o cancelar.
- Mantener progreso acotado y redactado, fuera del contexto y del historial durable; los estados se generan en el motor.
