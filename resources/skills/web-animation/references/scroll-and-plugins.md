# Scroll and specialized motion / Scroll y plugins

Attach a ScrollTrigger to the owning top-level animation; avoid a child tween competing with its parent timeline for playback control. Pin a stable wrapper and animate descendants. Build triggers in document order so earlier pin spacing is included in later measurements. If explicit `refreshPriority` is needed, **higher values refresh earlier**, not lower ones. Refresh after relevant layout/font/image changes; `update()` alone does not recompute start/end geometry.

Use SplitText only when text segmentation is needed; preserve semantic text and accessible naming, rebuild for relevant font/layout changes and revert the split. Use Flip for layout-state transitions, MotionPath for a specified path, and SVG morph/draw plugins for those actual operations. Register only the used plugins and verify installed-version imports. Plugin licensing is separate from the MIT license of the upstream skill text; consult the library's current terms when distribution matters.

Use `gsap.utils` for normalization, clamping, wrapping or scoped target selection when they simplify an actual animation. Use reusable setters/tweens for frequent pointer updates; avoid constructing a timeline each frame. Measure frame work, layout and paint on the target device. Applying `will-change`, `force3D`, pinning or numerical scrub does not itself prove a performance improvement.

Verify keyboard navigation, resize, back/forward navigation, removed targets and reduced-motion changes. Dispose only this component's instances; global `killAll()` can break unrelated interfaces.

Sources: [ScrollTrigger sort and refresh order](https://gsap.com/docs/v3/Plugins/ScrollTrigger/static.sort()/), [ScrollTrigger](https://gsap.com/docs/v3/Plugins/ScrollTrigger/).

ES: usa un propietario por secuencia, conserva el orden de recalculado y mide rendimiento real. Una prioridad mayor se recalcula antes. Elimina únicamente los efectos del componente y conserva accesibilidad en texto y navegación.
