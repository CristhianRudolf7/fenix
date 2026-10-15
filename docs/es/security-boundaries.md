# Protección de secretos, destinos e instrucciones

> Documento histórico del despliegue compartido anterior a 0.3.2. El sandbox y sus scripts se retiraron de la distribución personal. Véase [ejecución local actual](local-execution.md); las políticas aquí descritas no están activas en 0.3.2.


Estas protecciones se ejecutan en el motor y en el servicio de ejecución, sin llamadas
adicionales al modelo. Complementan el aislamiento por usuario y las aprobaciones.
La especificación completa está en [inglés](../en/security-boundaries.md).

## Secretos

El motor y el cliente de proveedores comparten el detector. Oculta credenciales
registradas y variantes habituales (base64, hexadecimal y codificación de URL), tokens
reconocibles, contraseñas etiquetadas, claves privadas y campos sensibles de JSON.
Se aplica a las solicitudes de los modelos, incluidas las tareas de memoria y
planificación, y conserva la autenticación real en los encabezados HTTP separados.

El streaming retiene fragmentos incompletos para evitar fugas entre bloques. Una salida
sensible de terminal inconclusa que supere 64 KiB detiene el proceso y marca el registro
incompleto. Si los datos opacos de continuidad del proveedor contienen secretos, la
respuesta se rechaza: no se alteran firmas ni se ejecutan sus herramientas.
La memoria también filtra secretos reconocibles al guardar. No se reescribe todo el
historial ni se garantiza detectar cualquier secreto desconocido u ofuscado.

## Destinos de red

La configuración del servicio admite `network_hosts` por perfil (`web`, `development`)
y por usuario. Las listas del usuario se intersectan con las globales; el agente no
puede ampliarlas. Omitir un perfil conserva el acceso web público existente; una lista
vacía bloquea todos sus destinos. Solo se admiten nombres exactos en minúsculas, sin
comodines. Los cambios se aplican a nuevas ejecuciones tras reiniciar el servicio.

En ejecuciones restringidas se resuelven y fijan las IPv4 públicas antes de arrancar.
El firewall admite únicamente esas IP en TCP 80/443, conserva el bloqueo de destinos
privados/del host y elimina el tráfico DNS. Se usan entradas estáticas de hosts y se
desactiva el reenvío DNS del entorno. El monitor comprueba esa misma política.

Es aislamiento por IP: no distingue servicios que comparten una IP de CDN ni valida
rutas HTTP o SNI. Para operaciones remotas estrictas de herramientas no confiables,
se debe combinar ejecución sin red con el intermediario de credenciales y sus acciones
fijas. Las dependencias y redirecciones necesitan destinos explícitamente autorizados.

El lector web admite, por separado, `allowed_hosts` en `web-search.json`: `null` mantiene
la web pública y `[]` deniega solicitudes. Comprueba cada acceso, robots y redirección
antes de abrir conexiones. Esta configuración no borra documentos ya indexados ni
reemplaza la política del navegador o de comandos en el servicio de ejecución.

## Instrucciones y memoria

Se señalan directivas explícitas para anular instrucciones/permisos o exportar secretos.
Una coincidencia es una señal, no una prueba de ataque. Se aceptan negaciones directas
como «Never bypass approvals». El contenido de referencia sigue siendo consultable.

- Las escrituras, ediciones y movimientos mediante parches hacia archivos de instrucciones
  protegidos se revisan antes de modificar archivos. Los parches múltiples se validan
  completos. Incluye AGENTS, SOUL, IDENTITY, SYSTEM, USER, MEMORY, HEARTBEAT, TOOLS y SKILL.
- Al cargar instrucciones del repositorio o del propietario se vuelve a comprobar el
  contenido, incluso si se escribió mediante comandos. Los originales se conservan;
  las fuentes sospechosas se sustituyen por un aviso fijo en el contexto de instrucciones.
- Los resultados de skills indican que son referencias y no conceden permisos.
- Una memoria sospechosa permanece como candidata sin fijar; repetirla no la confirma
  ni permite reemplazar un hecho confirmado. Se excluye de la recuperación automática,
  también si procede de registros anteriores o tarjetas de entidades.
- Un checkpoint mezcla fuentes y no permite confirmar una memoria: se exige evidencia
  del mensaje original del propietario, con cita y ámbito verificados. Los resúmenes
  por ámbito se revisan antes de introducirlos como contexto de instrucciones.

No se añade un auditor LLM ni se reescriben instrucciones automáticamente. Los ejemplos
legítimos de ataques pueden guardarse en documentos de referencia. El detector no
garantiza entender todas las paráfrasis: las aprobaciones, el aislamiento y la procedencia
siguen siendo controles necesarios.

## Comprobación

Las pruebas locales cubren secretos fragmentados, autenticación, datos opacos, promoción
de memoria, archivos protegidos, políticas de destinos, DNS y redirecciones. Las pruebas
reales con Docker/gVisor/Kata son optativas. Esta implementación no modifica el firewall
de producción ni equivale a una prueba de penetración de la infraestructura desplegada.
