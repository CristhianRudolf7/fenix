# Espacios virtuales en servidores propios

[English](../en/self-hosted-microvms.md) · [Política de ejecución](execution-isolation.md)

**Backend opcional.** La opción predeterminada ahora es [gVisor con systrap](self-hosted-gvisor.md), que no requiere KVM. Esta página describe la alternativa Kata seleccionada explícitamente.

Fénix incorpora un backend de máquinas virtuales Linux mediante **Kata Containers y Cloud Hypervisor**, sin cuenta, API ni suscripción de E2B. Docker Engine/containerd invoca el runtime de Kata; los procesos del usuario se ejecutan dentro de una VM con su propio kernel.

El adaptador y las pruebas están implementados. **Este servidor no dispone de `/dev/kvm`**, por lo que aquí no se ha validado el arranque real, la compatibilidad de Kata, la contabilización de recursos del hipervisor ni Chromium dentro de la VM. La configuración rechaza el arranque si falta el aislamiento; no vuelve automáticamente a runc.

## Decisión técnica

Kata aporta un supervisor de procesos invitado y la integración de archivos mediante virtio-fs. Esto permite que las herramientas nativas de Fénix y los programas de la VM trabajen con los mismos archivos, sin desarrollar nuestro propio hipervisor ni un servidor de archivos. Firecracker es un motor de virtualización; desplegarlo directamente requeriría resolver también esas funciones. Esta implementación no copia ni necesita el servicio de E2B.

Referencias: [arquitectura de Kata](https://github.com/kata-containers/kata-containers/blob/main/docs/design/architecture/README.md), [hipervisores disponibles](https://github.com/kata-containers/kata-containers/blob/main/docs/hypervisors.md), [runtimes de Docker](https://docs.docker.com/engine/daemon/alternative-runtimes/). Debe verificarse la versión concreta instalada; consultar `main` no fija una versión.

## Funcionamiento

- El controlador conserva conversaciones, identidad, credenciales y programación de tareas fuera de las VM de trabajo. Se mantienen la autorización, el intermediario de credenciales y el control de red existentes.
- Cada ejecución recibe una VM nueva. Un espacio puede tener varias ejecuciones, sujetas a límites y conflictos de escritura. No se comparte una VM activa entre clientes ni se reutilizan máquinas preparadas con estado de otro usuario.
- Los archivos permanecen en el sistema de archivos de capacidad limitada de la cuenta. `persistent_environments: true` añade un directorio privado de herramientas por proyecto en `CUENTA/environments/HASH`. Allí se conservan paquetes de usuario, entornos virtuales y cachés. También consumen la cuota de disco.
- Fénix configura `HOME`, `PATH` y los directorios de npm, Python y Rust. Los paquetes del sistema pertenecen a la imagen del operador. No se ofrece acceso administrativo al servidor.
- Los servidores de lenguaje reciben el proyecto y su entorno de herramientas en modo de solo lectura y sin red. Los comandos restringidos a archivos concretos no reciben ese directorio adicional. El navegador conserva sus perfiles y descargas delimitados.
- Al terminar una ejecución se elimina su VM, pero permanecen los archivos y las herramientas. Las sesiones activas continúan hasta completar, cancelar, desconectar, revocar o agotar su plazo. La siguiente ejecución arranca una VM nueva. **No se implementan snapshots de RAM, restauración de procesos suspendidos ni migración en vivo.**
- `Dockerfile.workspace` incorpora herramientas Linux para DOCX, XLSX, PPTX, PDF y multimedia. No equivale a Microsoft Office nativo de Windows; Windows y GPU necesitan otro backend explícito.

La persistencia sigue siendo local al servidor. Deben respaldarse los volúmenes de cuenta, incluidos los directorios de herramientas. Las copias de archivos del proyecto no incluyen ese directorio separado. Este adaptador no añade almacenamiento replicado ni recuperación automática entre servidores.

## Seguridad y cuotas

El backend exige KVM y un manifiesto privado con hashes de configuración, runtime, hipervisor, kernel, imagen invitada, virtiofsd y programa de arranque. Cambiar esos archivos obliga a detener las ejecuciones y validar la actualización.

La preparación activa seccomp en el invitado y el hipervisor; desactiva consolas de depuración, cachés de VM y cambios mediante anotaciones; y evita exponer dispositivos de bloques del host. Los montajes de solo lectura se protegen también en el servidor. La configuración utiliza virtio-fs sin caché para mantener la coherencia con las herramientas nativas.

Antes de ejecutar el comando, Fénix comprueba que el kernel invitado es distinto al del host y a los de otras ejecuciones activas, identifica el proceso real de Cloud Hypervisor y verifica su grupo de recursos de cuenta. Después vuelve a comprobar la autorización y el plazo. Estas son comprobaciones de despliegue, **no una atestación criptográfica**. Chromium mantiene su sandbox interno y utiliza el perfil seccomp dentro del invitado; no se aplica como perfil invitado el AppArmor específico del host runc.

La admisión reserva la memoria de la tarea más `microvm_overhead_mib`. Continúan los límites de memoria, CPU, procesos, disco y concurrencia. Debe verificarse su aplicación efectiva al hipervisor y a virtiofsd en el host de destino.

Cada cuenta puede definir `execution_seconds_per_month`. Se reserva el plazo máximo antes de ejecutar, se contabiliza el tiempo real al limpiar y se devuelve la parte no utilizada. Tras un fallo abrupto se consumen conservadoramente las reservas pendientes para evitar reiniciar la cuota al reiniciar el servicio. Los meses son UTC y cada ejecución carga al mes de admisión. Es una cuota de tiempo de ejecución, no facturación monetaria ni contabilización de tokens. `execution_status` muestra el uso de la cuenta autenticada.

## Instalación y comprobación

Instalar una versión verificada de Kata con Cloud Hypervisor, kernel, imagen y virtiofsd en su prefijo compilado, normalmente `/opt/kata`. Comprobar primero las sumas de los artefactos oficiales; el manifiesto de Fénix fija archivos locales, pero no demuestra por sí mismo su procedencia.

Seguir la [instalación de aislamiento](execution-isolation.md) para preparar cuentas y volúmenes. Detener las tareas y el servicio antes de cambiar el runtime. Usar una instalación de Kata dedicada: la preparación sustituye su `configuration.toml` predeterminado y conserva una copia `configuration.before-fenix.toml`.

```sh
sudo scripts/install-isolation.sh
sudo python3 /opt/fenix/isolation/microvm-preflight.py
sudo python3 /opt/fenix/isolation/prepare-microvm.py --register-runtime
sudo systemctl reload docker
sudo python3 /opt/fenix/isolation/microvm-preflight.py \
  --manifest /etc/fenix/microvm/manifest.json
```

La preparación conserva los demás ajustes de Docker y su runtime predeterminado, valida el JSON y guarda una copia. No recarga Docker por sí misma. Sin `--register-runtime`, genera únicamente un fragmento para administración externa. Un manifiesto existente no se sobrescribe automáticamente; revisar el fragmento y las copias si una preparación queda incompleta.

Construir `Dockerfile.workspace` con una referencia de registro y digest inmutable (`imagen@sha256:...`), o una etiqueta local verificada, y suficiente espacio libre, como indica la [guía en inglés](../en/self-hosted-microvms.md#prepare-an-execution-host). BuildKit no admite un ID local `sha256:...` aislado en `FROM`. La imagen incluye el [entorno de subtítulos de YouTube](youtube-summary.md). Crear la configuración a partir de `execution.microvm.example.json`, ajustando identidades, propietario del registro, imagen y capacidad real. Conservar los UUID y volúmenes existentes; no reformatear datos. Recrear los controladores con la configuración correspondiente.

El [registro de validación](../research/self-hosted-microvms-validation-2026-09-20.json) distingue las pruebas de política, las regresiones reales en OCI y las pruebas de VM todavía pendientes. La ejecución conjunta tuvo un fallo de arranque de Chromium; la repetición aislada pasó sin debilitar su sandbox. Se comprobó la resolución de dependencias de documentos, pero no se construyó la imagen completa por falta de espacio suficiente. Estos resultados no demuestran compatibilidad con Kata ni estabilidad del navegador bajo carga.

Las pruebas de política funcionan sin KVM:

```sh
python3 -m unittest discover -s tests/isolation -v
```

En un host de pruebas con KVM, runtime preparado e imagen workspace construida:

```sh
sudo env FENIX_MICROVM_TESTS=1 \
  FENIX_MICROVM_MANIFEST=/etc/fenix/microvm/manifest.json \
  FENIX_TEST_IMAGE=fenix-workspace:local \
  python3 -m unittest discover -s tests/isolation -p test_microvm_acceptance.py -v
```

La suite verifica identidad invitada, persistencia, documentos, terminales, archivos de solo lectura, túneles privados, revocación, eliminación de procesos y Chromium/CDP. Si se solicita expresamente ejecutarla y falta KVM, falla. Sus resultados reales y los hashes instalados deben registrarse antes de habilitar tráfico de clientes y después de actualizar el runtime.
