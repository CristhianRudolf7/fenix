# Índice de estudio de Fénix

Este índice conserva el orden acordado para estudiar el funcionamiento del agente. La numeración es estable: al pedir «el punto 1» se refiere al primer título de esta lista. El navegador se estudió por separado y se omite aquí. Cada explicación debe contrastarse con el código actual y distinguir implementación, configuración opcional y validaciones pendientes.

1. **Arranque, configuración y ciclo de vida del agente.** — Ya leído.
2. **Inicio de sesión, perfil y sesiones del propietario.**
3. **Proveedores de modelos y roles rápido e inteligente.**
4. **Conversación continua e interlocutor del usuario.**
5. **Recepción, encolado y ejecución de mensajes.**
6. **Ciclo de razonamiento y ejecución del agente maestro.**
7. **Herramientas: descubrimiento, selección y ejecución.**
8. **Skills: catálogo, carga bajo demanda y recursos.**
9. **Creación, edición, traducción y aprendizaje de skills.**
10. **Gestión de proyectos y separación de contextos.**
11. **Construcción del contexto y selección de información.**
12. **Compactación del contexto y continuidad del trabajo.**
13. **Memoria automática, hechos y preferencias del usuario.**
14. **Recuperación de memoria, relaciones y mantenimiento.**
15. **Planificación de tareas largas e hitos.**
16. **Requisitos, especificaciones y criterios de finalización.**
17. **Subagentes y delegación de tareas en paralelo.**
18. **Revisión de planes y auditoría de resultados.**
19. **Autonomía, dudas, decisiones y aprobaciones textuales.**
20. **Eventos, comunicación interna y notificaciones de progreso.**
21. **Control de tareas: modificar, pausar, cancelar y replanificar.**
22. **Procesos interrumpidos y recuperación después de reinicios.**
23. **Ejecución de comandos, terminales y procesos persistentes.**
24. **Lectura, búsqueda, creación y edición de archivos.**
25. **Arnés de programación: búsqueda de código, símbolos, LSP y parches.**
26. **Git, worktrees y modificaciones simultáneas.**
27. **Historial de archivos y restauración de cambios.**
28. **Resultados de herramientas: conservación, reducción y búsqueda.**
29. **Búsqueda web propia y lectura ligera de páginas.**
30. **Biblioteca documental: importación, extracción y OCR.**
31. **Fragmentación, embeddings y consultas sobre documentos.**
32. **Creación y edición de documentos, hojas de cálculo y presentaciones.**
33. **Diagramas, diseños, plantillas y artefactos visuales.**
34. **Exploración de archivos y visualización de resultados.**
35. **Recordatorios, cron y tareas programadas.**
36. **Workflows: automatizaciones con múltiples pasos.**
37. **Integraciones y autorización de servicios externos.**
38. **Telegram y entrega de mensajes por canal.**
39. **Seguridad: secretos, credenciales, contenido no confiable y acciones de riesgo.**
40. **Almacenamiento interno, base de datos y registros durables.**
41. **Seguimiento de tokens, consumo y métricas de ejecución.**
42. **Evaluaciones, benchmarks y auditoría del arnés.**
43. **Compatibilidad entre sistemas operativos y capacidades disponibles.**
44. **Instalación, actualizaciones e inicio automático.**

## Código inicial para el punto 1

- `apps/server/src/main.rs`: CLI, servidor HTTP, arranque y cierre.
- `crates/fenix-agent/src/lib.rs`: apertura de Runtime, servicios y trabajadores.
- `crates/fenix-agent/src/config.rs`: configuración del ejecutor.
- `crates/fenix-files/src/layout.rs`: directorios por plataforma.
- `crates/fenix-process/src/shutdown.rs`: señales y petición de cierre.

El atlas interactivo complementa este índice, pero no reemplaza su numeración.

## Progreso de lectura

- Navegador: ya estudiado por separado.
- Punto 1: leído, incluidas las consultas sobre configuración, almacenamiento y ciclo de vida.
- Siguiente: **2. Inicio de sesión, perfil y sesiones del propietario.**
