# Navegador local


[English — full technical reference](../en/browser.md)

Fénix controla Chromium instalado en esta máquina mediante un controlador CDP propio
en Rust. No necesita proveedores de navegadores, Playwright ni Selenium en producción.
Chromium proporciona HTML, CSS y JavaScript: no se ha creado un motor de renderizado
nuevo. Su sandbox permanece habilitado; actualízalo con el sistema operativo.

## Uso y sesiones

En **[Navegador](/browser)**, crea un perfil como `personal`, introduce una URL y pulsa Abrir.
Puedes hacer clic en la pantalla, escribir, pegar y desplazarte. Completa allí las
contraseñas, QR, MFA o CAPTCHA cuando sea necesario. El agente también puede escribir las contraseñas que le proporciones y enviar el login autorizado sin intervención manual. Activa **Permitir acceso al agente** cuando quieras
que Fénix use ese perfil. **Tomar control** impide nuevas acciones del agente; una
acción que ya se ejecutaba puede finalizar. La interacción manual también lo desactiva.

Cerrar el perfil libera sus procesos y conserva cookies, localStorage, IndexedDB y
otros datos bajo `<data-dir>/browser/profiles/<nombre>`. Reabrirlo reutiliza esos
datos. Cada sitio puede caducar o revocar el acceso; las cookies solo de sesión
pueden perderse al cerrar. No se garantiza una sesión eterna. No se ha iniciado sesión
ni comprobado ninguna cuenta real de WhatsApp o de una universidad.

El panel de contraseñas guarda un almacén cifrado con AES-256-GCM y Argon2id. Necesita
una clave maestra de al menos 12 caracteres para guardar, completar o eliminar el
acceso del perfil y sitio actuales. Completar no envía el formulario. Las contraseñas
no pasan al chat ni al registro de herramientas; la clave maestra no se guarda.
El gestor nativo de contraseñas de Chromium queda deshabilitado. El cifrado del almacén
**no cifra todo el perfil de Chromium**; los directorios tienen permisos 0700 y el
almacén 0600. El puerto de depuración local no tiene autenticación: otros procesos de esta máquina y los administradores del sistema quedan fuera de esa protección.

El agente puede crear un perfil personal nuevo al abrir el navegador con un
nombre explícito para una tarea recurrente autorizada. La sesión se conserva al
cerrar el navegador. No se reactiva un perfil existente cuyo control haya retomado
el usuario. Las observaciones omiten los valores de los campos. También se admiten
clic derecho, clic central y controles de rango. Se respetan las restricciones
reales de la página y las verificaciones externas que requieran al usuario.

## Agente y pruebas

El skill `browser-navigation` guía al agente para descubrir nueve herramientas reales:
abrir, observar, actuar, redimensionar, inspeccionar, esperar, manejar pestañas,
comprobar y cerrar. Las acciones incluyen clic, escritura, teclas, scroll, selección,
casillas, arrastre, diálogos y carga de archivos autorizados del espacio de trabajo.
La observación devuelve texto y geometría, con referencias al DOM y soporte de shadow
roots abiertos. Los frames inaccesibles requieren intervención o una nueva observación.

Sin perfil explícito, cada tarea usa uno temporal separado, eliminado al cerrar el
perfil. Las acciones individuales se serializan; los flujos simultáneos de varios
pasos deben usar perfiles separados para no compartir la página activa. Prueba tamaños como 390×844, 768×1024 y 1280×800, verifica estados visibles y
el desbordamiento horizontal, inspecciona fallos y guarda capturas. El tamaño se aplica
a las pestañas del perfil. La comparación de capturas es exacta y necesita una imagen
de referencia solicitada; contenido dinámico o animaciones pueden producir diferencias.
El modelo recibe texto y geometría, no las capturas como visión. La emulación no prueba
otros motores ni dispositivos físicos. No todos los sitios, controles de canvas,
frames complejos o sistemas antibots pueden automatizarse sin ayuda.

## Memoria

Configura `<data-dir>/browser.json` y reinicia Fénix. Por defecto: Chromium en
`/usr/bin/chromium`, dos perfiles abiertos, cuatro pestañas por perfil,
600 segundos para considerar una sesión del usuario inactiva y 60 segundos para
considerar la suspensión cuando todas sus tareas esperan respuesta. El navegador
de una tarea terminada se cierra si nadie más lo necesita. Una cola global considera
RAM y límites por cuenta. Sin soporte de suspensión se conserva la página activa.
Consulta el [ciclo de vida](browser-lifecycle.md) para configuración, garantías y
validación realizada del checkpoint real. Mirar la pantalla cuenta como actividad.
No se consume RAM de Chromium hasta abrirlo.

Consulta los detalles y comandos de prueba en la documentación en inglés. Las pruebas
reales usan un sitio local, sin cuentas personales ni proveedores externos.


## Verificación local (2026-09-15)

Pasaron 211 pruebas del workspace y 12 de la interfaz. La prueba optativa con
Chromium 152 real y sandbox habilitado también pasó: formularios, iframe del mismo
origen, tres tamaños, capturas, almacén de contraseñas y persistencia tras cerrar y
reabrir el proceso. La página de reapertura no vuelve a crear los datos de sesión.
En una observación con dos pestañas se sumaron 1165 MiB de RSS; no es el pico ni la
memoria exclusiva, ni una medición de WhatsApp o de una universidad. No se usaron
cuentas reales.

## Búsqueda y navegación

La barra de `/browser` abre URLs HTTP(S). Pide la investigación en la conversación:
el agente usa `web_search` / `web_read` y puede abrir un resultado con el navegador
cuando necesita interactuar. La API local sigue siendo `POST /api/web/search`, sin
proveedor externo. Consulta los [límites de cobertura](web-search.md).

Las descargas tienen ID, estado y bytes recibidos independientes del historial de
eventos. Una espera vencida puede indicar que siguen pendientes: espera por el
mismo ID sin repetir el clic. Una descarga cancelada devuelve estado terminal.
La finalización se comprueba con el archivo local, incluso tras reiniciar Chromium.
El registro conserva como máximo 128 transferencias. Consulta la
[revisión](browser-automation-review.md) y la [auditoría](browser-workflow-audit.md).

## Pruebas de regresión ampliadas

La [auditoría del navegador](browser-regression-audit.md) documenta la matriz de
casos difíciles, el formulario React, las pruebas responsivas y cómo reproducirlas.
Antes de ejecutar todas las pruebas ignoradas, compila el formulario con
`node apps/web/tests/fixtures/build-browser-controlled.mjs`. La prueba del formulario
público de Selenium requiere internet.
