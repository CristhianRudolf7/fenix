# Atlas interactivo del motor

El [atlas](/atlas/) forma parte de la documentación estática de Fénix. Presenta el recorrido de una tarea y dieciséis áreas funcionales: conversación, motor, planificación, contexto, memoria, skills, programación, procesos, navegador, búsqueda web, documentos, automatización, artefactos, seguridad, datos y evaluaciones.

## Navegación

- Selecciona un bloque del diagrama o una tarjeta para abrir su proceso.
- Sigue los pasos y las preguntas de estudio; después abre sus archivos para explorar los símbolos y el código en GitHub.
- Busca un proceso, ruta o nombre de función en todo el inventario.
- Usa acercar, alejar o restablecer en el diagrama. En pantallas pequeñas puedes desplazarlo horizontalmente sin desbordar la página.
- Comparte una sección con enlaces como `/atlas/#browser` o `/atlas/#planning`.

## Cobertura y precisión

`apps/docs/scripts/prepare-atlas.mjs` recorre código Rust y auxiliares de `crates/*/src`, `crates/*/tests`, `apps/server/src`, `apps/server/tests` y `scripts`. Cada archivo se asigna a un área; su huella y los nombres de funciones Rust se extraen sin llamadas a modelos. Los manifiestos aportan las dependencias declaradas entre crates. El frontend y los paquetes de skills quedan fuera del inventario del motor.

El mapa de procesos y sus explicaciones son documentación funcional. No representan un grafo exhaustivo de llamadas y no se deduce el comportamiento exclusivamente de imports. El listado léxico puede incluir funciones de pruebas dentro de archivos de implementación; no resuelve invocaciones dinámicas, macros ni rasgos.

La compilación reconstruye el inventario a partir del árbol local, incluidas modificaciones aún no publicadas. Los enlaces apuntan a GitHub `main`, que puede diferir hasta publicar esos cambios. Las explicaciones viven en `apps/docs/src/data/atlas.ts` y deben revisarse cuando cambia la implementación.

## Construcción y comprobación

```sh
pnpm --dir apps/docs build
```

La compilación comprueba Astro y TypeScript y verifica los hashes de scripts de la política CSP. Las pruebas de navegador se ejecutan con la documentación construida servida localmente:

```sh
FENIX_DOCS_TEST_URL=http://127.0.0.1:4186 pnpm --dir apps/web exec playwright test tests/docs-atlas.spec.js
```

Estas pruebas comprueban cobertura del inventario, selección de procesos, búsqueda de símbolos, enlaces de sección, zoom y ausencia de desbordamiento en móvil. No validan todas las funcionalidades del agente ni la producción.
