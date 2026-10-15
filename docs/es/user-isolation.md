> Documento histórico: el gateway multiusuario y el registro/acceso con Google se retiraron. Para la versión personal actual, consulta [acceso remoto](remote-access.md).

# Cuentas y entornos aislados por usuario

> Documento histórico del despliegue compartido anterior a 0.3.2. El sandbox y sus scripts se retiraron de la distribución personal. Véase [ejecución local actual](local-execution.md); las políticas aquí descritas no están activas en 0.3.2.


El gateway compartido autentica al usuario y selecciona su controlador desde un registro local protegido. El cliente no puede elegir otra cuenta mediante una ruta, cabecera, herramienta o respuesta del modelo. Cada cuenta conserva sus propias conversaciones, base de datos, proyectos, skills, Telegram y tareas programadas.

La implementación multiusuario separa el controlador de los comandos, terminales, servidores de lenguaje y Chromium. Estos procesos se ejecutan en contenedores con montajes seleccionados y límites. La [guía de aislamiento, instalación y migración](execution-isolation.md) describe la arquitectura actual y sus verificaciones. Actualizar el repositorio no migra las cuentas que ya están ejecutándose.

## Administración de cuentas

Las cuentas con contraseña se aprovisionan localmente. El [acceso y registro con Google](google-sign-in.md) permite asignar un entorno nuevo y preparado después de verificar la identidad. No hay registro público mediante contraseña. La administración de entornos sigue siendo local:

```sh
# La contraseña se lee desde stdin, nunca desde argumentos.
fenix accounts create REGISTRO DIRECTORIO_DE_PREPARACION CORREO http://127.0.0.1:4180
fenix accounts disable REGISTRO CORREO
fenix accounts enable REGISTRO CORREO
fenix accounts passwd REGISTRO CORREO
```

Los comandos se ejecutan como el propietario confiable del registro. Crear la cuenta prepara un directorio, pero no crea el volumen con cuota ni inicia el controlador. El operador del host mueve ese directorio al árbol de cuentas y provisiona su sistema de archivos siguiendo la guía. El gateway no necesita permiso de escritura sobre el padre protegido de los volúmenes.

Cada cuenta necesita UUID, token y puerto local exclusivos. El registro rechaza duplicados y se consulta durante la autenticación y las solicitudes; las escrituras administrativas utilizan bloqueo y reemplazo atómico. Deshabilitar, habilitar o cambiar la contraseña incrementa la revisión de autorización. El controlador detenido debe reprovisionarse con la revisión actual antes de arrancar.

Las carpetas incluyen `projects`, `workspace`, `memory`, `skills`, `artifacts`, `sessions` y `state`. La base de datos real permanece en `state/fenix.sqlite3`; los otros directorios no la duplican. Conservar los UUID y respaldar SQLite de forma consistente durante la migración. Detener la instancia anterior y su polling de Telegram antes de activar la nueva.

## Sesiones y revocación

Las contraseñas utilizan Argon2id con sal individual. El registro y los tokens internos permanecen fuera de los montajes de las cuentas. Las cookies son HttpOnly, SameSite=Strict y Secure con HTTPS. Configurar el origen público en el gateway.

Las sesiones duran exactamente 168 horas (604 800 segundos) desde el login; la actividad no renueva el plazo. Se guardan en `accounts.sessions.json`, junto al registro, con permisos exclusivos y reemplazo atómico. Reiniciar el navegador o gateway conserva el vencimiento; cerrar sesión la revoca.

Cada sesión queda vinculada al UUID estable, hash de contraseña y revisión de autorización. Reordenar el registro no cambia el propietario. Eliminar o deshabilitar una cuenta, cambiar su contraseña o avanzar la revisión invalida sesiones anteriores. Volver a habilitarla no las revive. Los streams comprueban la autorización incluso durante periodos SSE silenciosos, con revisión cada segundo.

El login limita intentos y trabajo de hashing. Se comprueba el origen de login/logout. Cada controlador requiere su token y publica únicamente un puerto loopback del host. El gateway limita concurrencia, cuerpos y tiempos de recepción de cabeceras sin imponer ese límite corto a los streams de respuesta.

## Despliegue

Utilizar `fenix-execution.service` y la unidad de sistema `fenix-controller@UUID.service` de la [guía de migración](../en/execution-isolation.md#installation-and-migration). Antes del arranque se verifican identidad, autorización actual, imagen, volumen con cuota, montajes, límites, cortafuegos y servicio de ejecución. Se mantiene desactivado el reinicio independiente de Docker. El gateway no necesita root ni acceso a Docker.

Las credenciales y bases de datos permanecen en el controlador. Los procesos de tareas reciben solo montajes seleccionados y variables mínimas. El servicio del host y el operador forman parte de la base de confianza; los contenedores probados comparten el kernel. El modo local individual no ofrece esta separación multiusuario. No se habilitan conversaciones compartidas, puertos públicos arbitrarios ni permisos entre cuentas.
