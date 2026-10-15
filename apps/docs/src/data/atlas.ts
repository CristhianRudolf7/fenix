export type Area = {
  id: string;
  title: string;
  subtitle: string;
  summary: string;
  steps: string[];
  questions: string[];
  match: string[];
  note?: string;
};
export const areas: Area[] = [
  {
    id: "conversation",
    title: "Conversación",
    subtitle: "De tu mensaje a una tarea",
    summary:
      "Web y Telegram llegan al mismo runtime personal. El interlocutor comunica avances observables y remite instrucciones; el motor conserva la conversación y ejecuta el trabajo.",
    steps: [
      "Recibir y registrar el mensaje en su canal",
      "Interpretar la intención y resolver respuestas pendientes",
      "Encolar el trabajo con su contexto y proyecto",
      "Publicar progreso y entregar el resultado al canal",
    ],
    questions: [
      "¿Qué sucede si llega otro mensaje mientras hay una tarea activa?",
      "¿Qué información puede comunicar el interlocutor sin inventar avances?",
    ],
    match: [
      "conversation",
      "interlocutor",
      "telegram",
      "progress",
      "model_roles",
      "usage_tracking",
    ],
  },
  {
    id: "engine",
    title: "Motor del agente",
    subtitle: "El ciclo que hace el trabajo",
    summary:
      "Runtime reúne servicios, trabajadores y estado. El ejecutor construye contexto, consulta al modelo, procesa llamadas de herramientas y decide si continuar o cerrar con evidencia.",
    steps: [
      "Abrir el runtime y sus servicios locales",
      "Construir los mensajes y el catálogo disponible",
      "Pedir la siguiente acción al proveedor",
      "Ejecutar herramientas y registrar resultados",
      "Comprobar el cierre o continuar el ciclo",
    ],
    questions: [
      "¿Dónde se detecta una tarea incompleta?",
      "¿Qué pertenece al modelo y qué comprueba el motor?",
    ],
    match: [
      "engine",
      "execution_state",
      "completion_gate",
      "completion_audits",
      "stop_gate",
      "restart_recovery",
      "harness_improvements_tests",
      "lib.rs",
    ],
    note: "La finalización del runtime no equivale a aprobar un benchmark: las evaluaciones registran esa diferencia.",
  },
  {
    id: "planning",
    title: "Planes y subagentes",
    subtitle: "Objetivos, hitos y delegación",
    summary:
      "Los planes descomponen trabajo en hitos con dependencias. Los subagentes reciben un objetivo concreto y trabajan en segundo plano; los eventos permiten al usuario cambiar la dirección del trabajo.",
    steps: [
      "Definir objetivo, requisitos y dudas iniciales",
      "Crear hitos y revisar los que necesitan comprobación",
      "Delegar objetivos concretos con contexto específico",
      "Recibir evidencia, bloqueos y resultados",
      "Replanificar o auditar la finalización",
    ],
    questions: [
      "¿Qué ocurre cuando se modifica un plan que ya está en curso?",
      "¿Cómo se comunican resultados sin bloquear la conversación?",
    ],
    match: [
      "planning",
      "subagents",
      "plan_reviews",
      "project_specs",
      "decisions",
      "autonomy",
    ],
    note: "La implementación limita la delegación a un nivel; los subagentes tienen como máximo tres intentos. No se aplica una auditoría adicional idéntica a todas las tareas pequeñas.",
  },
  {
    id: "context",
    title: "Proyectos y contexto",
    subtitle: "Mantener el hilo sin mezclar ámbitos",
    summary:
      "La conversación general convive con proyectos y tareas. El motor selecciona contexto activo, conserva información operativa y compacta cuando corresponde; los requisitos del proveedor siguen siendo independientes.",
    steps: [
      "Identificar el ámbito de la petición",
      "Recuperar contexto activo y requisitos del proyecto",
      "Construir contexto del maestro o subagente",
      "Reducir historial cuando se alcanza el umbral",
      "Conservar información necesaria para continuar",
    ],
    questions: [
      "¿Qué se comparte entre proyectos y qué debe permanecer separado?",
      "¿Qué puede perderse durante una compactación?",
    ],
    match: [
      "context",
      "context_budget",
      "active_context",
      "operational_context",
      "projects",
      "project_tools",
    ],
    note: "El valor inicial configurado de ventana es 200.000 tokens; no aumenta la ventana real ni las cuotas del proveedor.",
  },
  {
    id: "memory",
    title: "Memoria",
    subtitle: "Hechos, fuentes y recuperación",
    summary:
      "La memoria combina captura de candidatos, procedencia, ámbito, recuperación y revisión. El almacenamiento separa recuerdos, conocimiento y mantenimiento; la captura automática y los embeddings dependen de la configuración.",
    steps: [
      "Detectar candidatos de memoria fuera del turno principal",
      "Guardar hechos con procedencia y ámbito",
      "Recuperar evidencia relevante para una consulta",
      "Relacionar conocimiento y revisar conflictos",
      "Mantener la salud y evaluar la recuperación",
    ],
    questions: [
      "¿Cómo se diferencia un hecho de una inferencia?",
      "¿Qué memoria se introduce realmente en el contexto?",
    ],
    match: ["memory", "knowledge", "semantic_memory"],
    note: "Los nombres de archivos describen responsabilidades; las relaciones del atlas son recorridos didácticos, no un grafo exhaustivo de llamadas.",
  },
  {
    id: "skills",
    title: "Skills y herramientas",
    subtitle: "Descubrir, cargar y aprender",
    summary:
      "El catálogo permite localizar skills y cargar sus instrucciones bajo demanda. La gestión conserva recursos, versiones y traducciones; el descubrimiento de herramientas evita tener que presentar todas sus definiciones en cada turno.",
    steps: [
      "Consultar el catálogo y las categorías internas",
      "Seleccionar un skill o una herramienta relevante",
      "Leer instrucciones y recursos bajo demanda",
      "Ejecutar con las políticas del runtime",
      "Gestionar revisiones, aprendizaje y traducciones persistidas",
    ],
    questions: [
      "¿Cuándo entra el contenido completo de un skill en el contexto?",
      "¿Qué cambia cuando el usuario edita sus instrucciones?",
    ],
    match: [
      "skills",
      "skill_",
      "tool_discovery",
      "tool_program",
      "tool_receipt",
      "tools",
    ],
    note: "No existe marketplace en la versión personal actual. Las instrucciones internas conservan su idioma; la interfaz utiliza las traducciones guardadas.",
  },
  {
    id: "coding",
    title: "Programación y archivos",
    subtitle: "Explorar, editar y comprobar",
    summary:
      "El arnés de programación reúne búsqueda, símbolos, LSP, parches y contexto del entorno. Los worktrees separan modificaciones concurrentes y el historial permite consultar revisiones de archivos.",
    steps: [
      "Inspeccionar el repositorio y su estado real",
      "Buscar símbolos y consultar el servidor de lenguaje",
      "Editar archivos o aplicar parches",
      "Ejecutar comprobaciones ligadas al requisito y riesgo",
      "Revisar cambios e integrar worktrees",
    ],
    questions: [
      "¿Qué diferencia hay entre búsqueda textual y navegación con LSP?",
      "¿Cómo se evitan cambios simultáneos sobre el mismo archivo?",
    ],
    match: [
      "coding",
      "files",
      "file_history",
      "workspace_view",
      "worktrees",
      "workspace",
    ],
    note: "El workspace es una ubicación de trabajo, no una barrera de acceso. Rigen los permisos del usuario del sistema operativo.",
  },
  {
    id: "processes",
    title: "Terminal y plataforma",
    subtitle: "Procesos locales, entrada y cierre",
    summary:
      "El motor nativo adapta intérpretes, argumentos, terminales y supervisión a cada plataforma. Conserva salida, entrada, cancelación y notificaciones de finalización.",
    steps: [
      "Detectar y verificar el intérprete disponible",
      "Lanzar un proceso con directorio y entorno",
      "Recibir salida y gestionar entrada o terminal interactiva",
      "Notificar finalización, fallo o necesidad de intervención",
      "Cancelar y cerrar ordenadamente",
    ],
    questions: [
      "¿Cuándo se usa una PTY y cuándo una tubería?",
      "¿Qué queda pendiente para los hosts móviles?",
    ],
    match: ["processes", "fenix-process", "fenix-execution"],
    note: "iOS no permite ejecutar shells o procesos externos como un escritorio. La base portable no equivale a una aplicación móvil completa.",
  },
  {
    id: "browser",
    title: "Navegador",
    subtitle: "Observar, actuar y conservar sesión",
    summary:
      "Chromium local se controla mediante CDP. Los perfiles conservan datos de navegación; las herramientas observan páginas, actúan sobre elementos y gestionan descargas. La vista del propietario permite interacción directa.",
    steps: [
      "Abrir el perfil y conectar a Chromium",
      "Observar pestañas, DOM y frames",
      "Actuar sobre elementos y comprobar el estado visible",
      "Gestionar credenciales, archivos y descargas",
      "Mantener sesiones y cerrar las que ya no se necesitan",
    ],
    questions: [
      "¿Qué se guarda en el perfil y qué sigue solo en RAM?",
      "¿Qué ocurre con un formulario cuando la tarea espera al usuario?",
    ],
    match: ["fenix-browser", "browser.rs"],
    note: "Cookies y datos persistentes no son un checkpoint completo de RAM. No se reenvían acciones para restaurar formularios. El catálogo móvil omite el navegador autónomo.",
  },
  {
    id: "web",
    title: "Búsqueda web",
    subtitle: "Buscar y leer con menos recursos",
    summary:
      "El buscador propio y la lectura ligera ofrecen una alternativa a abrir Chromium cuando basta con recuperar contenido web. El índice y la configuración se guardan localmente.",
    steps: [
      "Recibir la consulta o una URL",
      "Buscar fuentes o recuperar contenido",
      "Procesar y limitar el contenido obtenido",
      "Entregar resultados con sus referencias",
    ],
    questions: [
      "¿Cuándo basta lectura ligera y cuándo hace falta un navegador?",
      "¿Qué fuentes respaldan una respuesta?",
    ],
    match: ["fenix-web"],
  },
  {
    id: "documents",
    title: "Documentos y biblioteca",
    subtitle: "De archivos a evidencia consultable",
    summary:
      "La biblioteca coordina extracción, fragmentación e indexación de documentos. La búsqueda devuelve fragmentos y referencias; los embeddings son una parte configurable, no una lectura completa del documento por el modelo en cada consulta.",
    steps: [
      "Registrar el archivo y preparar su extracción",
      "Extraer contenido con herramientas locales según formato",
      "Dividir e indexar fragmentos",
      "Generar embeddings si están configurados",
      "Buscar fragmentos y responder con evidencia",
    ],
    questions: [
      "¿Qué datos se generan al importar un PDF?",
      "¿Qué coste tiene preparar el índice frente a consultarlo?",
    ],
    match: ["fenix-documents", "documents", "library", "embeddings"],
    note: "OCR y algunos formatos necesitan dependencias externas. En iOS no se ejecuta la extracción basada en procesos externos.",
  },
  {
    id: "automation",
    title: "Automatización y conexiones",
    subtitle: "Horarios y flujos de trabajo",
    summary:
      "Recordatorios y cron lanzan tareas programadas. Los workflows conectan pasos, condiciones y ejecución. Los conectores guardan autorización cifrada para los servicios disponibles.",
    steps: [
      "Crear horario o definir nodos y conexiones del flujo",
      "Guardar configuración y credenciales protegidas",
      "Activar el trabajo cuando corresponde",
      "Ejecutar pasos con políticas de autorización",
      "Registrar resultados y notificar por el canal",
    ],
    questions: [
      "¿Qué diferencia hay entre un recordatorio y un workflow?",
      "¿Cómo se maneja una conexión que necesita autorización?",
    ],
    match: [
      "schedules",
      "workflows",
      "connectors",
      "integrations",
      "fenix-connectors",
    ],
    note: "Google Drive fue retirado del conector nativo. Una capacidad depende de su conexión, autorización y configuración; no todas funcionan sin preparación.",
  },
  {
    id: "artifacts",
    title: "Diseños y artefactos",
    subtitle: "Resultados que puedes abrir",
    summary:
      "Los diseños, diagramas y opciones visuales conectan la generación de contenido con archivos y vistas del usuario. El backend mantiene versiones y selecciones de diseño.",
    steps: [
      "Recoger preferencias y ofrecer alternativas",
      "Registrar la selección de una plantilla",
      "Generar o modificar el artefacto",
      "Guardar su versión y entregar una referencia",
    ],
    questions: [
      "¿Dónde se conserva la elección del usuario?",
      "¿Cómo se relaciona un diagrama con su archivo?",
    ],
    match: ["designs", "design_choice"],
  },
  {
    id: "security",
    title: "Seguridad y permisos",
    subtitle: "Proteger datos y pedir permiso",
    summary:
      "La instalación personal autentica al propietario, protege secretos y detecta señales de riesgo. Las aprobaciones textuales se vinculan a acciones concretas; no hay sandbox multiusuario ni un subagente de seguridad exclusivo.",
    steps: [
      "Validar sesión, origen y configuración",
      "Redactar secretos y proteger credenciales",
      "Examinar la acción y sus posibles efectos",
      "Pedir una respuesta textual si requiere permiso",
      "Registrar la decisión y ejecutar o rechazar",
    ],
    questions: [
      "¿Qué protege una aprobación y qué no puede garantizar?",
      "¿Qué datos nunca deberían aparecer en el progreso?",
    ],
    match: [
      "security",
      "permissions",
      "command_safety",
      "content_safety",
      "redaction",
      "effects",
      "auth",
      "fenix-credentials",
    ],
    note: "Detectar comandos de riesgo no certifica código arbitrario. Sigue pendiente la comprobación real del firewall del despliegue.",
  },
  {
    id: "foundation",
    title: "Datos y proveedores",
    subtitle: "La base compartida",
    summary:
      "Tipos del núcleo, adaptadores de modelos y almacenamiento SQLite sostienen las demás funciones. Los módulos de almacenamiento específicos aparecen también bajo la funcionalidad que implementan.",
    steps: [
      "Validar configuración y seleccionar roles de modelos",
      "Construir solicitudes y procesar streaming",
      "Persistir tareas, eventos y consumo",
      "Abrir archivos privados y aplicar permisos nativos",
    ],
    questions: [
      "¿Dónde se guardan los estados durables?",
      "¿Qué cambia al utilizar otro proveedor?",
    ],
    match: [
      "fenix-core",
      "fenix-providers",
      "fenix-storage",
      "fenix-files",
      "config",
      "assets",
      "model_roles",
    ],
    note: "El atlas describe código del árbol local. Un enlace a main puede diferir si este cambio todavía no se ha publicado.",
  },
  {
    id: "evaluation",
    title: "Evaluaciones",
    subtitle: "Medir sin confundir ejecución con éxito",
    summary:
      "El adaptador nativo registra trazas, uso y resultados de benchmarks. Las pruebas verifican componentes; una simulación o un doble de prueba no certifica producción ni el resultado de un proveedor.",
    steps: [
      "Configurar modelo, tarea y entorno de evaluación",
      "Ejecutar el agente y capturar eventos",
      "Recoger evidencia, métricas y resultado del evaluador",
      "Separar tarea completada de tarea aprobada",
    ],
    questions: [
      "¿Quién determina si se aprobó la tarea?",
      "¿Qué métricas pueden faltar en una respuesta del proveedor?",
    ],
    match: ["evaluation", "memory_eval", "tests"],
  },
];
export function areaFor(file: { path: string; test: boolean }): Area {
  if (
    file.path.startsWith("scripts/") &&
    /benchmark|eval|harbor/.test(file.path)
  )
    return areas.find((a) => a.id === "evaluation")!;
  if (file.path.startsWith("scripts/") && /ocr|pdf|document/.test(file.path))
    return areas.find((a) => a.id === "documents")!;
  if (file.test) return areas.find((a) => a.id === "evaluation")!;
  const crate = file.path.split("/")[1];
  const dedicated: Record<string, string> = {
    "fenix-browser": "browser",
    "fenix-web": "web",
    "fenix-documents": "documents",
    "fenix-skills": "skills",
    "fenix-process": "processes",
    "fenix-execution": "processes",
    "fenix-credentials": "security",
    "fenix-files": "coding",
    "fenix-providers": "foundation",
    "fenix-core": "foundation",
  };
  if (file.path.startsWith("crates/") && dedicated[crate])
    return areas.find((a) => a.id === dedicated[crate])!;
  if (file.path === 'crates/fenix-storage/src/plans.rs') return areas.find(a => a.id === 'planning')!;
  if (file.path === 'crates/fenix-storage/src/memory_eval.rs') return areas.find(a => a.id === 'evaluation')!;
  if (file.path === 'crates/fenix-storage/src/recovery.rs') return areas.find(a => a.id === 'engine')!;
  const short = file.path.replace("crates/fenix-agent/src/", "");
  return (
    areas.find(
      (a) =>
        a.id !== "foundation" &&
        a.id !== "evaluation" &&
        a.match.some((m) =>
          m === "lib.rs"
            ? file.path === "crates/fenix-agent/src/lib.rs"
            : short.includes(m),
        ),
    ) ?? areas.find((a) => a.id === "foundation")!
  );
}

export const fileGuides: Record<string, string> = {
  engine:
    "Coordina el ciclo de ejecución: mensajes del modelo, llamadas de herramientas, resultados y continuidad de la tarea.",
  lib: "Punto de entrada del crate: declara módulos, tipos compartidos y operaciones públicas. En el agente reúne los servicios en Runtime.",
  main: "Arranque del servidor: configuración de rutas, apertura del runtime y cierre ordenado.",
  auth: "Autenticación del propietario, contraseñas y sesiones de siete días; mantiene la sesión sin convertir el servidor en multiusuario.",
  conversation:
    "Gestiona el ingreso de mensajes y su relación con tareas, respuestas y conversación continua.",
  interlocutor:
    "Interpreta mensajes del usuario y comunica progreso observable; no ejecuta por su cuenta las acciones del maestro.",
  subagents:
    "Crea y supervisa delegaciones en segundo plano con objetivos específicos, concurrencia y reintentos.",
  planning:
    "Orquesta hitos, controles de planes, replanificación y entrega de eventos.",
  plan_reviews:
    "Revisión de planes y evidencia de los puntos que requieren validación.",
  completion_audits:
    "Auditoría de finalización, fuentes de evidencia y observaciones del trabajo largo.",
  completion_gate:
    "Comprueba criterios de cierre antes de dar por terminada una tarea.",
  stop_gate:
    "Evalúa si un intento de detenerse tiene evidencia suficiente o debe continuar.",
  context: "Construye mensajes y contexto para el maestro y los subagentes.",
  context_budget:
    "Calcula y aplica el presupuesto de contexto sin cambiar las cuotas del proveedor.",
  active_context:
    "Selecciona y conserva el contexto activo de la conversación y sus tareas.",
  operational_context:
    "Mantiene información operativa necesaria para continuar el trabajo y sus verificaciones.",
  memory:
    "Captura, guarda o consulta memoria con ámbito y evidencia, según el crate al que pertenece.",
  semantic_memory:
    "Almacenamiento y recuperación de información semántica de la memoria.",
  memory_health:
    "Detecta y gestiona condiciones de mantenimiento de la memoria.",
  memory_retrieval: "Recuperación y selección de candidatos de memoria.",
  knowledge: "Persistencia de conocimiento y relaciones entre elementos.",
  skills:
    "Catálogo e integración de instrucciones especializadas bajo demanda.",
  bundle: "Carga y gestión del paquete de skills distribuido con el agente.",
  management: "Operaciones de gestión de skills y sus recursos.",
  skill_categories:
    "Catálogo de categorías internas para organizar y descubrir skills.",
  skill_localization:
    "Versiones traducidas persistentes de las instrucciones de skills.",
  skill_learning:
    "Captura y gestión del aprendizaje de procedimientos reutilizables.",
  tool_discovery:
    "Descubrimiento selectivo de herramientas para evitar cargar siempre el catálogo completo.",
  tools:
    "Define o despacha las herramientas disponibles dentro del servicio al que pertenece.",
  tool_program: "Coordina programas y secuencias de herramientas.",
  tool_receipt:
    "Registro de resultados y evidencia de llamadas de herramientas.",
  projects: "Gestión y persistencia de proyectos y sus ámbitos de contexto.",
  project_specs:
    "Especificaciones y requisitos de proyectos, con su estado de validación.",
  project_tools:
    "Selecciona herramientas y permisos disponibles para el contexto del proyecto o tarea.",
  decisions: "Gestión de decisiones y comunicación de opciones al usuario.",
  autonomy:
    "Política de autonomía y tratamiento de dudas sin eliminar la verificación de requisitos.",
  processes:
    "Supervisa comandos, entrada/salida, sesiones de terminal y finalización dentro del agente.",
  output: "Lectura y presentación de resultados de procesos.",
  shutdown:
    "Fuente portable de cierre: señales del escritorio y solicitud explícita del host móvil.",
  transport:
    "Comunicación de evaluación local mediante loopback TCP o socket Unix.",
  files: "Herramientas de lectura, escritura y operaciones sobre archivos.",
  private: "Operaciones privadas de archivos, permisos y bloqueos nativos.",
  file_history: "Historial y restauración de revisiones de archivos.",
  workspace_view:
    "Navegación de archivos y referencias visibles en el espacio de trabajo.",
  worktrees:
    "Separación de cambios concurrentes mediante worktrees Git y su integración.",
  lsp: "Inicio y comunicación con servidores de lenguaje para navegación y diagnósticos.",
  symbols: "Consulta y manipulación de símbolos del código.",
  patch: "Aplicación de cambios mediante parches.",
  search: "Búsqueda de contenido de código y selección de resultados.",
  instructions: "Instrucciones específicas del arnés de programación.",
  environment:
    "Inspección del entorno de programación y sus herramientas disponibles.",
  cdp: "Comunicación con Chromium mediante Chrome DevTools Protocol.",
  actions:
    "Acciones del navegador sobre páginas y gestión de datos de credenciales.",
  frames: "Observación y control de frames del navegador.",
  downloads: "Seguimiento y recuperación de descargas.",
  vault: "Persistencia protegida de credenciales del navegador.",
  whatsapp:
    "Gestión de la sesión de WhatsApp y su disponibilidad para el navegador.",
  observe:
    "Extrae información observable de la página para orientar al agente.",
  dom: "Utilidades de inspección del DOM para el control del navegador.",
  prepare_input: "Prepara interacciones con campos de entrada del navegador.",
  condition: "Evalúa condiciones observables en la página.",
  target: "Resuelve los objetivos de interacción de una página.",
  documents:
    "Operaciones de documentos y conexión con su almacenamiento o API.",
  extract:
    "Extracción local de contenido para preparar documentos consultables.",
  embeddings:
    "Generación y almacenamiento de representaciones vectoriales de fragmentos documentales.",
  library: "Registro, indexación y recuperación de documentos y fragmentos.",
  schedules:
    "Cron, recordatorios y ejecución según horario; la zona inicial es America/Lima.",
  execute: "Ejecución de nodos y pasos del workflow.",
  model: "Tipos y reglas de representación del workflow.",
  connectors: "Operaciones de conexiones y autorizaciones de servicios.",
  integrations: "Rutas y configuración de las integraciones disponibles.",
  designs: "Gestión de diseños, versiones y artefactos visuales.",
  diagram:
    "Representación y modificación de diagramas generados por el agente.",
  design_choice: "Recepción y registro de la selección visual del usuario.",
  permissions: "Control de acciones que necesitan una respuesta del usuario.",
  command_safety:
    "Detección determinista de señales de riesgo en comandos locales.",
  content_safety:
    "Tratamiento de contenido no confiable y riesgos de instrucciones externas.",
  redaction: "Redacción de secretos en información que puede salir del motor.",
  effects: "Coordinación de efectos y cambios sobre archivos.",
  security: "Políticas de seguridad del runtime.",
  config:
    "Configuración del servicio y valores que condicionan su comportamiento.",
  model_roles: "Separación y configuración de los roles rápido y ejecutor.",
  usage_tracking: "Registro de consumo de modelos por actividad.",
  usage: "Persistencia y consulta de métricas de consumo.",
  events: "Eventos durables y comunicación de cambios.",
  progress: "Estados observables del trabajo y notificaciones de avance.",
  restart_recovery:
    "Tratamiento de trabajo pendiente tras reiniciar el runtime.",
  execution_state:
    "Estado operativo de una ejecución y su relación con el cierre.",
  evaluation: "Adaptador nativo para ejecutar y medir evaluaciones con trazas.",
  interaction: "Protocolo interactivo de mensajes y control del benchmark.",
  tool_output_search: "Búsqueda sobre resultados de herramientas conservados.",
  assets: "Entrega de recursos estáticos y configuración del servidor.",
};
export function guideFor(file: { path: string; test: boolean }) {
  if (file.test)
    return "Prueba o fixture de verificación. Sirve para estudiar los comportamientos esperados y sus casos de error; no representa una comprobación de producción.";
  const stem = file.path
    .split("/")
    .at(-1)!
    .replace(/\.[^.]+$/, "");
  if (stem === "lifecycle" && file.path.includes("fenix-browser"))
    return "Gestiona propiedad, espera, reconexión y cierre de sesiones locales del navegador; no restaura checkpoints remotos como perfiles locales.";
  return (
    fileGuides[stem] ??
    `Pieza de ${areaFor(file).title.toLocaleLowerCase("es")}. Consulta sus símbolos y el archivo fuente para seguir su responsabilidad dentro del proceso.`
  );
}
