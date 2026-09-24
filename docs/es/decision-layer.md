# Decisiones opcionales con Jev

[English](../en/decision-layer.md)

Jev complementa al modelo ejecutor: ordena candidatos autorizados de skills y memoria, puede ampliar una búsqueda de memoria de forma acotada, clasifica pares de recuerdos para revisión, propone rutas sencillas hacia un proyecto y solicita auditorías adicionales. Rust conserva el control de permisos, ámbitos y verificaciones obligatorias. Los hechos originales se conservan.

Está desactivado por defecto. La configuración autenticada por usuario se consulta y modifica en `/api/decisions/settings`. El documento en inglés incluye el JSON completo. `shadow: true` registra propuestas sin aplicarlas; `shadow: false` permite utilizarlas cuando superan el umbral. La confianza no garantiza que una respuesta sea correcta.

Ante errores, baja confianza o respuestas inválidas se mantiene el flujo anterior. Hay un tiempo máximo de espera, una pausa temporal tras fallos y caché de 60 segundos aislada por usuario. Solo se envían fragmentos acotados y redactados, que aun así pueden contener información privada. Jev no puede declarar una tarea terminada ni eliminar auditorías. Las consultas de varios proyectos siguen utilizando el router existente.

La evaluación registra llamadas, caché, fallos, tokens y coste estimado de estas decisiones por separado. Las llamadas adicionales pueden aumentar el coste; el ahorro debe comprobarse comparando las mismas tareas con y sin Jev. El precio configurable es una estimación, no una factura.

La prueba real del 24 de septiembre de 2026 devolvió HTTP 403, `customer_verification_required`, en unos 0,70 segundos: Vercel exige registrar una tarjeta válida. No se obtuvo una inferencia real ni se pudo medir ahorro. Las pruebas locales con proveedor simulado cubren errores, aislamiento, caché, confianza, modo observación y ordenación; las pruebas de almacenamiento comprueban la solicitud de revisión adicional.
