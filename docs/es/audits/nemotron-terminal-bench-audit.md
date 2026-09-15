# Auditoría del arnés de Fénix con Nemotron

[English](../../en/audits/nemotron-terminal-bench-audit.md) · [Protocolo de ejecución](nemotron-terminal-bench-protocol.md)

Finalizada: 2026-09-15. Versión evaluada: Fénix 0.3.0 con los cambios locales de memoria adaptativa registrados en el protocolo.

## Alcance y medición

Se ejecutan veinte tareas oficiales de Terminal-Bench 2.1, seleccionadas antes de ver resultados, con el motor Rust de Fénix congelado y con Terminus 2 de Harbor. Ambos utilizan `nemotron-3-5-lightning-30b` de RunInfra con pensamiento predeterminado activado. La referencia permite encontrar tareas que el modelo resuelve mediante otro arnés; no aísla causalmente todas las diferencias de arquitectura.

El [protocolo](nemotron-terminal-bench-protocol.md) registra los hashes del código y binario, commit del conjunto de tareas, recursos, tiempos máximos, configuración de contexto, correcciones del adaptador y selección. Son veinte tareas de diagnóstico: no el benchmark completo, una clasificación oficial ni una estimación general de éxito en producción. Hay un intento principal por arnés y tarea. Ningún agente recibe las instrucciones del verificador oculto.

Se utiliza el mismo modelo y límite nominal de salida, pero difieren el cálculo de contexto, las instrucciones, las herramientas, los reintentos y la interpretación del límite de entrada. Fénix conserva sus valores predeterminados. La preparación del entorno se separa del tiempo de ejecución del agente. Los dos contenedores concurrentes comparten una máquina de desarrollo; los tiempos absolutos no representan una medición aislada de velocidad del modelo.

| Métrica | Fénix | Terminus 2 |
| --- | ---: | ---: |
| Tareas resueltas | 1/20 (5%) | 10/20 (50%) |
| Llamadas API registradas | 334 | 1,130 |
| Tokens de entrada, incluyendo caché | 3,172,921 | 15,806,466 |
| Entrada reutilizada desde caché | 1,862,656 | 11,550,208 |
| Tokens de salida, incluyendo pensamiento | 160,842 | 1,215,409 |
| Ejecución acumulada del agente (s) | 917.62 | 7,742.80 |
| Mediana de duración por intento (s) | 41.02 | 315.66 |
| Costo reportado por el proveedor (USD) | $0.108266 | $0.510626 |

El menor tiempo y costo de Fénix refleja en gran parte terminaciones tempranas, no mayor rendimiento útil. Solo la tarea de combinación de datos pasó en ambos: 44,04 s frente a 98,24 s. La entrada total ya incluye los tokens de caché; la salida incluye pensamiento. Una conexión de referencia terminó con broken pipe al vencer el tiempo de Python/C, pero se registró todo su consumo. Ningún fallo HTTP del proveedor explica los fallos de Fénix.

| Motivo de terminación de Fénix | Tareas |
| --- | ---: |
| Resumen de compactación inválido o truncado | 9 |
| Sin corte permitido para el lote reciente | 2 |
| Salida del modelo agotada durante la tarea | 3 |
| Esperando aclaración del usuario | 3 |
| Finalización declarada, verificador fallido | 2 |
| Éxito oficial | 1 |

Las 14 detenciones por contexto o salida son mecanismos observados, no prueba de que corregirlos produzca 14 éxitos adicionales. Un verificador de la tarea de R no pudo ejecutar las pruebas semánticas después del daño al gestor de paquetes causado durante el intento; su recompensa oficial sigue siendo cero y se señala abajo. El fallo inicial excluido del adaptador no realizó llamadas al modelo y no es uno de los 20 intentos.

Fénix realizó 132 lanzamientos de comandos y 132 consultas de estado; 80 consultas devolvieron estado completado en menos de 50 ms. Eso no significa que los 80 comandos hubieran terminado durante una espera inicial corta, y no se probó una intervención para medir ese ahorro de tiempo.

## Qué está fallando en Fénix

### 1. La compactación puede terminar trabajo recuperable

[active_context.rs](../../../crates/fenix-agent/src/active_context.rs) fija la salida del resumen en 2.048 tokens. Varias respuestas reales de Nemotron consumen ese presupuesto sin producir texto visible del resumen y terminan con `finish_reason=length`. El motor hace bien en rechazar un resumen inválido, pero [engine.rs](../../../crates/fenix-agent/src/engine.rs) propaga ese error y termina la tarea. Conservar un checkpoint durable no garantiza continuar automáticamente.

Ocurrió durante recuperaciones, compilaciones, migración y configuración de servicios. Algunos resúmenes anteriores sí funcionaron: el defecto es que un fallo posterior resulte fatal, no que todas las solicitudes de 2.048 tokens fallen. En la recuperación del WAL, ambos agentes ya habían consultado la base original antes de conservar ese archivo, que desapareció. Ese daño anterior no debe atribuirse al resumen que falló después.

**Primera corrección que conviene probar:** presupuesto de resumen compatible con pensamiento, un reintento acotado, conservación de los identificadores de tarea y operaciones, y una alternativa explícita si resumir sigue fallando. Hay que validar el resumen y el espacio que libera. Aumentar el límite no demuestra por sí solo fidelidad de la memoria ni éxito de la tarea.

### 2. Un lote válido de herramientas puede impedir cualquier corte de contexto

El compactador conserva lotes completos de llamadas y resultados, exige que la parte reciente quepa en 6.000 tokens estimados y excluye cortar al final de todos los mensajes. Tres lecturas de logs devolvieron juntas 40.096 caracteres de contenido. No había un corte que permitiera conservar ese lote reciente, por lo que se abortó antes de crear un checkpoint de compactación. La tarea de calendarios alcanzó la misma condición después de una respuesta con mucho razonamiento y nuevas lecturas de los mismos archivos.

**Qué hace falta:** presupuesto conjunto para las salidas de un lote, referencias a artefactos recuperables para los resultados grandes y una alternativa que permita resumir un lote completo demasiado grande sin romper la correspondencia entre llamadas y resultados. El cálculo debe reflejar el contenido realmente enviado al proveedor y la capacidad del modelo. Los 32.768 tokens configurados en Fénix son muchos menos que la capacidad anunciada por el proveedor; el cálculo conservador por bytes y la repetición del razonamiento añaden presión.

### 3. Agotar la salida del modelo no tiene una vía de recuperación

Fénix aborta ante una respuesta truncada antes de ejecutar las acciones propuestas. Eso evita utilizar argumentos incompletos, pero también impide solicitar de forma acotada acciones más pequeñas y completas. Las dos tareas políglotas y la macro de Vim expusieron este comportamiento. Terminus también recibió respuestas truncadas: logró resolver Rust/C++, pero agotó el tiempo en Python/C.

**Qué hace falta:** distinguir el agotamiento de salida de otros motivos de terminación, pedir una acción menor o ajustar un presupuesto admitido, y limitar las repeticiones sin progreso. No ejecutar llamadas incompletas ni repetir efectos ya realizados. Los resultados no justifican reintentos ilimitados.

### 4. La duración y cancelación de procesos necesitan contratos más claros

En gRPC, Fénix ejecutó `nohup ... &` con `persistent=true`. El shell devolvió cero y después la limpieza normal del grupo de procesos mató al hijo que había quedado en segundo plano. El motor seguía activo durante la verificación: no fue un apagado del adaptador. Las utilidades de inspección ausentes y un `echo` alternativo no demostraban que el puerto escuchara, aunque la respuesta final lo afirmó.

El contrato que funciona actualmente es ejecutar el servicio en primer plano dentro de una sesión administrada con `persistent=true`. Debe explicitarse o reemplazarse por una operación de servicio con duración definida, estado de salida temprana y comprobación de disponibilidad. No conviene eliminar globalmente la limpieza de procesos descendientes.

En R, un intento de `R --version` fue rechazado porque seguía una instalación activa. El modelo canceló el instalador, volvió a instalar y el posterior aborto de compactación interrumpió otra vez la gestión de paquetes. El verificador no pudo preparar sus dependencias porque `dpkg` quedó interrumpido. Es daño al entorno durante la ejecución, no evidencia sobre la calidad numérica de un muestreador implementado. Conviene mostrar identificadores y progreso de procesos activos, favorecer la espera frente a cancelar para hacer una consulta y registrar qué operaciones interrumpidas necesitan recuperación.

La comprobación de pMARS con el resumen ampliado reveló otra interacción: después de dos compactaciones correctas, tres rechazos iguales por un proceso ocupado activaron el detector de repeticiones, aunque la instalación seguía mostrando avances entre ellos. El detector acumula coincidencias de acción y resultado incluso cuando hay progreso intermedio. Conviene representar explícitamente los bloqueos temporales, con el identificador del proceso activo y una política de espera o cola, distinguiéndolos de acciones repetidas sin progreso general.

### 5. Las preguntas innecesarias impiden terminar sin intervención

Fénix pidió información disponible en el historial de Git, solicitó reconfirmar una configuración de servidor ya indicada y preguntó si publicar automáticamente cuando el flujo solicitado de `push` y consulta web ya lo establecía. No se inventaron respuestas del usuario para ayudarlo a terminar.

Las instrucciones del sistema ya indican que no se repitan preguntas resueltas. Es un problema de cumplimiento de instrucciones e interacción entre modelo y arnés, no simplemente una frase que faltaba. Se debe conservar la preferencia del usuario de comunicar dudas reales, distinguiendo información faltante de autorización ya concedida. Hay que revisar la evidencia y el requerimiento antes de preguntar. Las repeticiones sin cambios que se detallan abajo permiten observar la variabilidad del caso de Git.

### 6. Más llamadas y más pruebas propias no garantizan mayor calidad

Los comandos rápidos normalmente requieren otra llamada al modelo para consultar `command_status`. Una espera local corta podría devolver la salida ya disponible en el primer resultado, manteniendo la ejecución asíncrona para procesos largos. Los recuentos siguientes miden la oportunidad de mejora, no una aceleración ya implementada y demostrada.

Ambos arneses fallaron en la limpieza asíncrona durante cancelación. Las pruebas generadas por Fénix utilizaron limpieza síncrona y no cubrieron la interacción de limpieza con espera que falló. La referencia también generó valores SQLite incorrectos y un log de Nginx sin una comilla de cierre. Conviene priorizar pruebas derivadas del comportamiento solicitado y sus casos relevantes, sin exigir otro revisor LLM para cada acción.

### 7. El trabajo auxiliar necesita presupuestos compatibles y relevancia

Una extracción de memoria en segundo plano utilizó 1.800 tokens y terminó sin JSON útil al agotarlos. Es independiente del error fatal de corte de contexto en calendarios. El trabajador registra los fallos de extracción, pero estas tareas no establecen la calidad de la memoria a largo plazo.

La búsqueda web de la tarea de R utilizó un índice léxico local, nuevo y limitado. Una consulta ampliada devolvió una página sin relación con el algoritmo y otra no obtuvo resultados. La herramienta informó de su cobertura limitada, pero el resultado irrelevante consumió contexto. Ante poca cobertura conviene devolver evidencia útil o abstenerse. La calidad de recuperación no se evaluó de forma independiente aquí.

## Comprobaciones controladas

Son diagnósticos adicionales, excluidos del resultado principal de 20 tareas. No se modificó el código Rust del motor.

**Resúmenes con entrada idéntica.** Dos entradas que fallaron originalmente, tres repeticiones por entrada y límite, alternando límites. Solo cambió `max_tokens` en la solicitud repetida. Aceptación significa terminación normal, texto no vacío y ausencia de llamadas a herramientas; no mide fidelidad semántica.

| Tarea de origen | Límite de salida | Aceptados | Media de tokens de salida | Media de segundos API | Costo USD |
| --- | ---: | ---: | ---: | ---: | ---: |
| db-wal-recovery | 2,048 | 0/3 | 2048.0 | 5.49 | $0.001563 |
| db-wal-recovery | 16,384 | 3/3 | 5242.7 | 13.28 | $0.003001 |
| build-cython-ext | 2,048 | 1/3 | 1510.3 | 4.29 | $0.001441 |
| build-cython-ext | 16,384 | 3/3 | 1078.7 | 3.25 | $0.001160 |

En estas dos entradas fijas, la aceptación fue 1/6 con 2.048 frente a 6/6 con 16.384. La muestra es pequeña y la generación sigue siendo estocástica. Esto respalda un problema de compatibilidad del presupuesto en estos casos, no una tasa universal de éxito ni una garantía de memoria fiel.

**Repetición completa de pMARS.** El proxy cambió únicamente el límite del resumen interno a 16.384. Se aceptaron dos resúmenes, que consumieron 4.850 y 502 tokens de salida. La tarea siguió fallando: tres errores iguales por proceso ocupado activaron el detector de repeticiones mientras avanzaba la instalación. Este resultado no sustituye ninguno de la ronda principal.

- pMARS: recompensa 0; 82.80 s; 55 llamadas API; $0.015483.

**Repeticiones de Git sin cambios.** Los dos intentos adicionales fallaron. Uno volvió a pedir aclaración y el otro se detuvo al compactar. Incluyendo el original, el resultado es 0/3 en esta tarea. La ruta de fallo varía; tres intentos no establecen una probabilidad universal de fallo.

| Intento | Resultado | Segundos | Llamadas API | Costo USD |
| --- | --- | ---: | ---: | ---: |
| fix-git__EFVcckW | Esperando aclaración | 35.27 | 16 | $0.003514 |
| fix-git__EsgXbME | Fallo de compactación | 47.39 | 26 | $0.007063 |

**Gasto total registrado: US$0.65216044.** Ronda principal US$0.61889244; tres repeticiones completas US$0.02605965; 12 pruebas de resumen US$0.00716545; comprobación inicial directa US$0.00004290. Las 1,573 solicitudes por proxy tienen costo reportado, además de una comprobación directa. Es consumo API reportado, no una factura del proveedor.

## Resultados y análisis por tarea

[CSV con métricas por intento](../../en/audits/nemotron-terminal-bench-results.csv). Cada resultado indica recompensa oficial y, entre paréntesis, comprobaciones aprobadas/total. Las comprobaciones parciales sirven para diagnosticar, no otorgan fracciones de tarea resuelta. Los tiempos son segundos de ejecución del agente.

| Tarea | Resultado Fénix | Resultado referencia | Fénix s | Referencia s | Fénix USD | Referencia USD |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cancel-async-tasks | 0 (5/6) | 0 (5/6) | 52.2 | 26.1 | $0.00769 | $0.00145 |
| fix-git | 0 (0/2) | 1 (2/2) | 43.6 | 38.9 | $0.00474 | $0.00288 |
| db-wal-recovery | 0 (0/7) | 0 (0/7) | 39.3 | 643.8 | $0.00685 | $0.06423 |
| multi-source-data-merger | 1 (3/3) | 1 (3/3) | 44.0 | 98.2 | $0.00735 | $0.00683 |
| build-cython-ext | 0 (2/11) | 0 (9/11) | 122.3 | 541.0 | $0.01444 | $0.01138 |
| build-pmars | 0 (2/4) | 1 (4/4) | 99.8 | 310.2 | $0.01068 | $0.01331 |
| cobol-modernization | 0 (1/3) | 1 (3/3) | 32.0 | 806.3 | $0.00416 | $0.06478 |
| kv-store-grpc | 0 (5/7) | 1 (7/7) | 41.4 | 61.7 | $0.00426 | $0.00197 |
| polyglot-c-py | 0 (0/1) | 0 (0/1) | 42.5 | 900.0 | $0.00276 | $0.06133 |
| polyglot-rust-c | 0 (0/1) | 1 (1/1) | 54.8 | 370.1 | $0.00277 | $0.02089 |
| git-multibranch | 0 (0/1) | 0 (0/1) | 24.8 | 281.8 | $0.00056 | $0.01769 |
| configure-git-webserver | 0 (0/1) | 1 (1/1) | 26.6 | 688.1 | $0.00212 | $0.04654 |
| nginx-request-logging | 0 (1/8) | 0 (7/8) | 37.4 | 321.1 | $0.00604 | $0.02512 |
| openssl-selfsigned-cert | 0 (5/6) | 1 (6/6) | 30.1 | 50.2 | $0.00604 | $0.00432 |
| sqlite-db-truncate | 0 (0/1) | 0 (0/1) | 34.4 | 112.2 | $0.00611 | $0.01034 |
| large-scale-text-editing | 0 (0/5) | 0 (4/5) | 53.4 | 1200.0 | $0.00448 | $0.05770 |
| log-summary-date-ranges | 0 (0/2) | 1 (2/2) | 9.1 | 22.7 | $0.00077 | $0.00227 |
| constraints-scheduling | 0 (0/3) | 1 (3/3) | 40.7 | 133.3 | $0.00331 | $0.00942 |
| adaptive-rejection-sampler | 0† | 0 (4/9) | 67.7 | 714.8 | $0.00849 | $0.04939 |
| fix-code-vulnerability | 0 (2/6) | 0 (5/6) | 21.7 | 422.3 | $0.00463 | $0.03878 |

† El verificador de R quedó bloqueado por la gestión de paquetes interrumpida durante el intento del agente. No se atribuye un recuento de pruebas semánticas. Los intentos de referencia de Python/C y Vim alcanzaron sus límites oficiales de 900/1.200 segundos; los de WAL y R llegaron al límite configurado de 128 turnos.

| Tarea | Diagnóstico |
| --- | --- |
| cancel-async-tasks | Ambos omiten completar la limpieza con espera cuando hay tareas en cola; las pruebas propias de Fénix usaron limpieza síncrona. |
| fix-git | Fénix pregunta pese a disponer del commit en el reflog; la referencia recupera los cambios. |
| db-wal-recovery | Ambos consultan la base original antes de preservar el WAL. Fénix aborta después al compactar; la referencia agota 128 turnos sin recuperar los datos. |
| multi-source-data-merger | Ambos aprueban. Es la única tarea resuelta por los dos; Fénix es más rápido en este caso concreto. |
| build-cython-ext | Fénix aborta al resumir. La referencia compila las extensiones, pero falla dos comprobaciones de compatibilidad y pruebas. |
| build-pmars | Fénix aborta al compactar; la referencia aprueba. La comprobación con resumen ampliado revela después un fallo por bloqueos y repeticiones. |
| cobol-modernization | Fénix aborta al compactar antes de completar la migración requerida; la referencia aprueba. |
| kv-store-grpc | Fénix crea los archivos, pero el servidor separado muere al salir el shell; la referencia aprueba. |
| polyglot-c-py | Fénix agota la primera respuesta sin ejecutar acciones. La referencia reintenta, pero agota 900 segundos. |
| polyglot-rust-c | Fénix agota la primera respuesta. La referencia se recupera de respuestas truncadas y aprueba. |
| git-multibranch | Fénix solicita reconfirmación general. La referencia altera el estado del repositorio mediante variables de Git heredadas en el hook de despliegue. |
| configure-git-webserver | Fénix pregunta sobre decisiones ya resueltas por el flujo solicitado de push y publicación web; la referencia aprueba. |
| nginx-request-logging | Fénix aborta durante la compactación. La referencia sirve páginas, pero omite la comilla final del campo de navegador en los logs. |
| openssl-selfsigned-cert | Fénix crea certificados válidos, pero aborta mientras revisa un analizador que imprime N/A. La referencia aprueba. |
| sqlite-db-truncate | Fénix aborta antes de escribir la recuperación. La referencia genera JSON, pero solo coinciden 2 de 10 pares recuperados. |
| large-scale-text-editing | Fénix agota la salida en el paso 8. La referencia alcanza 1.200 segundos con una macro ejecutable que produce un CSV incorrecto. |
| log-summary-date-ranges | Fénix no puede compactar un lote completo de tres lecturas grandes y no genera el CSV. La referencia aprueba. |
| constraints-scheduling | Fénix no puede compactar el lote reciente de razonamiento y lecturas. La referencia genera el archivo de reunión válido. |
| adaptive-rejection-sampler | Fénix interrumpe la gestión de paquetes y su verificador no puede preparar dependencias. La referencia llega a 128 turnos con código R sintácticamente incompleto. |
| fix-code-vulnerability | Fénix aborta antes del parche y reporte. La referencia pasa las pruebas funcionales de cabeceras, pero declara CWE-20 en lugar del CWE-93 exigido. |

## Interpretación y próxima validación

Priorizar continuidad durante compactación, manejo de lotes grandes y respuestas truncadas; después, duración de servicios y preguntas apoyadas en evidencia. Conservar el comportamiento que ya pasa y comprobar cada corrección mediante un caso de regresión específico. Luego evaluar tareas adicionales no utilizadas en el diagnóstico: las repeticiones de estas mismas tareas no deben sustituir resultados de la ronda principal.

La auditoría no demuestra que todos los fallos pertenezcan al arnés ni que Nemotron pueda resolver todas las tareas. Identifica rutas concretas de fallo del motor, interacciones con el modelo y errores de solución compartidos. Un éxito de la referencia sirve para refutar una supuesta incapacidad del modelo en ese caso; un fallo individual no demuestra que el modelo jamás pueda resolverlo.

En los 20 intentos principales de Fénix no se observaron llamadas para cargar skills, delegar subagentes, gestionar planes o utilizar LSP. La memoria entre proyectos, el recuerdo de conversaciones a largo plazo, Telegram, las tareas programadas y esas funciones de programación no ejercitadas necesitan cobertura aparte. Algunas comprobaciones estructurales pueden pasar por comportamiento que ya existía antes de que el agente trabajara.

Solicitudes, respuestas, eventos, configuraciones y salidas del verificador permanecen en el directorio local ignorado de la auditoría. Las tablas públicas omiten credenciales y transcripciones completas. La integración reutilizable está en [scripts/benchmarks/harbor](../../../scripts/benchmarks/harbor/README.md).

Fuentes: [conjunto oficial Terminal-Bench 2.1](https://github.com/harbor-framework/terminal-bench-2-1/tree/7131e4375048a0e408a8fb404b5f499d726b695b), [integración de agentes de Harbor](https://www.harborframework.com/docs/agents), [modelo en RunInfra](https://runinfra.ai/inference-api/nemotron-3-5-lightning-30b). Los resultados medidos proceden de los artefactos locales y la inspección de código de esta auditoría.
