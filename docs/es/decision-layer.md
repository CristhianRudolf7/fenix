# Decisiones opcionales con Jev

[English](../en/decision-layer.md)

Jev complementa al modelo ejecutor: ordena candidatos autorizados de skills y memoria, puede ampliar una búsqueda de memoria de forma acotada, clasifica pares de recuerdos para revisión, propone rutas sencillas hacia un proyecto y solicita auditorías adicionales. Rust conserva el control de permisos, ámbitos y verificaciones obligatorias. Los hechos originales se conservan.

Está desactivado por defecto. La configuración autenticada por usuario se consulta y modifica en `/api/decisions/settings`. El documento en inglés incluye el JSON completo. `shadow: true` registra propuestas sin aplicarlas; `shadow: false` permite utilizarlas cuando superan el umbral. La confianza no garantiza que una respuesta sea correcta.

Ante errores, baja confianza o respuestas inválidas se mantiene el flujo anterior. Hay un tiempo máximo de espera, una pausa temporal tras fallos y caché de 60 segundos aislada por usuario. Solo se envían fragmentos acotados y redactados, que aun así pueden contener información privada. Jev no puede declarar una tarea terminada ni eliminar auditorías. Las consultas de varios proyectos siguen utilizando el router existente.

La evaluación registra llamadas, caché, fallos, tokens y coste estimado de estas decisiones por separado. Las llamadas adicionales pueden aumentar el coste; el ahorro debe comprobarse comparando las mismas tareas con y sin Jev. El precio configurable es una estimación, no una factura.

La prueba real del 24 de septiembre de 2026 devolvió HTTP 403, `customer_verification_required`, en unos 0,70 segundos: Vercel exige registrar una tarjeta válida. No se obtuvo una inferencia real ni se pudo medir ahorro. Las pruebas locales con proveedor simulado cubren errores, aislamiento, caché, confianza, modo observación y ordenación; las pruebas de almacenamiento comprueban la solicitud de revisión adicional.

## Validación después de verificar la cuenta (24 de septiembre de 2026)

Pasaron 14 comprobaciones reales dirigidas en instancias aisladas de Rust y 41 pruebas de regresión. Los diez casos de clasificación dieron la etiqueta esperada: promedio de 329 ms y rango de 226–468 ms. La consulta de varios proyectos produjo abstención; una etiqueta correcta de relevancia de skill no se aplicó por baja confianza. También funcionaron la ordenación de skills, caché sin tokens, señal de auditoría y evaluación persistida de recuerdos sin alterar los hechos ni llamar al modelo generativo (843 ms).

Las diez respuestas de clasificación reportaron US$0 con la promoción y US$0,00016338 de coste de mercado. Las 13 llamadas no almacenadas en caché consumieron en total 5.503 tokens de entrada y 707 de salida: estimación de tarifa de US$0,000231126, no factura. Las otras llamadas integradas no conservaron el coste facturado. No se midió una comparación completa contra el modelo anterior: todavía no hay porcentaje de ahorro por tarea. No se activó ni desplegó en producción. [Resultados](../benchmarks/jev-runtime-2026-09-24.json).

### Un intento de programación con Luna low (25 de septiembre de 2026)

Con GPT-6 Luna low, las instrucciones mejoradas, dos cuotas de organizaciones verificadas y Jev activo, `build-cython-ext` pasó 9/11 comprobaciones oficiales: todavía no aprueba (reward 0). El agente terminó en 714,31 segundos, frente al timeout y 2/11 del intento anterior con max. Las esperas por cuota bajaron de 379,6 a 13,862 segundos, sin errores 429. Las claves atendieron 18 y 17 solicitudes, incluida la validación inicial.

Jev intervino una vez (477 ms), recomendó reparar con confianza 0,41 y su consejo no se aplicó por quedar bajo el umbral 0,85. No se demostró una mejora atribuible a Jev; cambiaron varias variables a la vez.

La instalación global, los imports y el ejemplo funcionaron. Persistió `np.int` en una ruta de la extensión Cython y falló la suite del repositorio. El agente observó que faltaba pytest y aun así cerró como completado sin instalarlo ni terminar las pruebas requeridas. No se repitió el benchmark ni se desplegó en producción. [Resultados](../benchmarks/luna-low-diagnostic-jev-2026-09-25.json).
