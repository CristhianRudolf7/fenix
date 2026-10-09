(() => {
  const root = document.documentElement;
  let savedTheme;
  try { savedTheme = localStorage.getItem('fenix-docs-theme'); } catch {}
  root.dataset.theme = savedTheme || 'light';
  document.querySelector('.theme').addEventListener('click', () => {
    root.dataset.theme = root.dataset.theme === 'dark' ? 'light' : 'dark';
    try { localStorage.setItem('fenix-docs-theme', root.dataset.theme); } catch {}
  });
  const menu = document.querySelector('.menu');
  const sidebar = document.querySelector('.sidebar');
  menu.addEventListener('click', () => {
    if (!sidebar) { window.location.href = '/docs/inicio/'; return; }
    const open = sidebar.classList.toggle('open');
    menu.setAttribute('aria-expanded', String(open));
  });
  document.addEventListener('keydown', (event) => {
    if (event.key === 'Escape') { sidebar?.classList.remove('open'); menu.setAttribute('aria-expanded','false'); }
  });
  document.querySelectorAll('[data-page]').forEach(link => {
    if (link.tagName === 'A' && link.dataset.page === document.body.dataset.page) {
      link.classList.add('active'); link.setAttribute('aria-current','page');
    }
  });
  const outline = document.getElementById('outline-links');
  document.querySelectorAll('.prose h2').forEach(heading => {
    const link = document.createElement('a');link.href = `#${heading.id}`;link.textContent = heading.textContent;outline?.append(link);
  });
  document.querySelectorAll('.prose table').forEach(table => {
    const wrapper = document.createElement('div');wrapper.className = 'table-wrap';table.before(wrapper);wrapper.append(table);
  });
  document.querySelectorAll('.prose pre').forEach(pre => {
    const button = document.createElement('button');button.className = 'copy-code';button.textContent = 'Copiar';button.setAttribute('aria-label','Copiar código');
    const text = pre.textContent;
    button.addEventListener('click', async () => {
      try { await navigator.clipboard.writeText(text);button.textContent='Copiado'; }
      catch { button.textContent='No se pudo copiar'; }
      setTimeout(() => {button.textContent='Copiar';},1800);
    });pre.append(button);
  });
  const dialog = document.querySelector('.search-dialog');
  const input = document.getElementById('search-input');
  const results = document.getElementById('search-results');
  let index;
  const normalize = value => value.toLowerCase().normalize('NFD').replace(/[\u0300-\u036f]/g,'');
  async function openSearch() {
    if (!dialog.open) dialog.showModal();input.focus();
    if (!index) {
      results.textContent='Preparando búsqueda…';
      try {
        const response=await fetch('/docs-assets/search.json');if(!response.ok)throw new Error('Search unavailable');
        index=await response.json();showResults();
      } catch {results.textContent='La búsqueda no está disponible. Puedes navegar por las guías.';}
    } else showResults();
  }
  function showResults() {
    if (!index) return;
    const terms=normalize(input.value.trim()).split(/\s+/).filter(Boolean);
    const matches=index.map(page=>{
      const title=normalize(page.title),text=normalize(page.text);
      return {...page,score:terms.length?terms.reduce((total,term)=>total+(title.includes(term)?12:0)+(text.includes(term)?1:0),0):page.url.includes('/referencia/')?0:1};
    }).filter(page=>page.score>0).sort((a,b)=>b.score-a.score).slice(0,10);
    results.replaceChildren();
    if(!matches.length){results.textContent='No encontramos resultados. Prueba otras palabras.';return;}
    matches.forEach(page=>{
      const link=document.createElement('a');link.href=page.url;
      const title=document.createElement('strong');title.textContent=page.title;
      const description=document.createElement('p');description.textContent=page.description;
      link.append(title,description);results.append(link);
    });
  }
  document.querySelector('.search-open').addEventListener('click',openSearch);
  document.querySelector('[data-close-search]').addEventListener('click',()=>dialog.close());
  dialog.addEventListener('click',event=>{if(event.target===dialog){const rect=dialog.getBoundingClientRect();if(event.clientX<rect.left||event.clientX>rect.right||event.clientY<rect.top||event.clientY>rect.bottom)dialog.close();}});
  input.addEventListener('input',showResults);
  document.addEventListener('keydown',event=>{if((event.ctrlKey||event.metaKey)&&event.key.toLowerCase()==='k'){event.preventDefault();openSearch();}});
})();
