# Historial de cambios

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
