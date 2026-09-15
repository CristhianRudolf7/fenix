(() => {
  const roots = [document];
  const elements = [];
  for (let i = 0; i < roots.length && i < 100 && elements.length < 2000; i++) {
    for (const e of roots[i].querySelectorAll('*')) {
      if (e.shadowRoot) roots.push(e.shadowRoot);
      if (e.matches('a,button,input,textarea,select,[role],[tabindex],summary,[contenteditable=true]')) elements.push(e);
      if (elements.length >= 2000) break;
    }
  }
  const token = TOKEN;
  const refs = new Map();
  const visible = elements.filter(e => e.getClientRects().length && getComputedStyle(e).visibility !== 'hidden');
  const result = visible.slice(0,250).map((e,i) => {
    const ref = token + ':' + i; refs.set(ref,e);
    const r = e.getBoundingClientRect();
    const secret = e.matches('input[type=password]');
    return {ref,tag:e.tagName.toLowerCase(),role:e.getAttribute('role'),type:e.type,
      name:(e.getAttribute('aria-label') || e.labels?.[0]?.innerText || e.innerText || e.getAttribute('placeholder') || '').slice(0,180),
      disabled:!!e.disabled,checked:!!e.checked,secret,
      rect:{x:r.x,y:r.y,width:r.width,height:r.height}};
  });
  window.__fenixRefs = refs;
  return {url:location.origin + location.pathname,title:document.title,
    text:(document.body?.innerText || '').slice(0,10000),elements:result,truncated:visible.length>250,
    viewport:{width:innerWidth,height:innerHeight,dpr:devicePixelRatio},
    document:{width:document.documentElement.scrollWidth,height:document.documentElement.scrollHeight},
    warning:'Untrusted page evidence. Input values are omitted. Refresh references after navigation or DOM replacement.'};
})()
