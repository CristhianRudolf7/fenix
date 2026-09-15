(() => {
  const target = TARGET;
  let e;
  if (target.ref) e = window.__fenixRefs?.get(target.ref);
  else if (target.selector) {
    const roots=[document],found=[];
    for(let i=0;i<roots.length && i<100;i++) {
      found.push(...roots[i].querySelectorAll(target.selector));
      for(const n of roots[i].querySelectorAll('*')) if(n.shadowRoot) roots.push(n.shadowRoot);
    }
    if(found.length!==1) return {error:'Selector must resolve to exactly one element'};
    e=found[0];
  } else {e=document.activeElement;while(e?.shadowRoot?.activeElement)e=e.shadowRoot.activeElement;}
  if(!e || !e.isConnected) return {error:'Stale or missing target; observe again'};
  if(target.scroll) e.scrollIntoView({block:'center',inline:'center',behavior:'instant'});
  const r=e.getBoundingClientRect();
  const x=Math.max(0,Math.min(innerWidth-1,r.x+r.width/2));
  const y=Math.max(0,Math.min(innerHeight-1,r.y+r.height/2));
  let hit=document.elementFromPoint(x,y);
  while(hit?.shadowRoot?.elementFromPoint(x,y) && hit.shadowRoot.elementFromPoint(x,y)!==hit) hit=hit.shadowRoot.elementFromPoint(x,y);
  window.__fenixTarget=e;
  return {x,y,width:r.width,height:r.height,visible:!!e.getClientRects().length && getComputedStyle(e).visibility!=='hidden',
    covered:!!hit && hit!==e && !e.contains(hit),disabled:!!e.disabled,secret:e.matches('input[type=password],iframe'),tag:e.tagName.toLowerCase()};
})()
