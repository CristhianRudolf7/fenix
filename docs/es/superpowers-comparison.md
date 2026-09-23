# Comparación de Superpowers con Fénix

[English](../en/superpowers-comparison.md) · [Evidencia verificable](../skills/superpowers-review.json) · [Consultas de prueba](../skills/superpowers-discovery-cases.json)

Este informe registra la primera iteración con 142 skills. Una [revisión posterior de límites](workflow-skill-boundaries.md) alcanzó 145 entradas, la [revisión de cuatro repositorios](repository-skills-comparison.md) llegó a 156 la [revisión de OpenClaw](openclaw-skills-review.md) alcanzó 164 y la actual [revisión de diez repositorios](ten-repository-skills-review.md) llega a 167. Las mediciones originales siguientes se conservan como evidencia histórica.

Revisión del **19 de septiembre de 2026**, aplicada al agente Fénix de este repositorio.

## Dictamen

**Fénix ya cubre los objetivos de los 15 skills de Superpowers, pero conviene profundizar algunos procedimientos y mejorar su recuperación.** Se reforzaron nueve skills existentes y se añadieron cinco referencias opcionales bilingües. El catálogo conserva 142 skills. Esa es nuestra adaptación: procedimientos compatibles con el motor de Fénix, sin instalar un segundo sistema de coordinación.

La diferencia entre cobertura y eficacia es esencial. Verifiqué archivos, mecanismos del runtime y recuperación de skills. **No se ejecutó una comparación con un modelo real que permita afirmar que ahora programa mejor, tarda menos o consume menos tokens.** Las mejoras de comportamiento tienen una justificación concreta, pero su magnitud sigue sin medirse.

## Fuente y alcance

Repositorio: [obra/superpowers](https://github.com/obra/superpowers/tree/5bf4e78011075bcfc0dc295f0724994cd123ee71), versión **6.4.1**, commit `5bf4e78011075bcfc0dc295f0724994cd123ee71`. La fecha del commit equivale al 19 de septiembre de 2026 a las 00:31:35 UTC. Los enlaces fijan esa revisión; no dependen de lo que publique después la rama principal.

Se leyeron los 15 `SKILL.md`, referencias seleccionadas, manifiestos y hook de inicio; también se inspeccionó la organización de pruebas y su documentación para Claude. No se ejecutaron scripts de Superpowers ni se auditaron todos sus auxiliares, plataformas o evaluaciones externas. Su README remite a otro proyecto de evaluaciones, que no se evaluó aquí. Se revisó la [licencia MIT](https://github.com/obra/superpowers/blob/5bf4e78011075bcfc0dc295f0724994cd123ee71/LICENSE). Las instrucciones añadidas son originales; no se redistribuyen sus textos, scripts ni recursos.

En Fénix se inventariaron los 142 skills y se revisaron los procedimientos pertinentes, instrucciones del agente y mecanismos de búsqueda, carga, instalación, planificación y recuperación. No equivale a una auditoría exhaustiva de cada skill de oficina, ciencia o negocio. La [revisión anterior de 66 repositorios](skill-source-review.md) solo había consultado secciones de tres skills de Superpowers; este informe amplía ese análisis sin alterar sus cifras históricas.

Los skills personales de Codex y los que Fénix distribuye son sistemas distintos. La documentación de OpenAI explica Codex; las capacidades de Fénix se comprobaron en su propio código. [Documentación oficial de skills](https://learn.chatgpt.com/docs/build-skills).

## Comparación de los 15 skills

La tabla inglesa enlaza cada archivo original al commit revisado. Aquí se presenta el resultado completo en español. «Cubierto» significa que existe un procedimiento o capacidad equivalente, no que ambos agentes rindan igual.

| Superpowers | Cobertura previa en Fénix | Resultado y cambio |
| --- | --- | --- |
| `using-superpowers` | Búsqueda, carga bajo demanda y sugerencias del runtime | La selección ya existe. Se mejoraron descripciones y alias; no se añadió invocación obligatoria para toda conversación. |
| `brainstorming` | `requirements-and-planning`, `architecture-design`, briefs del plan | Se aclaró cuándo basta una edición directa, cuándo comparar diseños y cuándo realizar un experimento. Se conservan decisiones y autorización previas. |
| `writing-plans` | Hitos, dependencias y criterios verificables | Se añadieron interfaces productoras/consumidoras, archivos asignados y comprobación de compatibilidad entre hitos. El registro sigue siendo el plan nativo. |
| `executing-plans` | Planificador, diarios de trabajadores, contratos y checkpoints | La persistencia ya estaba implementada. Se reforzaron evidencia de entrega y recuperación de trabajo completado; no se añadió otro diario paralelo. |
| `subagent-driven-development` | `task-delegation`, subagentes de un nivel y revisión adaptativa | Se precisaron entregables, interfaces, fuentes y revisión posterior. Se distingue aprobar un plan de comprobar su implementación. |
| `dispatching-parallel-agents` | Delegación independiente, dependencias y coordinación de efectos | Ya estaba cubierto. Se conserva el mecanismo y se refuerzan los traspasos sin crear otra skill equivalente. |
| `systematic-debugging` | `test-driven-repair`, `local-troubleshooting` | Faltaba detalle para seguir datos hasta su origen, comparar casos válidos, descartar hipótesis y diagnosticar intermitencias. Se añadió una referencia específica. |
| `test-driven-development` | Reparación reproducible y `test-strategy` | Se reforzaron pruebas de comportamiento nuevo, resultados esperados independientes y sensibilidad de las regresiones. Se conserva implementación válida. |
| `verification-before-completion` | Evidencia del sistema, recibos de herramientas y verificación por skill | Se añadió una tabla que relaciona afirmaciones con comprobaciones. Compilar, pasar una prueba, responder una petición e integrar son resultados distintos. |
| `requesting-code-review` | `code-review` y delegación | Se separó cobertura de requisitos de calidad del código y se acotó la segunda revisión a correcciones y regresiones plausibles. |
| `receiving-code-review` | Desacuerdos resueltos mediante evidencia, principalmente desde el revisor | Se añadió un procedimiento para verificar comentarios recibidos y clasificarlos como confirmados, no sustentados, pendientes de información o aplazados. |
| `using-git-worktrees` | `git-worktree-coordination` | Se añadieron reutilización del aislamiento, distinción de submódulos, directorio ignorado y comprobaciones iniciales para atribuir fallos. |
| `finishing-a-development-branch` | Worktrees, recuperación Git y publicación | Se precisaron base/head, cambios pendientes, pruebas y estado de integración. La propiedad de un worktree no se deduce de su nombre. |
| `writing-skills` | `skill-authoring`, diseño de experimentos y pruebas del catálogo | Se añadió evaluación con línea base, casos de presión y casos negativos. Se separan formato, recuperación y comportamiento del modelo. |
| `diagnosing-superpowers` | Diagnóstico local, historial y trazas de tareas | Se añadió atribución específica de fallos del agente, acciones repetidas, skills omitidas y cálculo cuidadoso de tiempo/tokens mediante eventos nativos. |

## Qué aporta realmente

El catálogo de Fénix cubre más áreas: documentos, investigación, datos, operaciones y trabajo personal además de programación. Contar 142 frente a 15 no demuestra superioridad. Superpowers concentra mucha más explicación en unos pocos procedimientos de desarrollo, y esa profundidad fue lo aprovechable.

Fénix también dispone de infraestructura ejecutable que un archivo de instrucciones no sustituye: planes versionados, dependencias, permisos, historial de efectos, búsqueda de skills, carga de recursos y supervisión de procesos. Instalar Superpowers no implementaría esas funciones de nuevo ni garantiza que el modelo siga cada paso.

Las incorporaciones tienen mecanismos útiles identificables:

- Seguir el primer dato incorrecto hasta su productor ayuda a elegir una corrección sustentada en vez de acumular cambios especulativos.
- Derivar resultados esperados de forma independiente ayuda a detectar pruebas que se validan a sí mismas.
- Contrastar sugerencias de revisión con consumidores y compatibilidad evita aplicar una recomendación incorrecta por confianza en el revisor.
- Asociar una afirmación con la versión comprobada evita usar un resultado antiguo después de modificar el comportamiento.
- Distinguir búsqueda fallida, instrucciones omitidas y rechazo del runtime permite corregir la capa adecuada.

Son razones técnicas para incorporar las instrucciones; no porcentajes medidos de mejora del modelo.

## Qué no conviene trasladar literalmente

Superpowers contiene reglas que, aplicadas como un paquete completo, chocan con Fénix o añaden trabajo innecesario aquí:

1. Invocar skills antes de cualquier respuesta ampliaría el proceso a conversaciones que no requieren ingeniería.
2. Repetir aprobaciones de diseño, documento y plan añadiría interrupciones a cambios ya autorizados. Se mantienen las preguntas necesarias y la política del runtime.
3. Borrar código funcional para recrear el orden de TDD arriesga trabajo útil. Se comprueba la sensibilidad de la prueba en una copia aislada.
4. Exigir una suite completa tras cada paso o repetir evidencia sin cambios añade coste sin demostrar más sobre ese artefacto. Se ejecutan comprobaciones exigidas y pertinentes, indicando su alcance.
5. Revisores obligatorios, cambios de modelo y nuevas rondas de reintento interferirían con la configuración y límites nativos.
6. Otro diario de progreso competiría con planes, contratos y checkpoints existentes.
7. Inferir que una carpeta `.worktrees` pertenece al agente no basta para autorizar su eliminación.
8. El servidor visual opcional duplicaría funciones que Fénix ya ofrece mediante Design y navegador.

Estas decisiones se refieren a la compatibilidad de Fénix; no pretenden descalificar el flujo de Superpowers en otros agentes.

## Archivos incorporados y activación

Se actualizaron `software-implementation`, `test-driven-repair`, `test-strategy`, `requirements-and-planning`, `code-review`, `git-worktree-coordination`, `task-delegation`, `local-troubleshooting` y `skill-authoring`.

Las cinco referencias cubren [causa raíz](../../resources/skills/test-driven-repair/references/root-cause-investigation.md), [evidencia de finalización](../../resources/skills/software-implementation/references/completion-evidence.md), [comentarios de revisión](../../resources/skills/code-review/references/review-feedback.md), [diagnóstico del agente](../../resources/skills/local-troubleshooting/references/agent-workflow-diagnosis.md) y [evaluación de skills](../../resources/skills/skill-authoring/references/behavioral-evaluation.md). Sus textos solo se cargan cuando la tarea los necesita.

Se conservaron las nueve versiones anteriores como fixtures y se registraron sus hashes exactos en `resources/skills-legacy-hashes.json`. El instalador existente puede renovar originales conocidos sin sobrescribir personalizaciones ni reactivar skills desactivadas. Las versiones personalizadas pueden requerir incorporar manualmente el contenido pertinente.

Existe un coste de contexto: los nueve archivos principales pasan, en conjunto, de **20.019 a 27.978 bytes UTF-8**; las referencias opcionales añaden **13.653 bytes**. Son tamaños de archivo, no tokens medidos ni consumo por turno. Solo se carga lo seleccionado. No se presume que las instrucciones adicionales sean gratuitas ni que ya hayan compensado su coste mediante menos errores.

La integración está en el código fuente y se comprueba con directorios de runtime aislados. Un ejecutable de Fénix compilado antes no recibe los nuevos recursos solo porque cambie Markdown en el repositorio: necesita el proceso normal de compilación e inicio del binario actualizado. **No se afirma que un servicio en ejecución haya sido recompilado, reiniciado o desplegado.** Tampoco se modificó la configuración personal de Codex.

## Pruebas y límites de la conclusión

La suite inicial de skills pasó 11 pruebas. Antes de las mejoras, un conjunto nuevo de 18 solicitudes en español e inglés encontraba el skill esperado entre los tres primeros resultados en 11 casos; fallaba en 7. La prueba emparejada reproduce las nueve versiones anteriores y luego las actualiza en el mismo runtime, manteniendo iguales los otros skills y el buscador. La versión mejorada recupera el skill esperado **en primera posición en los 18 casos**, frente a **11/18 entre los tres primeros** antes. La suite final de skills pasa **14 pruebas**. El [manifiesto de evidencia](../skills/superpowers-review.json) conserva resultados por consulta y estado de las comprobaciones.

Las pruebas añadidas usan el buscador real, comprueban la actualización de los nueve originales, conservación de personalizaciones y activación, y carga de las cinco referencias. Las pruebas existentes también cubren metadatos inválidos, consultas irrelevantes y límites de acceso a recursos. No se mide calidad buscando frases en los nuevos textos.

| Comprobación | Resultado observado |
| --- | --- |
| Mismas 18 consultas, skill esperado en primera posición | Antes 10/18; después 18/18 |
| Mismas 18 consultas, skill esperado entre los tres primeros | Antes 11/18; después 18/18 |
| Suite nativa de skills | 14 correctas, 0 fallos |
| Nueve skills en el validador de skill-creator | 9 correctas, 0 fallos |
| Análisis estático Rust sin admitir advertencias | Correcto |
| Formato Rust, enlaces locales e integridad de hashes | Correctos |
| Calidad, tokens, coste y latencia con modelo real | Sin medir |

Las 18 consultas son casos de desarrollo y regresión, no una muestra independiente: mejorar en ellas demuestra recuperación para esos casos. La instalación correcta tampoco demuestra que un modelo aplique las instrucciones. No se atribuyen mejoras medidas de tasa de acierto, latencia, tokens o coste de inferencia.

Para comprobar ese beneficio haría falta ejecutar Fénix antes y después con igual modelo y configuración, tareas aisladas y resultados evaluados independientemente. Los casos deberían incluir intermitencias, sugerencias erróneas del revisor, compilación correcta con comportamiento defectuoso, pruebas obsoletas tras editar, recuperación de efectos completados y cambios triviales que no deben generar nuevas aprobaciones. Se necesitan casos de transferencia, repeticiones donde haya variabilidad y conservación de fallos. El protocolo queda incorporado para poder medirlo sin confundir intención con resultados.
