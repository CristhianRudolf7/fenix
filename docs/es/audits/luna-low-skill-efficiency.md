# Eficiencia de skills y prompt — GPT-6 Luna low

[English](../../en/audits/luna-low-skill-efficiency.md)

El 23 de septiembre de 2026 se probaron dos revisiones y una confirmación alternada. Ambas versiones conservaron las 95 comprobaciones. En la confirmación disminuyeron las solicitudes al modelo un 23,1 %, los tokens de entrada un 27,6 %, el coste estimado un 15,6 % y el tiempo total un 7,0 %. Es una mejora de eficiencia en cuatro tareas pequeñas, no una demostración de superioridad general frente a Codex.

## Cambios

Se reutilizan herramientas y skills ya disponibles, se agrupa la preparación independiente y se prefieren rutas relativas y ediciones coherentes. Se evita borrar/recrear archivos innecesariamente, incluir herramientas incompatibles en programas y consultar procesos ya terminados. Los skills distinguen errores de preparación, expectativas y comportamiento: la prueba debe usar el mismo recurso que inicializa y conservar los requisitos y pruebas de aceptación suministradas.

Solo se actualizan versiones originales de skills mediante coincidencia exacta de hash; se conservan personalizaciones y estados de activación. Versiones: software-implementation 1.6.0, test-driven-repair 1.4.0 y python-development 1.1.0. No se relajaron permisos, validadores ni límites, y no se añadió otro revisor LLM.

## Método y resultados completos

Se reutilizaron exactamente los mismos archivos y comprobadores: caché TTL/LRU (17), transferencias SQLite (28), planificador de dependencias y CLI (23), persistencia JSON atómica y CLI (27). Mismo GPT-6 Luna `low`, servicio estándar, espacios y estados nuevos, evaluador nativo, 300 segundos por tarea y 330 por proceso; ninguna ejecución agotó esos plazos. El proxy registró consumo y solicitudes sin alterar sus parámetros.

La primera revisión tuvo una mejora modesta y todavía cometió errores de composición de herramientas y rutas. La segunda corrigió instrucciones solapadas del prompt maestro. Para confirmarla se congelaron ambos binarios y se alternó el orden anterior/nuevo en cada tarea, sin ejecuciones simultáneas. No se volvió a ejecutar Codex: esta comparación aísla los cambios de Fénix.

| Run | Checks | Requests | Input tokens | Output tokens | Wall seconds | Estimated USD |
|---|---:|---:|---:|---:|---:|---:|
| Original reference | 95/95 | 52 | 620,766 | 9,304 | 208.0 | 0.02039 |
| Candidate 1 | 95/95 | 46 | 544,988 | 9,098 | 212.6 | 0.01824 |
| Candidate 2 | 95/95 | 41 | 473,809 | 8,557 | 205.2 | 0.01660 |
| Confirmation: original | 95/95 | 39 | 427,904 | 8,676 | 201.2 | 0.01608 |
| Confirmation: candidate 2 | 95/95 | 30 | 309,987 | 7,991 | 187.0 | 0.01357 |

Se conservan también los resultados menos favorables. La segunda revisión redujo la entrada un 23,7 % frente a la referencia original, con solo un 1,3 % menos tiempo. La confirmación reprodujo la reducción de entrada frente a una nueva ejecución de control. No son intervalos de confianza.

## Confirmación por tarea

| Task | Original requests | New requests | Original seconds | New seconds | Checks each |
|---|---:|---:|---:|---:|---:|
| ttl-cache | 9 | 7 | 52.8 | 38.9 | 17 |
| ledger | 8 | 6 | 41.7 | 45.1 | 28 |
| dependency-plan | 13 | 6 | 58.7 | 41.0 | 23 |
| atomic-json | 9 | 11 | 48.0 | 62.1 | 27 |

JSON atómico empeoró en solicitudes y tiempo; las transferencias necesitaron menos solicitudes, pero tardaron más. Persisten lecturas innecesarias y variación en las pruebas generadas. Un resultado de comando distinto de cero no siempre es un defecto: una regresión reproducida intencionadamente puede ser útil.

## Límites y evidencia

Los tokens de entrada incluyen caché y acumulan todas las solicitudes. El tiempo incluye arranque, herramientas, latencia del modelo y cierre. El coste estimado mantiene las tarifas de la auditoría previa por millón: entrada USD 0,10; caché USD 0,01; escritura de caché USD 0,125; salida USD 0,50. Se añade solo la prima de escritura sobre la entrada ya contada; no equivale a una factura.

Los hashes de fixtures coinciden, se compilaron y repitieron los comprobadores externos y los archivos originales sin reparar siguieron fallando. Los registros verifican modelo, esfuerzo y consumo completo. No se incorporaron respuestas específicas del benchmark a los skills.

Son cuatro tareas Python utilizadas tanto para ajustar como para confirmar: falta evaluación con tareas nuevas. No demuestra resultados en SWE-bench, Terminal-Bench ni producción. Ambas versiones ya acertaban; mejoró la eficiencia, no el porcentaje de éxito. No se recortaron instrucciones de otras funciones del asistente.

[Resultados y hashes](../../benchmarks/luna-low-skill-efficiency-2026-09-23.json). El archivo local `luna-skill-efficiency-*20260923` conserva solicitudes, trazas, comprobadores y binarios. Los cambios acumulados se subieron primero a main (`d385b1e`). Estas mejoras de prompt y skills todavía no se desplegaron en producción.

## Validación

El servidor compiló y pasó la regresión existente `hash_upgrade_only_replaces_the_exact_pre_library_release`. Se verificaron los tres hashes originales del registro de migración y la exclusión de contenidos personalizados. Hubo 16 ejecuciones nuevas entre dos candidatos y la confirmación alternada: reutilizan cuatro tareas, no son 16 tareas diferentes. También se validaron los enlaces y el JSON de resultados.
