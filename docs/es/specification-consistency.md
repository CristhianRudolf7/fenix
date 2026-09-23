# Coherencia de especificaciones en Fénix

[English](../en/specification-consistency.md)

Se adaptaron ideas de [GitHub Spec Kit](https://github.com/github/spec-kit/tree/d4229c071c7ea3885b43e8a7739847300f618f13) mediante instrucciones propias en tres skills existentes, sin instalar su CLI ni copiar sus prompts:

- **Planificación:** detectar requisitos ambiguos, contradicciones entre contratos y tareas, y criterios sin comprobación prevista. Distinguir resultados técnicos de métricas de negocio que requieren observación posterior.
- **Implementación:** conservar referencias a requisitos y ejemplos de aceptación, respetar límites y errores de los contratos y revisar cobertura además de pruebas aprobadas.
- **Verificación:** separar incumplimientos observados de falta de evidencia, convertirlos en correcciones acotadas y evitar duplicar pendientes o reiniciar trabajo válido.

Las referencias detalladas se cargan para tareas relevantes con varios requisitos o contratos. Se reutilizan contratos de proyecto, planes versionados y evidencias; los cambios propuestos no se consideran aplicados. No se imponen documentos, ramas ni aprobaciones adicionales para tareas sencillas.

Las versiones pasan a planificación 1.3.0, implementación 1.5.0 y verificación 1.1.0. La migración existente reconoce las versiones originales anteriores y conserva personalizaciones, referencias modificadas y skills desactivadas. Las instalaciones existentes reciben las mejoras al reconstruir el runtime e instalar sus recursos o iniciarlo.

La validación comprueba metadatos, búsqueda nativa, recursos incluidos y migración sin pérdida de personalizaciones. No demuestra un aumento medido de inteligencia ni garantiza que una revisión del agente sea correcta. Este cambio no añade controles de backend ni despliega producción.

[Registro de validación](../research/specification-consistency-validation-2026-09-21.json): 39 pruebas nativas aprobadas, incluidos los dos casos nuevos de migración y búsqueda; metadatos válidos en las tres skills.
