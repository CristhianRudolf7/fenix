# Auditoría del arnés con Nemotron: protocolo

Estado: completado. Consulta el [informe de resultados](nemotron-terminal-bench-audit.md).

Se evalúan 20 tareas oficiales de Terminal-Bench 2.1 con Fénix y Terminus 2, usando el mismo Nemotron 3.5 Lightning de RunInfra y su razonamiento predeterminado activado. La selección se fijó antes de observar resultados. Es una muestra diagnóstica, no el puntaje del benchmark completo.

Se conserva el código actual de Fénix, incluida la memoria adaptativa. El agente ejecuta sus herramientas reales dentro de contenedores nuevos, con memoria independiente por intento. El controlador entrega la instrucción oficial por la API de chat y no resuelve tareas ni proporciona respuestas a preguntas. Los tests oficiales se ejecutan después de que termina el agente.

Cada contenedor conserva los recursos y plazos de la tarea: 1 CPU, 2 GiB y el tiempo indicado en la tabla del protocolo en inglés. Se ejecutan hasta dos intentos simultáneos. Fénix conserva su contexto total predeterminado de 32.768 tokens; la referencia declara 32.768 tokens de entrada en los metadatos del modelo. Ambos tienen un techo de salida de 16.384 por llamada y 128 pasos principales. Sus políticas de cálculo y compactación son diferentes: Fénix estima bytes y reserva espacio de salida. Ambos acceden al mismo modelo con capacidad real de 262.144 tokens. Se mantiene el límite interno de 2.048 tokens del resumen de Fénix para observar su efecto. Se mantiene el comportamiento de subagentes y memoria de Fénix; todas sus llamadas adicionales se contabilizan. No se aplica un presupuesto monetario externo.

Comandos y escrituras están autorizados dentro del contenedor. Si solo quedan tareas esperando al usuario durante 15 segundos, el intento autónomo termina y se ejecuta el evaluador. No se inventan respuestas. Esto distingue la autonomía sin asistencia del uso interactivo habitual.

Se guardan trayectorias, argumentos y resultados de herramientas, estados, compactaciones, comprobaciones oficiales, tokens y costos que devuelve el proveedor. El tiempo de instalación se separa del tiempo del agente. La suma de tiempos de API no equivale al tiempo total si hubo llamadas simultáneas. La comprobación inicial de conexión de Fénix se identifica y cuenta en el gasto.

Antes de las tareas de servidores se ajustó el adaptador para conservar el proceso de Fénix y sus servicios persistentes durante la verificación, cerrando su conexión de inferencia. Desde la tarea 6 se observan también los planes activos y se esperan dos segundos sin trabajo pendiente para dar tiempo a los eventos pub/sub. Ese margen se registra y cuenta en el tiempo de Harbor. Las primeras cinco tareas no crearon planes ni subagentes ni necesitaban servidores persistentes.

La clave real permanece en el host. Un proxy registra solicitudes y respuestas; un túnel local respeta la política HTTPS de Fénix. No se modifican instrucciones ni respuestas del modelo. El primer fallo de configuración del adaptador no alcanzó al modelo: se conserva como incidencia de integración y no se cuenta como fallo de una tarea.

La auditoría separará fallos de infraestructura, interpretación, planificación, preguntas, herramientas, terminal, contexto y cierre prematuro. Que el arnés de referencia acierte no demuestra por sí solo qué causó un fallo de Fénix: las discrepancias requieren inspección y repeticiones controladas.

Los artefactos privados permanecen en `.fenix/audits/terminal-bench-20260915/`, excluidos de Git. El protocolo completo, las versiones, los hashes, los precios y la lista de tareas están en [la versión en inglés](../../en/audits/nemotron-terminal-bench-protocol.md).
