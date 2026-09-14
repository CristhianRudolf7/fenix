# Proyectos en una sola conversación

[English](../en/projects.md)

Fénix conserva una conversación visible del propietario entre web y Telegram.
Los proyectos organizan el contexto y el trabajo internamente; no crean chats
separados ni exigen seleccionar un proyecto antes de escribir. La conversación
general tiene su propio contexto. No se mantiene un agente permanente por proyecto.

Estos proyectos del asistente son distintos del ámbito de credenciales del
proveedor. Cambiar de proyecto no selecciona otro modelo ni API key; el principal
y sus hijos comparten la conexión configurada.

## Crear y administrar proyectos

Abre **Proyectos** en la web. Cada proyecto tiene nombre, alias opcionales,
descripción y contexto operativo. La carpeta de trabajo es opcional, debe existir
y sirve como ubicación predeterminada para sus tareas. Los proyectos personales,
de oficina o planificación pueden no tener archivos.

Los nombres y alias no se pueden compartir entre proyectos. El contexto puede
organizarse con objetivo, restricciones y preferencias, progreso (completado, en
curso y bloqueado), decisiones clave, archivos relevantes, próximos pasos y
contexto crítico. La sección permite editar el proyecto y consultar sus tareas.
Cambiar su carpeta no modifica una ejecución que ya tenga su ubicación asignada.

En **Memoria**, selecciona memoria general o un proyecto para consultar o guardar
datos. Las preferencias generales y las decisiones específicas se mantienen
separadas. Editar un recuerdo no permite moverlo incidentalmente a otro ámbito.

## Identificar el destino antes de recuperar información

Los mensajes ingresan a la misma cola. Al recibirlos se conserva su referencia
de respuesta y una indicación del proyecto que se estaba tratando. Los nombres y
alias claros, saludos y órdenes explícitas de conversación general se pueden
resolver con reglas. Los demás casos utilizan una petición acotada al modelo
configurado, con catálogo y referencias relevantes, antes de cargar memorias del
proyecto o habilitar herramientas de ejecución. Esa petición consume tokens; no
selecciona automáticamente otro modelo más barato.

Se distingue conversación general, un proyecto, coordinación de varios y destino
ambiguo. Un mensaje puede contener varias intenciones, conservando su orden. Si no
hay destino claro, se pregunta por texto antes de producir efectos. Una respuesta
inválida del enrutador detiene la tarea. Identificar B no basta para saber qué
significa “mejorarlo”: el agente debe aclarar un objetivo impreciso.

Cada ejecución conserva una asignación con destino, proyectos consultables,
referencia a la tarea respondida, carpeta y versión del contexto. Cambiar de tema
no cambia esa asignación. Una comparación autorizada puede leer varios proyectos
sin fusionar sus memorias ni permitir escrituras arbitrarias entre ellos.

El contexto combina instrucciones, preferencias generales aplicables, resumen del
proyecto, referencias de tareas, memoria e historial del ámbito y catálogo breve
de skills. Las instrucciones completas de los skills se cargan bajo demanda.
Los historiales y resúmenes se separan internamente; los mensajes originales
permanecen conservados y visibles en la conversación común.

## Ejemplo: consultar A y después mejorar B

Ante “Dime cómo va A y después mejora el inicio de sesión de B”, el principal
coordina las dos peticiones en ese orden. Puede consultar directamente el estado
registrado de A y delegar el trabajo extenso de B con objetivo y entrega esperada.
No se crea un subagente por cada proyecto mencionado. La delegación depende del
trabajo, mediante herramientas e instrucciones; no es una receta fija para toda
frase posible.

Al delegar desde coordinación, se indica el proyecto de destino. El hijo recibe
el encargo relevante y su asignación, sin copiar toda la conversación principal.
No puede crear otros subagentes. El maestro sigue disponible mientras trabajan o
esperan una respuesta textual; se conservan los límites existentes de reintentos
y concurrencia.

El resultado de B vuelve asociado a B aunque entonces estés hablando de A. Las
notificaciones no cambian el foco de la conversación. Las aprobaciones y preguntas
pendientes conservan su tarea de origen. Puedes usar **Responder** en la web o la
respuesta a un mensaje en Telegram para referirte a una tarea anterior; hacerlo no
aprueba por sí mismo ninguna acción.

## Datos existentes y límites

Las migraciones de SQLite agregan proyectos, asignaciones y memoria e historial
por ámbito. Se conservan los mensajes y recuerdos previos como información
general. No se deduce automáticamente a qué proyecto antiguo pertenecía cada
mensaje ni se reasigna el historial al crear un proyecto. Los datos permanecen
en el directorio privado configurado.

La separación es de contexto y ejecución, no un sandbox del sistema operativo.
Los comandos conservan el acceso del usuario de Linux. La interpretación del
lenguaje puede equivocarse: los controles comprueban la asignación resuelta, pero
no demuestran que esa interpretación sea correcta.

Grafos, embeddings, extracción autónoma, clasificación retrospectiva del historial,
recuperación avanzada y jerarquías de permisos por proyecto siguen en el roadmap.
Las tareas interrumpidas quedan registradas y no se repiten automáticamente.

Las pruebas del repositorio cubren ámbitos e interfaz con verificaciones locales,
proveedores simulados, API y navegador. No constituyen un benchmark con modelos
reales ni prueban una precisión universal o porcentaje de ahorro de tokens.
Una evaluación real debe medir errores de destino, consultas y efectos cruzados,
preguntas innecesarias, tareas completadas, tokens y latencia. El contrato de los
endpoints se describe en la [documentación en inglés](../en/projects.md#storage-and-api).
