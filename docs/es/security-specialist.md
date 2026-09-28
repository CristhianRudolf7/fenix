# Tareas de seguridad en el catálogo compartido

El maestro y los subagentes normales pueden descubrir el catálogo completo de
skills habilitados, incluidos los de seguridad. Ya no existe un tipo de subagente
exclusivo para seguridad. Para una auditoría se delega un objetivo concreto, con
resultado esperado y restricciones de alcance, y se cargan los skills necesarios.

Las categorías organizan la búsqueda, no conceden permisos. Se conservan los skills
desactivados, el aislamiento por usuario y proyecto, las aprobaciones, los revisores
de planes de solo lectura y el límite de un nivel de delegación.

Cualquier tarea puede consultar informes locales SARIF/Semgrep mediante
`security_scan_read` dentro de sus rutas autorizadas. Los resultados son candidatos
con cobertura explícita. Los subagentes pueden adjuntar un `security_report`
estructurado; cuando se proporciona, se valida su alcance, cobertura y evidencias.
Se mantienen las comprobaciones normales de finalización y requerimientos.

Los perfiles de seguridad históricos siguen siendo legibles, pero no restringen
el catálogo ni las herramientas. Las nuevas asignaciones usan el perfil normal,
incluso cuando revisan seguridad. El filtro independiente para publicar skills
se conserva.
