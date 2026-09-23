# Revisión de los siete repositorios sandbox y la seguridad de Fénix

Fecha: 19 de septiembre de 2026. Estado: **investigación y propuesta; esta revisión no implementa cambios de seguridad**. [English](../en/sandbox-security-review.md) · [Plan de implementación](user-isolation-plan.md) · [Evidencias](../research/sandbox-security-2026-09-19.json).

La mejora principal para Fénix consiste en conservar el gateway autenticado y los entornos privados por cuenta, reforzar su aprovisionamiento y separar la ejecución de tareas del estado y las credenciales del agente. Los siete proyectos aportan ideas útiles, pero ninguno garantiza por sí solo que un usuario de Fénix solo acceda a lo suyo. Las skills pueden explicar las reglas; el servidor, el sistema operativo y las pruebas deben hacerlas cumplir.

## Qué se investigó

Se tomaron los siete repositorios del área `sandbox` del catálogo local `/home/Rudy/proyectos/repo/repo.sqlite`, consultado sin modificarlo. Se inspeccionaron los inventarios de funciones de sus README, documentación seleccionada de arquitectura, seguridad y ciclo de vida, código específico de autenticación/SDK y documentación oficial de los productos.

Se fijó una revisión de cada repositorio y se verificaron los archivos descargados contra sus hashes Git; el manifiesto conserva además SHA-256. **Esto no equivale a leer y auditar línea por línea todo el código de siete proyectos.** Tampoco se afirma que cada archivo descargado se leyera completo. No se ejecutaron sus instaladores ni se desplegaron sus servicios. Se revisó código local de Fénix y se ejecutaron sus pruebas de cuentas; no se inspeccionaron secretos ni se atacaron cuentas, contenedores o redes reales.

| Repositorio | Revisión examinada | Naturaleza |
|---|---|---|
| daytonaio/daytona | `ec4c21b2d597091ac09ecc278f3bcc172575a987` | Estado del repositorio y documentación del producto, diferenciados |
| firecracker-microvm/firecracker | `23b09b943fa3cd4a04004fe55724f0b5c2410f45` | Motor de microVMs, aislamiento del proceso y requisitos del host |
| sandboxie-plus/Sandboxie | `e82c55ba4969af79abddcd31155bb1bfe316bee6` | Aislamiento de aplicaciones Windows |
| opensandbox-group/OpenSandbox | `ff4d1bc269cf6a9be03592649bed19633bf45222` | Plataforma de ejecución y gestión de sandboxes |
| coder/coder | `a698b026fef7d1dbc15b4fbf6a4bd4c7c885c17c` | Entornos de desarrollo, agentes y permisos |
| e2b-dev/E2B | `ccaf9fc0ffe6ac39c7ec786af7608ab1de19467b` | SDK y contrato del servicio; infraestructura separada |
| TencentCloud/CubeSandbox | `e4e5dc244daaddc0fc706ff67fa4d4e51948cd86` | Plataforma de microVMs, red y ciclo de vida |

## Daytona

**Funcionalidades.** Gestión de entornos, ejecución de procesos, archivos, Git, terminales, snapshots, recursos, SDK y publicación de aplicaciones. La documentación del producto describe restricciones de salida y secretos sustituidos por un proxy HTTPS, evitando entregar el valor real al proceso. [README examinado](https://github.com/daytonaio/daytona/blob/ec4c21b2d597091ac09ecc278f3bcc172575a987/README.md), [secretos](https://www.daytona.io/docs/en/secrets/).

**Aprendizaje de seguridad.** La autorización para administrar un entorno, acceder a su aplicación y salir a Internet son permisos diferentes. Un token estándar de preview tiene alcance sobre el sandbox y sus puertos internos; las URLs firmadas permiten delimitar puerto, caducidad y revocación. Configurar una variable de proxy no impide que un programa intente otra ruta de salida. [Previews](https://www.daytona.io/docs/en/preview/), [red](https://www.daytona.io/docs/en/network-limits/).

**Limitación decisiva.** El README fijado anuncia que el núcleo pasó a ser privado en junio de 2026 y que este repositorio público no recibirá nuevas correcciones ni versiones. Su árbol actual contiene documentación y recursos gráficos. No podemos describir el motor actual como código abierto inspeccionado y mantenido ni prometer un fork equivalente al producto actual.

**Para Fénix.** Adoptar previews privados, temporales, revocables y limitados a un servicio. Considerar el servicio gestionado solo como adaptador opcional; no construir nuestra seguridad principal sobre un fork histórico sin mantenimiento.

## Firecracker

**Funcionalidades.** Motor Linux/KVM en Rust con un kernel invitado separado, configuración de CPU y memoria, dispositivos de disco/red, API de control, snapshots, métricas, registros y límites de transferencia. No incluye por sí mismo la identidad de los usuarios de nuestra aplicación. [README](https://github.com/firecracker-microvm/firecracker/blob/23b09b943fa3cd4a04004fe55724f0b5c2410f45/README.md).

**Seguridad.** Su jailer reduce privilegios, separa el contexto del proceso, elimina entorno/descriptores heredados y admite cgroups y límites. Recomienda identidades no privilegiadas distintas y rutas protegidas. Seccomp restringe llamadas al sistema por hilo; las compilaciones de desarrollo no tienen necesariamente las mismas garantías. La protección requiere mantener actualizados host, invitado y microcódigo. [Jailer](https://github.com/firecracker-microvm/firecracker/blob/23b09b943fa3cd4a04004fe55724f0b5c2410f45/docs/jailer.md), [host de producción](https://github.com/firecracker-microvm/firecracker/blob/23b09b943fa3cd4a04004fe55724f0b5c2410f45/docs/prod-host-setup.md), [seccomp](https://github.com/firecracker-microvm/firecracker/blob/23b09b943fa3cd4a04004fe55724f0b5c2410f45/docs/seccomp.md).

**Límites.** El integrador debe autenticar, cifrar y controlar los snapshots. Una clonación también copia memoria, secretos y ciertos estados aleatorios de las aplicaciones; VMGenID no corrige automáticamente todo eso. Una microVM no repara un permiso de aplicación concedido al usuario equivocado. [Snapshots](https://github.com/firecracker-microvm/firecracker/blob/23b09b943fa3cd4a04004fe55724f0b5c2410f45/docs/snapshotting/snapshot-support.md), [entropía](https://github.com/firecracker-microvm/firecracker/blob/23b09b943fa3cd4a04004fe55724f0b5c2410f45/docs/snapshotting/random-for-clones.md).

**Para Fénix.** Candidato para probar una frontera más fuerte frente a código hostil, sujeto a KVM, compatibilidad, consumo total y capacidad de operación. Reutilizar el motor mantenido; desarrollar nuestra autorización y gestión, no un hipervisor propio.

## Sandboxie Plus / Classic

**Funcionalidades.** En Windows, virtualiza acceso a archivos/registro, administra múltiples cajas, snapshots, recuperación e importación/exportación. Incluye controles de procesos, memoria, red WFP, DNS/proxy, IPC, portapapeles, dispositivos, almacenamiento cifrado y eventos del ciclo de vida. Comparte el kernel del host. [Inventario](https://github.com/sandboxie-plus/Sandboxie/blob/e82c55ba4969af79abddcd31155bb1bfe316bee6/README.md).

**Aprendizaje de seguridad.** Proteger al host contra escrituras no impide leer datos privados. El modo de privacidad limita lecturas; el modo reforzado restringe llamadas y dispositivos. Algunas funciones requieren certificado de soporte. Además, la propiedad de los directorios importa: `LockBoxToUser` limita ACL que, por defecto, pueden permitir acceso a otros usuarios autenticados de Windows. [Privacidad](https://sandboxie-plus.github.io/sandboxie-docs/PlusContent/privacy-mode/), [modo reforzado](https://sandboxie-plus.github.io/sandboxie-docs/PlusContent/security-mode/), [propiedad](https://sandboxie-plus.github.io/sandboxie-docs/Content/LockBoxToUser.html).

**Para Fénix.** Incorporar pruebas de lectura de datos, perfiles de navegador, IPC y propiedad real de archivos. No sirve como backend del despliegue Linux actual. Su README distingue licencias de Classic y Plus; no conviene reducir todo el proyecto a una etiqueta de licencia de GitHub.

## OpenSandbox

**Funcionalidades.** API de ciclo de vida y ejecución, backends Docker/Kubernetes, SDK de varios lenguajes, CLI, archivos, comandos, intérpretes, ejemplos de navegador/escritorio, ingreso/salida de red, runtimes opcionales, vault de credenciales y sesiones aisladas dentro de un sandbox. [README](https://github.com/opensandbox-group/OpenSandbox/blob/ff4d1bc269cf6a9be03592649bed19633bf45222/README.md).

**Autenticación.** El middleware inspeccionado admite clave global o proveedor de identidad por tenant; sin ambos puede omitir autenticación. También existen credenciales separadas para endpoints. La existencia de ese proveedor no demuestra que cualquier integración futura con Fénix autorice correctamente todos los recursos. [Middleware](https://github.com/opensandbox-group/OpenSandbox/blob/ff4d1bc269cf6a9be03592649bed19633bf45222/server/opensandbox_server/middleware/auth.py), [proveedor](https://github.com/opensandbox-group/OpenSandbox/blob/ff4d1bc269cf6a9be03592649bed19633bf45222/server/opensandbox_server/tenants/provider.py).

**Red y secretos.** Separa reglas obligatorias del operador de las solicitadas por el trabajo. Credential Vault mantiene secretos en un componente de salida y los inserta según destino, puerto, método y ruta. Requiere control de paquetes; solo DNS no basta. Tras recrear el componente hay que reinyectar credenciales desde un servicio confiable. [Red](https://github.com/opensandbox-group/OpenSandbox/blob/ff4d1bc269cf6a9be03592649bed19633bf45222/docs/architecture/network-isolation.md), [Vault](https://github.com/opensandbox-group/OpenSandbox/blob/ff4d1bc269cf6a9be03592649bed19633bf45222/docs/guides/credential-vault.md).

**Lo que no copiaría automáticamente.** El arranque documentado de sesiones con bubblewrap concede `CAP_SYS_ADMIN` y desactiva el confinamiento AppArmor. La comprobación de tamaño del overlay es de admisión, no una cuota continua, y diff/commit figuran como funciones pendientes. Hay incompatibilidades documentadas entre su salida de red, gVisor y ciertos proxies transparentes. [Sesiones](https://github.com/opensandbox-group/OpenSandbox/blob/ff4d1bc269cf6a9be03592649bed19633bf45222/docs/guides/isolation-sessions.md).

Su explicación de NetworkPolicy exagera límites generales de Kubernetes: este sí permite políticas de denegación de ingreso/salida para todos los pods seleccionados, con un complemento de red compatible. No debemos copiar conclusiones sin contrastarlas. [Documentación de Kubernetes](https://kubernetes.io/docs/concepts/services-networking/network-policies/).

**Para Fénix.** Adoptar separación de API, políticas obligatorias, credenciales acotadas y detección de capacidades. Un adaptador es evaluable; aumentar privilegios para habilitar una función no sería una mejora automática de seguridad.

## Coder

**Funcionalidades.** Aprovisionamiento de entornos mediante Terraform, contenedores/VMs, acceso remoto a terminal/IDE, ciclo de vida, detención por inactividad, organizaciones, permisos y herramientas de agentes. Su diseño mantiene la coordinación y llamadas al modelo fuera del entorno donde se ejecuta el código, conectado mediante un túnel autenticado. [README](https://github.com/coder/coder/blob/a698b026fef7d1dbc15b4fbf6a4bd4c7c885c17c/README.md), [arquitectura](https://github.com/coder/coder/blob/a698b026fef7d1dbc15b4fbf6a4bd4c7c885c17c/docs/ai-coder/agents/architecture.md).

**Aprendizajes.** Permisos por recurso, OAuth individual, control de herramientas MCP y conversaciones compartidas en lectura son capas independientes. Compartir la conversación puede incluir subtareas y adjuntos; no debe permitir ejecutar comandos. Las cabeceras de identidad no sustituyen la autenticación. [Organizaciones](https://github.com/coder/coder/blob/a698b026fef7d1dbc15b4fbf6a4bd4c7c885c17c/docs/ai-coder/agents/platform-controls/organizations.md), [MCP](https://github.com/coder/coder/blob/a698b026fef7d1dbc15b4fbf6a4bd4c7c885c17c/docs/ai-coder/agents/platform-controls/mcp-servers.md), [compartición](https://github.com/coder/coder/blob/a698b026fef7d1dbc15b4fbf6a4bd4c7c885c17c/docs/ai-coder/agents/chat-sharing.md).

**Límites.** Un secreto inyectado deliberadamente en el workspace puede ser leído por su código. El cifrado de base de datos es opcional, cubre campos concretos y requiere migrar registros existentes. Varias organizaciones, permisos de plantillas y auditoría tienen restricciones Premium. Hay límites entre código AGPL y componentes empresariales. [Secretos](https://github.com/coder/coder/blob/a698b026fef7d1dbc15b4fbf6a4bd4c7c885c17c/docs/admin/security/secrets.md), [cifrado](https://github.com/coder/coder/blob/a698b026fef7d1dbc15b4fbf6a4bd4c7c885c17c/docs/admin/security/database-encryption.md), [plantillas](https://github.com/coder/coder/blob/a698b026fef7d1dbc15b4fbf6a4bd4c7c885c17c/docs/admin/templates/template-permissions.md), [auditoría](https://github.com/coder/coder/blob/a698b026fef7d1dbc15b4fbf6a4bd4c7c885c17c/docs/admin/security/audit-logs.md).

**Para Fénix.** Es una referencia especialmente útil para separar controlador y ejecución y gobernar el acceso. Mantendría nuestros controladores privados por cuenta; no centralizaría automáticamente toda la memoria y credenciales de usuarios en un único agente compartido.

## E2B

**Funcionalidades.** SDK para entornos, archivos, comandos/PTY, plantillas, tiempos de vida, pausa/reanudación y URLs de aplicaciones, con paquetes relacionados para intérprete y escritorio. El servicio documenta microVMs Firecracker; el repositorio inspeccionado representa principalmente el contrato cliente, no toda la infraestructura. [README](https://github.com/e2b-dev/E2B/blob/ccaf9fc0ffe6ac39c7ec786af7608ab1de19467b/README.md), [seguridad del servicio](https://e2b.dev/security).

**Distinciones.** En el SDK fijado, el acceso a envd siempre se protege y la opción antigua `secure` está obsoleta. El ingreso a las aplicaciones es otra opción: `allowPublicTraffic` figura con valor predeterminado verdadero. También aparecen políticas de red, transformaciones e identidad del trabajo; su disponibilidad depende del backend. El proxy de salida nombrado no está soportado por el runtime abierto según el propio contrato. [SDK](https://github.com/e2b-dev/E2B/blob/ccaf9fc0ffe6ac39c7ec786af7608ab1de19467b/packages/js-sdk/src/sandbox/sandboxApi.ts).

**Para Fénix.** Adoptar plazos explícitos, cancelación, reconexión autenticada y detección de funciones. Mantener claves del servicio fuera del runner y resolver IDs desde nuestro registro de propiedad. Un tipo del SDK no demuestra que cualquier servidor compatible implemente la misma seguridad. No se pudo recuperar la guía separada de self-hosting; no se afirma una auditoría de esa infraestructura.

## CubeSandbox

**Funcionalidades.** Plataforma KVM con motor Rust, plantillas, ejecución, clonación copy-on-write, snapshots, rollback, pausa/reanudación, volúmenes persistentes y superficies compatibles con E2B. Combina control de paquetes eBPF y proxy HTTP/HTTPS; algunas opciones distribuidas están en preview. Sus cifras de rendimiento no se midieron aquí. [README](https://github.com/TencentCloud/CubeSandbox/blob/e4e5dc244daaddc0fc706ff67fa4d4e51948cd86/README.md), [ciclo de vida](https://github.com/TencentCloud/CubeSandbox/blob/e4e5dc244daaddc0fc706ff67fa4d4e51948cd86/docs/guide/snapshot-rollback-clone.md).

**Seguridad y valores predeterminados.** La autenticación de API depende de configurar un callback. La guía de endurecimiento identifica servicios internos sin autenticación, listeners amplios y canales sin cifrado. Las URLs de aplicaciones son públicas por defecto; se puede exigir token, pero la guía no ofrece rotación durante la vida de ese sandbox. [Autenticación](https://github.com/TencentCloud/CubeSandbox/blob/e4e5dc244daaddc0fc706ff67fa4d4e51948cd86/docs/guide/authentication.md), [endurecimiento](https://github.com/TencentCloud/CubeSandbox/blob/e4e5dc244daaddc0fc706ff67fa4d4e51948cd86/docs/guide/network-hardening.md), [acceso público](https://github.com/TencentCloud/CubeSandbox/blob/e4e5dc244daaddc0fc706ff67fa4d4e51948cd86/docs/guide/restrict-public-access.md).

**Aprendizajes.** El proxy puede inyectar credenciales fuera del invitado y registrar decisiones por destino/método/ruta. Los puertos no interceptados siguen sujetos a otra capa de política. Los permisos de paquetes prevalecen sobre las denegaciones: hay que impedir que una excepción solicitada por una tarea debilite la política del operador. Los snapshots sobreviven al proceso y necesitan propiedad, cuotas y borrado propios. [Red](https://github.com/TencentCloud/CubeSandbox/blob/e4e5dc244daaddc0fc706ff67fa4d4e51948cd86/docs/guide/network-policy.md), [proxy](https://github.com/TencentCloud/CubeSandbox/blob/e4e5dc244daaddc0fc706ff67fa4d4e51948cd86/docs/guide/security-proxy.md).

**Para Fénix.** Tomar control externo de red, generaciones del ciclo de vida y limpieza de clones. No adoptar el quickstart como despliegue seguro ni asumir equivalencia total con E2B. Un rollback local no deshace un correo enviado o una operación externa. Su licencia declara Apache-2.0 con excepciones de terceros.

## Qué ya protege Fénix y qué falta

El código ya tiene usuarios con UUID estable, hashes Argon2, archivos privados de cuentas y sesiones, sesión fija de una semana, selección del runtime a partir del usuario autenticado y credencial interna distinta por runtime. Cada cuenta tiene su entorno y base de datos; el mercado de skills es un subsistema compartido intencional con operaciones por propietario y contenido aprobado público. [Gateway](../../apps/server/src/accounts.rs), [sesiones](../../apps/server/src/accounts/sessions.rs), [mercado](../../apps/server/src/skill_guard/mod.rs).

El aprovisionamiento pide imagen de solo lectura, capacidades eliminadas, no-new-privileges, montaje de una sola cuenta, tmpfs privado, publicación en loopback y puente dedicado. No monta el socket Docker. El modo de lectura incorpora Landlock y seccomp y rechaza una plataforma incapaz de aplicarlos. Son bases que debemos conservar. [Contenedor](../../scripts/tenant-container.py), [modo de lectura](../../crates/fenix-agent/src/read_only.rs).

| Hallazgo | Evidencia | Implicación | Prioridad |
|---|---|---|---|
| El guardián confía en que exista la cadena del firewall | Código y ejecución con comandos simulados | Una cadena incompleta puede no repararse; no se demostró exposición del despliegue real | P0 |
| El UID del contenedor viene del usuario que aprovisiona | Simulación de root produjo `--user 0:0` | La garantía de no-root depende de cómo se invoque | P0 |
| Sin cuotas explícitas de CPU, RAM, PID y disco persistente | Argumentos Docker y documentación | Un usuario puede agotar capacidad compartida | P0 |
| Código con escritura comparte montaje/identidad con el controlador | Procesos y estructura del runtime | Puede alcanzar estado y credenciales de su propia cuenta; no prueba acceso a otra | P1 |
| Distintos caminos heredan distinto entorno | Shell filtra nombres; LSP hereda entorno | Hace falta una política mínima común | P1 |
| Chromium usa `--no-sandbox` en la imagen de tenant | Dockerfile | Menos defensa dentro del contenedor | P1 |
| Salida general a Internet público | Guardián de red | Datos que la tarea pueda leer podrían salir a destinos permitidos | P1 |
| Cambios de cuenta/contraseña dependen de recargar el registro | Implementación de sesiones | Falta una vía explícita de revocación inmediata | P1 |
| Cobertura parcial de recursos y tareas en segundo plano | Pruebas actuales | Ampliar pruebas negativas; no equivale a una vulnerabilidad demostrada | P0/P1 |

Si todas las cuentas se aprovisionan con el mismo UID del host, los permisos 0700 no son una barrera adicional entre procesos del host con esa misma identidad; el montaje del contenedor es la frontera principal. Un UID distinto o user namespaces pueden añadir defensa. El administrador del host sigue siendo confiable: no se promete protección contra quien controla el host y sus claves.

## Comprobaciones realizadas y conclusión

Pasaron **7 pruebas** de `cargo test -p fenix-server --lib accounts:: --locked`: autenticación, sesiones, rutas por cuenta y propiedad del mercado, entre otras. No se ejecutó toda la suite ni pruebas de escape del kernel.

Dos comprobaciones locales con datos ficticios y todos los comandos externos simulados confirmaron la propagación de UID 0 y la falta de reparación de una cadena existente. No ejecutaron Docker, sudo ni iptables. El manifiesto conserva sus resultados.

La propuesta es construir **nuestra propia gestión de permisos, recursos, ejecución y evidencias**, apoyada en componentes mantenidos. No instalar los siete proyectos, no aumentar skills como sustituto del aislamiento y no prometer seguridad completa a partir de estas pruebas. El [plan](user-isolation-plan.md) concreta fases, archivos, migración y criterios verificables.
