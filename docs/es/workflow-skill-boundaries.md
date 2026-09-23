# Tres skills con responsabilidades distintas

> Revisión histórica de 145 skills. El catálogo actual contiene 167 tras la [revisión de diez repositorios](ten-repository-skills-review.md); las mediciones siguientes corresponden a la revisión anterior.

[English](../en/workflow-skill-boundaries.md) · [Comparación inicial](superpowers-comparison.md) · [Comprobaciones registradas](../skills/workflow-boundaries-review.json)

## Decisión revisada

El catálogo ahora contiene **145 skills**. La revisión de 142 demostraba cobertura amplia y mejoras de recuperación; no demostraba que esa cantidad fuese óptima. Tras revisar entradas, resultados y acciones permitidas, tres procedimientos merecen tener una entrada propia y encontrarse directamente.

| Skill nueva | Propósito concreto | Dónde estaba antes |
| --- | --- | --- |
| [agent-workflow-diagnosis](../../resources/skills/agent-workflow-diagnosis/SKILL.md) | Explicar mediante trazas por qué el agente repitió acciones, omitió una skill o consumió tiempo y tokens. Diagnóstico de solo lectura. | Referencia de local-troubleshooting, que también permite reparar servicios y ejecutar comandos. |
| [completion-verification](../../resources/skills/completion-verification/SKILL.md) | Comprobar si archivos y resultados entregados cumplen los requisitos, incluyendo documentos y datos. | Referencia de software-implementation, cuyo propósito principal es crear programas. |
| [review-feedback](../../resources/skills/review-feedback/SKILL.md) | Evaluar comentarios recibidos y aplicar correcciones válidas cuando estén autorizadas. | Referencia de code-review, cuyo alcance y efectos declarados eran revisión de solo lectura. |

Una solicitud de comprobar hojas de cálculo entregadas no debería depender de seleccionar implementación de software. Aplicar correcciones de una PR tiene límites diferentes de revisar un diff sin editar. Explicar acciones repetidas del agente requiere comenzar por su historial, no por reparar un servicio local. Estas diferencias se verifican en la organización anterior; no prueban por sí solas que un modelo haya cometido esos errores.

## Integración y compatibilidad

Los tres skills anteriores conservan sus responsabilidades principales y remiten al especialista cuando corresponde. Las nuevas entradas son autosuficientes, utilizan herramientas nativas y respetan los permisos del motor. Los metadatos describen efectos esperados; no sustituyen controles ejecutables.

Las referencias anteriores se mantienen por compatibilidad con skills instaladas o personalizadas que todavía las enlacen. Las instrucciones actuales remiten a las nuevas entradas; no es necesario cargar ambas versiones. Se registraron los hashes exactos de los tres padres anteriores y se conservaron como fixtures para verificar la migración. Las personalizaciones y desactivaciones se preservan.

El empaquetado existente incorpora automáticamente las nuevas carpetas. No se añadió otro coordinador, proveedor, hook o política de permisos. La activación requiere iniciar el binario recompilado con estos recursos; este cambio no reinicia ni despliega el servicio en ejecución.

## Cómo se comprueba

Antes de redactar los nuevos cuerpos se fijaron [24 consultas](../skills/workflow-boundary-cases.json): 18 para los tres procedimientos y seis de tareas vecinas que deben seguir seleccionando sus skills anteriores. La prueba emparejada desactiva las tres entradas y carga los padres anteriores para representar la línea base; después actualiza los padres y activa los especialistas. El resto del catálogo y el buscador permanecen iguales dentro de la prueba.

Los objetivos anteriores son los padres; los nuevos son los especialistas. Se compara el acceso al procedimiento pertinente bajo dos organizaciones, no la recuperación de nombres idénticos. Los seis casos negativos comprueban que los nuevos skills no ocupen las primeras dos sugerencias en esas tareas. No garantizan ausencia universal de activaciones incorrectas.

También se conservan las regresiones del catálogo y las 18 consultas anteriores. Dos destinos históricos ahora apuntan a los especialistas correspondientes; los archivos y resultados históricos originales siguen intactos. Se comprueban instalación, carga de recursos, actualización de originales, personalizaciones y activación. Buscar frases en Markdown no cuenta como prueba de calidad del agente.

La ejecución final pasó **15/15 pruebas nativas de skills**. Los **145 archivos de entrada** pasaron la validación de formato y sus herramientas declaradas existen en el motor. Clippy pasó para la biblioteca y sus pruebas, tratando advertencias como errores. La comparación de recuperación dio estos resultados:

| Grupo de consultas | Organización anterior: top 1 / top 3 | Organización actual: top 1 / top 3 |
| --- | ---: | ---: |
| 18 solicitudes de los nuevos procedimientos | 7/18 · 9/18 | 18/18 · 18/18 |
| 6 tareas vecinas | 3/6 · 5/6 | 5/6 · 6/6 |
| 18 solicitudes históricas | 10/18 · 11/18 | 18/18 · 18/18 |

Ninguno de los especialistas nuevos apareció entre las dos primeras sugerencias de las seis tareas vecinas. La primera versión reveló un fallo previo: implementar un parser CSV, solicitado en español, no encontraba su skill. Añadir alias de implementación corrigió ese caso conocido; no constituye una evaluación independiente. Pasaron las comprobaciones de migración, personalizaciones y desactivaciones.

Se validó una copia aislada porque cambios concurrentes en el manifiesto del servidor requerían actualizar el archivo de dependencias. Se añadieron esas entradas solo en la copia, manteniendo las versiones; esta tarea no cambió el `Cargo.lock` del proyecto. También se corrigió una llamada incompatible a `renameat2` del importador de skills concurrente, pasando los directorios ya abiertos en ambas copias. Los skills, fixtures, registro de migraciones y `skills.rs` probados coincidían con el proyecto. La comprobación cubre el subsistema de skills, no toda la aplicación modificada concurrentemente.

Los resultados exactos están en [el registro de comprobaciones](../skills/workflow-boundaries-review.json). Son pruebas de integración y desarrollo, no una evaluación independiente del modelo. No se atribuyen mejoras medidas de calidad, latencia, tokens o coste de inferencia. Añadir entradas también tiene un coste de contexto y selección.

## Por qué estos tres

La investigación de causa raíz sigue formando parte de reparar un defecto: comparte entrada y resultado con test-driven-repair. Preparar un worktree y entregar su rama comparten estado, propiedad e integración. Separarlos más no tiene ahora una justificación igualmente clara y añadiría selección redundante.

**145 tampoco es un límite definitivo.** Se añadirán procedimientos cuando las tareas muestren resultados recurrentes con requisitos distintos, fallos de selección o dificultad real para aplicar el procedimiento dentro de su skill actual. Asegurar mejoras generales del agente requiere ensayos aislados y repetidos con resultados evaluados independientemente, según la [guía de evaluación](../../resources/skills/skill-authoring/references/behavioral-evaluation.md).
