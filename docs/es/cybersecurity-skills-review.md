# Revisión específica de skills de ciberseguridad

[English](../en/cybersecurity-skills-review.md) · [Evidencia de lectura](../skills/cybersecurity-review.json) · [Inventario](../skills/cybersecurity-inventory.json) · [Consultas de búsqueda](../skills/cybersecurity-cases.json)

Revisión del **20 de septiembre de 2026** de [mukul975/Anthropic-Cybersecurity-Skills](https://github.com/mukul975/Anthropic-Cybersecurity-Skills/tree/54a798831d2266a3ca61ce68a7acb80b81160d57). La rama descargada sigue en el commit `54a798831d2266a3ca61ce68a7acb80b81160d57`, ya inventariado anteriormente. Esta revisión profundiza la lectura; no presupone una versión nueva. Es un proyecto comunitario independiente de Anthropic que declara Apache-2.0.

## Qué se revisó realmente

Se inventariaron metadatos y hashes de **818 skills**. La lectura detallada abarca **30 skills: 18 completas y 12 por secciones documentadas**, además de **ocho auxiliares Python: seis completos y dos por secciones**. El manifiesto conserva rutas, rangos, hashes y enlaces al commit. No se ejecutó ni importó código, instaladores, assets o dependencias del repositorio.

Esto amplía sustancialmente la revisión previa, pero **no equivale a auditar las 818 skills y los 1096 auxiliares Python**. Se priorizaron las necesidades del asistente: usuarios separados, agentes/RAG/MCP, archivos, API, vulnerabilidades, backups, credenciales, contenedores, CI e incidentes. Firmware, sistemas industriales, wireless, blockchain y plataformas empresariales especializadas quedaron inventariados para una futura necesidad concreta.

## Cambios en Fénix

La biblioteca pasa de **167 a 171 skills propias**. Se añadieron cuatro y se mejoraron trece existentes, con catorce referencias nuevas que se cargan cuando hacen falta.

| Skill nueva | Utilidad |
| --- | --- |
| [agent-security-evaluation](../../resources/skills/agent-security-evaluation/SKILL.md) | Evalúa inyección de instrucciones, memoria entre usuarios, herramientas, revocación y reuso de aprobaciones mediante canarios y efectos registrados. |
| [api-security-testing](../../resources/skills/api-security-testing/SKILL.md) | Prueba acceso a objetos y campos protegidos entre cuentas; incluye trabajos diferidos y WebSocket tras revocar sesiones. |
| [vulnerability-management](../../resources/skills/vulnerability-management/SKILL.md) | Prioriza hallazgos con evidencia y contexto; conserva responsables, antigüedad, excepciones y cierre verificado. |
| [security-artifact-triage](../../resources/skills/security-artifact-triage/SKILL.md) | Inspecciona PDF, Office, comprimidos y otros archivos sospechosos sin activar su contenido, conservando límites y evidencia. |

Las trece existentes mejoradas son `backup-and-restore`, `dependency-and-secret-audit`, `release-and-ci`, `authentication-integration`, `container-workflows`, `kubernetes-operations`, `identity-access-review`, `phishing-triage`, `incident-response`, `detection-engineering`, `security-code-review`, `rag-evaluation` y `network-diagnostics`.

## Qué aprovechamos y qué corregimos

- **Agentes y RAG:** aprovechamos identidades por operación, pruebas entre usuarios y superficies de inyección. El auxiliar revisado envía el texto como mensaje del usuario y busca palabras en la respuesta; eso no prueba exposición por documentos recuperados ni efectos reales. Nuestra versión registra el canal, la exposición y la ejecución, incluyendo tareas legítimas y resultados inconclusos.
- **API:** aprovechamos cuentas pareadas, propiedades protegidas y autorización posterior al handshake. Un ejemplo considera segura cualquier excepción al probar un token expirado. Nuestra versión verifica datos, estado y efectos parciales; un fallo de conexión no pasa una prueba de seguridad.
- **Vulnerabilidades:** aprovechamos KEV, contexto del activo, responsables y vencimiento. Los ejemplos SSVC deducen una prueba de explotación a partir de EPSS y usan árboles simplificados diferentes. Nuestra versión separa probabilidad, evidencia de explotación y datos ausentes; consulta el modelo SSVC aplicable y conserva la política real de plazos. [FIRST define EPSS como probabilidad](https://www.first.org/epss/) y [CERT/CC publica SSVC](https://certcc.github.io/SSVC/).
- **SBOM y procedencia:** conservamos identidades de componentes y verificaciones del firmante. El auxiliar revisado puede perder `bom-ref`, convertir errores de consulta en listas vacías y tratar resultados por palabras como vulnerabilidades aplicables. Otro informa verificación pese a fallar la decodificación de procedencia. Nuestra versión separa firma, digest, identidad esperada y política del build, sin declarar éxito ante errores.
- **Backups:** aprovechamos restauraciones de prueba y resistencia al borrado. El auxiliar revisado puede pasar sin baseline, comparar errores iguales como si fueran hashes y escribir dentro del árbol restaurado. Creamos un comparador propio de manifiestos que conserva esos límites.
- **Contenedores y CI:** verificamos runtime, permisos, montajes, admisión, red y origen de artefactos realmente aplicados. Evitamos eliminar indiscriminadamente shells o compiladores que necesita el asistente. Una configuración o etiqueta no demuestra aislamiento.
- **Credenciales y correo:** ampliamos JWT/OAuth, consentimiento, rotación y sesiones derivadas. Un correo autenticado puede ser fraudulento; una clave nueva no invalida necesariamente sesiones existentes.
- **Incidentes, detecciones, criptografía y TLS:** ampliamos adquisición, huecos de telemetría, hipótesis refutables, ciclo de vida de claves y diferencia entre emitir un certificado y servirlo correctamente.

La [tabla en inglés](../en/cybersecurity-skills-review.md#findings-that-changed-our-implementation) detalla cada decisión. Los defectos descritos proceden de lectura de los ejemplos; no se presentan como resultados de ejecutar el software ajeno.

## Auxiliar y comprobaciones

El [comparador de backups](../../resources/skills/backup-and-restore/scripts/compare_manifests.py) lee dos JSON del formato Fénix, exige el mismo snapshot, rechaza claves duplicadas y hashes/rutas ambiguos y distingue coincidencia, diferencias y evidencia incompleta. No modifica archivos. Un inventario vacío o incompleto nunca obtiene éxito. **Compara manifiestos; no demuestra por sí solo que los archivos existen, que no hay malware o que la aplicación se recupera.**

Se fijaron **39 consultas de búsqueda antes de editar**: doce de capacidades nuevas, diecisiete de existentes y diez de tareas vecinas. Se comparan los cuerpos anteriores con los nuevos mediante la búsqueda nativa. También se validan metadatos, enlaces, carga de recursos y migración, preservando texto personalizado y estado desactivado. El auxiliar tiene once pruebas ejecutables. Los resultados y cualquier corrección de pruebas se registran en la evidencia JSON.

No hubo campaña con un modelo real, escaneo externo autenticado, ejecución de malware, pentest de producción ni nueva validación del aislamiento. Estas comprobaciones verifican recursos, búsqueda y comportamiento del auxiliar; no demuestran inmunidad a ataques ni inteligencia general superior.

Resultado final: **29/29 pruebas nativas y 80/80 pruebas Python aprobadas**, incluidas las once nuevas del comparador. **17/17 paquetes modificados** pasan el validador de skill-creator; también pasan formato Rust, enlaces locales y espacios del diff. Las **39/39 consultas** encuentran el resultado esperado entre los tres primeros (**37/39** en primera posición). Para skills existentes se pasa de **23/27 a 27/27**; las doce consultas nuevas y las diez de tareas vecinas pasan. El primer candidato falló dos búsquedas: creación ordinaria de PDF y revisión de código en español. Se corrigieron descripción y alias sin cambiar las consultas, y también pasan los conjuntos históricos.

## Incorporación

Los cambios están en el código fuente y se incorporan en la siguiente compilación. Al iniciar esa versión se instalan las skills nuevas y se actualizan solo cuerpos históricos reconocidos. Se conservaron trece snapshots/hashes para esa migración. Las personalizaciones y el estado de activación permanecen; las referencias anteriores no se sobrescriben y el detalle nuevo usa rutas nuevas. No se reinició ni desplegó el servicio ni se alteraron directorios privados de usuarios.
