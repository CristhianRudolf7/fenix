# Fiabilidad del arnés de programación

Desplegado el 22 de septiembre de 2026. Los cambios responden a los seis hallazgos concretos de la
comparación con Luna medium del 21 de septiembre; no demuestran todavía una mejora
de puntuación o velocidad. Ambos agentes aprobaron aquella muestra. Repetir los
tests después de modificar el código de ledger era correcto.

1. **Espera de comandos:** `execution.command_yield_ms` empieza en 1000 ms. Las
   ejecuciones cortas exitosas de la misma tarea y directorio permiten adaptar la
   espera hasta 10 segundos. Los procesos interactivos o persistentes ceden en
   hasta 100 ms. `command_run.yield_ms` permite una espera explícita de 0–30000 ms.
   Esto no reduce la duración permitida del proceso ni cambia las aprobaciones.
2. **Permisos explícitos:** `execution_mode: workspace` sirve para tests y builds
   que necesitan escribir. `input_paths` declara entradas sin seleccionar permisos.
   `read_only` exige entradas y conserva las restricciones de escritura y red.
   `read_paths` antiguo sigue siendo restrictivo; nunca se amplían permisos
   automáticamente. Los fallos de ejecución y las salidas no exitosas del programa
   tienen respuestas estructuradas distintas, sin atribuir su causa sin evidencia.
3. **Procesos:** identificadores nuevos `p_` más 16 caracteres hexadecimales,
   manteniendo compatibilidad con logs UUID. Una referencia inválida devuelve los
   procesos accesibles y no repite comandos ni adivina identificadores.
4. **Entorno:** `environment_inspect` consulta manifiestos e intérpretes disponibles
   en el entorno real de ejecución. No ejecuta scripts del proyecto, no instala
   paquetes ni consulta modelos. Caché por tarea de cinco minutos, invalidada al
   cambiar manifiestos o ejecutar comandos mutables. `refresh` fuerza actualización
   ante cambios externos. Declarar una dependencia no demuestra que esté instalada.
5. **Herramientas básicas:** listado, glob, lectura, búsqueda, parches, diagnóstico
   del entorno y programas de herramientas disponibles desde el inicio cuando los
   permisos de la tarea lo permiten. Las búsquedas vacías y los argumentos fuera
   de los límites declarados se rechazan antes de ejecutar.
6. **Operaciones agrupadas:** lecturas independientes de archivos y web en paralelo,
   con resultados en orden y registro individual. Los lotes con escrituras siguen
   ordenados. Los programas validan las dependencias antes de modificar archivos;
   las referencias como `{"$ref":"paso/process_id"}` evitan copiar identificadores.

Se añaden regresiones con procesos locales y proveedor simulado. Falta repetir la
comparación y evaluar tareas nuevas para medir precisión, tiempo y tokens. No se
introduce un auditor obligatorio ni se modifica el presupuesto de contexto.

[Detalles técnicos completos en inglés](../en/coding-harness-reliability.md).
