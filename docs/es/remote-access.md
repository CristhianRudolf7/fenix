# Acceso remoto con Cloudflare Tunnel

El frontend compilado y la API se sirven desde el mismo proceso Rust. `fenix.service` gestiona ambos; Vite solo se usa en desarrollo. Compila primero la interfaz y después el servidor, como indica el README.

Ejecuta el gateway de autenticación con `fenix gateway ~/.config/fenix/accounts.json`, usando `packaging/linux/fenix-gateway.service` como plantilla del servicio principal. Las cuentas se crean localmente, según [Aislamiento por usuario](user-isolation.md). El gateway sirve el login de correo y contraseña y exige una sesión para las API privadas. El túnel público debe apuntar al gateway, nunca a una instancia independiente ni al puerto de un usuario.

La variable `FENIX_PUBLIC_ORIGIN=https://tu-host.example.com` permite ese host y origen HTTPS exactos en el gateway. Se mantiene el acceso local y se rechazan otros hosts, orígenes incompatibles y solicitudes entre sitios. Esta variable selecciona el origen permitido; el gateway proporciona la autenticación.

Un túnel de Cloudflare con nombre conecta con el gateway en `http://127.0.0.1:4173` y conserva el Host público. La regla final debe devolver `http_status:404`. Configura un CNAME proxificado hacia `<tunnel-id>.cfargotunnel.com`. Cloudflare Access no es necesario: el visitante debe llegar directamente al login de Fénix.

Si el dominio ya tenía Access, comprueba primero que el gateway devuelve `401` al consultar una API privada sin sesión. Guarda una copia de la aplicación de Access, sus políticas y la configuración del túnel. Retira `originRequest.access` de la regla de ese dominio y elimina su aplicación de Access; comprueba también si existen requisitos heredados de la configuración global del túnel. Conserva los demás dominios y políticas. Dejar la validación obligatoria del JWT impediría entrar después de retirar Access. Una segunda autenticación con Access es opcional si se solicita expresamente; el dominio de producción de Fénix no la utiliza.

Guarda el token del túnel en `~/.config/fenix/cloudflared.token` con permisos `0600`. La plantilla `packaging/linux/fenix-tunnel.service` administra el túnel; el servicio principal puede leer `~/.config/fenix/server.env`. Conserva el directorio de datos existente para mantener historial, modelos y vinculación de Telegram.

Activa ambos con `systemctl --user enable --now fenix.service fenix-tunnel.service`. El usuario necesita lingering para arrancar sin iniciar sesión. Consulta el estado con `systemctl --user status fenix.service fenix-tunnel.service` y los registros con `journalctl --user -u fenix -u fenix-tunnel`.

Comprueba desde una sesión nueva del navegador que la raíz pública muestra el login propio sin redirigir a Cloudflare Access; que `/api/auth/session` y las API privadas devuelven `401` sin sesión; que funcionan el correo y la contraseña; y que cerrar sesión vuelve a bloquear las API privadas. Verifica la carga de los recursos de la interfaz y la conexión de eventos SSE autenticados. El micrófono requiere HTTPS y permiso del navegador. La computadora y su conexión deben permanecer disponibles. Reiniciar Fénix interrumpe el trabajo activo: actualiza el ejecutable cuando no haya tareas ejecutándose. Los cambios de configuración remota del túnel no requieren reiniciar el agente.

[English](../en/remote-access.md)
