# Revisión de gstack y mejoras propias de Fénix

[English: análisis completo y matriz por funcionalidad](../en/gstack-review.md)

Revisión del 21 de septiembre de 2026 sobre [garrytan/gstack, commit a6b3a57512ca6d5c6aa5b68f74f736195021f96e](https://github.com/garrytan/gstack/tree/a6b3a57512ca6d5c6aa5b68f74f736195021f96e), versión declarada 1.87.4.

Sí aporta mejoras útiles, aunque Fénix ya tenía muchos procedimientos equivalentes. Añadimos **tres skills originales y mejoramos ocho existentes**. El catálogo pasa de 177 a 180. No instalamos gstack ni copiamos sus prompts, ejecutables o assets.

## Alcance del análisis

El [inventario reproducible](../research/gstack-source-inventory-2026-09-21.json) registra rutas, tamaños y hashes: 61 archivos `SKILL.md`, de los cuales 53 son workflows principales, uno es el enrutador, cuatro son adaptaciones OpenClaw, uno es un ejemplo de navegador y dos son fixtures de pruebas. También hay 102 plantillas, incluidas secciones de carga diferida.

Inventariamos todas las entradas y los esquemas de las plantillas; examinamos instrucciones, secciones vinculadas y límites relevantes del navegador, hooks y arquitectura. Profundizamos en experiencia del desarrollador, salud del código, retrospectivas, planificación, QA, publicación, documentación, contexto y creación de skills. Esto no equivale a leer línea por línea todo el código, todos los preámbulos generados o todas las plantillas. No ejecutamos el instalador, sus pruebas, navegador, puente iOS, proveedores de modelos ni memoria remota. La matriz inglesa identifica todas las familias y decisiones, incluidas las entradas que solo tienen plantilla.

## Skills nuevas

| Skill | Qué mejora | Diferencia respecto de gstack |
| --- | --- | --- |
| `developer-experience-review` | Audita el recorrido desde documentación e instalación hasta la primera operación útil de una API, CLI o SDK. | Separa pruebas reales de estimaciones de un plan; mide tiempos observados y errores recuperables sin imponer puntuaciones ni tiempos universales. |
| `code-health-review` | Ejecuta los checks configurados y entrega resultados, fallos y cobertura ausente. | Conserva el código de terminación real; sin checks no declara éxito y no calcula una nota arbitraria de calidad. |
| `engineering-retrospective` | Examina entregas, reprocesos, fallos y bloqueos durante un periodo. | Usa ventanas comparables y evidencia de PR/despliegue; no deduce horas trabajadas ni productividad personal de los commits. |

## Skills mejoradas

| Skill | Cambio |
| --- | --- |
| `requirements-and-planning` | Conserva requisitos aceptados y revisa dependencias y comprobaciones cuando cambia una decisión. Entrega el plan actualizado. |
| `architecture-design` | Explicita quién detecta y recupera cada fallo importante, efectos parciales y cómo se distribuye o actualiza un nuevo artefacto. |
| `frontend-design-and-accessibility` | Diferencia auditoría visual y corrección autorizada; comprueba apariencia e interacción por separado. |
| `browser-regression-testing` | Diferencia informar de bugs y repararlos, conserva reproducciones y verifica resultados persistidos sin transacciones reales no autorizadas. |
| `performance-diagnosis` | Compara condiciones equivalentes de navegador, conserva la línea base y distingue mediciones ausentes de valores cero. |
| `release-and-ci` | Vincula pruebas al artefacto final, incluidos manifiestos y configuración. Añade una referencia de observación posterior al despliegue. |
| `technical-documentation` | Relaciona cambios públicos con guías, referencias, ejemplos y diagramas afectados, conservando historial y ambos idiomas. |
| `skill-authoring` | Convierte flujos observados en procedimientos reutilizables con casos de variación/fallo y fixtures sin secretos; no confunde una captura antigua con compatibilidad actual. |

La referencia de publicación comprueba entorno y versión realmente servida, diferencia una línea base previa de una captura posterior, acota la vigilancia y exige verificar la recuperación real. No promete supervisión continua al terminar la tarea.

## Qué aprovechamos sin duplicar

- Producto, alcance y especificaciones: ya existen descubrimiento, investigación de usuarios, planificación y gestión de issues.
- Revisión de código y depuración: ya existen trazado de consumidores, reproducción, corrección y validación de hallazgos.
- Diseño: ya existen tokens, comparación visual y presentación de opciones mediante herramientas nativas.
- Navegación y extracción: conservamos el navegador y la búsqueda propios de Fénix.
- Memoria y recuperación: mantenemos proyectos, historial y checkpoints existentes, sin crear otra base de contexto.
- Seguridad: mantenemos el perfil especialista y sus procedimientos de evidencia, reproducción y validación.
- Documentos, diagramas y PDF: conservamos nuestras herramientas; esta revisión no añade el motor de maquetación HTML de gstack.
- Evaluación de modelos: ya existen selección de modelos y diseño de experimentos con límites de evidencia.
- iOS: los procedimientos existentes necesitan herramientas reales disponibles; una skill no proporciona por sí sola un iPhone o su puente de control.

## Qué no trasladamos

No incorporamos importación general de cookies, emparejamiento remoto con autoridad amplia, sincronización de conversaciones a servicios externos, perfiles psicológicos inferidos, telemetría de gstack ni sus actualizaciones automáticas. Tampoco imponemos su esquema de versiones o publicación automática de todo lo que exista en el árbol de trabajo.

Los hooks `freeze` y `careful` ayudan a evitar errores, pero no sustituyen aislamiento entre usuarios: la propia documentación indica que restringir Edit/Write no restringe escrituras mediante Bash. En Fénix la protección efectiva sigue correspondiendo al runtime y su aislamiento. Este análisis no certifica la seguridad de ninguno de los sistemas.

## Verificación y activación

Las 11 entradas afectadas pasaron el validador de formato. Añadimos 26 consultas de descubrimiento en inglés/español, incluidas ocho de tareas vecinas, y una prueba de actualización que conserva personalizaciones, activación y referencias modificadas. Los resultados finales y sus límites están en la [evidencia de validación](../research/gstack-validation-2026-09-21.json).

Son regresiones de desarrollo, no un benchmark independiente que demuestre un aumento medido de inteligencia. No se probaron integraciones autenticadas externas ni se ejecutó gstack.

Conservamos los originales y hashes de las ocho skills modificadas para actualizar solo copias de fábrica reconocidas. Cargo incorpora automáticamente las nuevas entradas y recursos; las personalizaciones y skills desactivadas permanecen intactas. Los cambios están en el repositorio y se activarán al instalar la nueva compilación. Esta tarea no reinició ni desplegó el servicio activo.
