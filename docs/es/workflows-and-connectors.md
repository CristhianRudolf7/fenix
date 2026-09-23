# Flujos programados y conexiones nativas

[English](../en/workflows-and-connectors.md)

**Programadas** reúne recordatorios, prompts programados y flujos completos. Describe el procedimiento en el chat: Fénix pide los datos que falten, guarda los pasos y abre el diagrama en el panel de trabajo. Puedes hacer zoom, inspeccionar cada paso, consultar ejecuciones y pedir cambios por texto. En la vista completa se colapsa el menú; en pantallas pequeñas el detalle aparece debajo del diagrama.

Cada ejecución tiene ID, proyecto, carpeta y versión propios. Guarda resultados y estado de cada paso en SQLite. El motor funciona en segundo plano: cerrar la web no lo cancela y el chat continúa disponible. Admite herramientas, scripts, navegador, conectores, instrucciones para el agente, condiciones, preguntas y avisos. Los pasos de una ejecución siguen el orden de dependencias; puede haber varias ejecuciones independientes en paralelo.

Las descargas pueden usar una ruta automática por ejecución o una ruta como `automations/{{run_id}}/informe.pdf`, para que las siguientes ejecuciones no sobrescriban archivos anteriores.

Las conexiones entre datos usan referencias como `{"$ref":"buscar/files/0/id"}`. No se inventan archivos, carpetas, actividades universitarias ni sesiones: el agente debe consultarlos o preguntarlos. El navegador reutiliza sus herramientas DOM y comprueba el resultado; una subida sin comprobante no demuestra que se entregó el trabajo.

**Probar sin efectos** ejecuta pasos reconocidos como de lectura y se detiene antes de una acción con efectos o desconocida. No simula una entrega correcta ni valida las acciones que no ejecutó. Las escrituras no se reintentan automáticamente; una ejecución interrumpida conserva sus pasos completados y requiere una decisión textual antes de repetir una acción incierta. No se promete entrega exactamente una vez en servicios externos. Las ediciones generan versiones nuevas y no cambian las ejecuciones ya iniciadas.

La programación conserva **America/Lima**, fecha única, intervalo y cron. No recupera automáticamente horarios perdidos mientras Fénix estaba apagado. Evita solapar una misma automatización. Pausar una programación afecta sus próximas ejecuciones; cancelar una ejecución detiene el trabajo actual.

## Integraciones disponibles

- **Google Workspace:** OAuth con renovación; Drive, Docs, Sheets, Slides, Gmail y Calendar mediante sus API autorizadas.
- **Microsoft 365:** OAuth con renovación; OneDrive/SharePoint, Outlook, calendario y Excel mediante Microsoft Graph.
- **Slack:** OAuth o token; canales, historial y mensajes.
- **GitHub:** token personal; repositorios, incidencias y API REST.
- **Correo:** IMAP con TLS en 993; SMTP con TLS en 465 o STARTTLS obligatorio en 587. Gmail y Outlook también están disponibles mediante sus conectores OAuth.
- **PostgreSQL y SQLite:** consultas parametrizadas; solo lectura por defecto. SQLite utiliza bases existentes dentro del espacio del usuario.
- **Telegram y WhatsApp Web:** conservan la configuración y el navegador propios de cada cuenta.
- **Búsqueda web, PDF y ofimática:** reutilizan las herramientas de Fénix. Los documentos locales no requieren una suscripción a Microsoft.

Las credenciales se introducen en **Integraciones**, nunca en el chat o en un flujo. Cada usuario mantiene su propia bóveda cifrada y sus propias conexiones. Los archivos descargados quedan en su espacio, conservan hash y no sobrescriben destinos existentes. Las descargas admiten hasta 25 MiB; las respuestas JSON, hasta 3 MiB. La bóveda utiliza una clave local en el directorio privado del servicio: no protege frente a un proceso o administrador del host comprometido.

## Configuración y alcance de las pruebas

El regreso desde el proveedor abre una página estática pública que no accede a ninguna cuenta. Esa página borra el código de la URL y completa la conexión con un POST desde el mismo origen, autenticado por la sesión de Fénix. Se conservan la cookie `SameSite=Strict`, la caducidad existente y la validación de estado OAuth de un solo uso. La autorización de la cuenta nunca se intercambia mediante un GET público.

El administrador debe registrar las aplicaciones OAuth en los proveedores y autorizar la URL `https://tu-host/api/connectors/oauth/callback`. Puede configurar las variables `FENIX_GOOGLE_CLIENT_ID`, `FENIX_GOOGLE_CLIENT_SECRET` y equivalentes para `MICROSOFT` y `SLACK`. También se pueden introducir credenciales de una aplicación propia en el diálogo. El proveedor puede requerir habilitar API, verificar la aplicación o recibir consentimiento del administrador de la organización.

Las pruebas locales comprueban contratos de API con servidores simulados, separación de usuarios, cifrado, referencias, condiciones, versiones, recuperación, consultas SQLite e interfaz de escritorio/móvil. No sustituyen el consentimiento OAuth real, el envío real de correo ni una entrega en una universidad. Consultar una conexión no envía mensajes de prueba.

Consulta las [mejoras nativas de Google Workspace](google-workspace.md): selección de servicios, comprobaciones, esquemas, paginación, borradores y subidas.
