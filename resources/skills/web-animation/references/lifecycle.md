# Motion lifecycle / Ciclo de vida de animaciones

Choose installed APIs, then make each component own its animations and cleanup. In React, `useGSAP` from `@gsap/react` can bind a scope and revert its context; use `revertOnUpdate` when changing dependencies should rebuild the effect. With a normal effect, return `context.revert()`. In Vue or Svelte, create the context in the browser mount hook and revert it at teardown. Keep DOM-dependent execution out of SSR.

`contextSafe` associates animations created in later callbacks with a GSAP context. It does not automatically unregister listeners, cancel a timer/fetch, or make every callback inert after unmount. Remove listeners, cancel cancellable work and guard stale asynchronous results explicitly. Test a callback resolving after disposal.

Use `gsap.matchMedia` for responsive and reduced-motion conditions, and revert that media context at teardown. Do not hide content permanently in CSS to stage an entrance. Prefer reduced/no movement over a zero-duration animation that still hides or relocates meaningful content. Existing focus and accessibility semantics remain authoritative.

The optional `assets/scoped-reveal.mjs` takes a GSAP instance and a root element. It animates only `[data-reveal]` descendants under `prefers-reduced-motion: no-preference`; disposal reverts owned styles and media handlers. It does not add listeners, fetches or global defaults.

Sources: [GSAP React integration](https://gsap.com/resources/React/), [matchMedia](https://gsap.com/docs/v3/GSAP/gsap.matchMedia()/).

ES: cada componente limpia sus animaciones y suscripciones. `contextSafe` no cancela trabajo asíncrono. Conserva el estado estático legible y prueba montaje repetido, desmontaje y cambio de preferencias.
