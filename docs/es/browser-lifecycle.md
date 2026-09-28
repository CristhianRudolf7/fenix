# Memoria del navegador y esperas persistentes

Fénix prioriza `web_read` para páginas públicas cuyo texto sea suficiente. Conserva
Chromium para JavaScript, sesiones autenticadas, formularios, capturas e
interacciones. No añade servicios externos ni sustituye silenciosamente acciones
del navegador.

## Cierre según las tareas

Cada perfil registra sus tareas. Al completar o cancelar la última, se cierra el
navegador y su ejecución. El perfil personal permanece en disco; los temporales se
eliminan tras confirmar la limpieza. Otra tarea activa, una sesión controlada por
el usuario, una descarga pendiente o un perfil `integration-*` impiden el cierre
automático. WhatsApp conserva su conexión habitual.

Cuando todas las tareas esperan al usuario, puede suspenderse tras 60 segundos.
Las sesiones del usuario sin tareas activas utilizan 600 segundos de inactividad.
El mantenimiento comprueba cada cinco segundos. Una página de una tarea activa no
se elimina simplemente por estar inactiva.

## Guardado y recuperación

Con gVisor y Docker compatibles se guarda el estado del proceso y se detiene el
sandbox. Esto busca conservar formularios sin enviar y estado JavaScript, además
de cookies y almacenamiento. Guardar únicamente el perfil no garantiza conservar
el formulario. El navegador detenido no mantiene procesos residentes; el motor,
el sistema operativo y pequeños metadatos siguen utilizando memoria.

Se espera a que no haya actividad de red pendiente, descargas, diálogos ni comandos
en curso. Se desconecta el control del navegador y se guarda el proceso. La copia
se cifra y autentica por bloques, sin cargarla entera en RAM. Los metadatos guardan
pestaña, tamaño y tareas, sin añadir valores de formularios a la conversación.

Recuperar comprueba usuario, imagen, runtime y archivos del perfil; entra en la cola
global y usa una credencial de conexión nueva. El perfil permanece exclusivo de
su ejecución durante la suspensión, pero libera su reserva de RAM y cómputo.
Antes de actuar, el agente debe observar nuevamente la página. No se repiten
acciones anteriores. Las copias válidas pueden sobrevivir al reinicio del servicio
y se consumen al restaurar.

Los sitios externos pueden caducar sesiones o formularios durante la suspensión.
Conexiones abiertas o runtimes incompatibles también pueden impedirla. Sin soporte
o sin espacio suficiente se conserva el navegador activo. Si falla el cifrado
después de detenerlo, se retiene la copia nativa en el directorio privado de Docker;
ese caso se audita, no garantiza recuperación tras reiniciar el servicio y no se
anuncia como guardado cifrado exitoso.

## Cola global

El servicio combina memoria disponible real (`MemAvailable`), reserva para el host,
concurrencia y límites por cuenta. Las solicitudes esperan sin iniciar Chromium,
sin reservar memoria del nuevo proceso y sin llamar al modelo. Los usuarios se
turnan. Cancelar retira la solicitud; abandonar una recuperación todavía no
iniciada conserva la copia suspendida. Esperar admisión no bloquea otros navegadores.

La cola es acotada y está en memoria: tras reiniciar el servicio, el cliente debe
solicitar admisión nuevamente. No sustituye la cola durable de tareas. Las copias
tienen límites por archivo, usuario, total y vencimiento. El aislamiento y los
límites de los contenedores siguen vigentes.

## Configuración y validación

`<data-dir>/browser.json` admite `hibernate_after_seconds` (60 por defecto, entre
10 y 86400). `idle_seconds` mantiene su valor inicial 600. `memory_budget_mib` sigue
siendo compatible, pero ya no causa cierres que descarten páginas activas. Un
navegador local sin ejecutor no puede hibernar.

Consulta la [tabla de configuración del host](../en/browser-lifecycle.md#configuration)
para `/etc/fenix/execution.json`. Por defecto una copia admite 2 GiB, cada cuenta
4 GiB, el total 8 GiB y el vencimiento es de 24 horas. Se requiere `checkpoint_key`
y espacio temporal adicional. La cola admite 128 solicitudes, 8 por cuenta,
durante 900 segundos como máximo, salvo configuración explícita.

**Requisito de despliegue:** Docker debe permitir checkpoints y usar el runtime
gVisor fijado. No se reinicia Docker ni se reduce el aislamiento automáticamente.
El 28 de septiembre de 2026 se habilitó el soporte y pasaron dos ciclos reales de
suspensión y recuperación con gVisor. El formulario sin enviar y el estado
JavaScript sobrevivieron, incluso reiniciando el servicio entre ambos pasos.
Se comprobó el apagado del navegador, el cifrado en disco, el rechazo de otro
usuario y la ausencia de envíos duplicados. Los ciclos completos tardaron 29,78
y 31,83 segundos en ese escenario sintético.

El controlador Rust desplegado también suspendió automáticamente un navegador de
prueba: la memoria contabilizada de su ejecución bajó de 302.628.864 bytes (unos
289 MiB) a cero, sin tareas residentes. La copia ocupó 153.781.569 bytes (unos
147 MiB). Esta medición no incluye el controlador, el servicio ni la caché del sistema.

La recuperación en producción conservó el texto del formulario y el estado
JavaScript sin enviarlo. Una ejecución en cola esperó mientras el navegador
ocupaba la capacidad y fue admitida al cerrarlo. Se eliminó el navegador temporal
y se restauró el tiempo de inactividad original. Las consultas autenticadas a la
web pública y los cinco servicios systemd funcionaron tras el reinicio final.
Consulta el [registro de validación](../research/browser-hibernation-validation-2026-09-28.json)
para las mediciones y la cobertura de las pruebas.

Las pruebas locales cubren cifrado, usuarios, reservas, cola, cancelaciones,
errores y reinicio del servicio con un backend simulado. La prueba de Chromium
cubre formularios cuando no se puede suspender, observación antes de actuar,
cierre según tareas y persistencia de cookies y almacenamiento. Las cuentas de
aceptación son temporales y la comprobación en producción utiliza un formulario
sintético. Se debe repetir al cambiar imagen, runtime o servidor. Los sitios reales
pueden caducar sus sesiones y requieren pruebas propias; una sola página no permite
afirmar una capacidad universal de usuarios simultáneos.

Los comandos de reproducción están en la [versión en inglés](../en/browser-lifecycle.md#deployment-and-validation-boundary).
