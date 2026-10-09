# Fénix — Hoja de ruta

Revisión documental: 9 de octubre de 2026, hora de Perú. Esta hoja distingue ampliaciones pendientes de capacidades ya presentes en el código. No constituye una promesa de fechas ni una certificación del despliegue. Los contratos actuales están en [REQUIREMENTS.md](REQUIREMENTS.md).

## Prioridad: validar y consolidar lo existente

- Comprobar el firewall en la infraestructura real: destinos autorizados y bloqueados, restricciones DNS y cancelación cuando se pierde protección. Las pruebas locales no completan esta validación.
- Evaluar tareas completas de programación, documentos, oficina y navegador con configuraciones reproducibles, informes de fallos y medidas de éxito, tiempo, tokens y costo por tarea resuelta.
- Separar pruebas simuladas, escenarios sintéticos reales y evaluación de usuarios. No comparar arneses con modelos, tareas o permisos distintos como si fueran equivalentes.
- Validar carga concurrente, aislamiento entre cuentas, consumo de RAM, recuperación de sesiones y capacidad de la biblioteca con documentos extensos.
- Actualizar documentación e instalación para distinguir ejecución individual y servicio compartido. Mantener la licencia del proyecto y las guías de contribución/seguridad como decisiones pendientes si se abre la distribución.

## Instalación con un solo comando y configuración inicial

Propuesta pendiente: un instalador público que detecte sistema y arquitectura, descargue una versión publicada del binario Rust y del frontend ya compilado, verifique su integridad y configure los servicios systemd correspondientes. La instalación normal no debería exigir clonar el repositorio ni instalar Cargo, Node o pnpm; la compilación desde código quedaría como alternativa para desarrolladores. El script actual `scripts/install-local.sh` requiere esas dependencias y compila desde el clon: no equivale todavía a esta distribución simplificada.

- Separar archivos de aplicación, configuración, secretos y datos personales. Reinstalar o actualizar no debe borrar usuarios, archivos, memoria, sesiones ni credenciales existentes.
- Abrir la bienvenida en el navegador local al finalizar; si no hay entorno gráfico, mostrar la dirección y explicar cómo acceder desde otro equipo mediante un canal autenticado.
- Configurar en la web el proveedor, modelo y credencial del modelo inteligente y del modelo rápido. Permitir reutilizar una conexión compatible sin obligar a ingresar dos claves. Mantener opciones avanzadas plegadas y comprobar cada conexión antes de finalizar, indicando que esa comprobación puede consumir tokens.
- Guardar las credenciales únicamente en el servidor, fuera de Git y del almacenamiento persistente del navegador. Limitar la configuración inicial al propietario; no dejar un asistente de instalación abierto públicamente sin autenticación.
- Mantener Telegram y otras conexiones como pasos opcionales. Usar inicialmente la zona horaria de Perú, con posibilidad de cambiarla. Permitir retomar una configuración incompleta y volver a editarla después.
- Distinguir instalación autogestionada, donde el propietario configura proveedores, del servicio alojado, donde las credenciales del operador no se muestran ni se piden a los usuarios finales.
- Prever actualización, diagnóstico y desinstalación conservadora. Las dependencias opcionales del navegador, aislamiento u ofimática deben indicar su estado real y no presentarse como disponibles si faltan.

Referencias de configuración e instalación: [OpenClaw](https://docs.openclaw.ai/start/wizard), [instalador de OpenClaw](https://docs.openclaw.ai/install), [Hermes Agent](https://hermes-agent.nousresearch.com/docs/getting-started/installation) y [configuración de Hermes](https://hermes-agent.nousresearch.com/docs/user-guide/configuration). Son referencias de experiencia de uso, no dependencias que se incorporarán a Fénix.

## Organizaciones, grupos y relaciones entre archivos

Propuesta pendiente: permitir que una misma cuenta pertenezca a varias organizaciones o grupos, con membresías, roles y permisos independientes. Conservar un espacio personal separado y un ámbito explícito para cada tarea, archivo, memoria, proyecto y conexión.

- Relacionar archivos entre sí y con proyectos, organizaciones o grupos mediante identificadores y metadatos: documento de origen, referencia, versión, adjunto o resultado derivado. No requerir duplicar físicamente un archivo para crear una relación.
- Definir propiedad y acceso por separado: relacionar o mencionar un archivo no concede permiso para leerlo. Compartir requiere autorización explícita y debe poder revocarse.
- Filtrar navegación, búsqueda, embeddings y contexto del agente según la membresía y los permisos vigentes. Un usuario con acceso a dos organizaciones no autoriza mezclar automáticamente sus datos.
- Resolver el destino de las instrucciones en la conversación continua; preguntar cuando el grupo u organización sea ambiguo y mostrar el ámbito activo de forma comprensible.
- Investigar si convienen bases independientes por organización o particiones con aislamiento verificable. Definir invitaciones, salida de miembros, auditoría y conservación de archivos sin romper el aislamiento existente por usuario.

## Memoria y recuperación avanzada

La base actual ya incluye memoria por ámbito, entidades, relaciones, procedencia, revisiones, captura de candidatos, recuperación léxica/relacional y opciones semánticas. No se vuelve a proponer todo ello como una función ausente. Quedan por investigar y validar:

- Qué información guardar automáticamente, qué debe comenzar como candidata y qué repetición o confirmación permite promover una preferencia. Conservar fuente, fecha, ámbito, confianza y vigencia; distinguir hechos, tareas, decisiones, requisitos, artefactos, supuestos y dudas.
- Comparar memoria relacional/híbrida con memoria lineal vectorial en calidad, actualización de hechos, costo y complejidad.
- Calibrar clasificación de consultas de entidad, temporal, evento, concepto y general; evaluar variantes de consulta sin deriva semántica.
- Comparar ramas BM25/léxica, vector, título, grafo y tiempo. Identificar entidades ancla, recorrer vecinos con profundidad acotada, comprobar ámbito, correlacionar resultados y separar evidencia explícita, complementaria y conflictiva antes de fusionar puntuaciones.
- Definir qué vectorizar dinámicamente y cuándo invalidar embeddings usando hashes y revisiones. Comparar wiki, tarjetas de entidades y documentos completos frente a fragmentos.
- Decidir campos de tarjetas rápidas por tipo y cómo recuperarlas sin imponer un esquema empresarial a todos los usuarios.
- Evaluar la interfaz de siete verbos inspirada en GBrain: verificar su inventario contra una versión identificada, comparar navegación y síntesis y medir si aporta valor frente a herramientas más simples. `tilt` representa una hipótesis de selección de vista; `inspect` consulta y previsualiza, mientras `restore` o `rollback` cambia el estado.
- Curar duplicados, contradicciones, registros huérfanos o vencidos mediante propuestas auditables y reversibles; no borrar evidencia útil silenciosamente.
- Investigar un mantenimiento nocturno que extraiga información relevante de sesiones anteriores, con cursores, deduplicación, presupuesto y protección de tareas activas.
- Evaluar Tree-sitter para ediciones estructurales de Markdown, con secciones permitidas y protección del resto del documento.

## Contexto, compactación y retención

Los checkpoints, resultados recuperables y búsqueda de artefactos ya tienen una base implementada. Falta medir y ampliar su comportamiento:

- Comparar resumen extractivo/abstractivo, retención reciente y recuperación selectiva del original sin perder decisiones, pendientes, bloqueos, supuestos ni efectos realizados.
- Antes de compactar: capturar la ventana, redactar secretos, calcular hash y guardar un checkpoint durable antes de extracción adicional. Después: reconstruir resumen, recuperar contexto pertinente y conservar identificadores de tarea/sesión.
- Mantener el resumen operativo con objetivo; restricciones y preferencias; progreso completado/en curso/bloqueado; decisiones; archivos; próximos pasos y contexto crítico.
- Calibrar retención: `critical` completo mientras sea necesario; `active` mientras haya trabajo/reintentos; `reference` fuera del contexto pero recuperable; `transient` resumible o eliminable después de procesarse.
- Medir guardado y recuperación con precisión, cobertura, falsos recuerdos, contradicciones, filtraciones entre proyectos, pérdida de contexto, latencia y costo. Una mejora media no debe esconder regresiones en consultas exactas o exhaustivas.
- Mejorar tokenización y presupuesto de contexto por modelo, sin confundir ventana configurada con límites de tokens por minuto del proveedor.

## Autonomía prolongada y recuperación

Existe cola durable, deduplicación, contratos y recuperación conservadora. Las ampliaciones deben evitar repetir efectos inciertos:

- Presupuestos por tarea y objetivos prolongados, supervisión externa y criterios de progreso verificable.
- Recuperación más amplia ante interrupciones, con reconciliación de herramientas y acciones externas, sin autorizar reejecución únicamente porque una tarea quedó pendiente.
- Restauración por tarea, turno, llamada de herramienta o archivo cuando la operación lo permita. Distinguir restauración de archivos, cancelación y compensación de efectos externos.
- Extender historial de eventos, identificadores por intento, versiones de contrato y explicación de lo que puede restaurarse. El historial privado actual no equivale a deshacer cualquier acción.
- Evaluar revisión de hitos y finalización según riesgo, evitando auditorías adicionales en todas las acciones simples. Medir desacuerdos entre revisores, correcciones perjudiciales y costo de revisión.
- Mantener autonomía elegida por el usuario, aclaraciones de requisitos indispensables y un modo explícito de evaluación sin interlocutor.

## Heartbeat y recordatorios proactivos

El heartbeat del asistente continúa como ampliación. No confundirlo con cron ni con comprobaciones técnicas de procesos:

- Revisar una lista explícita y estados conocidos, con cadencia, horario activo y presupuesto.
- Comunicar únicamente novedades relevantes, conservar cambios no entregados y evitar avisos repetidos.
- Resolver comprobaciones deterministas mediante código; consultar modelos cuando sea necesaria interpretación.
- No utilizarlo como autorización para acciones nuevas ni como permiso de reanudar efectos interrumpidos.
- Detectar compromisos e intenciones persistentes y evaluar cuándo proponer recordatorios automáticamente sin inventar obligaciones.

Referencia: [heartbeat de OpenClaw](https://docs.openclaw.ai/gateway/heartbeat).

## Skills, configuración y conexiones

El catálogo bajo demanda, edición localizada y aprendizaje selectivo ya existen. Quedan por evaluar:

- Búsqueda semántica de skills únicamente si mejora frente a la recuperación léxica actual con catálogos grandes y tareas reales.
- Calidad del aprendizaje automático, preservación de capacidades, regresiones, versiones y reversión. La aceptación semántica de un candidato no prueba su utilidad.
- Autoconfiguración por conversación con versiones, validación y restauración de ajustes; evitar herramientas nuevas para cada campo de configuración.
- Composición más amplia de herramientas y soporte MCP donde exista una necesidad real.
- Revisión de seguridad de publicaciones con conjuntos etiquetados; no inferir precisión general de unas pocas pruebas del servicio Sentinel.
- Aprovisionamiento automático de capacidad para registro con Google y operaciones multiusuario, antes de prometer crecimiento sin administración manual.

## Programación, búsqueda y navegador

Ya están implementados LSP configurable, parches, worktrees, búsqueda propia y control de Chromium. Futuro trabajo:

- Evaluar instalación automática de servidores LSP y compatibilidad con repositorios/lenguajes adicionales.
- Investigar indexación del código por símbolos o grafos y demostrar mejoras en benchmarks antes de introducir complejidad permanente.
- Ampliar streaming entre protocolos conservando argumentos de herramientas, eventos finales, consumo y metadatos firmados.
- Evaluar cobertura de búsqueda web, frescura, variantes, fuentes especializadas y costo del rastreador. Conservar el requisito de independencia respecto de proveedores comerciales de búsqueda.
- Visión para el navegador, cobertura de frames entre procesos, compatibilidad de sitios, pruebas físicas y entre motores y límites de memoria del sistema operativo.
- Medir concurrencia, admisión global y restauración de checkpoints bajo carga; no extrapolar los ciclos sintéticos de hibernación a cualquier web.
- Ampliar compatibilidad de gVisor/microVM con evidencia de instalación y escenarios adversariales. No debilitar el aislamiento para conseguir compatibilidad.

## Documentos, voz y experiencia de usuario

- Evaluar calidad de extracción en documentos extensos, tablas, OCR, citas y recuperación de evidencia completa; investigar índices vectoriales para volúmenes mayores.
- Ampliar capacidades DOCX/XLSX/PPTX y representación visual solo con formatos y verificación explícitos. El motor actual no recalcula fórmulas ni sustituye una suite ofimática completa.
- Evaluar voz distinta de STT/TTS del navegador si la latencia o compatibilidad lo requiere; estudiar WebRTC y una aplicación instalada posteriormente.
- Mejorar narración del interlocutor con evidencia y lenguaje natural, sin convertir cada evento técnico en un mensaje ni ocultar fallos relevantes.
- Medir experiencia responsiva, accesibilidad y visores de resultados, conservando la conversación continua y los archivos privados por cuenta.

## GitHub gestionado y conexión personal

Propuesta pendiente: una identidad u organización de GitHub administrada por Fénix para usuarios no técnicos, junto a conexión opcional con el GitHub de programadores. Investigar propiedad, GitHub App, permisos, aislamiento de repositorios y exportación/transferencia. No confundir worktrees locales implementados con una cuenta GitHub gestionada ya disponible.

## Godot, animaciones 2D y sociedades simuladas

Propuesta pendiente: mundos 2D con personajes, objetivos, memoria y rutinas; creación por conversación y visualización en el panel derecho. Mantener simulaciones aisladas por usuario y separar acontecimientos ficticios de memoria y tareas reales.

Referencia: [Pixel Agents](https://github.com/pixel-agents-hq/pixel-agents).

## Investigación y referencias

Comparar cada propuesta con una base reproducible: corpus, tareas, versiones, permisos, semillas, modelos y configuración. Medir éxito, tokens, tiempo, costo y RAM. Investigar funciones relevantes de Grok y otros agentes sin copiar afirmaciones comerciales como resultados propios.

Referencias iniciales de memoria/contexto, que requieren revisar versión y metodología antes de utilizarlas como evidencia:

- [MemGPT](https://arxiv.org/abs/2310.08560).
- [RECOMP](https://arxiv.org/abs/2310.04408).
- [Lost in the Middle](https://arxiv.org/abs/2307.03172).

Los detalles implementados y sus límites se documentan en `docs/es/`; los informes históricos conservan sus resultados originales. Las conversaciones sobre un ERP empresarial adaptable siguen siendo ideas exploratorias, no funcionalidad ya programada ni compromiso de esta versión.
