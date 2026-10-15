> La versión en desarrollo usa un único propietario y login en el propio servidor. Configúralo antes del primer arranque siguiendo [acceso personal](remote-access.md).

# Despliegue independiente del frontend y backend

En producción hay dos servicios de usuario **systemd**:

- `fenix-web.service`: Caddy sirve el frontend React/Vite en el puerto local 4173
  y reenvía `/api` al backend en 4174.
- `fenix.service`: el gateway Rust mantiene el login, las sesiones y el acceso
  separado a cada usuario. Los servicios aislados del agente siguen funcionando.

El dominio y el túnel conservan su configuración. No hay servidor de desarrollo
Vite ni servidor Node en producción. Caddy mantiene las cabeceras de origen y
las cookies; la validación de autenticación continúa en Rust.

Para actualizar solamente la interfaz:

```sh
pnpm --dir apps/web build
bash scripts/publish-frontend.sh
```

La publicación cambia atómicamente el directorio activo. No compila Rust ni
reinicia el agente. Conserva versiones anteriores para restauración y nombres de
assets anteriores para pestañas abiertas; revisar periódicamente su uso de disco.

Rust ya no incluye archivos web en su ejecutable ni los necesita para compilar.
Para desarrollo local puede servirlos desde `FENIX_WEB_DIST`; un valor vacío
inhabilita esta compatibilidad. Las actualizaciones del backend siguen requiriendo
compilación y reinicio de los servicios correspondientes.

Las plantillas, configuración del origen HTTPS, pasos de migración y pruebas
están en la [guía completa en inglés](../en/frontend-deployment.md).
