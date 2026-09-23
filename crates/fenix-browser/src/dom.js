// Shared isolated-world DOM semantics for actions, observations and conditions.
if (!window.__fenixDOM) {
  const parent = (e) =>
    e?.assignedSlot || e?.parentElement || e?.getRootNode()?.host;
  const contains = (owner, node) => {
    for (let n = node; n; n = parent(n)) if (n === owner) return true;
    return false;
  };
  const query = (selector, limit = 2000) => {
    const roots = [document],
      found = [];
    let visited = 0;
    for (let i = 0; i < roots.length && i < 100; i++) {
      for (const e of roots[i].querySelectorAll(selector)) {
        found.push(e);
        if (found.length >= limit) return found;
      }
      const walker = document.createTreeWalker(
        roots[i],
        NodeFilter.SHOW_ELEMENT,
      );
      for (
        let e = walker.nextNode();
        e && visited < 20000;
        e = walker.nextNode()
      ) {
        visited++;
        if (e.shadowRoot) roots.push(e.shadowRoot);
      }
    }
    return found;
  };
  const active = () => {
    let e = document.activeElement;
    while (e?.shadowRoot?.activeElement) e = e.shadowRoot.activeElement;
    return e;
  };
  const disabled = (e) => {
    for (let n = e; n; n = parent(n))
      if (n.inert || n.getAttribute?.("aria-disabled") === "true") return true;
    return e.matches(":disabled");
  };
  const visible = (e) =>
    [...e.getClientRects()].some((r) => r.width > 0 && r.height > 0) &&
    !["hidden", "collapse"].includes(getComputedStyle(e).visibility);
  const hit = (x, y) => {
    let n = document.elementFromPoint(x, y);
    while (n?.shadowRoot) {
      const next = n.shadowRoot.elementFromPoint(x, y);
      if (!next || next === n) break;
      n = next;
    }
    return n;
  };
  const point = (e) => {
    for (const r of e.getClientRects()) {
      let left = Math.max(0, r.left),
        right = Math.min(innerWidth, r.right),
        top = Math.max(0, r.top),
        bottom = Math.min(innerHeight, r.bottom);
      for (let n = parent(e); n; n = parent(n)) {
        const css = getComputedStyle(n),
          box = n.getBoundingClientRect();
        const sx = n.offsetWidth ? box.width / n.offsetWidth : 1,
          sy = n.offsetHeight ? box.height / n.offsetHeight : 1;
        if (/hidden|clip|scroll|auto/.test(css.overflowX)) {
          left = Math.max(left, box.left + n.clientLeft * sx);
          right = Math.min(
            right,
            box.left + (n.clientLeft + n.clientWidth) * sx,
          );
        }
        if (/hidden|clip|scroll|auto/.test(css.overflowY)) {
          top = Math.max(top, box.top + n.clientTop * sy);
          bottom = Math.min(
            bottom,
            box.top + (n.clientTop + n.clientHeight) * sy,
          );
        }
      }
      if (right <= left || bottom <= top) continue;
      for (const [fx, fy] of [
        [0.5, 0.5],
        [0.2, 0.2],
        [0.8, 0.2],
        [0.2, 0.8],
        [0.8, 0.8],
      ]) {
        const x = left + (right - left) * fx,
          y = top + (bottom - top) * fy;
        if (contains(e, hit(x, y))) return { x, y, covered: false };
      }
    }
    const r = e.getBoundingClientRect();
    return { x: r.x + r.width / 2, y: r.y + r.height / 2, covered: true };
  };
  window.__fenixDOM = Object.freeze({
    parent,
    contains,
    query,
    active,
    disabled,
    visible,
    point,
  });
}
