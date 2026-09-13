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
| `fenix-storage` | Migraciones SQLite, sesión, cola, mensajes, eventos, FTS5 y entregas. |
| `fenix-agent` | Motor, contexto, permisos, herramientas, procesos, cron y adaptador de Telegram. |
| `apps/server` | Ejecutable, API de loopback y archivos estáticos incrustados. |
| `apps/web` | Interfaz en módulos ES, HTML y CSS, sin servidor Node durante el uso. |

El servidor tiene dos hilos de Tokio. Las operaciones de archivos utilizan el pool
de trabajo bloqueante; las llamadas al modelo, Telegram y los comandos utilizan
I/O asíncrona. La versión inicial ejecuta una tarea del agente a la vez.

## Ciclo y permisos

El mensaje se redacta, se guarda y se encola. Una respuesta textual a una pregunta
pendiente se entrega directamente a esa solicitud. El motor toma la tarea más
antigua, construye contexto, consulta al modelo, valida las herramientas, aplica
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
`cancelar` detiene la tarea activa; `cancelar <task_id>` admite una tarea en cola.

Las herramientas de archivos validan rutas, protegen datos privados y no crean
archivos encima de otros existentes. La edición exige una coincidencia única y
verifica la escritura. Los comandos pueden usar cualquier directorio accesible al
usuario de Linux; **la carpeta de archivos no es un sandbox de comandos**. Los
procesos gestionados tienen timeout, salida acotada y limpieza de su grupo.

## Persistencia y memoria

SQLite guarda conversación original, metadatos de los proveedores, tareas,
eventos, configuración, recuerdos y programaciones. WAL, transacciones breves y
un bloqueo de instancia evitan dos ejecutores simultáneos sobre los mismos datos.
Las claves no se devuelven en la API de configuración ni en el historial visible.

La memoria explícita guarda título, contenido, origen y fechas. La normalización
de contenido evita duplicados exactos; FTS5 utiliza palabras Unicode, búsqueda OR
y ranking. No hay embeddings ni grafos. `MEMORY.md` es una exportación de referencia;
la fuente principal es la base de datos. Los otros cuatro Markdown de instrucciones
se editan desde configuración. Los skills predeterminados no sobrescriben los
archivos personales existentes.

Al comenzar una tarea, si el historial activo supera aproximadamente 64 KB, el
motor intenta resumir tareas antiguas completas y conservar las recientes. El
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

- Una tarea activa, 32 pasos de modelo y hasta 16 herramientas por respuesta.
- Hasta dos reintentos adicionales de inferencia transitoria, sin repetir efectos
  de herramientas automáticamente. No existe presupuesto acumulado de tokens.
- Una hora por tarea y 30 minutos de espera para una respuesta del propietario.
- Timeout HTTP de 120 segundos; hasta 4.096 tokens de salida por llamada del agente.
- Solicitud activa de hasta 512 KB, argumentos de herramientas de hasta 128 KB y
  resultados grandes recuperables en páginas.
- Archivos UTF-8 regulares de hasta 1 MiB, con búsqueda y listados acotados.
- Ocho procesos gestionados a la vez, timeout de 1–600 segundos y hasta 128 KiB de
  stdout/stderr combinado. El límite predeterminado de comando es 120 segundos.
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
