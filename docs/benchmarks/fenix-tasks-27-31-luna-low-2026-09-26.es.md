# Fénix: auditoría de las tareas 27–31

[English](fenix-tasks-27-31-luna-low-2026-09-26.md) · [Español](fenix-tasks-27-31-luna-low-2026-09-26.es.md)

Resultado oficial: **3/5 tareas aprobadas (60%)**. Solo Fénix, GPT‑6 Luna low, sin Flex ni Jev; un intento por tarea. Esta selección no representa el resultado general de Terminal-Bench.

## Correcciones anteriores a las cinco tareas

- Una aclaración opcional o una pregunta técnica sin investigar ya no crean por sí solas un bloqueo por falta de usuario en modo no interactivo. Se exige tipo de dependencia, cita del encargo y justificación. Se conservan las preguntas legítimas, los permisos y las comprobaciones de requisitos.
- El cierre exitoso rechaza requisitos declarados sin verificar y admite limitaciones ajenas al encargo sin dispensar las pruebas obligatorias. Son controles de consistencia y procedencia, no una prueba semántica universal.
- Guías y skills de interpretación de formatos, datos e imágenes: obtener convenciones de evidencia real, comprobar la suposición de mayor riesgo y no confundir archivo escrito con resultado correcto.
- Validación previa: 261 pruebas Rust aprobadas, 3 opcionales omitidas, 4 pruebas del medidor aprobadas y compilación correcta.

## Resultados medidos

| # | Task | Resultado | Pruebas | Tiempo agente | Espera por cuota | Coste USD registrado |
|---|---|---|---|---|---|---|
| 27 | `hf-model-inference` | Aprobó | 4/4 | 202.6 s | 20.9 s | $0.02287 |
| 28 | `largest-eigenval` | Aprobó | 3/3 | 72.9 s | 0.0 s | $0.00901 |
| 29 | `make-doom-for-mips` | Falló | 0/3 | 900.3 s | 521.2 s | $0.08657* |
| 30 | `merge-diff-arc-agi-task` | Aprobó | 5/5 | 355.1 s | 46.6 s | $0.02590 |
| 31 | `model-extraction-relu-logits` | Falló | 0/1 | 471.7 s | 59.9 s | $0.03860 |

Coste estimado de los recibos disponibles: **$0.18295**. *Falta el uso de una llamada interrumpida al finalizar Doom; no es un coste total exacto. Tiempo total de los trabajos, incluida preparación/verificación/limpieza: 44.3 min. El tiempo del agente ya incluye la espera por cuota; no deben sumarse ambas columnas.

Tokens registrados: 5,726,195 de entrada (4,725,644 de lectura y 769,223 de escritura de caché, incluidos en la entrada), 32,822 de salida. 167 llamadas con respuesta HTTP; la interrupción mencionada limita estos totales.

## Auditoría por tarea

### 27. hf-model-inference

Descargó el modelo, creó el endpoint, detectó el error de formato con solicitudes reales, corrigió la suposición de lista anidada, reinició el servicio y verificó casos positivos, negativos y entrada inválida. Pasó las cuatro pruebas. Hubo llamadas evitables por búsqueda repetida de herramientas, sondeos y una prueba anterior al reinicio.

Ajuste derivado: Reforzar el skill de servicios: dependencias listas, reinicio intencional tras editar, espera acotada del endpoint y comprobación funcional. Reutilizar herramientas visibles; no automatizar reinicios indiscriminados ni añadir revisores.

### 28. largest-eigenval

Primero intentó iteración de potencias compleja y falló el evaluador suministrado. Cambió a scipy.linalg.eig(check_finite=False), seleccionó el autovalor de mayor módulo y volvió a verificar. Pasó las pruebas oficiales de par propio, dominancia y velocidad. No pidió aclaraciones ni intentó abandonar.

Ajuste derivado: Preferir una primitiva numérica probada antes de una optimización propia salvo que las mediciones la justifiquen; revisar sus hipótesis para matrices no simétricas y pares complejos. Conservar la recuperación exitosa, sin añadir controles numéricos al motor.

### 29. make-doom-for-mips

Compiló un ELF MIPS, pero no produjo el fotograma requerido. El motor rechazó un cierre fallido prematuro y continuó. Probó variantes de ABI/glibc y modificó la máquina virtual sin validar primero un programa mínimo compatible. Varias comprobaciones imprimían que faltaba la imagen pero terminaban con código cero. Agotó 900 segundos; 521,185 fueron espera preventiva del medidor para respetar la cuota. La última llamada se interrumpió sin recibo de tokens: el coste registrado es incompleto.

Ajuste derivado: Conservar el final de errores largos en el estado recuperable; probar primero un ejecutable mínimo y su entrada/salida en el destino. Preservar el ejecutor de aceptación y evitar emulaciones vacías sin validar. Comprobar la salida requerida con aserciones que fallen. Reforzar los skills existentes sin crear otro subsistema.

### 30. merge-diff-arc-agi-task

Pasó las cinco comprobaciones oficiales de Git y transformación. El nuevo control rechazó una aclaración opcional y continuó con los ejemplos. Un diálogo interactivo de tzdata causó esperas repetidas y un intento prematuro de cerrar como fallo. Después canceló el instalador atascado, reparó configuración/PATH y completó la fusión. También corrigió el intento de actualizar una rama activa mediante una referencia remota y checkout separado. Pasó todos los ejemplos y el caso oculto.

Ajuste derivado: Mostrar la salida reciente redactada cuando un proceso queda en silencio, sin avanzar cursores ni responder automáticamente. Reforzar instalación automática y atención a preguntas explícitas en el skill existente. Conservar la recuperación Git y el rechazo efectivo de dudas opcionales; no crear otro gestor.

### 31. model-extraction-relu-logits

Creó un extractor por consultas y diferencias finitas, pero leyó internals de forward.py y fijó en 20 una cantidad que era desconocida. Comparó reiteradamente contra A1 real y ajustó pasos, densidad y agrupamiento; el motor rechazó varios cierres fallidos sin evidencia suficiente. Una edición conservó los mismos hashes y aun así contó como reparación. Finalmente pasó su comprobación local de coseno, pero el verificador sustituyó la red por otra de 30 neuronas y distinta escala/semilla: 20 de 30 direcciones no cumplieron la tolerancia. La cantidad fija y la falta de generalización incumplieron la forma desconocida; consultar pesos internos no demuestra una solución de caja negra.

Ajuste derivado: Usar hashes nativos antes/después para no contar ediciones idénticas como reparaciones ni invalidar pruebas vigentes. Reforzar guías para respetar la caja negra, inferir dimensiones con evidencia permitida, variar entradas y usar aserciones que fallen. No introducir la respuesta del test ni un extractor específico en el arnés. Su eficacia necesita una evaluación futura controlada.

## Cambios después de la ronda

- Los resúmenes durables conservan el principio y el final de los errores. Errores distintos al final de una salida larga ya no se confunden solo por compartir advertencias iniciales; el registro completo sigue recuperable.
- Una espera por silencio devuelve hasta 1.200 bytes recientes del log redactado, identificados como repetidos. No mueve cursores, responde preguntas ni mata procesos automáticamente; mantiene propiedad de tarea, cancelación y timeout.
- Una edición con hashes nativos iguales no cuenta como reparación ni invalida verificaciones vigentes. Sin evidencia de igualdad se conserva el comportamiento conservador.
- Se actualizaron skills existentes de servicios, entorno, compilación, rendimiento, pruebas e ingeniería inversa: reinicio tras editar servicios, instalaciones sin diálogos innecesarios, ABI mínimo antes de un port grande, algoritmos probados, aserciones reales y respeto de límites de información. Versiones y hashes preservan copias personalizadas.
- Validación posterior: **265 pruebas Rust aprobadas, 3 opcionales omitidas**, compilación del servidor correcta. Las cuatro regresiones nuevas fallaron antes de la corrección y pasan después. No se añadieron modelos, revisores ni subsistemas.
- Estos cambios finales no se reevaluaron en Terminal-Bench: el resultado de la tabla corresponde al ejecutable anterior, congelado durante las cinco tareas. No se desplegó ni se subió a GitHub.

## Método y límites

- Selección fijada antes de observar resultados: siguientes tareas alfabéticas sin ejecutar, entrada textual, CPU y límite oficial ≤900 s. Una ejecución, recursos oficiales: 1 CPU y 2 GiB salvo ARC, con 4 GiB.
- Contenedor nuevo por tarea, sin intervención manual. Verificadores originales auditados solo después del resultado. Aprendizaje de skills desactivado; otras llamadas auxiliares están contabilizadas.
- Por espacio, la imagen de Hugging Face se construyó con el Dockerfile oficial sin editar. Las otras usaron imágenes publicadas. No se modificaron instrucciones, recursos ni verificadores.
- Doom falló con una importante espera de cuota; no hay prueba de que con más tiempo activo hubiera pasado. La extracción ReLU consultó internals: no demuestra un método limpio de caja negra, independientemente de su resultado oficial.
- No se puede atribuir una mejora porcentual comparando estas tareas con las cinco anteriores, que eran distintas. Los controles y guías tampoco garantizan corrección semántica.
- Tarifas guardadas por millón: entrada $0.10, caché lectura $0.01, caché escritura $0.125, salida $0.50. Estimación de uso, no factura; excluye infraestructura e impuestos.

Dataset: `7131e4375048a0e408a8fb404b5f499d726b695b`.
Evaluated binary SHA-256: `452d4439a01eaac559de9c8e108e5ad2f4f9151fc47c34f90230f2e6c94e06e2`.
Post-round binary SHA-256: `0b9fbcd3a0183de4d76396e3638f99719ea01351334fda6a0bc146161360ced0`.
Git HEAD: `7a0b28e6697abf3d71af001463da744ea5aee2e6`; `main`, with snapshotted local changes.

[JSON de métricas](fenix-tasks-27-31-luna-low-2026-09-26.json). Las trazas y credenciales permanecen fuera del repositorio.
