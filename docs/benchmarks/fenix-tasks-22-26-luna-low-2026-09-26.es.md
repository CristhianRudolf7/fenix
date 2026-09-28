# Fénix: auditoría de las tareas 22–26

[English](fenix-tasks-22-26-luna-low-2026-09-26.md) · [Español](fenix-tasks-22-26-luna-low-2026-09-26.es.md)

Resultado: **1/5 tareas aprobadas (20%)**. Solo Fénix, GPT‑6 Luna low, sin Flex ni Jev; un intento por tarea. Cambios en `main`, sin despliegue ni push. Esta muestra no es el resultado general de Terminal-Bench.

## Cambios anteriores a la ronda

- Inspección acotada de cambios Git al consultar el entorno, sin contenido de archivos ni descubrimiento de repositorios superiores. Es una captura con caché, no una vista en vivo.
- Consejo de caso mínimo cuando la misma prueba vuelve a fallar tras una reparación observada. Reutiliza el control de cierre existente, sin otro modelo ni cambios de permisos.
- Prompts y skills `test-driven-repair`, `test-strategy` y `huggingface-hub` reforzados: distinguir hipótesis de requisitos, comprobar progresivamente y reutilizar datos. Versiones y hashes previos permiten actualizar copias sin sobrescribir personalizaciones.
- Validación: **258 pruebas Rust aprobadas, 3 opcionales omitidas**, 4 pruebas del medidor aprobadas y compilación del servidor correcta. Las pruebas de permisos, preguntas legítimas y cierre se conservaron.

## Resultados

| # | Task | Resultado | Pruebas oficiales | Tiempo agente | Coste USD estimado | Entrada | Salida |
|---|---|---|---|---|---|---|---|
| 22 | `extract-elf` | Falló | 1/2 | 51.8 s | $0.00846 | 175,428 | 2,781 |
| 23 | `gcode-to-text` | Falló | 1/2 | 134.6 s | $0.01656 | 434,328 | 7,165 |
| 24 | `git-leak-recovery` | Aprobó | 5/5 | 37.2 s | $0.00505 | 84,145 | 1,384 |
| 25 | `gpt2-codegolf` | Falló | 0/1 | 43.1 s | $0.00886 | 93,441 | 1,746 |
| 26 | `headless-terminal` | Falló | 0/7 | 28.3 s | $0.00385 | 53,018 | 349 |

Total estimado: **$0.04278**, 50 llamadas. El tiempo del agente incluye esperas por cuota: 30.1 s acumulados, casi todos en G-code. Los cinco trabajos completos, con preparación, verificación y limpieza, tardaron aproximadamente 9.7 min.

No hubo errores HTTP, timeouts, fallos de memoria ni registros de uso pendientes. Las 50 respuestas del proveedor fueron HTTP 200. Los tiempos cortos de los bloqueos no significan eficiencia: no completaron el trabajo.

## Auditoría por tarea

### 22. extract-elf

Creó el extractor y comprobó algunas palabras del binario, pero inventó una base 0x400000 a partir del ejemplo. El verificador usaba direcciones virtuales de secciones: coincidencia 0%, frente al mínimo de 75%. Sus pruebas repetían esa base y fijaban 700 entradas, así que no eran independientes. También corrigió errores de invocación y de un offset de su propia prueba.

Mejora recomendada: Validar la suposición de mayor riesgo contra metadatos reales del formato y un caso independiente; no convertir valores del ejemplo en requisitos ni copiar expectativas de la implementación. No hace falta un revisor obligatorio.

### 23. gcode-to-text

Intentó reconstruir trayectorias. La expresión regular inicial omitía decimales como E.01926 y produjo varios resultados vacíos. Después rasterizó solo 92 segmentos de capas bajas. Al no encontrar librerías gráficas, escribió la etiqueta Embossed text y admitió que no verificó la inscripción, pero cerró como completado. Falló la comprobación del contenido.

Mejora recomendada: Validar un ejemplo real del formato y sus modos, revisar cobertura de capas antes de filtrar y usar renderizado/OCR si la proyección textual no basta. Distinguir escritura correcta de archivo de respuesta correcta: una incertidumbre central no debe convertirse en éxito mediante una advertencia.

### 24. git-leak-recovery

Inspeccionó referencias, reflogs y blobs; recuperó el secreto ficticio fuera del repositorio y eliminó los objetos inalcanzables dentro del entorno autorizado. Pasó las cinco pruebas oficiales, incluida conservación de commits válidos y checksum del contenido.

Mejora recomendada: Conservar este flujo acotado; esta tarea no justifica una capa adicional. Usó comandos Git directos, por lo que no demuestra un efecto causal del nuevo resumen de entorno.

### 25. gpt2-codegolf

Detectó que faltaba el índice TensorFlow, preguntó al propietario no disponible y se bloqueó sin crear gpt2.c. No intentó consultar documentación pública ni localizar metadatos compatibles. No hubo timeout, falta de memoria ni error del proveedor. Faltaban metadatos, pero no había evidencia de que solo el usuario pudiera aportarlos.

Mejora recomendada: Separar un desconocimiento técnico de una decisión que realmente depende del propietario. Antes de cerrar por falta de respuesta, conservar evidencia de recuperación autorizada pertinente. Un blocking_reason escrito por el modelo no demuestra que la consulta sea indispensable. Esto no prueba que obtener el índice baste para resolver toda la tarea.

### 26. headless-terminal

Preguntó por el contrato de BaseTerminal sin leer base_terminal.py, que estaba disponible. Su propia justificación decía que el objetivo era claro y no hacía falta bloquear. Tras listar el archivo, volvió a decir que debía continuar, pero solicitó blocked y el motor lo aceptó porque ask_user no tenía un usuario disponible. No escribió implementación: fallaron las siete pruebas por módulo ausente.

Mejora recomendada: No convertir el fallo de entrega de una pregunta opcional en prueba de dependencia del usuario. Contrastar el bloqueo propuesto con el objetivo y los archivos disponibles en el control actual de entrada/cierre, conservando dudas legítimas y permisos. Validar la longitud del texto blocking_reason no detecta contradicciones; no hace falta imponer múltiples revisores a todas las tareas.

## Prioridades sin añadir complejidad innecesaria

1. Reforzar el control existente de bloqueo: una pregunta no entregada no prueba que falte una decisión indispensable del usuario. Primero distinguir datos técnicos investigables de alcance, credenciales o permisos realmente ausentes. Conservar las consultas legítimas.
2. Revisar la evidencia del resultado central al cerrar. Guardar un archivo prueba persistencia, no que su contenido resuelva la solicitud; una incertidumbre reconocida sobre la respuesta no debe convertirse en éxito.
3. Dirigir la verificación independiente a la suposición de mayor riesgo: formato de direcciones, unidades, sintaxis o selección de datos. No añadir pruebas genéricas ni revisores a todas las tareas.

Estas mejoras quedan como conclusiones de la auditoría; no se cambió el ejecutable entre tareas ni se hicieron intentos adicionales. Las guías añadidas no resolvieron el bloqueo prematuro en esta ronda. No se puede atribuir causalmente el resultado a un solo cambio, ni comparar su porcentaje con el de tareas diferentes.

## Reproducibilidad y límites

- Terminal-Bench 2.1: `7131e4375048a0e408a8fb404b5f499d726b695b`.
- Fénix binary SHA-256: `e8efb479688cd5b6fdc97a35025edd6f8e58a5cfd7f2b9df11e93e7d785f59d8`.
- Git HEAD: `7a0b28e6697abf3d71af001463da744ea5aee2e6` (incluye cambios locales registrados en la captura privada).
- Selección previa: siguientes tareas alfabéticas sin ejecutar, de entrada textual, CPU y límite oficial de agente ≤900 s; se mantiene excluida la tarea de ajedrez con imagen. Recursos oficiales: 1 CPU, 2 GiB por tarea salvo GPT‑2, con 8 GiB.
- Contenedores nuevos; verificadores originales sin cambios, inspeccionados para auditar después de cada resultado. Aprendizaje automático de skills desactivado antes de iniciar; se contabilizan otras llamadas auxiliares. Sin intervención manual en las tareas.
- El consejo nuevo para fallos repetidos no se activó en estas trazas y no se cargaron los tres skills modificados. La evidencia local valida mecanismos, no una mejora de puntuación.
- Tarifa guardada por millón de tokens: entrada $0.10, lectura de caché $0.01, escritura de caché $0.125, salida $0.50. Se separan las categorías, sin doble contabilidad. Coste estimado, no factura; sin infraestructura ni impuestos. [Fuente de tarifa](https://developers.openai.com/api/docs/pricing), registrada el 2026-09-26.

[JSON](fenix-tasks-22-26-luna-low-2026-09-26.json) contiene métricas y hashes de artefactos; las credenciales y trazas completas permanecen fuera del repositorio.
