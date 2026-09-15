# Nemotron: cinco tareas nuevas con el evaluador nativo

[English](../../en/audits/nemotron-native-five-task-audit.md)

Fecha: 2026-09-15. Fénix 0.3.0 con el evaluador nativo y las mejoras previas del arnés. Se congeló el motor durante las cinco ejecuciones.

**Resultado: 1/5 tareas aprobadas (20%), con un costo de US$0.04838435.** Son tareas distintas de las 20 anteriores, elegidas para observar problemas diferentes; no constituyen una muestra aleatoria ni una puntuación representativa de todo el benchmark.

| Tarea | Resultado oficial | Segundos nativos | Llamadas | Tokens entrada | Tokens salida | USD |
|---|---:|---:|---:|---:|---:|---:|
| `git-leak-recovery` | 1 | 34.36 | 22 | 182,517 | 7,387 | 0.00640414 |
| `headless-terminal` | 0 | 58.52 | 18 | 136,810 | 19,783 | 0.00737083 |
| `pypi-server` | 0 | 75.28 | 32 | 260,973 | 22,712 | 0.01140713 |
| `query-optimize` | 0 | 130.14 | 20 | 162,642 | 26,752 | 0.00918554 |
| `regex-log` | 0 | 178.65 | 11 | 71,214 | 78,991 | 0.01401671 |


Total del proveedor: **103 llamadas, 814,156 tokens de entrada —incluidos 391,680 en caché— y 155,625 de salida.** La caché no se suma otra vez. Todas las respuestas registraron uso y costo; no hubo errores HTTP. La salida incluye el razonamiento contabilizado por el proveedor. El costo suma `usage.cost` real, no una estimación de tarifas.

La ejecución nativa acumuló 476.95 segundos entre las tareas. El trabajo completo de Harbor duró 1892.81 segundos e incluye preparación, contenedores, verificadores oficiales y concurrencia: no es latencia del modelo. El cronómetro nativo empieza después de la comprobación inicial de conexión; el costo incluye esa comprobación, llamadas principales, resúmenes y procesos auxiliares. Las métricas internas de uso tienen un alcance menor que la facturación total.

## Condiciones

- Terminal-Bench 2.1 oficial, revisión `7131e4375048a0e408a8fb404b5f499d726b695b`; Harbor 0.23.0.
- Un intento por tarea, sin reintentos del trabajo, hasta dos tareas simultáneas; motor y modelo constantes. No se enviaron pruebas ocultas al agente.
- `nemotron-3-5-lightning-30b` de RunInfra mediante el adaptador compatible `openrouter` y un proxy de registro; pensamiento predeterminado del proveedor, sin sobrescribir el nivel.
- Contenedores oficiales: una CPU y 2 GiB de RAM; 900 segundos para el agente. El verificador dispone de 900 segundos, salvo SQL, con 1.800 segundos oficiales.
- Motor nativo en Rust; Python solo orquesta Harbor y el proxy. Estado aislado nuevo y skills incluidos por tarea; `/app`, comandos y escrituras permitidos, sin servidor LSP configurado.
- Contexto configurado de 32.768 tokens, techo de salida de 16.384 por respuesta y presupuesto efectivo de entrada de 15.360 tras reservar salida y margen; máximo de 128 pasos. El techo por respuesta no es un presupuesto monetario.
- El evaluador nativo no inicia el rastreador web de fondo del servidor interactivo. Utiliza las herramientas disponibles y el flujo normal de trabajo y compactación.
- Se aprueba por el verificador oficial, no por el estado completado del agente. Las pruebas se inspeccionaron después de detenerse el intento correspondiente.

SHA-256 del ejecutable: `ff895b861c517db5d4b975906bab3b530ff41f12515fba74e9639fbdc705c637`. Se verificó que los 100 archivos fuente registrados y el ejecutable no cambiaron. Esta huella identifica el estado de trabajo, que incluía implementación previa todavía sin un commit nuevo.

## Qué falló y por qué

### Descubrimiento de herramientas: headless-terminal

El agente instaló `pexpect` y escribió una implementación, pero omitió inicialmente la herencia de `BaseTerminal`. Intentó corregirla con argumentos inventados para `file_edit` y `file_patch`. Las peticiones al proveedor solo exponían seis herramientas esenciales: los esquemas de edición no se habían cargado. El modelo usó nombres anunciados en el catálogo sin llamar a `tool_search`; el motor permitió intentar esas herramientas y devolvió errores de parámetros insuficientes para orientarlo.

Las relecturas del archivo sin cambios terminaron activando el detector de bucles. El verificador aprobó importación y cancelación, pero fallaron otras cinco pruebas de comportamiento. La implementación enviaba el salto de línea y Ctrl-C sin convertirlos al nombre esperado por `pexpect.sendcontrol`. Una comprobación local separada con las mismas versiones (`pexpect` 4.9.0 / `ptyprocess` 0.7.0) confirmó que ambos envíos escriben cero bytes: no se enviaba Enter. La prueba de cancelación solo comprueba que no exista un archivo; aprobarla no demuestra que la cancelación funcionara. Evidencia: `diagnostics/terminal-control.json`. El agente no ejecutó pruebas funcionales de la implementación.

**Mejora:** ante una herramienta autorizada pero no cargada, devolver primero su esquema; ante argumentos inválidos, indicar campos requeridos, recibidos y un ejemplo mínimo. Separar descubrimiento de permisos. Dar una oportunidad acotada de recuperación antes de terminar un bucle. El código incorrecto precedía al detector: desactivarlo no garantiza resolver la tarea.

### Duración del servidor y falso éxito: pypi-server

El paquete se construyó, pero el servidor arrancó con `&` y `persistent=false`. Al finalizar el shell, la limpieza normal del grupo de procesos eliminó el servidor. El esquema documentaba `persistent=true`, pero el modelo no lo usó. No fue un fallo de la conservación de servicios del evaluador: nunca se registró un servicio persistente.

Además, faltaba la página del proyecto `/simple/vectorops/` en el índice. La comprobación local de instalación devolvió `Requirement already satisfied`: el paquete ya estaba instalado desde el directorio de trabajo y no se comprobó su descarga. El verificador oficial desinstaló primero el paquete y la instalación falló. Fénix había marcado la tarea como completada.

**Mejora:** comunicar explícitamente la limpieza y duración de procesos, orientar el uso de servicios persistentes y comprobar la instalación desde un entorno limpio. Relacionar la afirmación de éxito con el comportamiento solicitado, no solo con un código de salida. No convertir comandos arbitrarios en servicios persistentes automáticamente.

También fallaron dos extracciones auxiliares de memoria por JSON inválido. Fueron errores no fatales, incluidos en el costo, y no explican la ausencia del servidor.

### Bloqueo demasiado amplio: query-optimize

El agente leyó la consulta y el esquema y lanzó la consulta original, lenta. Después generó una propuesta con CTE y funciones de ventana e intentó guardarla en `/app/sol.sql`. El motor esperó al comando anterior y devolvió `process_busy`, `retryable=true`, `executed=false`: el archivo nunca se escribió. El verificador oficial aprobó las dos comprobaciones de referencia/base de datos y falló las cuatro de solución porque no existía `sol.sql`. Su ejecución de pytest tardó 1.405,36 segundos (23 min 25 s), principalmente en la comparación inicial, aparte de los 130,14 segundos del intento nativo. El bloqueo de la tarea impidió una escritura independiente mientras seguía una lectura de la base de datos.

El modelo continuó consultando el proceso y releyendo el SQL original. El detector terminó la tarea por lecturas idénticas repetidas, antes de alcanzar los 900 segundos. Una consulta sin salida todavía puede estar trabajando. La propuesta no llegó a materializarse ni ejecutarse en el intento oficial. Después, una comprobación determinista separada extrajo el SQL de la traza y lo ejecutó sobre una copia de solo lectura: sus 500 filas y ocho nombres de columnas coincidieron exactamente con la referencia. El hash de la base coincidió con el esperado por el verificador y no cambió. Una ejecución local tardó 5,21 segundos para la propuesta y 2,83 para la referencia; no son mediciones oficiales ni demuestran que pasaría el umbral de rendimiento. No hubo llamadas al modelo, cambios en el contenedor evaluado ni un segundo intento del agente. Evidencia: `diagnostics/sql-result.json`. Esto refuerza la atribución de la falta del archivo al bloqueo del motor, sin cambiar la calificación oficial.

**Mejora:** gestionar dependencias y recursos con criterio conservador; permitir acciones independientes cuando se pueda demostrar que no interfieren, manteniendo la serialización si se desconocen los efectos de un comando. Acotar las mediciones de referencia y distinguir espera de fallo. Antes de detener un bucle, mostrar el bloqueo y permitir una alternativa limitada sin cancelar trabajo útil automáticamente.

### Agotamiento de salida: regex-log

Cuatro respuestas alcanzaron el límite configurado de 16.384 tokens de salida. Fénix guardó las respuestas truncadas sin ejecutarlas, hizo tres recuperaciones y terminó al agotarlas. Los metadatos mostraron consumo en razonamiento, no un archivo terminado demasiado grande. La instrucción genérica de dividir escrituras no resolvió ese problema.

El archivo final contenía únicamente un patrón de IPv4. La prueba esperaba fechas y recibió direcciones IP. No hubo una comprobación con `re.findall` ejecutada mediante comandos.

**Mejora:** diferenciar agotamiento del razonamiento de argumentos demasiado extensos; dirigir pasos pequeños de implementación y prueba, con un cambio de estrategia acotado. No ejecutar generaciones incompletas. Evaluar por separado el reparto de razonamiento y salida, sin suponer que aumentar el límite solucionará el fallo.

### Compactación: funciona el mecanismo, falta fidelidad

Las 15 compactaciones finalizaron sin error fatal de resumen o corte y conservaron checkpoints. Sin embargo, un resumen de la tarea de terminal inventó una corrección del usuario fechada el `2026-06-10`, cuando era una acción del asistente, y dejó la validación como algo externo. En SQL, otro resumen describió `sol.sql` como parcialmente escrito aunque la herramienta había devuelto `executed=false` y el archivo no existía. El contrato mantenía el objetivo original. Es un defecto observado de fidelidad; esta ejecución no permite medir cuánto causó el fallo final.

Cada petición de trabajo incluía un mensaje de sistema de 22.960–24.625 caracteres con instrucciones generales de conversación, proyectos y memoria. Es una medición de caracteres, no de tokens. Conviene cargar solo el contexto operativo pertinente para programación; esta ejecución no aísla el tamaño del prompt como causa del fallo.

Varias compactaciones conservaron cero mensajes recientes. Los resúmenes consumieron 58.868 tokens de salida. Conviene preservar estructuradamente errores recientes, esquemas, bloqueos y procedencia, evitando convertir inferencias del asistente en instrucciones del usuario. Guardar el original no garantiza que su resumen sea correcto.

## Prioridades posteriores

1. Recuperación del esquema de herramientas y errores de argumentos útiles.
2. Evitar bloquear escrituras independientes detrás de lecturas lentas cuando se pueda demostrar que no interfieren.
3. Comunicar la duración de servicios y comprobar el comportamiento de un consumidor limpio antes de declarar éxito.
4. Recuperación limitada de bucles considerando bloqueos, compactación reciente y estado real de procesos.
5. Preservar procedencia y estado crítico al compactar, reduciendo el costo de los resúmenes.
6. Distinguir agotamiento de razonamiento de acciones extensas y evaluar una recuperación adaptativa acotada.

Son recomendaciones; no se modificó el motor durante la auditoría. Git aprobó sus cinco comprobaciones oficiales. Esta muestra no permite concluir que el arnés mejoró o empeoró frente a las 20 tareas anteriores, ni atribuir todos los fallos exclusivamente al arnés o al modelo.

## Reproducción y evidencia

[Métricas CSV](nemotron-native-five-task-results.csv), [evaluador nativo](../native-evaluation.md). Evidencia local privada en `.fenix/evaluations/native-five-20260915/`: protocolo, selección, huellas, integridad, métricas, llamadas al proveedor y resultados por tarea. Cada tarea conserva `agent/native/report.json`, checkpoints y `verifier/`. No se publican credenciales, secretos sintéticos, razonamientos internos del proveedor ni el contenido de las pruebas.
