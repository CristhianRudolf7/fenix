# Fénix — Especificación de requisitos

## Alcance del documento

Este documento define el alcance actual de Fénix: un asistente personal funcional, con un único agente principal, conversación por texto, herramientas autorizadas, memoria básica explícita y controles operativos. Estas son las capacidades que se deben construir y validar primero.

- Telegram es la única integración externa de mensajería contemplada inicialmente.
- La interfaz propia se limita inicialmente a chat de texto y administración básica.
- La memoria avanzada, la autonomía prolongada, la voz en tiempo real, los subagentes y el resto de ampliaciones se conservan en [ROADMAP.md](ROADMAP.md).
- Este documento describe requisitos y criterios de cierre; no implica que las funciones ya estén implementadas.
- Las referencias de OpenClaw, ZeroClaw y NullClaw sirven para definir el conjunto básico de capacidades y se mantienen como antecedentes de diseño.

## Asistente funcional con memoria básica

### 1.1. Ejecución local y proveedores de modelos

- Mantener el núcleo en Rust como decisión de la conversación. El servicio debe poder iniciarse, detenerse, cargar su configuración y funcionar independientemente de que la interfaz esté abierta.
- Definir adaptadores para modelos remotos o locales compatibles con conversación y llamadas a herramientas. Permitir seleccionar proveedor y modelo, configurar credenciales y tratar errores de conexión, límites de uso y timeouts.
- Mantener separados el núcleo, los proveedores, las herramientas, la persistencia y los canales, para poder ampliar cada uno sin reescribir el bucle del agente.

### 1.2. Bucle del agente y cumplimiento básico de la tarea

- Seguir el ciclo: recibir mensaje → identificar sesión y objetivo → construir contexto → consultar modelo → validar y ejecutar herramientas → incorporar resultados → continuar o responder.
- Cada tarea debe conservar el pedido, el objetivo, las restricciones, el resultado esperado y los supuestos utilizados, identificados como tales. Una inferencia del agente no debe confundirse con una instrucción del usuario.
- Comprobar el resultado antes de declarar éxito: leer el archivo producido, comprobar un estado o ejecutar una prueba existente cuando corresponda. La validación debe ser proporcional a la tarea.
- Permitir consultar el estado y cancelar la ejecución. Cancelar detiene trabajo pendiente; no revierte por sí solo efectos que ya ocurrieron.

### 1.3. Conversación y sesiones persistentes

- Conversar por texto mediante Telegram y una interfaz propia básica; disponer de una entrada local para desarrollo y diagnóstico.
- Crear, recuperar y reiniciar sesiones. Guardar mensajes, respuestas y resultados de herramientas para continuar la conversación después de reiniciar el servicio.
- Mantener una sola ejecución activa por sesión al principio. Ordenar los mensajes que lleguen mientras está ocupada y procesar la cancelación sin esperar a que termine el turno.
- Conservar un espacio de trabajo explícito y mostrar la sesión activa. El enrutamiento automático entre varios proyectos simultáneos pertenece al trabajo futuro.

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
- Ejecutar comandos con directorio de trabajo explícito, timeout, salida acotada y control del proceso.
- Consultar páginas y realizar búsquedas web o peticiones HTTP mediante herramientas configuradas.
- Gestionar memoria explícita, consultar estado de tareas y crear o cancelar programaciones.
- Definir un registro extensible de herramientas con nombre, descripción, esquema de argumentos, permisos y resultado normalizado. Habilitar sólo las herramientas necesarias para el turno.
- Dejar preparados los límites de integración para nuevas herramientas o MCP. La automatización compleja del navegador y el arnés de programación se incorporarán más adelante.

### 1.7. Permisos y aislamiento básico

- Restringir quién puede utilizar el asistente, qué rutas puede modificar y qué comandos o herramientas tiene habilitados.
- Aplicar las comprobaciones antes de ejecutar. Utilizar un entorno de ejecución restringido y límites de proceso adecuados a las herramientas habilitadas.
- Solicitar aprobación cuando una acción exceda la autorización vigente o lo exija su política; conservar esa autorización dentro de su alcance.
- Validar argumentos y tratar documentos, páginas y salidas de herramientas como contenido no confiable.
- Redactar secretos en registros y respuestas. Los sistemas avanzados de permisos entre proyectos y de compensación de efectos se desarrollarán más adelante.

### 1.8. Skills predefinidos y manuales

- Cargar skills instalados o redactados explícitamente, seleccionarlos por su descripción y utilizarlos cuando encajen con la tarea.
- Cada skill debe explicar objetivo, entradas, pasos, herramientas permitidas, límites, salidas y condiciones de éxito.
- Permitir listar, habilitar y deshabilitar skills, conservando al menos nombre, descripción y versión utilizada.

### 1.9. Cron y recordatorios explícitos

- El planificador funciona fuera del modelo y del adaptador de Telegram. Persiste las programaciones para que sobrevivan a un reinicio.
- Admitir ejecuciones de una sola vez, intervalos y expresiones cron, con zona horaria definida.
- Permitir crear, listar, editar, pausar y cancelar tareas programadas o recordatorios pedidos por el usuario; conservar próxima ejecución, estado y último resultado.
- Aplicar timeout, reintentos acotados y protección contra solapamientos de la misma programación. Definir cómo tratar una ejecución vencida durante una interrupción del servicio.

Referencia de funcionamiento: [Automations/Cron de OpenClaw](https://docs.openclaw.ai/automation/cron-jobs).

### 1.10. Heartbeat básico

- Despertar periódicamente al agente para revisar una lista explícita de pendientes, por ejemplo en `HEARTBEAT.md`, y el estado de las tareas conocidas.
- Configurar cadencia, horario activo y presupuesto. Evitar solapamientos con otra revisión o con una sesión ocupada.
- Comunicar sólo asuntos que requieran atención; evitar avisos repetidos y ciclos del modelo cuando no hay trabajo configurado.
- Separar esta revisión periódica del cron de hora exacta y de los latidos técnicos usados para saber si un trabajador sigue vivo.

Referencia: [Heartbeat de OpenClaw](https://docs.openclaw.ai/gateway/heartbeat).

### 1.11. Límites, errores y prevención básica de bucles

- Establecer límites por tarea de tiempo, tokens, llamadas al modelo, herramientas y reintentos; registrar costo cuando el proveedor lo permita y fijar concurrencia máxima.
- Detectar repeticiones consecutivas de la misma herramienta con argumentos y resultados equivalentes. Al persistir la falta de progreso, detener el ciclo y explicar el bloqueo.
- Tratar fallos de proveedor, errores de herramientas, timeout y cancelación con estados explícitos. Reintentar sólo de forma acotada y cuando sea seguro.
- No volver a ejecutar automáticamente un efecto externo cuyo resultado sea desconocido.

Referencia: [Detección de bucles de OpenClaw](https://docs.openclaw.ai/tools/loop-detection).

### 1.12. Estado, registro mínimo y administración

- Generar `session_id`, `task_id`, `turn_id` y `tool_call_id` antes de procesar el trabajo correspondiente.
- Guardar herramienta, argumentos permitidos, resultado, error, código de salida, timestamps y consumo disponible. Mantener las salidas grandes fuera del contexto y ofrecer una referencia para recuperarlas.
- Mostrar estados como pendiente, ejecutando, bloqueado, completado, fallido, cancelado o interrumpido.
- Tras reiniciar, conservar historial y programaciones, e identificar las tareas que quedaron interrumpidas; el alcance actual no promete reanudar automáticamente cualquier efecto en curso.
- Administrar modelo, herramientas habilitadas, límites, skills, memoria explícita y horarios mediante opciones o comandos concretos. La autoconfiguración libre por lenguaje natural y el historial restaurable de configuración quedan para el trabajo futuro.

### Criterio de cierre del alcance actual

El asistente debe poder conversar, usar herramientas autorizadas, comprobar un resultado, recordar una preferencia explícita después de reiniciar, recuperar una sesión, ejecutar una programación persistida y detenerse ante cancelación, error o presupuesto agotado. También debe permitir inspeccionar lo ocurrido y corregir su memoria básica. El alcance actual puede cerrarse sin componentes de memoria avanzada ni orquestación de subagentes.

