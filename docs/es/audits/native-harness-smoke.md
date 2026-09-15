# Comprobación de integración nativa — 2026-09-15

[English](../../en/audits/native-harness-smoke.md) · [Guía del evaluador](../native-evaluation.md)

Tras implementar la evaluación nativa y las seis mejoras, pasaron **234 pruebas**
del workspace; cuatro pruebas opcionales quedaron omitidas. También pasaron
Clippy sin advertencias, las pruebas de documentación y la comprobación de
sintaxis del frontend. Las regresiones cubren contexto, recuperación, servicios,
espera de procesos, bucles, respuestas persistentes y paginación de tareas.

Se ejecutó un intento independiente de `log-summary-date-ranges`, tarea oficial
de Terminal-Bench 2.1, con revisión
`7131e4375048a0e408a8fb404b5f499d726b695b`, Nemotron
`nemotron-3-5-lightning-30b` mediante RunInfra, el evaluador Rust y Harbor 0.23.0.
Se conservaron instrucciones, recursos y verificador oficiales. El modo nativo
inicia un índice web nuevo sin rastreo en segundo plano; esta tarea no utilizó
herramientas web.

| Medición | Resultado |
| --- | --- |
| Recompensa oficial | 0/1 |
| Ejecución medida por el evaluador, incluida la espera final | 20,825 s |
| Solicitudes al proveedor, incluida configuración | 6 |
| Tokens de entrada, incluida caché | 46.687 |
| Entrada desde caché | 17.408 |
| Tokens de salida | 3.810 |
| Costo informado por el proveedor | USD 0,00220953 |
| Errores HTTP | 0 |
| Llamadas a herramientas registradas | 4 |
| Compactaciones activas | 1, correcta |

El evaluador exportó las trazas y entregó el resultado al verificador oficial.
La compactación redujo el contexto estimado de 16.843 a 12.819 tokens. Sin
embargo, el modelo listó el directorio de logs, intentó usar un argumento
`limit` no admitido y repitió el mismo listado. El detector detuvo la tercera
repetición sin progreso observado. No se produjo el archivo solicitado.
**La tarea falló; el recorrido de integración funcionó. Esto no demuestra una
mejora del porcentaje de aciertos.**

El informe nativo registra cinco respuestas de ejecución/resumen, con 46.667
tokens de entrada y 3.717 de salida. Los totales del proveedor también incluyen
la comprobación inicial de conexión. Hubo un primer lanzamiento detenido en la
comprobación de Docker, sin llamadas API; se reintentó después de confirmar que
el servicio estaba activo, sin reiniciarlo. El trabajo completo de Harbor tardó
aproximadamente 2m30s, incluyendo el contenedor y su preparación.

La auditoría original de 20 tareas sigue intacta. Este intento no equivale a
repetirla ni constituye una comparación con Terminus 2. Para medir mejoras de
calidad hay que repetir la evaluación pareada y añadir tareas nuevas.

Los artefactos privados están en `.fenix/evaluations/native-smoke-20260915/`,
fuera de Git: huellas del código, manifiesto del binario, resultados oficiales,
trazas y contabilidad. Se detuvieron el proxy temporal y el contenedor de la
tarea; solo se eliminó la imagen incorporada para esta comprobación.
