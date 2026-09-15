# Revisión adaptativa de planes

[English](../en/plan-reviews.md)

Los planes complejos se revisan antes de despachar hitos. Los revisores usan el mismo
proveedor, modelo y nivel de pensamiento que el maestro. El chat continúa disponible.

## Objetivo y activación

El maestro recupera decisiones confirmadas, inspecciona evidencia y comunica dudas.
El `brief` separa requisitos con procedencia (`requirements`), observaciones
(`observations`), suposiciones (`assumptions`) y preguntas (`open_questions`). Una
suposición no es un requisito: «mejora el login» no implica añadir login social.
Se agrupan preguntas, se reutilizan respuestas y se detallan los hitos próximos
conforme aparece información. Mientras se espera, avanza el trabajo independiente.

Hay revisión automática con tres o más hitos, cualquier dependencia, suposiciones,
preguntas pendientes o un `review_reason`. Es una regla inicial, no un clasificador
entrenado. El maestro debe indicar un motivo para planes cortos pero relevantes o
ambiguos. Los planes sencillos y claros evitan esa inferencia adicional.

`plan_manage review` requiere plan, versión y motivo. `review_nodes` limita el bloqueo
de nuevos despachos a esos hitos y sus dependientes. Un hito bloqueado/fallido activa
revisión una vez por ejecución. No se repiten sus efectos automáticamente ni se usa
el modelo para sondear el estado. La revisión no detiene retroactivamente acciones
en curso; para sustituirlas se usa replanificar o cancelar. Otras ramas continúan.

## Revisores y resolución

Un principal comprueba objetivo, alcance añadido sin fundamento, dependencias,
viabilidad y criterios de finalización. Puede pedir hasta dos especialistas con
preguntas concretas y skills sugeridos. El motor los crea como hijos del maestro,
al mismo nivel. Los especialistas no pueden solicitar más agentes.

Mediante `plan_review_read`, consultan petición original, brief, hitos completos en
páginas acotadas y respuestas confirmadas. No reciben conclusiones de otros revisores.
Pueden leer archivos, recuperar memoria/fuentes, cargar skills, consultar evidencia
de hitos y preguntar. El motor prohíbe comandos, escrituras, mutaciones de memoria,
ediciones de planes y delegación. Las pruebas que requieren comandos se proponen al
maestro. Un plan revisado no demuestra que se haya ejecutado correctamente.

El informe estructurado `subagent_finish.review` incluye veredicto y observaciones con
ID, categoría, gravedad, descripción, evidencia e hitos afectados. Un resultado
favorable no puede contener objeciones bloqueantes. Informes inválidos no liberan hitos.

Si todos pasan, se libera la revisión. Una objeción bloqueante crea un único turno de
resolución del maestro. Dos votos favorables no anulan una objeción. Los desacuerdos
factuales se resuelven con fuentes; las preferencias desconocidas se preguntan al
usuario. Este turno también está restringido a lecturas, preguntas y su propio plan.

`resolve_review` registra cada objeción `rol/id` como resuelta, descartada con evidencia
o pendiente del usuario. Si hay que corregir el plan, `revise` registra las resoluciones
y el brief actualizado, conservando efectos válidos. Se permite una respuesta del
maestro y una comprobación final de los hitos cambiados; si cambia el objetivo global,
se revisa el conjunto. La comprobación final no añade especialistas ni otra corrección
automática. Las objeciones restantes bloquean el ámbito afectado y vuelven al canal
original. Un fallo posterior no reinicia automáticamente el ciclo de corrección.
El usuario puede replanificar en cualquier momento e iniciar un ciclo nuevo.

## Persistencia, interfaz y límites

La migración 5 guarda revisiones y revisores vinculados a versiones. Editar cancela
revisiones y preguntas obsoletas. Los informes antiguos no liberan trabajo nuevo.
Reiniciar conserva auditoría, marca revisiones pendientes como interrumpidas y no
reanuda efectos. Se comparte el límite de cuatro inferencias de hijos simultáneas y
32 hijos activos; esperar respuestas libera capacidad. Los tres intentos del proveedor
son distintos de las rondas de revisión.

Tareas muestra estado y ámbito. El detalle incluye informes, resoluciones y tokens
reportados para revisores y resolución. El tiempo incluye esperas; se identifica el
consumo ausente y no se estiman precios. Los avisos utilizan el pub/sub y la cola de
entrega existentes, con una conversación continua en web y Telegram.

Las pruebas con proveedor simulado verifican orquestación, permisos, preguntas,
desacuerdos, límites, ediciones, persistencia y consumo. Detectar ambigüedades y juzgar
la evidencia depende del modelo. Las mejoras de calidad, ahorro y umbrales óptimos
requieren evaluaciones representativas con modelos reales; no se afirma una mejora
en benchmarks.
