# Fénix — Requisitos del producto

Revisión documental: 11 de octubre de 2026, hora de Perú. Versión declarada en los paquetes: **0.3.2 (en desarrollo)**. Este documento describe los contratos actuales del código; no certifica que todas las opciones estén habilitadas en cada instalación ni sustituye pruebas reales.

## Producto y modalidades de ejecución

Fénix es un asistente con una conversación continua, tareas concurrentes y subagentes. El núcleo utiliza Rust; la interfaz utiliza Vite, React, TypeScript, Tailwind y pnpm. El frontend se publica como archivos estáticos y puede ejecutarse mediante Caddy/systemd, separado del backend Rust. Node se utiliza para desarrollo y compilación, no como requisito del servidor estático de producción.

La distribución 0.3.2 se orienta a una instalación personal por cuenta del sistema operativo. Los comandos se ejecutan localmente; no hay sandbox de ejecución multiusuario. Las variables antiguas de aislamiento se rechazan y requieren migración explícita. El gateway multiusuario se ha retirado; el servidor incorpora autenticación de un único propietario. Véase [ejecución local](docs/es/local-execution.md).

## Propietario y sesiones

- El servidor personal ofrece acceso con correo y contraseña, perfil, cambio de contraseña, cierre de sesión y consumo de tokens registrado.
- Las sesiones duran siete días desde su creación; la actividad no renueva el vencimiento. Los cambios de contraseña y autorización invalidan sesiones anteriores.
- El propietario se configura localmente mediante entrada estándar. No hay registro público ni acceso con Google para cuentas; las conexiones OAuth con servicios externos son independientes.
- Los registros y rutas de cuenta conservados son compatibilidad histórica. La versión personal no admite separación de procesos por contenedor entre usuarios; no debe publicarse como un servicio compartido sin un nuevo diseño de aislamiento.
- La identidad de Google no concede acceso a Drive u otros servicios: esas conexiones se autorizan por separado.

Véanse [cuentas](docs/es/account-profile.md), [Google](docs/es/google-sign-in.md) y [fronteras de seguridad](docs/es/security-boundaries.md).

## Conversación, modelos y voz

- La web y los canales vinculados mantienen una conversación continua por cuenta. Los identificadores internos permiten coordinar tareas, pero no deben aparecer como requisito de uso normal ni en mensajes dirigidos a personas no técnicas.
- Los roles rápido/interlocutor e inteligente/ejecutor son configurables. El interlocutor comunica avances respaldados por eventos y entrega acciones al motor; no debe afirmar resultados sin evidencia.
- El valor de arranque del código para una configuración nueva sigue siendo `gpt-5.6-luna` con razonamiento bajo. No representa una obligación del producto ni garantiza que ese modelo esté disponible en el proveedor. La configuración de una instalación puede usar otros modelos.
- Telegram es opcional. Las conexiones adicionales, incluido WhatsApp, requieren sus propios servicios y configuración; que exista una tarjeta no demuestra conexión activa.
- El micrófono utiliza el reconocimiento de voz del navegador para dictar al borrador, sin enviarlo automáticamente. La compatibilidad y los permisos dependen del navegador. No existe modo Live ni síntesis de voz.
- Mensajes ordinarios, respuestas a preguntas, cancelaciones y cambios de plan deben enrutarse sin bloquear toda la conversación.

Véanse [Dictado](docs/es/dictation.md), [lenguaje conversacional](docs/es/conversational-language.md) e [integraciones](docs/es/integrations.md).

## Tareas, autonomía y planificación

- Cada tarea conserva objetivo, requisitos, restricciones, supuestos, evidencias, pendientes y estado. Una inferencia no equivale a una instrucción del usuario.
- El usuario expresa autonomía por conversación, proyecto o tarea. Las dudas indispensables sobre objetivos y criterios de aceptación se consultan; las decisiones técnicas reversibles pueden resolverse bajo autonomía autorizada.
- La autonomía no elimina las aprobaciones de comandos de riesgo, las políticas de conectores ni las validaciones de cumplimiento.
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
- Los skills son personales: se pueden consultar, editar y eliminar. No existe servicio de marketplace, publicación ni revisión para publicar.
- El descubrimiento de herramientas respeta la autorización vigente. Los programas de herramientas permiten agrupar operaciones sin omitir controles.

Véanse [skills](docs/es/skills-and-documents.md), [biblioteca](docs/es/skill-library.md).

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

## Portabilidad del motor de ejecución

Inicialmente orientado a Linux, el motor común utiliza Tokio y terminales nativas mediante `portable-pty`, con Bash en Linux/macOS y PowerShell en Windows. Conserva los contratos de procesos y permisos. La adaptación del resto de accesos Unix y de los instaladores sigue pendiente; véase [alcance y comprobaciones](docs/es/native-process-engine.md).

### Acceso al sistema de archivos en instalación personal

El workspace es la carpeta predeterminada para trabajar y crear proyectos, no una frontera de acceso. Las herramientas del agente admiten rutas absolutas, rutas relativas externas y enlaces a archivos, con los permisos del usuario que ejecuta Fénix. La búsqueda puede partir de una carpeta externa y el análisis LSP puede abrir archivos externos. Los comandos de riesgo siguen requiriendo aprobación textual; se conservan la ocultación de secretos en resultados y las comprobaciones contra cambios inesperados al editar. Los límites de tamaño de las herramientas de texto no son permisos del sistema: para archivos grandes o binarios se utilizan las herramientas apropiadas o comandos. Este cambio no concede permisos de administrador ni publica archivos externos mediante el visor web.

### Motor de archivos multiplataforma

La lectura, creación, edición, borrado, resolución de rutas y listado se concentran en `fenix-files`, con adaptación Unix/Windows. El chat y el explorador ya no requieren `/proc` para listar carpetas. Se mantienen las comprobaciones de contenido, los permisos del sistema y la separación entre acceso personal y archivos privados. Véase [el alcance y la validación del motor](docs/es/native-file-engine.md); esta adaptación no equivale a portar toda la aplicación ni a validar instalaciones completas en Windows/macOS.

### Gestión de skills y almacenamiento multiplataforma

Se incorpora `fenix-skills`, acceso por handles, bloqueo mediante archivos y protección de datos mediante permisos Unix o DACL de Windows. Se conservan traducciones, recursos, personalizaciones, revisión antes de guardar, eliminaciones durables, memoria y persistencia SQLite. Las nuevas ubicaciones predeterminadas respetan rutas explícitas y `.fenix` local existente. [Alcance y validación](docs/es/skills-storage.md): la ejecución nativa en Windows/macOS y el soporte completo de la aplicación siguen pendientes.

## Selección del intérprete de comandos

El motor detecta y verifica intérpretes disponibles, conserva su selección durante la instancia y registra ruta, familia y versión disponible por proceso. `FENIX_SHELL` permite una selección explícita antes del arranque; una configuración inválida no provoca sustitución silenciosa. `environment_inspect` presenta las alternativas al agente. Los comandos no se traducen entre lenguajes ni se repiten automáticamente con otro shell. Android incorpora selección del shell del sistema, sin declarar completa la aplicación Android. Véase [motor de procesos](docs/es/native-process-engine.md).

## Base móvil iOS

La base portable distingue iOS de macOS: almacena datos y skills en el contenedor de la aplicación y rechaza shells, terminales y procesos externos. El catálogo móvil no anuncia navegador autónomo. Se verificó la compilación cruzada de archivos y procesos para iOS ARM64; faltan el host nativo, puente Rust, permisos de documentos, Keychain, notificaciones, programación de segundo plano y validación con Xcode/dispositivos. No equivale a soporte completo de la aplicación iOS.

## Cierre y almacenamiento portable

El servidor y las evaluaciones usan una fuente de cierre por plataforma o por petición del host móvil. El transporte de ejecución remoto se ha eliminado; los benchmarks pueden utilizar sockets Unix o TCP de loopback. Configuración e índice del buscador, credenciales cifradas y perfiles del navegador utilizan el motor de almacenamiento privado. El núcleo compila para los cinco destinos; faltan validaciones nativas y los hosts móviles completos. Véase [núcleo portable](docs/es/platform-core.md).

## Configuración Windows y macOS

- Detección automática de navegadores Chromium compatibles, con selección explícita opcional.
- Medición nativa de memoria del navegador en Linux, Windows y macOS.
- Instalación desde código: LaunchAgent personal en macOS y tarea al iniciar sesión en Windows.
- Python configurable para extracción documental, conservando comprobaciones de identidad y hash.
- Validación completa de instalación y tareas en equipos Windows/macOS todavía pendiente; no se anuncia una release binaria validada. Véase [la guía](docs/es/desktop-setup.md).

## Disponibilidad de modelos en el chat web

El servidor personal comprueba conexiones cuando corresponde y conserva resultados por identidad de conexión. El chat bloquea nuevos envíos si uno de los modelos necesarios no está disponible o todavía necesita comprobación; conserva el borrador y permite comprobar o revisar configuración. Las consultas de estado no generan inferencias. El servidor aplica el mismo bloqueo a los mensajes web y las evaluaciones no inician estos sondeos automáticamente. Véase [el alcance y validación](docs/es/model-availability.md).
