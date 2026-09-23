(() => {
  const a = CONDITION,
    d = window.__fenixDOM;
  const es = a.selector ? d.query(a.selector) : [];
  const visible = es.some(d.visible),
    overflow = document.documentElement.scrollWidth - innerWidth;
  const content = a.selector
    ? es.map((e) => e.innerText || e.textContent || "").join("\n")
    : document.body?.innerText || "";
  const passed = {
    visible,
    hidden: !visible,
    text: content.includes(a.text || ""),
    url: location.href.includes(a.text || ""),
    ready: document.readyState === "complete",
    no_overflow: overflow <= 1,
  }[a.condition];
  return {
    passed,
    condition: a.condition,
    overflow_px: overflow,
    matches: es.length,
  };
})();
