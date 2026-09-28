# Comparación de arneses: tareas 19–21 de Terminal-Bench

[English](three-harness-tasks-19-21-luna-low-2026-09-26.md) · [Datos medidos](three-harness-tasks-19-21-luna-low-2026-09-26.json)

GPT-6 Luna low, servicio estándar, sin Flex. Un intento por combinación. Resultados: fenix 1/3, codex 3/3, opencode 3/3.

Cambios evaluados: inspección de muestras representativas, comprobaciones del comportamiento y restricciones obligatorias, skills de pruebas/datos actualizados y esquema explícito de herramientas agrupadas. Se mantiene desactivado el aprendizaje auxiliar durante evaluación. Los tres adaptadores mantienen el mismo contexto general de ejecución. Validación local: 256 pruebas del agente y cuatro del adaptador aprobadas, tres opcionales omitidas y compilación correcta.

En las tres evaluaciones nativas no hubo solicitudes de extracción de skills; todas las llamadas registradas devolvieron consumo. Las mejoras orientan la comprobación semántica; no convierten el motor en un verificador general de exactitud.

| Tarea | Arnés | Pruebas | Tiempo (s) | USD estimados | Tokens entrada | Tokens salida |
|---|---|---:|---:|---:|---:|---:|
| 19. adaptive-rejection-sampler | fenix | 7/9 | 900.39 | 0.05079362 | 2,212,138 | 15,932 |
| 19. adaptive-rejection-sampler | codex | 9/9 | 418.22 | 0.02395636 | 1,138,241 | 11,202 |
| 19. adaptive-rejection-sampler | opencode | 9/9 | 389.34 | 0.01719442 | 462,045 | 5,379 |
| 20. fix-code-vulnerability | fenix | 2/6 | 47.72 | 0.00825030 | 186,157 | 2,081 |
| 20. fix-code-vulnerability | codex | 6/6 | 21.34 | 0.00603471 | 115,478 | 1,246 |
| 20. fix-code-vulnerability | opencode | 6/6 | 41.25 | 0.00646198 | 111,847 | 1,161 |
| 21. count-dataset-tokens | fenix | 1/1 | 202.92 | 0.01325242 | 269,482 | 2,658 |
| 21. count-dataset-tokens | codex | 1/1 | 122.65 | 0.00601271 | 147,694 | 1,862 |
| 21. count-dataset-tokens | opencode | 1/1 | 139.82 | 0.00708930 | 184,676 | 1,941 |

≥ significa mínimo registrado cuando falta consumo de alguna solicitud. Los tokens de esos casos también pueden ser parciales. Las esperas por cuota y errores del transporte se conservan en el JSON; no se atribuyen automáticamente al arnés.

El tiempo corresponde a ejecución del agente: excluye descarga/preparación del contenedor, instalación del arnés y verificador externo. Incluye esperas del proveedor. Los tokens de entrada se acumulan entre llamadas; no representan una sola ventana de contexto. El coste de una tarea abandonada no es un coste por tarea resuelta.

[Precio oficial](https://developers.openai.com/api/docs/models/gpt-6-luna): por millón de tokens, entrada $0.10; lectura de caché $0.01; escritura de caché $0.125; salida $0.50. La entrada normal descuenta ambas categorías de caché. Estimación por uso, no factura; sin infraestructura ni impuestos.

## 19. Muestreo por rechazo adaptativo en R

Implementar un muestreador modular con validación, detección de falta de log-concavidad, pruebas estadísticas y archivo de muestras. Los tres instalaron R. Codex y OpenCode aprobaron las nueve comprobaciones; Fénix agotó el plazo de 900 segundos con siete aprobadas.

- **Fénix:** sus pruebas detectaron defectos reales: argumentos faltantes, intersecciones inválidas, dispersión incorrecta y problemas con densidades de pendiente constante. El motor rechazó un abandono prematuro y continuó trabajando. Sin embargo, alternó fórmulas sin estabilizar las propiedades matemáticas, y una prueba estadística larga quedó sin completar. Al final aprobó el muestreo básico oficial, pero la prueba formal excedió su plazo y faltaba el archivo requerido. Hizo 58 llamadas y acumuló 243,973 segundos de espera por cuota; esa espera contribuyó, pero no explica por sí sola el fallo.
- **Codex:** escribió código mientras instalaba R, aunque consultó procesos con esperas cortas repetidas. Investigó fallos intermitentes de la exponencial con varias ejecuciones, corrigió tangentes casi paralelas y pendientes iguales. Completó la suite con 3000 muestras y guardó normal_samples.txt. Aprobar el verificador no garantiza exactitud para todas las densidades posibles.
- **OpenCode:** reparó una instalación interrumpida, aisló el fallo exponencial con diez muestras y agrupó pendientes prácticamente iguales. Luego completó pruebas y archivo de muestras; aprobó el verificador.
- **Oportunidad pendiente:** comprobar primero propiedades matemáticas pequeñas y reproducibles, reparar el componente concreto y después ampliar la prueba estadística. Conservar lo que ya funciona, seguir los archivos exigidos como entregables y diagnosticar pruebas lentas a partir de progreso observado. No hace falta otro revisor obligatorio. Esta ronda no es una comparación pareada que demuestre mejora causal del prompt.

## 20. Reparación de validación de cabeceras en Bottle

Identificar y corregir una vulnerabilidad, generar report.jsonl con su CWE y ejecutar pytest. Codex y OpenCode aprobaron las seis comprobaciones oficiales y las 367 pruebas del repositorio. Fénix aprobó dos comprobaciones, no reparó el código ni creó el informe.

- **Fénix:** buscó errores de validación genéricos y se concentró en parse_date, que devuelve None ante ciertas fechas inválidas. Encontró una prueba que exigía ese comportamiento y lo trató como un conflicto con la tarea. Preguntó al usuario no disponible y el motor aceptó el bloqueo. Había consultado git status/log, pero no el diff ni la suite existente. El usuario nunca identificó parse_date como la vulnerabilidad: la supuesta duda nació de una hipótesis sin confirmar. Los programas usaron nombres correctos, pero algunas búsquedas independientes devolvieron solo uno de sus resultados seleccionados. Se registró una llamada de extracción de memoria; desactivar aprendizaje de skills no desactiva toda memoria.
- **Codex:** revisó el diff, encontró las comprobaciones eliminadas de CR/LF/NUL en _hkey y _hval, restauró ValueError, informó CWE-93 y pasó pytest.
- **OpenCode:** llegó al mismo parche mediante diff, código y pruebas. Su título inicial anticipaba CWE-20, pero el informe final basado en evidencia clasificó correctamente CWE-93.
- **Oportunidad pendiente:** separar hipótesis de diagnóstico de requisitos del usuario. Antes de preguntar sobre un cambio de comportamiento, revisar el código modificado y ejecutar la prueba existente pertinente; demostrar que el camino sospechado explica el fallo. Conservar preguntas legítimas y permisos. El control actual comprueba que existe una pregunta/evidencia, pero no demuestra que la hipótesis elegida sea pertinente. Preferir lecturas directas en paralelo cuando se necesitan todos los resultados.

## 21. Conteo de tokens de un dataset de Hugging Face

Leer el README, seleccionar el texto generado por DeepSeek de los dominios científicos, usar Qwen2.5-1.5B-Instruct y guardar el entero. Esta tarea amplía la selección original de veinte con la regla alfabética/de recursos registrada; el 21 no es un ordinal oficial del benchmark. Los tres aprobaron y escribieron 79586.

- **Fénix:** cargó el skill de Hugging Face y su referencia de datos, leyó el README, instaló dependencias e inspeccionó esquema y categorías reales. Corrigió el filtro literal science que no seleccionaba filas y contó reasoning y solution para biology, chemistry y physics. Comprobó subtotales y releyó answer.txt: 26 filas, 63841 tokens de razonamiento y 15745 de solución. Quedó sobrecoste por buscar en la web un repositorio ya conocido, recargar objetos y repetir conteos.
- **Codex:** descargó directamente README y Parquet de metadatos, inspeccionó frecuencias por dominio, contó ambos campos y guardó el resultado. Hizo 11 llamadas frente a 15 de Fénix; también repitió comprobaciones e instalaciones de dependencias.
- **OpenCode:** leyó README y metadatos, investigó por qué el filtro science estaba vacío, seleccionó las disciplinas científicas y verificó el entero guardado. Hizo 14 llamadas.
- **Oportunidad pendiente:** inspeccionar los valores categóricos reales antes de filtrar y agrupar cálculo final, subtotales independientes, escritura y lectura en un mismo script reutilizando objetos cargados. Resolver la ambigüedad con los datos disponibles. El acierto muestra un flujo observado correcto, no demuestra por sí solo que las nuevas instrucciones causaran una mejora.

## Protocolo y límites

Codex 0.157.1; OpenCode 1.18.32; Fenix SHA256 `e85925f73a7d6f7f1acf21e11bedaa7c663a7809cdc8dedf36348e0db280d113`; dataset `7131e4375048a0e408a8fb404b5f499d726b695b`.

Contenedor oficial nuevo por intento, 1 CPU, 2 GiB y 900 segundos de plazo del agente. Orden rotado: Fénix/Codex/OpenCode; Codex/OpenCode/Fénix; OpenCode/Fénix/Codex. Se congelaron binario, adaptadores y configuración. Sin cambios durante la medición, intentos adicionales de tarea ni exposición de tests ocultos. Los tres arneses mantienen el mismo contexto general del entorno que en la ronda anterior. La evaluación nativa mantiene desactivado el aprendizaje de skills en segundo plano. Se respetaron las cuotas mediante el mismo proxy. El JSON conserva respuestas, tiempos, tokens, esperas y rutas privadas de las trazas.

Las tareas 19 y 20 completan la selección original. La 21, count-dataset-tokens, amplía la selección: primera tarea restante por orden alfabético que usa texto, CPU y plazo oficial de hasta 900 segundos. Se eligió antes de observar resultados. No se desplegó ni se modificó el agente durante la medición. Tres tareas no permiten concluir superioridad general; las mejoras respecto de las tareas 16–18 no se midieron con una repetición pareada.
