# Marketplace de skills y Fénix Sentinel

Fénix Sentinel es un revisor propio implementado en Rust. No incorpora ni depende de NVIDIA SkillSpector. Combina reglas locales con el modelo rápido configurado, sin herramientas disponibles para el modelo auditor. Crear, editar o utilizar un skill privado no activa esta auditoría de publicación.

## Uso

Skills incluye Mis skills, Marketplace y Publicaciones. El usuario elige una categoría y solicita publicar. Solo las versiones aprobadas aparecen en el catálogo compartido, donde se puede buscar, filtrar, previsualizar e instalar una copia privada con un nombre libre. Nunca se sobrescriben carpetas existentes.

La publicación es una copia fija. Los cambios privados no modifican la versión pública: se solicita otra revisión mediante «Revisar cambios actuales». La versión anterior permanece pública hasta aprobar la nueva o retirarla. «Mantener privado» también cancela revisiones pendientes para impedir una publicación tardía. Retirar una publicación no elimina las copias que otros usuarios ya instalaron.

## Correcciones sin perder funciones

El modelo rápido identifica las capacidades legítimas con evidencia y propone cambios mínimos sobre una copia. Otra revisión comprueba la seguridad y una comparación separada verifica cada capacidad. El motor conserva archivos, permisos de ejecución, metadatos, encabezados, bloques de código, código en línea y enlaces. El original privado queda intacto; el autor puede ver el informe y las diferencias.

Una corrección que cambia código ejecutable queda pendiente de pruebas funcionales; el servicio no ejecuta scripts desconocidos ni inventa resultados. Tampoco publica si el análisis está incompleto o no puede establecer la conservación de funciones. La revisión semántica no garantiza equivalencia de comportamiento ni ausencia total de vulnerabilidades.

Esta versión admite paquetes completos de texto UTF-8: hasta 128 archivos y 128 KiB, con SKILL.md de hasta 32 000 bytes. Rechaza enlaces, rutas ocultas, archivos especiales, binarios y archivos comprimidos, sin omitirlos silenciosamente. No incluye consulta de CVE, YARA, análisis AST, desempaquetado de documentos ni verificación criptográfica de firmas. Posibles credenciales impiden enviar el contenido al modelo y requieren intervención del propietario.

## Servicio y protección contra abuso

El gateway autenticado mantiene el marketplace central fuera de los directorios accesibles a los agentes de usuarios. La identidad procede de la sesión. En ejecución individual, se reutiliza la conexión del modelo rápido de conversación.

En el servicio compartido, el operador configura `FENIX_SENTINEL_CONFIG` con la ruta de un JSON privado, con permisos 0600, que contiene `provider`, `model`, `reasoning_effort`, `base_url` y `api_key`. El despliegue actual utiliza GPT-6 Luna con pensamiento bajo. Véase el ejemplo y los detalles operativos en la [documentación en inglés](../en/skill-marketplace.md). Si no se define la variable, se usa `skill-exchange/sentinel.json` junto al registro del gateway, cuando existe. No existe un endpoint para que un usuario cambie estas credenciales.

La misma versión genera una sola revisión aunque se pulse repetidamente. El hash abarca todos los archivos y sus indicadores de ejecución. Se permiten cinco versiones nuevas por hora, veinte al día y dos pendientes por usuario, con una cola global de 64 trabajos y dos trabajadores. Hay un límite de 120 solicitudes por minuto y cuenta. Cada revisión usa como máximo cuatro llamadas al modelo. Los intentos están limitados y se conserva la cola en SQLite.

Los errores y las revisiones interrumpidas nunca equivalen a aprobación. Un resultado tardío no puede volver a publicar algo retirado. El análisis puede consumir tokens aunque el proveedor falle o la solicitud se interrumpa. Los límites son del servicio de publicación; no modifican el presupuesto del agente para sus tareas.

## Pruebas y alcance

Las pruebas con un endpoint simulado de Nemotron comprueban revisión, correcciones, evidencia, aislamiento de usuarios, deduplicación, límites y retirada de publicaciones. Las pruebas de navegador verifican las interacciones de la interfaz. Estas pruebas no miden la precisión real del modelo como auditor: eso requiere un conjunto etiquetado de skills legítimos y maliciosos.

También pasaron dos comprobaciones reales con `nemotron-3-5-lightning-30b`: revisión de un ejemplo legítimo y corrección de una instrucción maliciosa mediante cuatro llamadas. Se conservó la capacidad de preparar informes y el original quedó intacto. Son comprobaciones sintéticas de conexión y del flujo de corrección, no una medición general de precisión de seguridad.
