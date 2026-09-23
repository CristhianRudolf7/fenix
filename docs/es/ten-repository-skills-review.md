# Revisión de diez repositorios y mejoras propias de Fénix

[English](../en/ten-repository-skills-review.md) · [Evidencia y cobertura de lectura](../skills/ten-repository-review.json) · [Inventario de fuentes](../skills/ten-repository-inventory.json) · [Consultas de búsqueda](../skills/ten-repository-cases.json)

Revisión del **19 de septiembre de 2026**. Los diez repositorios proceden del área `skills`, categoría `valioso`, del catálogo local `proyectos/repo/repo.sqlite`, consultado en modo de solo lectura. Esta revisión continúa la [iteración anterior de 164 skills](openclaw-skills-review.md).

## Resultado

Fénix tiene ahora **167 skills propias**. Se añadieron tres procedimientos distintos, se mejoró el contenido de dieciocho existentes y se corrigió adicionalmente la búsqueda del calendario. Ocho referencias opcionales y dos auxiliares Python propios aportan profundidad. En total cambiaron 32 archivos dentro de estos paquetes.

La mejora comprobada está en la recuperación de skills y en las comprobaciones ejecutables de resultados pareados y tiempos de vídeo. Las instrucciones también cubren tareas que antes dependían de procedimientos generales. **Esto no demuestra que Fénix sea globalmente más inteligente que los agentes de esos repositorios.** No se ejecutó una comparación con un modelo real ni se activaron conexiones externas.

## Alcance exacto

Se enumeraron los diez árboles Git en commits fijos: 16 181 archivos y **3096 rutas `SKILL.md`**. Esas rutas incluyen traducciones, copias para plataformas, adaptadores y fixtures de pruebas; no equivalen a 3096 capacidades independientes.

Se descargaron 115 archivos seleccionados. **35 se leyeron completos y otros 27 por secciones documentadas**. Los restantes solo recibieron, como máximo, una inspección estructural; no se consideran auditados. La selección buscó carencias concretas de Fénix, debilidades de procedimientos existentes, contratos de integración representativos y mecanismos aparentemente redundantes.

La evidencia conserva rutas, rangos leídos, fechas, SHA-256 de descargas y hashes Git verificados. Por tanto, es una revisión detallada de mecanismos seleccionados en los diez proyectos, **no una auditoría completa de todos sus skills, scripts y dependencias**. No se importó ni ejecutó código, instaladores o activos de esos proyectos.

| Repositorio | Rutas `SKILL.md` | Contribución examinada |
| --- | ---: | --- |
| [microsoft/SkillOpt](https://github.com/microsoft/SkillOpt/tree/79124b37e9a6371e13b753f8bcd7adb1e493ade1) | 5 | Optimización y evaluación de skills |
| [affaan-m/ECC](https://github.com/affaan-m/ECC/tree/07756cee15788a54506031462794ad645719b028) | 903 | Recuperación de contexto y mantenimiento |
| [garrytan/gstack](https://github.com/garrytan/gstack/tree/a6b3a57512ca6d5c6aa5b68f74f736195021f96e) | 65 | Producto, revisión de código y QA |
| [Leonxlnx/taste-skill](https://github.com/Leonxlnx/taste-skill/tree/e79ca9ec7e071eb3a3b623c4fb752e853fc3ed58) | 13 | Dirección visual y rediseño |
| [google/skills](https://github.com/google/skills/tree/18152e0d310e4d7047e9c2ec25a37b0d22d6893e) | 151 | Cloud, identidades, IAM y SecOps |
| [calesthio/OpenMontage](https://github.com/calesthio/OpenMontage/tree/08e2151fa02de28a5d6a312b3d575692bf147ad7) | 138 | Montaje, narración y tiempos |
| [coreyhaines31/marketingskills](https://github.com/coreyhaines31/marketingskills/tree/5b2c0007766c6a1cf1d53fd8fc73e979e0821022) | 50 | Conversión, afirmaciones y visibilidad |
| [ComposioHQ/awesome-claude-skills](https://github.com/ComposioHQ/awesome-claude-skills/tree/be2a406907dbc61b73e6827ded415c96139d13a2) | 864 | Contratos de servicios conectados |
| [mukul975/Anthropic-Cybersecurity-Skills](https://github.com/mukul975/Anthropic-Cybersecurity-Skills/tree/54a798831d2266a3ca61ce68a7acb80b81160d57) | 818 | Detecciones y auditoría de MCP |
| [zhaoxuya520/reverse-skill](https://github.com/zhaoxuya520/reverse-skill/tree/7e2097fd90d25c2f976f6eba26d6c00aa88051df) | 89 | Binarios, protocolos y análisis forense |

La mayoría declara MIT; Google y el repositorio de ciberseguridad declaran Apache-2.0. OpenMontage tiene AGPL-3.0 en la raíz y diferencias en componentes. GitHub no detectó licencia raíz en el catálogo de Composio. Esos datos no resuelven las licencias de cada recurso anidado. El proyecto de ciberseguridad es de terceros; su nombre no acredita afiliación con Anthropic.

## Comparación por repositorio

### 1. SkillOpt

Fénix ya tenía autoría y diseño experimental, pero faltaba comprobar resultados repetidos por tarea. Se incorporó una revisión acotada a fallos observados y un auxiliar que empareja tareas y repeticiones, conserva intentos faltantes y separa selección de prueba final. Una mejora media no puede ocultar el deterioro de una repetición crítica.

Se descartaron la recolección automática nocturna de conversaciones, la sustitución automática de skills y las puntuaciones por lenguaje imperativo. El [evaluador de SkillOpt](https://github.com/microsoft/SkillOpt/blob/79124b37e9a6371e13b753f8bcd7adb1e493ade1/skillopt/evaluation/gate.py) contiene una bonificación opcional por densidad de directivas; eso no demuestra mejores resultados.

### 2. ECC

Su recuperación iterativa, evaluación y revisión del catálogo aportan disciplina útil. Se reforzó `repository-navigation` con preguntas pendientes, evidencia localizada y reformulación de búsquedas según el vocabulario real del proyecto. Autoría distingue ahora problemas de búsqueda, instrucciones, herramientas y ejecución antes de cambiar texto.

Diagnóstico de sesiones, verificación y contratos ya estaban cubiertos. No se añadieron duplicados ni límites rígidos de tres búsquedas. Tampoco se adoptaron probabilidades de relevancia sin calibración, promoción de preferencias por ausencia de correcciones o estimaciones de tamaño presentadas como consumo real de contexto.

### 3. gstack

Se aprovecharon detalles concretos de revisión y QA: buscar consumidores de nuevos estados fuera del diff, comprobar campos generados antes de denunciarlos como ausentes y conservar estado inicial, acción, resultado e identidad de cada defecto del navegador.

Descubrimiento de producto ahora examina la solución que la persona usa actualmente y qué evidencia cambiaría la propuesta, distinguiendo objetivos comerciales, internos y de aprendizaje. No se incorporó su segundo sistema de orquestación, memoria o equipos obligatorios de revisión; tampoco interrogatorios repetidos ni puntuaciones generales sin una escala validada.

### 4. taste-skill

La lectura del público, referencias e identidad visual mejora decisiones que antes quedaban demasiado generales. La referencia propia de frontend conecta jerarquía, tipografía, densidad, recursos y movimiento con el encargo, y exige comparar una vista renderizada representativa antes de extender el cambio.

Se conservaron accesibilidad y sistema visual existente. Se descartaron obligaciones de cambiar framework, iconos o estética, además de sugerencias de inventar cifras o fechas para aparentar naturalidad. Una preferencia visual no se presenta como medición objetiva.

### 5. Google

Se reforzó despliegue con selección explícita de proyecto, ubicación, identidad que despliega e identidad de ejecución. La revisión IAM conserva permisos ajenos y tokens de concurrencia; distingue políticas heredadas, condiciones, denegaciones y alcances de autorización.

La ingeniería de detecciones separa sintaxis, ejecución en el motor, resultados históricos, creación de reglas y activación de alertas. No presupone herramientas MCP disponibles ni considera una búsqueda de palabras una barrera de seguridad. Los umbrales absolutos de volumen no se generalizan a entornos con tamaños diferentes.

### 6. OpenMontage

Se incorporaron procedencia de tomas, duración medida de narración, planificación por fotogramas y comprobación del archivo final. El [auxiliar original de tiempos](https://github.com/calesthio/OpenMontage/blob/08e2151fa02de28a5d6a312b3d575692bf147ad7/lib/verify_scene_pacing.py) usa cercanía a hitos visuales; Fénix exige identificar el evento correspondiente para que otro evento cercano no oculte una desincronización.

El auxiliar propio valida frecuencias racionales, continuidad, límites y correspondencia de cues. No renderiza ni juzga el significado. Las reconstrucciones de interfaces deben identificarse como tales. No se importaron Remotion, Modal, código AGPL o proveedores de vídeo.

### 7. Marketing skills

Copywriting, SEO, analítica y pruebas A/B ya existían. Se añadió `conversion-optimization` porque diagnosticar abandono requiere población, eventos, periodo y denominadores propios; también distinguir fallos técnicos, pérdida de instrumentación e hipótesis de comportamiento.

Las mejoras de marketing reutilizan el brief existente y exigen pruebas auténticas para citas y cifras. SEO registra consultas, motor, fecha, citas y límites analíticos. Se descartaron porcentajes de mejora prometidos y obstáculos para cancelar. [Google no exige un archivo o esquema especial para sus funciones de búsqueda con IA](https://developers.google.com/search/docs/appearance/ai-features).

### 8. Composio

El catálogo incluye 832 entradas de proveedores, muchas con instrucciones de enrutamiento repetidas. Fénix necesita contratos de conexión fiables para sus procedimientos existentes, no copiar todas esas entradas.

La revisión encontró dos errores aprovechables: el skill llamado [googlebigquery](https://github.com/ComposioHQ/awesome-claude-skills/blob/be2a406907dbc61b73e6827ded415c96139d13a2/composio-skills/googlebigquery-automation/SKILL.md) describe Metabase; el de Search Console exagera la integridad que proporciona la paginación. [Google no garantiza devolver todas las filas](https://developers.google.com/webmaster-tools/v1/searchanalytics/query). La adaptación comprueba proveedor real, usuario, cuenta, esquema actual, efectos parciales y cobertura de datos.

### 9. Anthropic-Cybersecurity-Skills

Los auxiliares seleccionados aportan ejemplos útiles, pero parsear reglas o contar etiquetas ATT&CK no demuestra detectar ataques. El auxiliar MCP puede ejecutar un paquete bajo la etiqueta «static» y lanzar un servidor para enumerar herramientas: ambas acciones van más allá de leer archivos.

La versión propia conserva conversiones fallidas, prueba positivos, negativos benignos y límites, e inspecciona qué ejecuta cada escáner. Seguridad de agentes sigue contenido no confiable hasta memoria, recuperación y herramientas con efectos, verificando autorización en el punto real de ejecución.

### 10. Reverse-skill

Se creó `reverse-engineering`: reconstruir un formato o explicar un binario requiere entradas y evidencia distintas de revisar código fuente. Parte de hashes, formato y arquitectura; distingue offsets y direcciones virtuales, y vincula conclusiones a funciones, bytes o mensajes reconstruidos.

La ejecución dinámica depende de un entorno autorizado y comprobado. Una instrucción no proporciona aislamiento. Análisis de incidentes añade procedencia de adquisición e incertidumbre del reloj. No se importaron instaladores, acceso supuesto a laboratorios ni un sistema paralelo de diarios.

## Integraciones de Composio que sí pueden servir

**Sí resulta útil como capa de conexión para servicios que la persona utilice.** Se prioriza una conexión existente adecuada antes de añadir otra. Las Apps disponibles en la sesión de Codex que edita este repositorio no se transfieren automáticamente a Fénix.

| Necesidad y prioridad | Servicios candidatos | Procedimiento de Fénix | Comprobación decisiva |
| --- | --- | --- | --- |
| Alta: correo y agenda | Gmail/Outlook y calendarios | email-workflows, calendar-scheduling | Cuenta laboral/personal, hilo/evento, zona horaria, borrador/envío y notificaciones |
| Alta: coordinación | Slack/Teams, GitHub/Jira/Linear | team-messaging, issue-tracker-workflows | Espacio, proyecto, canal, duplicados y efectos de notificación |
| Alta si el conocimiento está distribuido | Notion/Confluence, Drive y documentos | knowledge-workspace-connectors y skills de Google | Cobertura de búsqueda, permisos, IDs y estructura documental |
| Según uso: ventas | Attio/HubSpot/Salesforce y Airtable | crm-workflows, data-analysis | Organización, campos personalizados, relaciones y lectura posterior |
| Según uso: medición web | Search Console y analítica | seo-review, analytics-instrumentation | Propiedad, dimensiones, periodo, latencia y límites de filas |
| Posterior, con necesidad concreta | Ashby o navegador alojado | hiring-and-interviews y procedimientos de navegador | Disponibilidad real, relaciones de registros, identidad de sesión y coste |

La [referencia propia de servicios conectados](../../resources/skills/mcp-integration/references/connected-services.md) define cómo comprobar las herramientas, vincular usuario/organización/cuenta, elegir transporte, resolver esquemas y verificar cada efecto. Un detalle relevante: el [MCP alojado de Composio](https://docs.composio.dev/docs/sessions-via-mcp) omite los modificadores locales del SDK y no expone herramientas personalizadas que se ejecutan en el proceso local. Los controles deben estar en la ruta que efectivamente ejecuta la operación.

No se encontraron herramientas Composio/Rube invocables en esta sesión ni configuración Composio en las búsquedas del repositorio. Se incorporaron instrucciones, no OAuth, un cliente MCP remoto ni conexiones activadas. Una conexión real requiere servicio elegido, entorno compatible, cuenta y credenciales. Esta revisión no envió mensajes ni modificó servicios externos.

## Cambios y validación

Las tres incorporaciones son [conversion-optimization](../../resources/skills/conversion-optimization/SKILL.md), [detection-engineering](../../resources/skills/detection-engineering/SKILL.md) y [reverse-engineering](../../resources/skills/reverse-engineering/SKILL.md).

Se revisaron autoría, evaluación experimental, navegación del repositorio, revisión de código, frontend, regresión del navegador, descubrimiento de producto, marketing, SEO, vídeo, despliegue cloud, IAM, MCP, API, CRM, incidentes, revisión de seguridad y revisión de fuentes. Calendario recibió alias de búsqueda. El informe en inglés incluye una tabla por skill; la evidencia conserva versiones previas y hashes de todos los recursos.

Los auxiliares [compare_outcomes.py](../../resources/skills/experiment-and-benchmark-design/scripts/compare_outcomes.py) y [check_timeline.py](../../resources/skills/video-editing/scripts/check_timeline.py) usan solo la biblioteca estándar de Python y leen JSON local. No ejecutan un modelo, instalan paquetes, renderizan vídeo ni cambian cuentas.

| Comprobación | Resultado | Alcance |
| --- | --- | --- |
| Formato, metadatos, secciones y límites | 167/167 | Paquetes consistentes y cargables |
| Pruebas nativas de skills | 26 aprobadas | Búsqueda, carga, recursos, activación y regresiones existentes |
| Migración de versiones previas | 19 fixtures exactos | Conservación de texto personalizado y desactivación |
| Pruebas Python de auxiliares | 24 aprobadas | Resultados faltantes/duplicados, regresiones críticas, JSON inválido y límites temporales |
| Clippy de la biblioteca y pruebas del agente | Aprobado sin advertencias | Comprobaciones Rust del objetivo probado |

Las 62 consultas bilingües se fijaron antes de editar: 36 de procedimientos revisados, 12 nuevos y 14 de tareas vecinas. En las 50 cuyo skill ya existía, la aparición entre los tres primeros resultados pasó de **45/50 a 50/50**. Las incorporaciones recuperan **12/12**. El resultado final es **62/62 entre los tres primeros y 61/62 en primera posición**. Ningún skill nuevo ocupa el primer lugar de una tarea vecina.

El primer candidato falló en tres consultas; se corrigieron alias naturales sin cambiar las consultas. Como sirvieron para ajustar los cambios, son casos de desarrollo, no un conjunto independiente. Se conservaron ambas ejecuciones. Las pruebas del comparador también detectaron un defecto inicial: una repetición crítica peor podía ocultarse bajo una media mejor; ahora se compara cada par crítico.

No se midieron éxito de tareas con un modelo real, latencia ni ahorro de tokens. No se probaron motores SIEM, binarios desconocidos, conexiones Composio ni percepción/escucha de medios en esta iteración. Pasar validadores no sustituye esas evaluaciones.

## Entrega

Cargo incorpora los recursos en la siguiente compilación. Al iniciar esa versión, Fénix instala las entradas nuevas y actualiza únicamente cuerpos históricos reconocidos sin personalizar. Conserva modificaciones del usuario y estado de activación. Se mantiene la política existente de conservar referencias del usuario. No se reinició ni desplegó el servicio, ni se modificó el catálogo SQLite o la instalación personal de skills de Codex.
