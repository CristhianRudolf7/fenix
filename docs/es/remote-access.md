# Acceso remoto personal

El servidor del agente incorpora directamente el login del único propietario. El gateway multiusuario y los comandos `fenix gateway`/`fenix accounts` se han retirado en la versión en desarrollo; no se necesita un proxy de cuentas para mensajes ni eventos.

Con el agente detenido, configura el propietario mediante `fenix owner CORREO --password-stdin [--data-dir RUTA]`, proporcionando la contraseña por la entrada estándar. Arranca después `fenix` con el mismo directorio de datos. No hay registro público de usuarios. Reconfigurar el propietario invalida las sesiones anteriores.

Cada sesión dura siete días exactos desde el login. La actividad no renueva ese plazo y las sesiones sobreviven a reinicios. Cerrar sesión revoca esa sesión; cambiar la contraseña revoca todas. Los eventos SSE se cierran al caducar o revocarse la sesión, pero las tareas del agente continúan.

Para acceso remoto, configura `FENIX_PUBLIC_ORIGIN` con el origen HTTPS y dirige Caddy o el túnel al servidor personal. Las cookies utilizan HttpOnly, SameSite=Strict y Secure en la configuración HTTPS. El frontend puede seguir siendo un proceso separado. Esta adaptación no se ha desplegado y no anuncia validación real del firewall de producción.
