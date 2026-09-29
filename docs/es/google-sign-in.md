# Inicio de sesión y registro con Google

Fénix permite entrar y crear una cuenta con Google. El acceso solo solicita
identidad, correo y nombre; Drive y los demás servicios se autorizan por separado
en Conexiones. El acceso con contraseña sigue disponible.

## Activación

En Google Cloud, crea un cliente OAuth de aplicación web con origen
`https://fenix.crehlabs.com` y estas redirecciones:

- `https://fenix.crehlabs.com/api/auth/google/callback`: acceso y registro.
- `https://fenix.crehlabs.com/api/connectors/oauth/callback`: conexiones opcionales.

Configura `FENIX_GOOGLE_CLIENT_ID` y `FENIX_GOOGLE_CLIENT_SECRET` en el archivo
privado de entorno del servicio gateway, junto a `FENIX_PUBLIC_ORIGIN`, y reinicia
ese servicio. No publiques el secreto en el frontend ni en Git. Los contenedores
que gestionan las conexiones necesitan recibir sus credenciales por separado.

## Registro y aislamiento

Cada nueva cuenta recibe un entorno **nuevo, vacío y preparado previamente**.
El comando de administración `fenix accounts reserve REGISTRY STAGING_USERS_ROOT
slot-N@signup.invalid http://127.0.0.1:PORT` crea su identidad reservada; después
el operador mueve el directorio vacío al árbol protegido de usuarios y completa el aprovisionamiento habitual de disco, UID/GID, política de ejecución,
contenedor y servicio systemd. No requiere contraseña por stdin.

El registro comprueba que el entorno responda antes de asignarlo. Si no hay
capacidad preparada, muestra un mensaje para reintentar; nunca reutiliza datos
de otro usuario ni crea una cuenta incompleta. La reserva se reclama bajo bloqueo
para impedir asignaciones simultáneas. Todavía no se repone capacidad automáticamente.
Consulta el procedimiento completo en [la documentación en inglés](../en/google-sign-in.md).

Las cuentas existentes de Gmail o Google Workspace pueden vincularse conservando
sus datos. Para otros proveedores de correo, entra primero con contraseña y
vincula Google desde Perfil. La identidad se asocia al identificador estable de
Google; las cuentas nuevas usan Google y no tienen contraseña local utilizable.

Las sesiones duran siete días exactos. Las pruebas usan respuestas simuladas de
Google; queda pendiente probar el recorrido real cuando se configuren las
credenciales en Google Cloud. No se han desplegado estos cambios.
