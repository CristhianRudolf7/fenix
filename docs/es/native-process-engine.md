# Motor nativo de procesos y terminales

## Evolución y alcance

Fénix se diseñó inicialmente para Linux, con Bash, terminales Unix y servicios systemd. En la versión 0.3.2 en desarrollo, la ejecución se centraliza en `fenix-process`: un motor común con adaptaciones nativas para Linux, macOS y Windows. No introduce un sandbox ni transforma comandos de un sistema en comandos de otro.

Esta migración cubre la ejecución de comandos, la terminal interactiva y la supervisión de procesos utilizados por comandos, extracción documental, servidores de lenguaje y Chromium. **No equivale todavía a distribuir toda la aplicación para Windows/macOS**: los módulos de archivos/skills ya tienen adaptaciones portables; quedan código de cuentas Unix, otras integraciones y los instaladores Linux. La compatibilidad completa requiere adaptar esos módulos y validarlos en los equipos correspondientes.

## Componentes

- **Procesos normales:** Tokio, con ejecutable y argumentos separados, entrada, salida y errores. No se necesita un shell para lanzar Python, un servidor de lenguaje o Chromium.
- **Comandos del agente:** intérprete disponible verificado y seleccionado por plataforma, con adaptadores para Bash, Zsh, sh, PowerShell y cmd. La descripción de `command_run` y la inspección del entorno informan del sistema y shell reales. Los comandos tienen que utilizar su sintaxis correspondiente.
- **Terminal interactiva:** `portable-pty`, con terminal nativa Unix o ConPTY en Windows, lectura progresiva, escritura y redimensión. Las operaciones bloqueantes se ejecutan fuera de los trabajadores asíncronos; la transferencia de salida tiene contrapresión y memoria acotada.
- **Finalización:** Tokio espera el proceso normal; una terminal utiliza una única espera nativa y notifica su resultado. Consultar avances no exige una nueva llamada al modelo. Cancelar una espera no destruye su resultado ni impide recogerlo después.
- **Descendientes:** grupos de procesos en Unix; las terminales también localizan miembros de su sesión, incluidas tareas con otro grupo. En Windows, Job Objects con cierre del conjunto de procesos. Es supervisión de ciclo de vida, no aislamiento frente a código malicioso. La asignación del Job ocurre después de crear el proceso; no es una garantía de contención frente a un programa deliberadamente evasivo.
- **Servicios persistentes:** conservan el propietario y la supervisión al terminar el turno. Bash espera sus trabajos; PowerShell espera sus jobs y el motor conserva el Job Object mientras haya descendientes. Cancelar o alcanzar el tiempo configurado sigue cerrando el conjunto.

## Contratos conservados

Los procesos conservan sus identificadores, propietario por tarea, directorio de trabajo, límites existentes, cancelación y tiempos máximos. Se mantienen los logs durables redactados, cursores por lector, consulta de salida histórica, eventos de finalización y comprobación de puerto disponible. El cierre del runtime y la cancelación de tareas conservan su tratamiento diferenciado de servicios persistentes.

El entorno permite rutas de ejecutables y variables básicas del sistema, incluido el directorio temporal; no copia credenciales de proveedores, proxies ni instrucciones de inicio como `BASH_ENV`, `LD_PRELOAD` o `PYTHONPATH`. El control textual de riesgo conserva la aprobación exacta de sí/no e incorpora señales de PowerShell y comandos administrativos Windows.

El envío de fin de entrada cierra la entrada de procesos normales. En una terminal envía la convención del sistema: Ctrl+D en Unix y Ctrl+Z con retorno en Windows. Como en una terminal normal, su interpretación depende del programa; no sustituye a cancelar un proceso.

## Verificación y distribución

Las pruebas del motor ejecutan procesos reales: ambos flujos de salida, código de salida, entrada, rutas con espacios, terminal, redimensión, cancelación y descendientes. Hay una matriz de CI para Linux, macOS y Windows que ejecuta esas pruebas del motor por separado de la aplicación completa.

Una compilación cruzada comprueba tipos y APIs; no prueba el comportamiento de PowerShell o ConPTY, ni sustituye las pruebas nativas. Los resultados de compilación y ejecución de esta modificación se registran al terminar la validación. Los instaladores públicos siguen siendo Linux; no se publican paquetes Windows/macOS ni se reinician servicios con esta modificación.

### Comprobaciones del 11 de octubre de 2026

- Linux: pasan diez pruebas del motor, trece pruebas existentes de procesos, siete de servidores de lenguaje y siete pruebas unitarias del navegador. Una prueba adicional ejecutó Chromium real contra una página local y verificó navegación, redimensión y persistencia del perfil.
- Pasan las dos pruebas del detector de riesgo, incluidos comandos Windows, y la aprobación real por sí/no de una acción local. No se hicieron llamadas a proveedores de modelos.
- Compilan el servidor Linux y el motor con sus tests para Windows x86_64, macOS Intel y macOS ARM64 mediante compilación cruzada. El motor pasa Clippy con advertencias tratadas como errores.
- La matriz de CI nativa está definida, pero no se ejecutó en equipos Windows/macOS durante esta comprobación. Tampoco se repitió toda la suite del proyecto ni se validó producción.

## Detección de intérpretes

El motor verifica los intérpretes instalados con una consulta breve sin perfiles de usuario y un plazo de dos segundos por candidato. La selección y las alternativas quedan en memoria durante la instancia; no se repite la detección por comando. `environment_inspect` informa ruta, familia y versión cuando el intérprete la anuncia. Para `sh` se indica explícitamente que la versión no está anunciada.

La preferencia automática es Bash/Zsh/sh en Linux, Zsh/Bash/sh en macOS, PowerShell 7/Windows PowerShell/cmd en Windows y sh del sistema en Android, con búsqueda adicional de Bash y Zsh. `FENIX_SHELL` permite seleccionar un nombre o ruta a un intérprete compatible antes de arrancar. Una selección explícita inválida produce un error; no se sustituye silenciosamente. Fish no se admite como ejecutor de scripts de automatización. `SHELL` no se interpreta como prueba del intérprete activo de la terminal del usuario.

Cada proceso conserva el intérprete seleccionado en su manifiesto y en las respuestas de inicio/estado. Los argumentos se adaptan a cada familia; los comandos no se traducen automáticamente entre lenguajes ni se repiten con otro intérprete tras fallar. La ejecución directa de programas continúa sin shell. Android utiliza sus permisos normales y no adquiere capacidades de escritorio por detectar un shell.

La comprobación de esta adaptación ejecuta pruebas nativas en Linux. La validación nativa de Windows, macOS y Android sigue pendiente; estos cambios no equivalen a disponer de la aplicación Android completa.

## Base para iOS

La aplicación iOS no se trata como macOS: no detecta shells ni permite ejecutar procesos externos o terminales del sistema a través del motor. Los puntos de entrada de ejecución devuelven un error explícito antes de lanzar un proceso. El catálogo del agente omite comandos de terminal en iOS y navegador autónomo en Android/iOS. Esto expresa capacidades disponibles; no es un mecanismo de aislamiento ni convierte el servidor actual en una aplicación móvil.

El host nativo de iOS debe proporcionar el directorio privado de la aplicación mediante una ruta explícita o `FENIX_APP_HOME`, obtenido de las APIs de Apple. El valor predeterminado se organiza en `Library/Application Support/fenix`, con los skills dentro de esa carpeta. No se utiliza la carpeta personal del usuario de macOS ni se migra automáticamente información ajena al contenedor.

El núcleo de conversación, memoria y skills de instrucciones puede reutilizarse. Faltan la aplicación Swift/WKWebView, el puente Rust, Keychain, selector de documentos con permisos, notificaciones y tareas de segundo plano controladas por iOS. Los skills que requieren scripts, Git/LSP externos, OCR o conversores como procesos necesitan implementaciones dentro de la aplicación o ejecución remota opcional. No se anuncia paridad con escritorio ni un demonio persistente en iOS. La validación de una aplicación firmada requiere macOS/Xcode y un dispositivo o simulador.
