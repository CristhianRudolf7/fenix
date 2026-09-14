# Arquitectura

[English](../en/architecture.md)

Fénix es un workspace Rust modular que inicialmente se ejecuta en un solo proceso.
La web y Telegram utilizan el mismo motor, la misma cola y una única conversación.

## Componentes

| Componente | Responsabilidad |
| --- | --- |
| `fenix-core` | Contratos de conversación, llamadas/resultados, razonamiento y proveedores. |
| `fenix-credentials` | Claves redactadas y resolución exacta por proyecto, sin fallback. |
| `fenix-providers` | HTTP reutilizable, cinco protocolos y autenticación de proveedores. |
| `fenix-storage` | Migraciones SQLite, proyectos, asignaciones fijas, memoria/historial por ámbito, cola, mensajes, eventos, FTS5 y entregas. |
| `fenix-agent` | Motor, contexto, permisos, herramientas, procesos, cron y adaptador de Telegram. |
| `apps/server` | Ejecutable, API de loopback y archivos estáticos incrustados. |
| `apps/web` | Interfaz en módulos ES, HTML y CSS, sin servidor Node durante el uso. |

El servidor tiene dos hilos de Tokio. Las operaciones de archivos utilizan el pool
de trabajo bloqueante; las llamadas al modelo, Telegram y los comandos utilizan
I/O asíncrona. Una tarea del principal atiende la conversación mientras hasta cuatro
hijos trabajan como tareas Tokio en el mismo proceso. Comparten cliente HTTP,
conexiones y configuración de modelo; no necesitan procesos o navegadores propios.

## Ciclo y permisos

El mensaje se redacta, se guarda y se encola. Una respuesta textual a una pregunta
pendiente se entrega directamente a esa solicitud. El motor toma la tarea más
antigua, resuelve su ámbito, fija la carpeta y construye el contexto correspondiente;
consulta al modelo, valida las herramientas, aplica
permisos, ejecuta y registra sus resultados. Continúa hasta responder, fallar,
cancelarse o alcanzar un límite operativo.

El orden de llegada de mensajes es diferente del orden del contexto del modelo:
un mensaje nuevo aparece en el chat, pero espera a que termine la tarea anterior
para incorporarse al contexto. El registro conserva objetivo, respuestas,
supuestos declarados visiblemente y evidencia de validación. No constituye una
prueba formal del cumplimiento de cada requisito en lenguaje natural.

Los comandos y cambios de archivos piden aprobación por defecto. El motor la
exige aunque el modelo no llame a `request_approval`. Cada aprobación autoriza una
herramienta y argumentos exactos, una sola vez. Se responde `aprobar <id>` o
`rechazar <id>`; un sí/no claro también resuelve la única aprobación pendiente.
Las aclaraciones usan `responder <id> <respuesta>`. Los demás mensajes se encolan.
`cancelar` detiene al principal; `cancelar <task_id>` admite trabajos en cola o
hijos. Varias solicitudes pueden estar pendientes simultáneamente, cada una con su
ID y resolución independiente. La espera de un hijo no pausa al principal.

Las herramientas de archivos validan rutas, protegen datos privados y no crean
archivos encima de otros existentes. La edición exige una coincidencia única y
verifica la escritura. Los comandos pueden usar cualquier directorio accesible al
usuario de Linux; **la carpeta de archivos no es un sandbox de comandos**. Los
procesos gestionados tienen timeout, salida acotada y limpieza de su grupo.

El programa serializa cambios de archivos, memoria y programaciones, y mantiene
un bloqueo exclusivo durante toda la ejecución de cada comando gestionado. Las
lecturas y llamadas al modelo pueden continuar en paralelo. La coordinación cubre
este proceso, no procesos externos ni otras instalaciones, y no constituye un
sandbox. No permite cambiar modelo ni permisos mientras haya trabajo activo.

## Proyectos y conversación general

`projects.rs` resuelve la intención antes de cargar información del proyecto. Las
referencias claras usan reglas; las peticiones compuestas o ambiguas utilizan una
llamada acotada al modelo configurado para obtener una ruta JSON. Consume tokens y
puede fallar. Un destino no resuelto requiere aclaración textual antes de ejecutar.

`project_tools.rs` impone destino y ámbitos consultables en el motor. Los filtros
se aplican antes de recuperar memoria e historial. La carpeta de las herramientas
de archivos queda fijada por tarea. La coordinación consulta proyectos solicitados
y delega ejecución con destino explícito. Los turnos que entregan informes pueden
consultar evidencia y notificar, sin iniciar trabajo de reemplazo ni nuevos efectos.

La conversación general tiene historial y resumen propios. Los turnos principales
de un proyecto reciben además una copia acotada de mensajes generales recientes de
usuario y asistente, sin transcripciones de herramientas ni detalles de otros
proyectos. El contexto operativo y las referencias de tareas aportan continuidad.
Las preferencias globales son explícitas; una escritura de proyecto no puede
promover automáticamente sus hechos a memoria general.

Se capturan referencias de respuesta y foco al recibir cada mensaje. El foco es
una pista, no la carpeta mutable de una ejecución activa. Los resultados tardíos
y aprobaciones conservan su asignación; los informes no cambian el foco. Todo sigue
visible en un mismo chat. La migración conserva los registros antiguos como
generales sin adivinar su propietario. Consulta [la guía de proyectos](projects.md).

## Delegación en segundo plano

`subagent_spawn` guarda propósito preciso y resultado esperado, con contexto y
restricciones opcionales, y devuelve de inmediato el ID del hijo. El principal
continúa y puede atender solicitudes posteriores mientras el hijo trabaja o espera
aprobación. El ejecutor y la base de datos imponen un único nivel: los hijos no
pueden crear hijos. El catálogo varía por rol y ámbito. Los hijos no tienen
historial principal ni delegación y requieren `subagent_finish`; sólo inspeccionan
su propia tarea. Coordinación e informes tienen restricciones adicionales. La
inspección del principal también respeta sus ámbitos consultables.

Un worker toma los trabajos delegados independientemente de la cola del principal.
SQLite guarda vínculos al padre, intentos, contexto independiente e informes. El
hijo recibe su encargo, contexto del proyecto, instrucciones estables, memoria
explícita pertinente del proyecto y general, y todos los metadatos de skills. No
recibe el historial principal; las herramientas de memoria conservan sus filtros. Los informes completos se
guardan y se encolan para el principal; los detalles quedan en el registro del hijo.

Los fallos reintentables admiten hasta tres intentos totales conservando el contexto
y los resultados de herramientas, sin repetir efectos. Cada intento dispone de
32 pasos y diez minutos. Se exige cierre estructurado con `subagent_finish`; un
bloqueo, cancelación o fallo definitivo puede terminar antes. Reiniciar marca los
hijos inconclusos como interrumpidos, sin restaurar su ejecución automáticamente.

`GET /api/status` incluye subagentes y `pending_requests`, conservando `pending` por
compatibilidad. Tareas actualiza propósito, entrega esperada, estados, IDs, intentos
e informes mediante el sondeo habitual. Sólo el principal utiliza la instantánea
SSE, por lo que un hijo no puede reemplazar su respuesta progresiva.

## Persistencia y memoria

SQLite guarda conversación original, metadatos de los proveedores, tareas,
eventos, configuración, recuerdos y programaciones. WAL, transacciones breves y
un bloqueo de instancia evitan dos ejecutores simultáneos sobre los mismos datos.
Las claves no se devuelven en la API de configuración ni en el historial visible.
El directorio de datos conserva base de datos, bloqueo e instrucciones; las skills
usan `~/.fenix/skills` o `--skills-dir`. La biblioteca es compartida, mientras cada
base de datos conserva su estado de habilitación. Al iniciar se copian recursos
antiguos de `<data-dir>/skills`, sin sobrescribir destinos, seguir enlaces ni borrar
originales. Los límites de migración se documentan en [uso local](local-operation.md).

Principal e hijos reciben todos los metadatos válidos, incluidas las skills
deshabilitadas. Las descripciones se acotan a 512 bytes y las versiones a 80,
indicando el truncamiento; no se omiten entradas por un máximo arbitrario.
`skill_load` carga las instrucciones habilitadas bajo demanda, hasta 32 KB.

La memoria explícita guarda título, contenido, origen, proyecto/tarea y fechas. La
normalización de contenido evita duplicados exactos dentro del mismo ámbito; FTS5 utiliza palabras Unicode, búsqueda OR
y ranking. No hay embeddings ni grafos. `MEMORY.md` es una exportación de referencia;
la fuente principal es la base de datos. Los otros cuatro Markdown de instrucciones
se editan desde configuración. Los skills predeterminados no sobrescriben los
archivos personales existentes.

Al comenzar una tarea, si el historial de su ámbito supera aproximadamente 64 KB,
el motor intenta resumir tareas antiguas completas y conservar las recientes.
La conversación general y cada proyecto tienen resumen y cursor separados; los
turnos de coordinación usan su propio contexto de tarea. El
resumen utiliza la estructura acordada: objetivo, restricciones y preferencias,
progreso, decisiones, archivos, próximos pasos y contexto crítico. El cursor sólo
avanza con un resumen completado; el historial original y las parejas de llamadas
y resultados permanecen guardados. Una tarea activa muy grande puede alcanzar el
límite de solicitud y necesitar un pedido posterior más pequeño.

## Programaciones y Telegram

El planificador funciona fuera del modelo, en `America/Lima`. Los recordatorios
entregan texto; las tareas programadas entran en la cola. Se comprueban duplicados
activos y solapamientos. Al reiniciar se omiten ejecuciones vencidas y las tareas
anteriores sin terminar quedan interrumpidas, sin repetición automática.

Telegram usa `getMe`, `getUpdates` y `sendMessage`. Un código aleatorio mostrado
localmente vincula al propietario; grupos, terceros y adjuntos no pueden generar
tareas. El cursor evita redeliveries normales de entradas y la bandeja de salida
registra intentos acotados. Una caída o un resultado de red ambiguo puede duplicar
un mensaje saliente: no se promete entrega exactamente una vez. Las entregas
fallidas permanecen registradas en la base de datos.

## Límites iniciales

- Una tarea principal y hasta cuatro hijos simultáneos, 32 pasos de modelo por
  tarea/intento y hasta 16 herramientas por respuesta.
- Hasta tres intentos totales por hijo ante fallos reintentables, conservando contexto;
  diez minutos por intento, incluida la espera de aprobación.
- Hasta dos reintentos adicionales de inferencia transitoria del principal, sin repetir efectos
  de herramientas automáticamente. No existe presupuesto acumulado de tokens.
- Una hora por tarea principal y 30 minutos de espera para una respuesta del propietario.
- Timeout HTTP de 120 segundos; hasta 4.096 tokens de salida por llamada del agente.
- Solicitud activa de hasta 512 KB, argumentos de herramientas de hasta 128 KB y
  resultados grandes recuperables en páginas.
- Archivos UTF-8 regulares de hasta 1 MiB, con búsqueda y listados acotados.
- Un comando gestionado a la vez por runtime, con límite de registro de ocho, timeout
  de 1–600 segundos y hasta 128 KiB de stdout/stderr combinado. El límite
  predeterminado de comando es 120 segundos.
- Hasta 32 KB por skill y 8 KB por archivo de instrucciones estable.

Las referencias primarias de bibliotecas y APIs están enlazadas en la
[documentación principal](../en/architecture.md).

## Progreso y respuestas progresivas

`GET /api/progress` entrega eventos SSE con las mismas restricciones de loopback.
Un canal watch de Tokio conserva una única instantánea reemplazable con IDs,
estado, herramienta y hasta 32 KB de texto provisional. Reconectar el navegador
recupera esa instantánea sin enviar otra solicitud al modelo. El sondeo habitual
sigue actualizando historial, aprobaciones y cola.

El streaming del proveedor está implementado para OpenAI Responses. Se muestran
fragmentos de texto visible; las herramientas, el consumo y los metadatos de
razonamiento se obtienen de la respuesta terminal. Los otros cuatro protocolos
conservan respuestas completas. Si un gateway devuelve JSON, se procesa la misma
respuesta sin solicitar otra inferencia.

Las vistas provisionales se publican como máximo cada 100 ms y retienen la última
palabra incompleta y los prefijos de claves conocidas antes de redactar secretos.
El texto provisional no se guarda en la base de datos, contexto ni Telegram.
Las herramientas se ejecutan únicamente después de decodificar la respuesta
terminal. Un corte sin evento terminal falla la tarea; no se reintenta la
inferencia automáticamente tras recibir fragmentos de texto. Cancelar cierra la
conexión local y retira la vista provisional; el proveedor puede haber facturado
ya la solicitud.

Los estados de herramientas provienen del motor y no añaden llamadas al modelo.
La web indica preparación, generación, herramienta activa, espera de respuesta,
reintento y reconexión. El mensaje definitivo reemplaza al provisional. Este
cambio no incorpora la propuesta de contratos de verificación de tareas.

Referencia: [Streaming de OpenAI](https://developers.openai.com/api/docs/guides/streaming-responses).
