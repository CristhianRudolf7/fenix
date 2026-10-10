# Instalación pública de Fénix

## Instalar

En Linux x86_64 con glibc 2.39 o posterior (por ejemplo Ubuntu 24.04), OpenSSL 3, zlib, zstd y una sesión systemd de usuario:

```bash
curl -fsSL https://fenix.crehlabs.com/install.sh | bash
```

No ejecutes con sudo. No necesitas clonar el repositorio ni instalar Rust, Node o pnpm para la aplicación base. Si no tienes `curl`, instálalo primero con el gestor de paquetes de tu distribución.

Al terminar, abre `http://127.0.0.1:4173`. La bienvenida permite elegir proveedor, modelo y clave. La comprobación de la conexión hace una solicitud real al proveedor y puede consumir tokens; no se realizó esa solicitud durante la validación del instalador.

Esta distribución es la instalación personal en localhost. El despliegue alojado con usuarios y aislamiento conserva su configuración independiente. El navegador, aislamiento y otras herramientas opcionales pueden requerir dependencias adicionales; el instalador no las da por disponibles.

## Otro puerto o versión

```bash
curl -fsSL https://fenix.crehlabs.com/install.sh | FENIX_PORT=4180 FENIX_VERSION=v0.3.1 bash
```

Un puerto ocupado por otro servicio provoca un error antes de reemplazar la aplicación.

## Datos y servicios

- Aplicación: `~/.local/share/fenix-personal/current`, con releases separadas.
- Datos y memoria: `~/.local/share/fenix-personal/data`.
- Skills personales: `~/.local/share/fenix-personal/skills`.
- Configuración opcional del proceso: `~/.local/share/fenix-personal/config/server.env`.
- Servicio: `fenix-personal.service`, independiente de los servicios alojados.

```bash
systemctl --user status fenix-personal.service
systemctl --user restart fenix-personal.service
```

Repite el comando de instalación para actualizar. Conserva tus datos y restaura la aplicación y unidad anteriores si no puede arrancar. La verificación SHA-256 detecta alteraciones o corrupción de la descarga; no constituye una firma criptográfica independiente de GitHub.

Para detenerlo sin borrar los datos:

```bash
systemctl --user disable --now fenix-personal.service
```

El servicio se inicia con la sesión systemd del usuario. Para mantenerlo funcionando después de cerrar esa sesión, el administrador puede habilitar la persistencia de usuario con `loginctl enable-linger`; no se cambia esa política automáticamente.

## Validación del 10 de octubre de 2026

- Compilación optimizada de Rust y comprobación TypeScript/build de la interfaz completas.
- Seis pruebas controladas: instalación, actualización conservando datos, checksum incorrecto, archivo con enlace inseguro, puerto ocupado y restauración ante fallo de arranque. Descargas y systemd se simulan en estas seis pruebas.
- Ejecución real del binario con `--help` dentro de Ubuntu 24.04, con sus bibliotecas de ejecución. No equivale a una instalación systemd dentro de ese contenedor.
- Instalación real desde `https://fenix.crehlabs.com/install.sh`, descargando la release pública 0.3.1 y verificando su checksum. Arranque real mediante systemd en el puerto 4199, separado de producción.
- Bienvenida real comprobada en Chromium: campo de clave enmascarado, 17 proveedores y ausencia de desbordamiento horizontal en móvil.
- Reinstalación real desde el mismo comando; conservación de un archivo de comprobación en el directorio de datos.
- Comprobación de que el modo alojado oculta el formulario de claves usando una respuesta de sesión controlada. No se presenta como prueba de una cuenta real en producción.
- La instancia personal usada para validar se detuvo después de las pruebas; se conservaron sus datos. Los servicios publicados existentes permanecieron activos.

GitHub Actions tiene un bloqueo de facturación de la cuenta. El workflow de publicación está implementado, pero esta primera release se compiló y publicó manualmente; no se afirma que Actions haya ejecutado estos pasos correctamente.
