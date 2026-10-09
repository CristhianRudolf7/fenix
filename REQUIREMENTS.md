# Fénix — Requisitos del producto

Revisión documental: 9 de octubre de 2026, hora de Perú. Versión declarada en los paquetes: **0.3.0**. Este documento describe los contratos actuales del código; no certifica que todas las opciones estén habilitadas en cada instalación ni sustituye pruebas reales.

## Producto y modalidades de ejecución

Fénix es un asistente con una conversación continua, tareas concurrentes y subagentes. El núcleo utiliza Rust; la interfaz utiliza Vite, React, TypeScript, Tailwind y pnpm. El frontend se publica como archivos estáticos y puede ejecutarse mediante Caddy/systemd, separado del backend Rust. Node se utiliza para desarrollo y compilación, no como requisito del servidor estático de producción.

Se conservan dos modalidades: ejecución individual con datos locales y despliegue compartido con gateway de cuentas y controladores aislados por usuario. La primera no proporciona por sí sola el aislamiento de la segunda. Las imágenes, políticas, credenciales y servicios se aprovisionan por separado. Véanse [arquitectura](docs/es/architecture.md), [despliegue del frontend](docs/es/frontend-deployment.md) y [aislamiento](docs/es/user-isolation.md).

## Cuentas, sesiones y aislamiento

- El gateway ofrece acceso con correo y contraseña, perfil, cambio de contraseña, cierre de sesión y consumo de tokens registrado.
- Las sesiones duran siete días desde su creación; la actividad no renueva el vencimiento. Los cambios de contraseña y autorización invalidan sesiones anteriores.
- El acceso y registro con Google están implementados, pero necesitan credenciales OAuth configuradas y entornos vacíos previamente reservados. No se promete registro ilimitado sin capacidad aprovisionada.
- Cada cuenta conserva datos, archivos, memoria, credenciales, navegador y ejecuciones separados. Las operaciones deben comprobar identidad, ámbito y permisos antes de acceder a recursos.
- La identidad de Google no concede acceso a Drive u otros servicios: esas conexiones se autorizan por separado.

Véanse [cuentas](docs/es/account-profile.md), [Google](docs/es/google-sign-in.md) y [fronteras de seguridad](docs/es/security-boundaries.md).

## Conversación, modelos y voz

- La web y los canales vinculados mantienen una conversación continua por cuenta. Los identificadores internos permiten coordinar tareas, pero no deben aparecer como requisito de uso normal ni en mensajes dirigidos a personas no técnicas.
- Los roles rápido/interlocutor e inteligente/ejecutor son configurables. El interlocutor comunica avances respaldados por eventos y entrega acciones al motor; no debe afirmar resultados sin evidencia.
- El valor de arranque del código para una configuración nueva sigue siendo `gpt-5.6-luna` con razonamiento bajo. No representa una obligación del producto ni garantiza que ese modelo esté disponible en el proveedor. La configuración de una instalación puede usar otros modelos.
- Telegram es opcional. Las conexiones adicionales, incluido WhatsApp, requieren sus propios servicios y configuración; que exista una tarjeta no demuestra conexión activa.
- El dictado y la voz Live utilizan STT/TTS del navegador. Compatibilidad, permisos y calidad dependen del navegador. Cerrar Live termina la conversación de voz y deja las tareas independientes en ejecución.
- Mensajes ordinarios, respuestas a preguntas, cancelaciones y cambios de plan deben enrutarse sin bloquear toda la conversación.

Véanse [Live](docs/es/live.md), [lenguaje conversacional](docs/es/conversational-language.md) e [integraciones](docs/es/integrations.md).

## Tareas, autonomía y planificación

- Cada tarea conserva objetivo, requisitos, restricciones, supuestos, evidencias, pendientes y estado. Una inferencia no equivale a una instrucción del usuario.
- El usuario expresa autonomía por conversación, proyecto o tarea. Las dudas indispensables sobre objetivos y criterios de aceptación se consultan; las decisiones técnicas reversibles pueden resolverse bajo autonomía autorizada.
- La autonomía no elimina permisos, aislamiento, políticas de conectores ni validaciones de cumplimiento.
- Los planes tienen hitos, dependencias y versiones; pueden editarse durante su elaboración o ejecución. Los resultados atrasados deben conciliarse con la versión vigente.
- Eventos, mensajes pendientes y entregas se conservan en SQLite. Los suscriptores utilizan cursores y avisos asíncronos; no se garantiza entrega externa exactamente una vez ante acuses inciertos.
- La revisión de planes y la auditoría de finalización son selectivas. Las tareas simples no requieren siempre otro agente; la verificación se vincula a requisitos y riesgos.
- La recuperación tras reinicios se limita a fronteras seguras y acciones cuya evidencia permita decidir. No supone repetir efectos externos automáticamente.

Véanse [planificación](docs/es/planning.md), [autonomía](docs/es/conversational-autonomy.md), [revisión](docs/es/plan-reviews.md) y [continuidad](docs/es/harness-continuity.md).

## Subagentes y proyectos

- El maestro puede delegar una tarea con un propósito y resultado esperado concretos mientras sigue disponible.
- Los subagentes tienen contexto de tarea y herramientas autorizadas; no reciben automáticamente todo el historial ni pueden crear otros subagentes. El catálogo de skills es común y se recupera bajo demanda.
- Se conservan seguimiento, estados, cancelación y contadores de reintento. El máximo configurado no autoriza repetir efectos inciertos.
- Una solicitud puede contener trabajo en varios proyectos. Se fija el ámbito antes de recuperar contexto o ejecutar y se conserva conversación general separada del contexto específico.
- Las escrituras simultáneas pueden utilizar worktrees de Git con integración local serializada. Esto no equivale a fusionar automáticamente cambios incompatibles.

Véanse [proyectos](docs/es/projects.md) y [worktrees](docs/es/parallel-worktrees.md).

## Skills y herramientas

- Se conservan un catálogo léxico incremental, búsqueda, listado paginado y carga de instrucciones/referencias bajo demanda. El contexto no contiene cientos de paquetes completos.
- Los paquetes canónicos del agente mantienen inglés; la interfaz usa Markdown español persistido. Las traducciones se generan al crear o editar, no cada vez que se abre un skill.
- La interfaz permite consultar, editar con guardado explícito y eliminar skills de la cuenta. Los cambios personales no sobrescriben paquetes de otras cuentas.
- El catálogo de seguridad está unificado: no se exige un subagente especializado permanente para acceder a esos skills.
- El aprendizaje selectivo conserva candidatos, revisión, activación, protección de personalizaciones y reversión. La revisión semántica no sustituye pruebas funcionales.
- La publicación y revisión de skills siguen disponibles en el backend; no forman parte del catálogo visible actual de la interfaz. No confundir existencia de APIs con una página de marketplace activa.
- El descubrimiento de herramientas respeta la autorización vigente. Los programas de herramientas permiten agrupar operaciones sin omitir controles.

Véanse [skills](docs/es/skills-and-documents.md), [biblioteca](docs/es/skill-library.md) y [servicio de publicación](docs/es/skill-marketplace.md).

## Memoria, contexto e historial

- La memoria tiene ámbito general, de proyecto y tarea; registra procedencia, entidades, alias, relaciones, vigencia y revisiones.
- La búsqueda combina opciones léxicas, temporales y relacionales. La recuperación semántica depende de configuración; no debe afirmarse que toda memoria siempre usa embeddings.
- Se conservan captura en segundo plano, revisión de candidatos, contradicciones y comprobaciones no destructivas. Las decisiones automáticas deben conservar trazabilidad y no inventar hechos.
- Los valores iniciales del contexto son 200.000 tokens de capacidad, compactación a 160.000 y 6.000 tokens recientes. Son ajustes del motor: no amplían los límites reales del modelo o proveedor.
- La compactación mantiene checkpoints y procedencia; puede recuperar originales y resultados guardados. El resumen no reemplaza la evidencia ni confirma una tarea terminada.
- El historial privado de archivos permite restauraciones cubiertas por sus herramientas. No revierte automáticamente correos, comandos arbitrarios ni otros efectos externos.

Véanse [memoria adaptable](docs/es/adaptive-memory.md), [continuidad](docs/es/harness-continuity.md) y [recuperación](docs/es/harness-recovery.md).

## Programación y terminal

- Búsqueda regex/glob, lectura y parches de varios archivos, instrucciones `AGENTS.md` según repositorio y servidores LSP configurables.
- Terminal con entrada interactiva, procesos persistentes, resultados incrementales y eventos de finalización. Los consumidores pueden esperar o consultar el proceso según su estado.
- Las pruebas deben responder a requisitos y riesgos; reutilizar pruebas existentes y evitar comprobaciones redundantes que no aporten evidencia.
- La finalización debe apoyarse en resultados verificables, especialmente después de errores, truncación o compactación.
- Los límites de pasos, tiempos y salida son configurables; el modo de supervisión externa no elimina las políticas ni los límites físicos y del proveedor.
- El código nativo incluye evaluación del agente y adaptadores/soporte para los benchmarks documentados. Los informes corresponden a las tareas y configuración realmente ejecutadas.

Véanse [arnés](docs/es/coding-harness.md), [ejecución](docs/es/execution-harness.md), [evaluación](docs/es/native-evaluation.md) y [informes](docs/benchmarks/).

## Navegador, búsqueda y conexiones

- Búsqueda propia con índice local y rastreador; no depende de un proveedor comercial de búsqueda. La cobertura no equivale a la de un buscador global.
- Chromium controlado por el agente, perfiles persistentes y acceso interactivo del propietario. Sesiones, credenciales y cookies pertenecen a la cuenta.
- Se conserva estado recuperable en disco cuando procede, apagado de navegadores inactivos, lectura HTTP ligera y coordinación global de capacidad. Un estado guardado no reproduce cualquier aplicación web en memoria ni evita caducidad de sesiones.
- Los sitios pueden bloquear automatización, requerir MFA o tener componentes no soportados. No prometer compatibilidad absoluta.
- Las conexiones OAuth y credenciales son opcionales y específicas de cada servicio. Las herramientas locales de documentos, PDF y búsqueda no se presentan como proveedores externos.

Véanse [navegador](docs/es/browser.md), [ciclo de vida](docs/es/browser-lifecycle.md), [búsqueda](docs/es/web-search.md) y [conectores y flujos](docs/es/workflows-and-connectors.md).

## Documentos, archivos y resultados visuales

- Biblioteca privada de archivos originales y versiones, extracción asíncrona, fragmentos con citas y búsqueda léxica/híbrida.
- Embeddings opcionales mediante `text-embedding-3-small`; se reutilizan fragmentos sin cambios. Un fallo del proveedor no elimina la búsqueda léxica.
- PDF e imágenes utilizan OCR configurado y aislado. Office utiliza extracción nativa; fórmulas no se recalculan y el texto extraído no reproduce toda la maquetación.
- Herramientas propias para crear, leer y editar DOCX, XLSX y PPTX, con límites documentados; no constituyen una suite Office completa.
- Propuestas de diseño en carrusel dentro del chat, previsualización modal y resultados en el panel derecho. No existe una página Design independiente en la navegación actual.
- El explorador modal y las pestañas de archivos solo acceden al espacio autorizado de la cuenta. Diagramas, gráficas, documentos y multimedia tienen visores específicos y límites explícitos.

Véanse [biblioteca](docs/es/document-library.md), [documentos nativos](docs/es/skills-and-documents.md), [diseños](docs/es/design.md) y [explorador](docs/es/workspace-viewer.md).

## Comprobación y límites pendientes

La existencia de código o de pruebas simuladas no certifica fiabilidad universal ni despliegue de cada función. Conservar fechas, configuración y alcance de las evaluaciones históricas. Sigue pendiente comprobar el firewall en la infraestructura real: destinos permitidos/bloqueados, DNS y detención al perder protección. Las opciones de microVM y compatibilidad ampliada tienen su propia evidencia, no una garantía general.

Los cambios futuros se registran en [ROADMAP.md](ROADMAP.md); los cambios ya realizados se resumen en [CHANGELOG.md](CHANGELOG.md).
