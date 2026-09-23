# Comparación con ECC y mejoras de Fénix

[English](../en/ecc-review.md)

La revisión de [affaan-m/ECC](https://github.com/affaan-m/ECC) dio lugar a **cinco skills propias nuevas y trece mejoradas**. La biblioteca pasa de 181 a **186 skills**. Gran parte de ECC ya tenía una correspondencia en Fénix; las incorporaciones responden a necesidades distintas o añaden precisión a procedimientos existentes.

## Alcance real

Se revisó el commit `2b6e839771e53096d8451a213d40dc64ec8acac0`, el 21 de septiembre de 2026. El repositorio contiene 292 skills canónicas y 903 archivos `SKILL.md` contando traducciones y adaptaciones. Se examinó el catálogo completo —nombres, extractos de descripción y encabezados—, se leyeron íntegramente **27 skills seleccionadas** y se inspeccionaron auxiliares relevantes de evaluación y aprendizaje.

El [mapa de 292 entradas](../skills/ecc-capability-map.md) explica cada correspondencia o limitación. El [inventario verificable](../skills/ecc-source-review.json) conserva revisión, huellas y profundidad de lectura. No se afirma haber leído todas las 76.166 líneas de instrucciones principales ni auditado todo el código: inventariar o calcular huellas no equivale a leer o probar. No se ejecutaron instaladores, hooks ni servicios de ECC.

## Qué se añadió

| Skill | Utilidad |
| --- | --- |
| `automation-audit` | Saber qué automatizaciones están configuradas, cuáles funcionan con evidencia y dónde duplican efectos. |
| `notification-triage` | Reducir alertas duplicadas conservando cambios de gravedad, reaperturas, responsables y comprobantes de entrega. |
| `llm-cost-optimization` | Analizar gasto por tarea resuelta, reintentos, caché y presupuesto sin sacrificar calidad ni cambiar proveedores automáticamente. |
| `patent-research` | Investigar patentes con fuentes oficiales, identificadores, familias y fechas precisas, sin convertir la búsqueda en un dictamen jurídico. |
| `inventory-planning` | Proponer reposición por producto y almacén considerando reservas, entregas previstas, incertidumbre y restricciones de compra. |

## Qué se mejoró

Las trece revisadas son `agent-workflow-diagnosis`, `api-integration`, `api-contract-design`, `browser-regression-testing`, `cache-design`, `customer-support`, `data-pipelines`, `memory-and-reminders`, `performance-diagnosis`, `scheduled-workflows`, `skill-authoring`, `team-messaging` y `experiment-and-benchmark-design`.

Los cambios ayudan a localizar en qué capa falla un agente; comprobar contratos desde cliente y servidor; detectar acciones de interfaz que se deshacen; invalidar cachés cuando cambia el procesamiento; conciliar entregas y devoluciones; acelerar datos sin perder registros; mantener aprendizajes dentro de su proyecto; limitar bucles; y comprobar que un evaluador no acepte resultados incorrectos. También se corrigió una zona horaria fija en la guía de memoria para utilizar la configuración del usuario.

Ocho referencias nuevas aportan detalle bajo demanda. Se guardaron las trece versiones anteriores y sus huellas para actualizar solo copias originales reconocidas, conservando instrucciones personalizadas, recursos editados y skills desactivadas.

## Decisiones y límites

El mapa clasifica 143 entradas como correspondencias por finalidad; 29 como fuentes para mejorar skills existentes; nueve agrupadas en las cinco nuevas; 45 dependientes de integraciones; 36 específicas del entorno ECC; 28 especializaciones aplazadas; y dos entradas obsoletas. Una correspondencia no significa igualdad de todas las APIs o ejemplos.

No se importaron observadores en segundo plano, almacenamiento alternativo de memoria, sincronización de conversaciones ni motores de permisos. Se conservan las identidades y mecanismos de aislamiento de Fénix. Tampoco se adoptaron precios fijos de modelos, porcentajes universales de inventario o plazos genéricos de reclamación.

Las instrucciones son originales. ECC tiene licencia MIT en la raíz, pero algunas skills declaran otras licencias y procedencias; no se copiaron cuerpos de prompts, scripts, pruebas ni recursos de esos autores. No se añadió una suscripción o conexión externa.

## Validación

Se prepararon 56 consultas en español e inglés para comparar descubrimiento antes y después, incluidos diez casos vecinos que no deben ser desplazados. Las pruebas nativas también comprueban actualización, personalizaciones, activación y rutas de recursos. Los [resultados exactos](../research/ecc-validation-2026-09-21.json) distinguen estas comprobaciones de una evaluación del comportamiento del modelo, que no se ejecutó.

Los cambios están en el repositorio. La instancia en ejecución necesita el despliegue habitual para recibirlos; este trabajo no reinició ni desplegó el servicio.

Resultados ejecutados: **18/18 validaciones de formato y 35/35 pruebas nativas superadas**. Las 56 consultas recuperaron la skill prevista entre las tres primeras opciones, 55 en primer lugar. En las 26 consultas sobre procedimientos existentes, el resultado pasó de 21 a 26; los diez casos vecinos funcionaron antes y después. Las veinte consultas sobre skills nuevas no tienen una línea base de la misma skill, porque aún no existía. Son casos de desarrollo, no un benchmark independiente de calidad.
