/** Optional GSAP entrance. The static DOM remains the accessible fallback. */
export function mountReveal(gsap, root, { duration = 0.35, distance = 12 } = {}) {
  if (!root || typeof root.querySelectorAll !== "function") {
    throw new TypeError("A component root element is required");
  }
  if (!Number.isFinite(duration) || duration < 0 || !Number.isFinite(distance)) {
    throw new TypeError("Animation dimensions must be finite and duration nonnegative");
  }
  const media = gsap.matchMedia();
  media.add("(prefers-reduced-motion: no-preference)", () => {
    const targets = root.querySelectorAll("[data-reveal]");
    if (targets.length && duration > 0) {
      gsap.from(targets, { y: distance, opacity: 0, duration, stagger: 0.04 });
    }
  }, root);
  let disposed = false;
  return () => {
    if (!disposed) {
      disposed = true;
      media.revert();
    }
  };
}
