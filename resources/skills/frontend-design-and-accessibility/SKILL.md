---
name: frontend-design-and-accessibility
description: Build or improve web interfaces with consistent components, responsive states and accessible interaction.
metadata:
  version: 1.4.0
  category: engineering
  tags: frontend, design, and, accessibility
  aliases: frontend, interfaz, accesibilidad, react, diseño responsivo, teclado, visual hierarchy, typography, diseño visual, jerarquía, identidad de marca
  required_tools: file_read,file_edit,file_write,tool_search
  requires: Web project; browser tools for rendered verification
  effects: local-write
---
# Frontend Design And Accessibility

## Scope

Use for interface design and implementation. Use browser-navigation for browser mechanics; this skill defines what the interface should preserve and verify.

## Inputs

User flow, existing stack/design language, viewport targets and content.

## Workflow

1. Inspect the actual framework, dependency versions and existing components. Reuse established tokens and patterns instead of migrating the stack without a reason.
2. Define the flow and its loading, empty, error, disabled and success states. Use semantic controls, visible labels and meaningful heading order.
3. Build responsive layout that tolerates long text, zoom and localization. Preserve keyboard navigation, visible focus, focus return after dialogs and reduced-motion preferences.
4. Keep data requests independent where possible, cancel stale requests and avoid unnecessary global state. Measure a suspected performance issue before adding memoization.
5. Inspect rendered pages at relevant widths and navigate the key flow by keyboard. Review screenshots together with console/network errors.

For visual delivery in Fénix, discover `design_templates`, `design_present`, `design_publish` and `design_import` with `tool_search`. Present a few template choices only when the choice matters; publish the finished structured result or import its workspace file so it appears in the chat panel and Design. Use the existing design ID and current revision for updates, preserve project scope, and label sample data. HTML previews disable scripts/network; Office imports extract content rather than reproduce original layout or recalculate formulas.

For timelines, GSAP, ScrollTrigger or animation lifecycle bugs, load web-animation. Keep ordinary layout and keyboard/accessibility work here; a static Fénix design preview cannot execute the animation.

For visual redesigns or a new interface direction, load [visual decisions and comparison](references/visual-decisions.md). Resolve hierarchy, content density and actual assets against the brief; preserve established brand choices. Improve a representative rendered view before propagating changes across the interface.

When asked only to audit appearance, report located visual defects and keep source unchanged. For an authorized correction, connect each change to a user-visible issue and verify the same representative view afterward. Evaluate visual hierarchy, interaction and accessibility separately; a polished screenshot cannot establish that a form, keyboard flow or persistence works. Preserve scope when an audit reveals unrelated aesthetic opportunities.

## Verification

Check the requested behavior and existing flows, contrast, focus order, overflow and screen-reader labels. State any visual or assistive-technology checks not performed.

## Limits

A screenshot cannot prove accessibility. Do not trade readability for decorative animation or introduce a new design system for a small component change.
