# Mejoras de Google Workspace en Fénix

Fénix amplía su conector propio: no instala Google Workspace CLI ni comparte credenciales de terminal. Mantiene OAuth, tokens cifrados por usuario y ejecución dentro del espacio autorizado. No añade una suscripción de integración; siguen aplicando disponibilidad, cuotas y licencias de Google.

## Qué cambia

- **Servicios elegibles:** Drive, Docs, Sheets, Slides, Gmail, Calendar, Tasks, Contacts, Forms, Chat y Meet. Las conexiones nuevas empiezan solo con Drive en lectura. Los permisos se construyen a partir de la selección; ampliar acceso requiere consentimiento de Google.
- **Comprobaciones sinceras:** autorización, lectura comprobada y comprobación parcial son estados diferentes. Un fallo en Gmail no queda oculto por una prueba exitosa de Drive. Docs, Sheets, Slides y Forms requieren leer un recurso concreto; la interfaz muestra esta limitación.
- **Esquemas oficiales:** el agente consulta métodos, parámetros y estructuras mediante `workspace_schema`. La consulta no transmite tokens ni concede permisos.
- **Previsualización:** `dry_run=true` muestra la solicitud sin llamar a Google. Valida su forma local, no todos los requisitos de la API ni los permisos del archivo.
- **Paginación limitada:** devuelve páginas, cobertura y continuación; se detiene por límite, error o token repetido. No repite escrituras.
- **Correo:** lectura, borradores, envío explícito y respuestas que conservan el hilo. Los destinatarios son explícitos; los adjuntos salen del espacio del usuario, con máximo de diez archivos y 5 MiB totales. Crear un borrador en Gmail también requiere escritura.
- **Archivos y agenda:** subida de archivos nuevos a Drive, de hasta 25 MiB, y búsqueda de eventos con calendario e intervalo explícitos.

Hay una skill nueva para flujos entre servicios y diez skills existentes actualizadas. El catálogo queda en 187. Las actualizaciones reconocen versiones originales y preservan personalizaciones y skills desactivadas.

## Límites

Meet permite consultar transcripciones existentes y accesibles; no graba ni transcribe reuniones nuevas. Las APIs administrativas y los observadores permanentes de eventos no se incluyen. Reducir los servicios seleccionados limita Fénix, pero no revoca automáticamente permisos antiguos en Google.

Las pruebas con servidores locales verifican contratos y límites; la consulta real de Discovery verifica documentación y scopes públicos. Ninguna de ellas demuestra envío real de correo o acceso a cuentas particulares. No se conectaron cuentas ni se activó un despliegue de producción.

Consulta el [detalle técnico en inglés](../en/google-workspace.md), el [registro de validación](../research/google-workspace-validation-2026-09-21.json) y el [repositorio de referencia](https://github.com/googleworkspace/cli).
