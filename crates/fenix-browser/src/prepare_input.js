(() => {
  const a = INPUT,
    e = window.__fenixTarget,
    d = window.__fenixDOM;
  if (!e?.isConnected) return { error: "Target detached; observe again" };
  if (d.disabled(e) || e.readOnly)
    return { error: "Target is disabled or read-only" };
  const input = e instanceof HTMLInputElement,
    textarea = e instanceof HTMLTextAreaElement;
  const textual = [
    "text",
    "search",
    "tel",
    "url",
    "email",
    "password",
    "number",
  ];
  const special = ["date", "time", "datetime-local", "month", "week", "color", "range"];
  if (!(
    textarea ||
    e.isContentEditable ||
    (input && [...textual, ...special].includes(e.type))
  ))
    return { error: "Target is not an editable text control" };
  if (a.action === "type" && input && special.includes(e.type))
    return { error: "Use fill with a complete value for this input type" };
  if (
    a.action === "fill" &&
    (input || textarea) &&
    e.maxLength >= 0 &&
    a.text.length > e.maxLength
  )
    return { error: "Text exceeds the input maxlength" };
  if (input && (special.includes(e.type) || e.type === "number")) {
    const probe = document.createElement("input");
    probe.type = e.type;
    probe.value = a.text;
    if (
      (a.text && !probe.value) ||
      (e.type === "color" && !/^#[0-9a-f]{6}$/i.test(a.text))
    )
      return { error: "Value is invalid for this input type" };
  }
  e.focus();
  if (!e.isConnected || (d.active() !== e && !d.contains(e, d.active())))
    return {
      error: "Target changed or could not receive focus; observe again",
    };
  if (input && special.includes(e.type)) {
    Object.getOwnPropertyDescriptor(
      HTMLInputElement.prototype,
      "value",
    ).set.call(e, a.text);
    e.dispatchEvent(new Event("input", { bubbles: true, composed: true }));
    e.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
    return { native: false };
  }
  if (a.action === "fill") {
    if (input || textarea) e.select();
    else {
      const r = document.createRange();
      r.selectNodeContents(e);
      const s = getSelection();
      s.removeAllRanges();
      s.addRange(r);
    }
  }
  return { native: true };
})();
