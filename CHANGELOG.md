# Historial de cambios

## Cambios pendientes: integraciones

- Retirada la integración opcional con PostgreSQL del backend y de la interfaz; SQLite continúa como almacenamiento interno. Las conexiones PostgreSQL previamente guardadas ya no se muestran ni se ejecutan.

## 0.3.2 — En desarrollo

- Añade disponibilidad cacheada por conexión LLM, comprobación al iniciar el servidor cuando corresponde, reintentos espaciados y bloqueo de nuevos mensajes web con aviso recuperable. Conserva borradores y registra consumo de sondeos. Prepara carpetas documentales de hosts Android/iOS y puente directo Rust. [Alcance](docs/es/model-availability.md). Sin desplegar.

- Configura el navegador de escritorio para Windows/macOS, con detección de Chromium/Chrome/Edge y medición portable de procesos. Añade instaladores desde código con LaunchAgent y tarea Windows del usuario; adapta extracción Python y añade CI nativa. Instalación real en ambos sistemas y releases binarias pendientes. [Guía](docs/es/desktop-setup.md). Sin desplegar.

- Incorpora un atlas interactivo en la documentación: procesos del backend, recorridos, búsqueda de archivos y funciones, zoom y enlaces al código. El inventario se regenera al construir la web; las explicaciones funcionales requieren revisión al cambiar el comportamiento.

- Centraliza el cierre por plataforma y por solicitud del host móvil. Retira el cliente y protocolo de ejecución remota; comandos, LSP y navegador conservan ejecución nativa. Migra permisos del buscador, conexiones y perfiles al motor privado y añade transporte TCP de loopback para benchmarks. [Alcance y validación](docs/es/platform-core.md). Sin desplegar.

- Elimina el gateway multiusuario y autentica al único propietario en el servidor del agente. Conserva correo/contraseña, perfil, cambio de contraseña y logout; las sesiones duran siete días exactos, sobreviven a reinicios y revocan los streams sin detener tareas. Sin desplegar.

- Prepara la base para iOS con almacenamiento dentro del contenedor de la aplicación y rechazo explícito de shells/procesos externos. El catálogo móvil omite el navegador autónomo. Aplicación nativa, puente Rust y validación en dispositivos pendientes; sin desplegar.

- Detecta y verifica los intérpretes disponibles, permite elegir con `FENIX_SHELL` y registra el intérprete de cada proceso. Adapta argumentos e inspección de entorno a Bash, Zsh, sh, PowerShell y cmd. Validación nativa fuera de Linux pendiente; sin desplegar.

- Retira Google Drive del conector de Workspace y elimina el marketplace de skills, sus APIs y sus revisiones en segundo plano. Conserva el catálogo personal con consulta, edición y eliminación. Sin desplegar.

- Elimina las llamadas Live del frontend y del backend. Conserva el dictado al borrador y el interlocutor de chat/Telegram; migra la configuración y el consumo del modelo rápido sin perderlos. No desplegado todavía.

- Centraliza comandos, terminales y supervisión de procesos en un motor Rust con adaptaciones Linux/macOS/Windows. Mantiene salida progresiva, entrada, redimensión, cancelación, logs y servicios persistentes; incorpora detección de comandos de riesgo Windows. La portabilidad completa de archivos/cuentas e instaladores sigue pendiente. [Alcance y validación](docs/es/native-process-engine.md).

- Orienta la distribución a instalaciones personales: comandos y navegador locales, sin conexión automática al servicio de ejecución aislado. Rechaza variables antiguas de sandbox para exigir migración explícita.
- Retira instaladores, backends y pruebas del despliegue aislado multiusuario y el helper de comandos de solo lectura. Conserva las protecciones de vistas web y los worktrees.
- Añade detección determinista de comandos de riesgo y aprobación textual de sí/no, vinculada a una acción exacta, también en workflows y con política de permitir. No certifica la seguridad de código arbitrario.
- Adapta PDF/imágenes a extracción Python local; las dependencias y modelos OCR siguen siendo opcionales y se instalan por separado.
- Rediseña la portada documental con composición editorial y reutiliza el ave y su animación del agente.
- Publicación del binario 0.3.2 pendiente; el instalador público mantiene la release 0.3.1.

## Sin publicar

- Incorporar instalador público para Linux x86_64 (glibc ≥ 2.39), empaquetado de releases y proceso de publicación por GitHub Actions. La cuenta de GitHub tiene Actions bloqueado por facturación; la release inicial se compila y publica manualmente.

- Migrar la documentación a Astro, Starlight y TypeScript, conservando la portada y las rutas; sustituir la búsqueda propia por Pagefind e incorporar navegación y resaltado de código de Starlight.

- Añadir una web pública de documentación con guías, búsqueda local, navegación responsiva, temas y animaciones suaves; conservar el acceso móvil existente en su ruta. Destacar el enlace en los README.

- Mejora las instrucciones y skills de programación para reutilizar herramientas, agrupar preparación independiente, coordinar ediciones y diagnosticar fallos de preparación de pruebas. Actualiza únicamente skills originales sin personalizaciones. [Evaluación](docs/es/audits/luna-low-skill-efficiency.md).

- Frontend separado con Caddy/systemd; Rust ya no incluye los archivos web. Publicación de interfaz sin recompilar ni reiniciar el agente. [Despliegue](docs/es/frontend-deployment.md).

- Mantiene la navegación móvil en un panel lateral izquierdo y reduce la altura del cuadro de mensaje, conservando las rutas y la navegación por teclado.

- Desplegado el 22 de septiembre de 2026: mejora la espera de comandos cortos, modos de ejecución explícitos, referencias de procesos, diagnóstico del entorno, herramientas básicas y lecturas agrupadas respetando dependencias. [Detalles](docs/es/coding-harness-reliability.md).

- Activa skills aprendidos revisados sin aprobación, conserva revisiones recuperables, protege ediciones manuales, busca procedimientos relacionados y archiva los aprendidos sin uso que no estén protegidos.

- Añade contexto basado en consumo, programas declarativos de herramientas, recuperación desde puntos seguros, versiones/restauración de archivos y aprendizaje selectivo mediante candidatos revisados. Véanse [funcionamiento y límites](docs/es/harness-continuity.md).

- Añade recuperación relacional por ámbito, evidencia de coincidencias, vigencia por tipo, propuestas de contradicciones ligadas a revisiones, diagnóstico no destructivo y evaluación reproducible de recuperación.

- **Pendiente de comprobar en el despliegue:** verificar el firewall en la infraestructura real, incluidos destinos permitidos/bloqueados, restricciones de DNS y detención de ejecuciones al perder la protección. Las pruebas locales pasaron, pero esta comprobación real todavía no se ha realizado. Véanse los [límites de validación](docs/es/security-boundaries.md).

- Evaluación nativa en Rust, informes aislados y adaptador para Harbor.
- Reserva de contexto, salidas acotadas y recuperación sin repetir efectos.
- Supervisión de servicios, espera de comandos, detección de bucles según progreso y contratos con respuestas persistentes.

- Memoria adaptable por ámbito con procedencia, candidatos, vigencia, retirada e historial.
- Entidades, alias, esquemas de relaciones versionados, recorridos acotados y paquetes de contexto.
- Captura durable en segundo plano, revisión de solapamientos, búsqueda semántica opcional y controles web.

- Incorporar skills de programación seleccionados, recuperación por campos y diagnóstico útil de esquemas de herramientas.
- Permitir escrituras independientes junto a comandos de solo lectura protegidos por Linux y aclarar la duración de servicios supervisados.
- Retirar observaciones del contexto conservando checkpoints y procedencia de ejecución.
- Añadir recuperación específica según fallos y grupos reproducibles de cinco tareas de evaluación nativa.
- Conservar la respuesta más reciente de memoria cuando solicitudes concurrentes terminan fuera de orden.
- Liberar bloqueos de lectura antes de esperar configuración o inferencia en segundo plano, evitando inversión del orden de bloqueos.
- Ignorar cargas tardías después de navegar para que ajustes en segundo plano no sustituyan la página actual.
- Recuperar generaciones interrumpidas con un contador de fallos consecutivos, conservar notas incompletas como no verificadas y reforzar la transición de diagnóstico a acción.
- Conservar identificadores de citas y embeddings cuando una nueva extracción no cambia texto, ubicación y posición del fragmento.
- Actualizar los Markdown de la raíz en español, mantener `README.en.md` como versión inglesa y distinguir requisitos implementados, propuestas y evidencia histórica. Las instrucciones internas y fixtures conservan su idioma operativo.

## 0.3.0 — 2026-09-15

Esta versión continúa los subagentes en segundo plano y el contexto por proyecto
publicados en 0.2.0 el 14 de septiembre, e incorpora el trabajo posterior del 14–15.

- Planes asíncronos editables con hitos, dependencias, preguntas vinculadas,
  eventos durables y actualizaciones entregadas al canal original.
- Revisión adaptativa de planes, especialistas y resolución de objeciones con
  evidencia, usando el modelo configurado.
- Programación con contexto del repositorio: búsquedas regex/glob, parches de
  varios archivos, instrucciones `AGENTS.md` y servidores LSP configurables.
- Compactación durante tareas activas con checkpoints y contexto recuperable.
- Entrada de terminal interactiva, servicios persistentes, logs incrementales
  redactados, recuperación por páginas y límites de ejecución configurables.
- Mensajes de seguimiento a subagentes existentes, conservando contexto e IDs,
  un nivel de delegación y contadores de intentos acotados.
- Descubrimiento determinista de herramientas bajo demanda sobre el catálogo autorizado.
- Búsqueda web pública propia con rastreador en segundo plano, índice SQLite FTS5,
  lectura HTTP concurrente y enlaces a las fuentes.
- Navegador Chromium local controlado desde Rust, perfiles persistentes, control
  manual, bóveda de contraseñas cifrada, vistas responsivas y un skill de navegación.
- Documentación en español e inglés y versiones de paquetes unificadas en 0.3.0.

Estado histórico de la entrega del 15 de septiembre: el modelo recibe texto y geometría del DOM, no capturas de pantalla. La cobertura
de búsqueda depende del índice local. Voz, aprendizaje autónomo de memoria y
reanudación automática tras caídas se describían como trabajo futuro en aquella entrega. Consulta `REQUIREMENTS.md` para el contrato actual y sus límites.

## 0.3.1 — 10 de octubre de 2026

- Distribuir binario y frontend compilados mediante el instalador público en `https://fenix.crehlabs.com/install.sh`.
- Mantener datos y skills de la instalación personal separados de los servicios alojados; verificar SHA-256 y restaurar la instalación anterior si falla el arranque.
- Permitir configurar proveedor, modelo y clave en la bienvenida del modo personal. Mantener sin cambios la bienvenida del servicio alojado.
- Validar seis contratos con pruebas controladas. Validar la descarga pública, el arranque con systemd, la bienvenida en navegador y una reinstalación conservando datos. La prueba del modo alojado usa una respuesta de sesión controlada, no una sesión de producción. [Detalle](docs/es/public-installation.md).

### Acceso al sistema de archivos en instalación personal

El workspace es la carpeta predeterminada para trabajar y crear proyectos, no una frontera de acceso. Las herramientas del agente admiten rutas absolutas, rutas relativas externas y enlaces a archivos, con los permisos del usuario que ejecuta Fénix. La búsqueda puede partir de una carpeta externa y el análisis LSP puede abrir archivos externos. Los comandos de riesgo siguen requiriendo aprobación textual; se conservan la ocultación de secretos en resultados y las comprobaciones contra cambios inesperados al editar. Los límites de tamaño de las herramientas de texto no son permisos del sistema: para archivos grandes o binarios se utilizan las herramientas apropiadas o comandos. Este cambio no concede permisos de administrador ni publica archivos externos mediante el visor web.

### Motor de archivos multiplataforma

La lectura, creación, edición, borrado, resolución de rutas y listado se concentran en `fenix-files`, con adaptación Unix/Windows. El chat y el explorador ya no requieren `/proc` para listar carpetas. Se mantienen las comprobaciones de contenido, los permisos del sistema y la separación entre acceso personal y archivos privados. Véase [el alcance y la validación del motor](docs/es/native-file-engine.md); esta adaptación no equivale a portar toda la aplicación ni a validar instalaciones completas en Windows/macOS.

### Gestión de skills y almacenamiento multiplataforma

Se incorpora `fenix-skills`, acceso por handles, bloqueo mediante archivos y protección de datos mediante permisos Unix o DACL de Windows. Se conservan traducciones, recursos, personalizaciones, revisión antes de guardar, eliminaciones durables, memoria y persistencia SQLite. Las nuevas ubicaciones predeterminadas respetan rutas explícitas y `.fenix` local existente. [Alcance y validación](docs/es/skills-storage.md): la ejecución nativa en Windows/macOS y el soporte completo de la aplicación siguen pendientes.
