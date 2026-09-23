# Especialista de seguridad

Fénix incorpora el perfil `security` en su motor de subagentes existente. El maestro conserva una instrucción breve para delegar; los skills especializados no aparecen en sus sugerencias, búsquedas ni catálogo habitual. Sigue disponible mientras se ejecuta la auditoría. Se mantiene un único nivel de delegación y el tratamiento existente de cancelación, seguimiento, permisos y hasta tres intentos ante errores transitorios del proveedor.

## Uso

El maestro llama a `subagent_spawn` con `profile: "security"`, objetivo concreto, resultado esperado y `constraints` que identifique el objetivo de la auditoría y las operaciones permitidas. El perfil persiste con la asignación y participa en la deduplicación. Un mensaje de seguimiento no puede cambiarlo. Las asignaciones antiguas conservan el perfil `general`.

Los hitos de `plan_manage` también admiten `profile: "security"`; el plan debe incluir restricciones de alcance. Cambiar el perfil en una revisión invalida el trabajo anterior mediante el mecanismo existente de versiones. Los revisores de planificación especializados mantienen sus restricciones de solo lectura y su propio formato de revisión.

El especialista utiliza el modelo configurado para tareas. No modifica el modelo de conversación ni la configuración del revisor Sentinel para publicación de skills.

## Separación de capacidades

El motor filtra antes de buscar, paginar o sugerir skills. También comprueba la carga directa por nombre, las referencias y los recursos montados para comandos. Los skills incluidos de seguridad permanecen restringidos aunque una copia antigua no tenga categoría. Los personalizados con `metadata.category: security` quedan disponibles para este perfil; los desactivados no se pueden cargar.

El especialista dispone además de un pequeño catálogo auxiliar de navegación de código, archivos y pruebas. No puede gestionar subagentes, crear recordatorios, modificar memoria ni invocar conectores. La interfaz administrativa del propietario conserva la biblioteca completa.

Esta separación organiza catálogos y herramientas; no constituye un nuevo sandbox del sistema operativo. Se reutiliza el aislamiento existente. Las restricciones redactadas de cada auditoría no son una nueva lista de destinos de red impuesta por el motor. Tener acceso autorizado a comandos tampoco se convierte automáticamente en acceso de solo lectura. El control adicional por destino requiere soporte específico del servicio de ejecución.

## Informes y herramientas

Para completar una auditoría, `subagent_finish.security_report` debe incluir alcance, cobertura y hallazgos. La cobertura distingue comprobado, inconcluso y no evaluado. Cada hallazgo tiene ID único, estado confirmado/sospechoso/descartado, gravedad, ubicación, condiciones, evidencia y propuesta de corrección. Un informe vacío no certifica seguridad. Rust valida la estructura, no la veracidad de las conclusiones del modelo. El maestro recibe un resumen acotado con recuentos y hasta ocho hallazgos; el resultado completo permanece recuperable por el ID del subagente.

La herramienta nativa `security_scan_read` interpreta archivos locales SARIF 2.1.0 o JSON de Semgrep. Entrega diez candidatos por página y conserva errores o ejecuciones incompletas. No ejecuta analizadores, instala paquetes, sigue enlaces del informe ni utiliza servicios externos. Mantiene el límite existente de archivos UTF-8 de 1 MiB; los informes mayores deben dividirse desde el productor. El código de salida del proceso y la cobertura real se verifican aparte.

## Skills propios

Se añaden seis procedimientos con referencias bajo demanda:

- `security-change-review`: revisión de cambios y sus consumidores.
- `security-finding-validation`: evidencia, falsos positivos y variantes.
- `security-static-analysis`: analizadores locales y cobertura.
- `security-rust-boundaries`: unsafe, FFI, concurrencia, cancelación y errores.
- `security-property-testing`: invariantes y secuencias de estados.
- `security-fix-validation`: reproducción original, variantes y comportamiento legítimo.

Se refuerzan también cuatro skills existentes: revisión de código, dependencias y secretos, evaluación de agentes y revisión de fuentes de skills. Las copias originales conocidas se actualizan al iniciar; las personalizadas se conservan.

Se estudiaron los procedimientos y scripts de Trail of Bits, junto con las ideas de PentAGI y Shannon. La implementación es propia y no incorpora esos frameworks. El análisis detallado, las fuentes y las limitaciones se encuentran en la [documentación en inglés](../en/security-specialist.md).

Las pruebas automatizadas comprueban separación, persistencia, llamadas directas, finalización estructurada y errores de analizadores. No equivalen a un benchmark real de detección de vulnerabilidades ni garantizan ahorro de tokens.

## Validación

Se ejecutaron las pruebas de biblioteca, planificación, subagentes y almacenamiento, además de validar diez paquetes de skills y sus referencias. Las respuestas de modelo en integración son simuladas y deterministas: verifican el motor, no la precisión del modelo al descubrir vulnerabilidades. Se comprueba también la compilación del servidor con los nuevos esquemas. No se reinició producción.
