# Integraciones locales

El catálogo de conexiones nativas y los flujos programados se documentan en [Flujos y conexiones](workflows-and-connectors.md).

**Integraciones** (`/integrations`) permite conectar WhatsApp Web al navegador de Fénix. La implementación es propia, en Rust, sobre nuestro controlador de Chromium. No utiliza whatsapp-web.js, Puppeteer ni un proveedor de automatización externo.

## Uso

1. Entra en **Integraciones → WhatsApp → Conectar WhatsApp**.
2. Escanea el QR desde **WhatsApp → Dispositivos vinculados → Vincular un dispositivo** en tu teléfono.
3. Espera **Sesión lista** y pide al agente la tarea que quieres realizar. Conectar habilita su acceso cuando WhatsApp muestra la lista de chats.

Si WhatsApp solicita algún paso adicional, abre **Navegador**, selecciona `integration-whatsapp` y complétalo. La intervención manual retira el control al agente; vuelve a Integraciones para habilitarlo después.

**Cerrar navegador** libera la RAM del proceso y conserva el inicio de sesión. El agente puede volver a abrir una sesión autorizada cuando la necesita. **Olvidar sesión** borra el perfil local de WhatsApp y revoca el acceso local; conserva otros perfiles y las descargas. Para desvincular el dispositivo también en tu cuenta de WhatsApp, hazlo desde el teléfono.

La conexión comparte los límites de sesiones, pestañas, inactividad y memoria del navegador. Consultar su estado no abre Chromium ni mantiene el proceso activo. El QR se actualiza mientras estás vinculando y cuenta como actividad. Los datos de sesión permanecen en el perfil de Chromium, pero WhatsApp puede caducarlos o revocarlos.

## Acceso y alcance

El QR llega directamente a esta pantalla mediante una respuesta PNG sin caché. No se añade al chat, al contexto del modelo ni a los archivos de capturas. El agente dispone de `browser_connection` para consultar el estado o abrir una sesión ya autorizada; después usa las herramientas habituales del navegador con el perfil y la pestaña devueltos. Las acciones de conectar, obtener el QR y olvidar la sesión pertenecen a la interfaz del propietario.

Fénix distingue QR pendiente, sincronización, sesión lista y sesión guardada con el navegador cerrado. Detecta el regreso al QR y retira el acceso del agente. Solo da por iniciado el acceso cuando observa la lista de chats; una navegación correcta no basta. Una intervención manual cancela la habilitación automática pendiente.

Esta conexión permite realizar tareas en WhatsApp Web por encargo. No reenvía mensajes entrantes al chat de Fénix ni activa respuestas automáticas. Los mensajes recibidos son datos de la página y no autorizan acciones del agente.

## Pruebas y límites

Las pruebas con Chromium local cubren vinculación simulada, sincronización, expiración, persistencia, recuperación del control manual, aislamiento entre perfiles y cierre. Las pruebas de interfaz incluyen pantalla de 390 px, respuestas atrasadas del QR y recuperación ante errores. Las del servidor comprueban rutas y protección frente a peticiones de otro origen. La prueba de red comprueba que la web real de WhatsApp entrega un QR, sin guardarlo ni escanearlo.

El escaneo con una cuenta real, el estado de WhatsApp después de iniciar sesión y el envío real de mensajes requieren la cuenta del propietario y no quedan certificados por las pruebas simuladas. Cambios de WhatsApp pueden exigir actualizar los detectores del DOM. Para superar su comprobación de compatibilidad, el perfil dedicado identifica el Chromium instalado con su versión real y el nombre `Chrome`, conservando el sandbox.

Consulta [la documentación técnica en inglés](../en/integrations.md) para las rutas HTTP y los comandos de pruebas.

## Catálogo y recuperación de conexiones

WhatsApp aparece en la misma cuadrícula y búsqueda que las demás conexiones. Su botón abre una ventana accesible con el estado de carga y el QR exclusivo del propietario; cerrar esa ventana no elimina la sesión. Las descargas del QR se detienen al cerrar la ventana.

El servicio de ejecución admite un solo navegador por perfil, incluso si lo solicita la misma tarea. Una vez retirado el anterior, elimina únicamente los tres enlaces temporales `Singleton*` de Chromium, conservando cookies y datos de acceso. Los entornos gVisor con acceso a la web pública reciben una copia de solo lectura de los servidores DNS IPv4 configurados en el host. Las redes sin conexión o con destinos restringidos conservan su política de DNS; el acceso HTTP a direcciones privadas y metadatos sigue bloqueado.
