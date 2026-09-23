# Arnés de programación con contexto de repositorio

[Ejecución interactiva, límites actualizados y herramientas bajo demanda](execution-harness.md)

[English](../en/coding-harness.md)

El maestro y sus subagentes tienen búsqueda de código, parches, instrucciones por
repositorio, LSP y compactación durante la tarea. Se mantiene una sola conversación.
La web permite configurar estas funciones en **Configuración → Opciones avanzadas
→ Herramientas de programación y contexto**. Las instalaciones existentes reciben
los valores de contexto predeterminados y ningún servidor de lenguaje activado.

## Búsqueda y parches

`files_search` conserva la búsqueda literal sin distinguir mayúsculas. Ahora
admite `regex`, `case_sensitive`, filtros `glob` y `exclude`, y `limit`.
`files_glob` encuentra rutas mediante patrones como `**/*.rs`. Se respetan
`.gitignore` y `.ignore`, las restricciones de archivos privados y la exclusión
de directorios generados; no se siguen enlaces simbólicos. Los resultados indican
truncamiento y errores omitidos. Límites: 500 resultados, 20.000 entradas,
profundidad 32, 64 MiB de texto explorado y 1 MiB por archivo UTF-8.

`file_patch` agrega, modifica, mueve o elimina hasta 32 archivos. Usa marcadores
`*** Begin Patch`, `*** Update File: ruta`, `@@`, `*** Add File: ruta`,
`*** Move to: ruta`, `*** Delete File: ruta` y `*** End Patch`. Cada línea de un
bloque empieza por espacio (contexto), `-` (eliminar) o `+` (agregar).
Los bloques `@@` no llevan números de línea. El contenido anterior debe coincidir
sin ambigüedad. Se valida el conjunto antes de escribir y se reutilizan las
aprobaciones textuales exactas y los controles de efectos existentes.

Un parche de varios archivos **no es una transacción del sistema de archivos**:
un error de disco puede dejar cambios parciales. El error y el registro indican
los archivos completados y el que falló; hay que inspeccionarlos antes de reintentar.
No se incorpora restauración automática.

## Instrucciones del proyecto

Se carga el `AGENTS.md` de la raíz del workspace al iniciar la tarea, también para
subagentes. Para un archivo se añaden las reglas de sus subcarpetas; las más
específicas solo aplican a ese subárbol. No se heredan reglas fuera del proyecto.
Prevalecen las instrucciones explícitas del usuario y las políticas del motor.
Los archivos se limitan a 32 KB cada uno / 64 KB por cadena, se redactan secretos
y se detectan cambios por hash.

Si una edición encuentra reglas que el modelo aún no recibió, devuelve
`executed: false` con esas instrucciones y no consume la aprobación ni escribe.
Leer y editar en una misma respuesta del modelo no permite saltar esta condición.
Tras compactar, las reglas anidadas pueden cargarse de nuevo. Estas instrucciones
no constituyen un aislamiento de los comandos de terminal.

## LSP y diagnósticos

Se configuran servidores instalados, por ejemplo:

```json
[
  {"name":"rust","command":["rust-analyzer"],"extensions":["rs"]},
  {"name":"python","command":["/ruta/absoluta/a/pylsp"],"extensions":["py"]}
]
```

También se admite `initialization_options`. Ninguna extensión puede tener dos
servidores. No se instalan programas automáticamente ni se ejecutan comandos de
configuración procedentes del repositorio. Los servidores configurados por el
propietario se ejecutan con sus permisos de usuario; LSP no es un sandbox.

`code_query` permite consultar definiciones, referencias, información del símbolo,
símbolos del archivo, implementaciones y diagnósticos. Las posiciones `line` y
`character` empiezan en 1; la columna cuenta unidades UTF-16. Se sincronizan cambios
del archivo y se reutilizan sesiones separadas por proyecto. Hay un máximo de
cuatro sesiones, 32 documentos por sesión y tiempos/tamaños acotados. Cancelar una
consulta fallida descarta el servidor; cambiar configuración o apagar el servicio
cierra sus procesos.

### Navegación progresiva por símbolos

- Localizar archivos con `files_glob` o `files_search`; después consultar
  `code_query` con `symbol_outline` para ver nombres jerárquicos, tipos y rangos.
  `limit` vale 50 por defecto (máximo 100); `offset` permite paginar. `symbols`
  conserva el resultado original del servidor.
- Buscar con `find_symbol` y `name_path: "Clase/metodo"`. Se compara el sufijo de
  componentes; una `/` inicial exige la ruta completa. La búsqueda distingue
  mayúsculas y se limita al archivo solicitado.
- Leer solo una función o clase con `read_symbol`. Devuelve cuerpo, rangos y hash
  SHA-256 del archivo. `max_bytes` vale 16 KiB por defecto, máximo 32 KiB; avisa
  si el contenido está truncado. Un nombre ambiguo produce un error; para
  sobrecargas se deben consultar los rangos y leer por posición.
- Consultar referencias, definición, información o implementaciones con un
  `name_path` único. `include_context: true` agrega fragmentos a un máximo de
  20 ubicaciones, cinco líneas y 1.000 bytes por fragmento. Se aplican los mismos
  permisos de archivos y separación entre usuarios, proyectos y worktrees.
  No se leen contenidos de dependencias externas al espacio permitido.

Los rangos devueltos empiezan en **cero y usan UTF-16**; los argumentos de posición
y `start_line` de los fragmentos empiezan en uno. Rangos inválidos o posiciones
que dividan un carácter Unicode producen un error.

Cada proyecto/worktree puede consultar su servidor en paralelo; solo se
serializan consultas del mismo servidor y espacio. Si hay cuatro sesiones,
se reutiliza el lugar de la inactiva menos reciente. Nunca se expulsa una sesión
con consultas activas o esperando; si todas están ocupadas se pide reintentar.
Cambiar la configuración cancela consultas antiguas y permite iniciar sesiones
nuevas.

La caché guarda hasta ocho respuestas de símbolos de 128 KiB por sesión. Antes de
consultar se actualizan los documentos abiertos, incluso ante cambios externos
del mismo tamaño, y se invalida la caché. Los archivos eliminados o ilegibles se
cierran y se informa del problema. Si cambia el archivo consultado durante la
petición, hay que reintentar. Esto no recorre todo el repositorio: el servidor
configurado sigue siendo responsable de indexar archivos no abiertos.

Las ediciones continúan con `file_edit` y `file_patch`, sus aprobaciones y reglas
actuales. No se añadió otro editor, un renombrado LSP entre archivos, embeddings
ni llamadas adicionales a modelos. Véanse los detalles del protocolo y límites
en la [documentación en inglés](../en/coding-harness.md#progressive-symbol-navigation).

Validación (21/09/2026): pasaron 39 pruebas seleccionadas de navegación, caché,
concurrencia, permisos, herramientas, actualización de skills y worktrees. Las
pruebas de protocolo usan un servidor simulado determinista; se omitió la prueba
opcional con el LSP real de Python por no tener configurado su ejecutable. No se
midieron mejoras de benchmarks ni compatibilidad con todos los servidores.

Al modificar archivos se consultan diagnósticos de hasta cuatro archivos si existe
un servidor configurado. Se informa si quedan otros pendientes. Un diagnóstico
pendiente o no disponible no indica éxito; un fallo del servidor tampoco revierte
una escritura ya realizada. Esto no sustituye compilar, ejecutar tests ni comprobar
el comportamiento solicitado. No usa embeddings ni llamadas adicionales a modelos.

## Compactación de una tarea activa

Valores iniciales: capacidad de 32.768 tokens, compactación al llegar a 22.000
estimados y conservación de 6.000 tokens recientes. Se deben ajustar a la capacidad
real del modelo. La estimación incluye instrucciones, herramientas y metadatos;
no equivale al conteo facturado ni a un presupuesto de dinero.

Antes de resumir se guarda el contexto original redactado en SQLite, con hash
SHA-256, proyecto, `task_id` y `session_id`. Se resumen bloques antiguos completos,
sin separar herramientas de sus resultados. Se mantienen instrucciones, objetivo,
resumen estructurado y bloques recientes con sus metadatos. Las salidas antiguas
grandes se representan mediante extractos y enlaces al original.

`task_inspect` con `checkpoint_id`, `offset` y `length` permite recuperar el JSON
original por páginas, sujeto a los permisos del proyecto. El resumen puede omitir
detalles: se deben recuperar del checkpoint cuando sean necesarios. Si falla la
compactación no se reemplaza el contexto original. Se conservan los identificadores,
contadores y límites de ejecución: no se repiten efectos ni se reanuda trabajo
interrumpido por reinicio. Si las instrucciones o un bloque completo no caben,
se detiene con evidencia preservada en lugar de perder contexto silenciosamente.

Consulta la [documentación en inglés](../en/coding-harness.md) para el protocolo de
parches y las pruebas deterministas y de LSP real.

Validación del 15/09/2026: pasaron las pruebas del workspace, la comprobación final
del agente (82 tests), los 11 tests de navegador y Clippy sin advertencias. También
se probó un servidor LSP real de Python: definición de función y diagnóstico de
error sintáctico tras editar. Las respuestas de modelos se simularon; no es un
nuevo benchmark de calidad de programación ni implicó consumo de APIs de pago.
