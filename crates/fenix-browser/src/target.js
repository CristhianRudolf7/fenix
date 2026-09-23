(() => {
  const target = TARGET,
    d = window.__fenixDOM;
  let e;
  if (target.ref) e = window.__fenixRefs?.get(target.ref);
  else if (target.selector) {
    const found = d.query(target.selector);
    if (found.length !== 1)
      return { error: "Selector must resolve to exactly one element" };
    e = found[0];
  } else e = d.active();
  if (!e?.isConnected)
    return { error: "Stale or missing target; observe again" };
  if (target.scroll)
    e.scrollIntoView({
      block: "center",
      inline: "center",
      behavior: "instant",
    });
  const r = e.getBoundingClientRect(),
    point = d.point(e);
  window.__fenixTarget = e;
  return {
    ...point,
    width: r.width,
    height: r.height,
    visible: d.visible(e),
    disabled: d.disabled(e),
    readonly: !!e.readOnly,
    secret: e.matches("input[type=password],iframe"),
    tag: e.tagName.toLowerCase(),
    inputType: e.type,
  };
})();
