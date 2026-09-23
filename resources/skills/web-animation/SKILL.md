---
name: web-animation
description: Implement or repair web motion with scoped timelines, scroll triggers, lifecycle cleanup and reduced-motion behavior, including GSAP. Animaciones web, parallax y ScrollTrigger.
metadata:
  version: 1.0.0
  category: engineering
  tags: animation, motion, gsap, timeline, scrolltrigger
  aliases: GSAP, animaciones web, animar, parallax, scroll, ScrollTrigger, SplitText, Flip, useGSAP
  required_tools: file_read,file_write,file_edit,command_run,tool_search
  requires: Web project and its chosen animation runtime; browser for behavior checks
  effects: local-write
---
# Web Animation

## Scope

Build purposeful motion in an existing web interface. Preserve the chosen library. CSS or the Web Animations API may suffice for a simple transition; GSAP is useful for coordinated timelines, scroll or specialized plugins. Interface layout and semantics remain with frontend-design-and-accessibility.

## Inputs

Requested interaction, framework and versions, target elements, lifecycle, responsive states and accessibility requirements. Establish the readable static state before adding motion.

## Workflow

1. Identify which elements and properties this component owns. Scope selectors to its root; avoid changing global defaults or killing other components' animations.
2. Express sequencing with a timeline and explicit relative positions. Animate transform/opacity where they satisfy the design. Measure actual bottlenecks before adding layers, batching or permanent `will-change`.
3. Put DOM work in the client lifecycle. Register required plugins, clean up animations, media-query contexts, event handlers, observers and timers. A context-safe callback records animations for cleanup; it does not cancel future events or promises for you.
4. Respect live changes to reduced-motion preferences and viewport conditions. Keep content readable with scripts disabled, motion reduced or setup failing. Avoid hiding focusable content solely to stage an entrance.
5. Test entry, exit, replay, interruption, unmount/remount and affected scroll states. Refresh scroll measurements after relevant layout changes, including loaded fonts/images, rather than on every animation frame.

## References

- [GSAP lifecycle and framework integration](references/lifecycle.md): React, Vue, Svelte and delayed callbacks.
- [Scroll, plugins and performance](references/scroll-and-plugins.md): pinning, refresh order, text/SVG/layout plugins and utility selection.
- `assets/scoped-reveal.mjs`: an optional GSAP entrance helper with explicit cleanup and reduced-motion handling; adapt it to the actual project.

## Verification

Use the browser to check visible end states, keyboard access, reduced motion and duplicate instances after navigation. Verify the actual rendering; code compilation and a still screenshot cannot establish motion behavior.

## Limits

Do not replace an established animation library or install every GSAP plugin. Fénix's static design preview disables scripts; run dynamic animation in the authorized web project's browser instead of claiming that preview executes it.

## Español

Implementa movimiento con selectores locales, secuencias claras y limpieza al desmontar. Conserva contenido visible y preferencias de movimiento reducido. Comprueba navegación, interrupción y scroll en un navegador real. La vista estática de diseños de Fénix no ejecuta estas animaciones.
