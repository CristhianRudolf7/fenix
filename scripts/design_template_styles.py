"""Original template compositions. No external assets, fonts, or copied layouts."""
from html import escape

PALETTES = {
    'midnight': ('#181b30', '#f6f3ff', '#b9a0ff', '#292d49'),
    'cobalt': ('#1535cd', '#ffffff', '#d4ff74', '#2949df'),
    'coral': ('#ffe6da', '#3c2425', '#c34930', '#f9c9b7'),
    'forest': ('#173d33', '#f2f0dc', '#d5e8a0', '#285044'),
    'plum': ('#482747', '#fff0e7', '#ffab8e', '#613858'),
    'sand': ('#f0e8d8', '#352b23', '#a44e2c', '#e4d8bf'),
    'mint': ('#dff2e7', '#153d35', '#246b55', '#c6e3d1'),
    'ink': ('#f4f2ed', '#222630', '#465cbb', '#e3e4eb'),
}


def art(theme, variant=0):
    bg, fg, accent, soft = PALETTES[theme]
    if variant % 5 == 0:
        shapes = ''.join(f'<circle cx="260" cy="250" r="{r}" fill="none" stroke="{accent}" stroke-width="26"/>' for r in [200, 158, 116, 74])
        shapes += f'<path d="M260 20V480M20 250H500" stroke="{bg}" stroke-width="15"/><circle cx="408" cy="99" r="30" fill="{fg}"/>'
    elif variant % 5 == 1:
        shapes = ''.join(f'<rect x="{50+i*63}" y="{135-i*19}" width="85" height="{220+i*8}" rx="42" fill="{accent}" opacity="{.3+i*.16}" transform="rotate(22 260 250)"/>' for i in range(5))
        shapes += f'<circle cx="103" cy="73" r="32" fill="{fg}"/>'
    elif variant % 5 == 2:
        shapes = f'<path d="M65 440V220a195 195 0 01390 0v220Z" fill="{accent}"/><path d="M136 440V224a124 124 0 01248 0v216Z" fill="{bg}"/><circle cx="260" cy="301" r="79" fill="{fg}"/><path d="M25 455H495" stroke="{fg}" stroke-width="2"/>'
    elif variant % 5 == 3:
        shapes = f'<ellipse cx="260" cy="278" rx="211" ry="140" fill="{soft}"/><ellipse cx="260" cy="240" rx="174" ry="118" fill="{fg}"/><ellipse cx="260" cy="227" rx="128" ry="83" fill="{accent}"/>'
        shapes += ''.join(f'<ellipse cx="{x}" cy="{y}" rx="43" ry="19" fill="{bg}" transform="rotate({r} {x} {y})"/>' for x,y,r in [(198,216,-35),(284,245,25),(303,194,-25)])
        shapes += f'<path d="M140 395Q230 350 357 400" fill="none" stroke="{accent}" stroke-width="5"/>'
    else:
        shapes = f'<path d="M80 400L260 60L445 400Z" fill="{accent}"/><circle cx="260" cy="279" r="101" fill="{bg}"/><circle cx="260" cy="279" r="65" fill="{fg}"/><path d="M65 439H455" stroke="{fg}" stroke-width="2"/>'
    return f'<svg viewBox="0 0 520 500" xmlns="http://www.w3.org/2000/svg" aria-label="Original geometric illustration" role="img">{shapes}</svg>'


CSS = '''*{box-sizing:border-box}body{margin:0;background:var(--brand-bg,var(--paper));color:var(--brand-text,var(--ink));font:15px/1.55 var(--brand-font,Arial,sans-serif)}nav{padding:22px 5%;display:flex;justify-content:space-between;align-items:center;border-bottom:1px solid color-mix(in srgb,var(--ink) 18%,transparent);font-size:11px}nav b{font-size:20px;letter-spacing:-1px}nav span{opacity:.75}main{max-width:1300px;margin:auto;padding:34px 5% 40px}h1{font-size:clamp(35px,6.4vw,78px);font-weight:600;line-height:1.01;letter-spacing:-.065em;margin:20px 0 25px;text-wrap:balance}h2{line-height:1.2;letter-spacing:-.035em;font-size:26px}h3{font-size:17px;line-height:1.25;margin:12px 0}p{margin:12px 0}small,.eyebrow{font-size:10px;letter-spacing:.16em;text-transform:uppercase}.eyebrow{display:flex;gap:9px;align-items:center}.eyebrow:before{content:"";width:7px;height:7px;background:var(--accent);border-radius:50%}.lead{font-size:17px;opacity:.76;max-width:380px}.hero{display:grid;grid-template-columns:1.1fr 1fr;gap:35px;align-items:center;min-height:370px}.hero svg{width:100%;max-height:420px}.serif h1,.serif h2{font-family:Georgia,serif;font-weight:400;letter-spacing:-.055em}.pill,button{display:inline-block;border:0;border-radius:30px;background:var(--brand-accent,var(--accent));color:var(--paper);padding:13px 23px;font:600 12px Arial,sans-serif}.button-light{background:var(--ink);color:var(--paper)}.line{border-top:1px solid color-mix(in srgb,var(--ink) 20%,transparent);margin-top:28px;padding-top:20px;display:flex;justify-content:space-between;gap:20px}.grid{display:grid;grid-template-columns:repeat(3,1fr);gap:18px}.card{padding:24px;border-radius:16px;background:var(--soft);color:var(--ink)}.card p{font-size:12px;opacity:.75}.card b.metric{font-size:42px;line-height:1.2;letter-spacing:-.06em;display:block;margin:12px 0}.card small{font-size:9px}.center{text-align:center}.center .lead{margin:0 auto 28px;max-width:600px}.center h1{max-width:850px;margin:20px auto}.tag{padding:5px 10px;background:var(--soft);border-radius:5px;font-size:10px}.visual{border-radius:20px;overflow:hidden;background:var(--soft)}.visual svg{display:block;width:100%}.project-grid{display:grid;grid-template-columns:1.3fr 1fr;gap:20px;margin-top:28px}.project-grid .visual{height:260px}.project-grid svg{height:100%;width:100%}.project-grid .caption{display:flex;justify-content:space-between;margin-top:10px;font-size:11px}.wide-title{font-size:clamp(40px,8vw,95px)}.light-card{background:#fff;color:#29322f}.dark-card{background:var(--ink);color:var(--paper)}.mock-app{border:1px solid color-mix(in srgb,var(--ink) 20%,transparent);border-radius:18px;background:var(--soft);padding:24px;box-shadow:0 25px 70px #0002;transform:rotate(-3deg)}.mock-head{display:flex;gap:6px;margin-bottom:24px}.mock-head i{width:7px;height:7px;border-radius:50%;background:var(--accent)}.spark{display:flex;align-items:end;gap:10px;height:150px;padding-top:20px}.spark i{flex:1;background:var(--accent);border-radius:6px 6px 0 0}.row{display:flex;justify-content:space-between;align-items:center;gap:15px}.tiny{font-size:11px;opacity:.7}.badge{display:inline-block;padding:5px 10px;border:1px solid currentColor;border-radius:20px;font-size:10px}.price{font-size:49px;letter-spacing:-.06em;line-height:1.2;margin:20px 0}.price em{font-size:12px;font-style:normal;letter-spacing:0}.list{list-style:none;margin:20px 0;padding:0;font-size:12px;line-height:2.3}.list li:before{content:"✓";margin-right:9px}.agenda{display:grid;grid-template-columns:70px 1fr auto;gap:18px;padding:22px 0;border-top:1px solid color-mix(in srgb,var(--ink) 20%,transparent)}.agenda h3{margin:0}.workspace{display:grid;grid-template-columns:170px 1fr;border-radius:16px;overflow:hidden;border:1px solid #dce2e0;background:#fff;color:#24302b}.workspace aside{background:#f1f4f1;padding:24px 20px}.workspace aside p{font-size:12px;margin:18px 0}.workspace article{padding:28px;min-width:0}.workspace h1{font-size:34px;letter-spacing:-.04em}.workspace h2{font-size:20px}.workspace pre{background:#192b25;color:#d9f2b5;border-radius:10px;padding:20px;white-space:pre-wrap;font:12px/1.8 monospace}.kanban{display:grid;grid-template-columns:repeat(3,1fr);gap:15px}.lane{background:#eff2f7;border-radius:12px;padding:14px}.task{background:white;padding:18px 14px;border-radius:9px;margin-top:13px;box-shadow:0 3px 8px #18283408}.task h3{font-size:13px}.avatar{display:inline-grid;place-items:center;width:24px;height:24px;border-radius:50%;background:var(--accent);color:var(--paper);font-size:9px}.schedule-grid{display:grid;grid-template-columns:repeat(4,1fr);gap:9px}.slot{border:1px solid #cad6ce;border-radius:7px;text-align:center;padding:12px;font-size:12px}.selected{background:var(--accent)!important;color:var(--paper)!important}.field{background:#f4f5f3;border:1px solid #d9dfda;border-radius:7px;padding:10px 12px;margin:8px 0 20px;font-size:13px}.toggle{display:inline-block;width:30px;height:17px;border-radius:20px;background:#326c50;padding:3px}.toggle:after{content:"";display:block;width:11px;height:11px;background:white;border-radius:50%;margin-left:auto}.orbital{position:relative}.orbital .badge{position:absolute;bottom:25px;left:10px}.stats{display:flex;gap:35px}.stats b{display:block;font-size:28px;letter-spacing:-.05em}.stats span{font-size:10px;opacity:.75}footer{padding:18px 5%;font-size:9px;letter-spacing:.09em;border-top:1px solid color-mix(in srgb,var(--ink) 15%,transparent);opacity:.7}button{cursor:default}.chips{display:flex;gap:8px;flex-wrap:wrap}.rule{width:45px;height:4px;background:var(--accent);margin:24px 0}@media(max-width:600px){main{padding:28px 6%}.hero{grid-template-columns:1fr;gap:12px}.hero svg{max-height:280px}.grid,.project-grid,.kanban{grid-template-columns:1fr}.workspace{grid-template-columns:1fr}.workspace aside{display:none}.project-grid .visual{height:230px}nav span{display:none}.stats{gap:18px}h1{font-size:45px}.schedule-grid{grid-template-columns:repeat(2,1fr)}}'''


def cards(data):
    return '<div class="grid">'+''.join(f'<div class="card"><small>{a}</small><h3>{b}</h3><p>{c}</p></div>' for a,b,c in data)+'</div>'


def web_design(slug):
    specs = {
        'saas-launch': ('midnight','Orbit','A little less busy.\nA lot more possible.'),
        'research-lab': ('ink','Horizon Lab','The next question\nchanges everything.'),
        'consulting': ('sand','Forma / Studio','Clarity is\na competitive edge.'),
        'portfolio': ('coral','Mara — Independent designer','Good ideas.\nUnexpected forms.'),
        'restaurant': ('forest','Savia','From the earth.\nFor the table.'),
        'product-store': ('sand','Everyday Objects','Less, but\nbetter considered.'),
        'conference': ('cobalt','OFFSCRIPT / 2026','A gathering for\nwhat comes next.'),
        'nonprofit': ('mint','Common Ground','A brighter future\nstarts on your street.'),
        'documentation': ('ink','Forma / Developers','Build your first workspace.'),
        'analytics-dashboard': ('ink','Pulse / Analytics','A clearer view of your growth.'),
        'project-dashboard': ('ink','Studio / Workspace','Make good work happen.'),
        'pricing-page': ('midnight','Orbit / Plans','Small start.\nRoom to grow.'),
        'course-page': ('plum','Open School','Stay curious.\nMake something new.'),
        'newsletter': ('sand','The Margin','Ideas worth\nmaking time for.'),
        'appointment': ('mint','Still / Wellbeing','A little time,\njust for you.'),
        'account-settings': ('ink','Studio / Account','Make yourself at home.'),
    }
    theme,brand,headline=specs[slug]
    bg,fg,accent,soft=PALETTES[theme]
    title=f'<h1>{escape(headline).replace(chr(10),"<br>")}</h1>'
    eyebrow='<div class="eyebrow">Independent thinking. Thoughtful design.</div>'
    if slug=='saas-launch':
        body=f'<div class="hero"><div><div class="eyebrow">A calmer way to work</div>{title}<p class="lead">Bring your projects, people and next big idea into one focused space.</p><p><span class="pill">Meet your workspace ↗</span></p><div class="line"><small>YOUR NEXT CHAPTER, TOGETHER.</small></div></div><div class="mock-app"><div class="mock-head"><i></i><i></i><i></i></div><div class="row"><small>PROJECT OVERVIEW</small><span class="badge">This week</span></div><h2>Good work, in motion.</h2><div class="spark">'+''.join(f'<i style="height:{v}%"></i>' for v in [30,55,43,78,63,91,82])+'</div><div class="line"><div><b>12</b><br><small>Projects</small></div><div><b>86%</b><br><small>On track</small></div><div><b>+24%</b><br><small>Momentum</small></div></div></div></div>'
    elif slug=='research-lab':
        body=f'<div class="hero serif"><div><div class="eyebrow">Horizon Lab · Field report 08</div>{title}<p class="lead">At the intersection of human curiosity and scientific discovery.</p><div class="line"><span>Explore our research</span><span>↗</span></div></div><div class="orbital">{art("ink",0)}<span class="badge">Open science. Wider horizons.</span></div></div>'+cards([('01 / DISCOVERY','Questions before answers','Mapping the edges of what we know.'),('02 / METHODS','Evidence in the open','Transparent experiments and shared tools.'),('03 / COMMUNITY','Better, together','A network of curious minds.')])
    elif slug=='consulting':
        body=f'<div class="hero serif"><div>{eyebrow}{title}<p class="lead">Strategy, identity and digital experiences for teams ready to move forward.</p><span class="pill">Start a conversation ↗</span></div><div>{art(theme,2)}</div></div><div class="line stats"><div><b>01</b><span>Find the real question</span></div><div><b>02</b><span>Shape the direction</span></div><div><b>03</b><span>Make it happen</span></div></div>'
    elif slug=='portfolio':
        body=f'<div class="row"><div>{eyebrow}{title}</div><span class="badge">Available for collaborations ↗</span></div><div class="project-grid"><div><div class="visual" style="background:#482747">{art("plum",1)}</div><div class="caption"><b>Kindred — Brand identity</b><span>2026 ↗</span></div></div><div><div class="visual" style="background:#1535cd">{art("cobalt",2)}</div><div class="caption"><b>Volume — Digital experience</b><span>2026 ↗</span></div></div></div>'
    elif slug=='restaurant':
        body=f'<div class="hero serif"><div><div class="eyebrow">Seasonal kitchen · Neighborhood table</div>{title}<p class="lead">A changing menu, local ingredients, and the pleasure of a long lunch.</p><span class="pill">Discover the menu ↗</span><div class="line"><small>TUESDAY — SUNDAY</small><small>12:00 — 21:00</small></div></div><div>{art(theme,3)}</div></div>'
    elif slug=='product-store':
        body=f'<div class="row serif"><div><div class="eyebrow">The slow-living collection</div>{title}</div><p class="lead">Useful objects.<br>Made to stay.</p></div><div class="grid">'+''.join(f'<div><div class="visual" style="background:{PALETTES[t][0]}">{art(t,v)}</div><div class="caption row"><h3>{name}</h3><span>↗</span></div><small>{desc}</small></div>' for t,v,name,desc in [('coral',2,'Arc vessel','Sculptural forms'),('forest',1,'Ridge collection','Natural textures'),('ink',4,'Balance objects','Everyday rituals')])+'</div>'
    elif slug=='conference':
        body=f'<div class="hero"><div><div class="eyebrow">Design, technology & the in-between</div>{title}<div class="chips"><span class="badge">18—19 SEP</span><span class="badge">LIMA + ONLINE</span></div><p><span class="pill">Explore the program ↗</span></p></div><div>{art(theme,1)}</div></div><div class="line"><small>TWO DAYS. MANY PERSPECTIVES.</small><small>ORIGINAL SAMPLE PROGRAM</small></div>'
    elif slug=='nonprofit':
        body=f'<div class="hero"><div><div class="eyebrow">Small actions. Shared possibilities.</div>{title}<p class="lead">Growing greener spaces and stronger connections, one neighborhood at a time.</p><span class="pill">Find your next small action ↗</span></div><div>{art(theme,2)}</div></div>'+cards([('GROW','Make room for green','Community gardens, shared responsibility.'),('CONNECT','Meet your neighbors','Bring a skill. Find a friend.'),('CARE','Leave things better','Practical projects for lasting change.')])
    elif slug=='documentation':
        body=f'<div class="workspace"><aside><b>Developer guide</b><p>◉ Overview</p><p>Getting started</p><p>Authentication</p><p>Projects</p><p>API reference</p><div class="rule"></div><small>VERSION 2.0</small></aside><article><small>GUIDES / QUICKSTART</small>{title}<p>From a blank canvas to your first project in three clear steps.</p><pre><span style="color:#86a293"># Create a new project</span>\nproject init my-workspace\n\n<span style="color:#86a293"># Start building</span>\nproject dev</pre>{cards([("01","Install","Set up your environment."),("02","Connect","Add your workspace."),("03","Create","Bring your idea to life.")])}</article></div>'
    elif slug=='analytics-dashboard':
        body=f'<div class="row"><div><div class="eyebrow">Workspace overview</div><h1 style="font-size:40px">{headline}</h1></div><span class="badge">Last 30 days ↗</span></div><div class="grid">'+''.join(f'<div class="card light-card"><small>{l}</small><b class="metric">{v}</b><span class="tiny">{d} · sample comparison</span></div>' for l,v,d in [('TOTAL REVENUE','$24,680','↑ 12.8%'),('ACTIVE CUSTOMERS','1,248','↑ 8.2%'),('CONVERSION','4.8%','↑ 0.6 pts')])+'</div><div class="project-grid"><div class="card light-card"><div class="row"><h3>Revenue over time</h3><small>ILLUSTRATIVE DATA</small></div><svg viewBox="0 0 600 190" width="100%"><path d="M0 145H600M0 95H600M0 45H600" stroke="#e9eeea"/><path d="M10 160L70 130L130 141L190 88L250 107L310 63L370 82L430 45L490 53L590 10V185H10Z" fill="#e2ebfa"/><path d="M10 160L70 130L130 141L190 88L250 107L310 63L370 82L430 45L490 53L590 10" fill="none" stroke="#4965c4" stroke-width="4"/></svg></div><div class="card dark-card"><small>LOOKING AHEAD</small><h2>Your next insight<br>starts here.</h2><p>Replace the sample metrics with your own verified data.</p><span class="badge">Explore the report ↗</span></div></div>'
    elif slug=='project-dashboard':
        body=f'<div class="row"><div><div class="eyebrow">Studio / Website launch</div><h1 style="font-size:43px">{headline}</h1></div><span class="badge">Team workspace</span></div><div class="line" style="margin:10px 0 25px"><span>Board view</span><small>6 TASKS · SAMPLE PROJECT</small></div><div class="kanban">'+''.join('<div class="lane"><div class="row"><b>'+state+'</b><span class="tiny">02</span></div>'+''.join(f'<div class="task"><span class="tag">{tag}</span><h3>{name}</h3><div class="row"><span class="tiny">{date}</span><span class="avatar">{owner}</span></div></div>' for tag,name,date,owner in tasks)+'</div>' for state,tasks in [('To explore',[('Research','Map the customer journey','Sep 24','AR'),('Strategy','Align on the project brief','Sep 25','SK')]),('In progress',[('Design','Build the visual language','Sep 26','LM'),('Content','Shape the launch story','Sep 27','AR')]),('Ready',[('Planning','Define success measures','Sep 20','SK'),('Discovery','Gather the first insights','Sep 21','LM')])])+'</div>'
    elif slug=='pricing-page':
        body=f'<div class="center"><div class="eyebrow" style="justify-content:center">A workspace for every chapter</div>{title}<p class="lead">Clear plans for individuals, growing teams and bigger ambitions.</p></div><div class="grid">'+''.join(f'<div class="card" style="background:{color};color:{text}"><div class="row"><small>{name}</small><span class="badge">{tag}</span></div><div class="price">{price}<em> / month</em></div><p>{desc}</p><ul class="list"><li>Focused workspace</li><li>{projects} projects</li><li>Shared visual library</li></ul><span class="pill" style="background:{text};color:{color}">Explore {name} ↗</span></div>' for name,price,tag,desc,projects,color,text in [('Solo','$12','YOUR SPACE','A thoughtful start for personal projects.','5','#292d49','#f6f3ff'),('Team','$29','MOST FLEXIBLE','More room for your next collaboration.','25','#b9a0ff','#181b30'),('Studio','$59','ROOM TO GROW','A connected home for ambitious teams.','Unlimited','#292d49','#f6f3ff')])+'</div>'
    elif slug=='course-page':
        body=f'<div class="hero serif"><div><div class="eyebrow">Open School / New perspectives</div>{title}<p class="lead">Hands-on courses for people who learn by doing.</p><span class="pill">Find your next workshop ↗</span></div><div>{art(theme,0)}</div></div>'+cards([('DESIGN / 6 LESSONS','Think in systems','Connect ideas through visual language.'),('WRITING / 4 LESSONS','Find your voice','Make every word earn its place.'),('CREATIVE / 5 LESSONS','Start with curiosity','Turn a small question into a project.')])
    elif slug=='newsletter':
        body=f'<div class="hero serif"><div><div class="eyebrow">The Margin / Issue 024</div>{title}<p class="lead">An occasional letter on design, culture and the spaces in between.</p><div class="line"><small>A LETTER, NOT A FEED.</small><span>↗</span></div></div><div>{art(theme,4)}</div></div>'+cards([('01 / PERSPECTIVE','The art of paying attention','Why the smallest details can change our work.'),('02 / IN THE STUDIO','Objects with a second life','A conversation about making things last.'),('03 / BOOKMARKED','A few good things','Reading, listening and looking more closely.')])
    elif slug=='appointment':
        body=f'<div class="hero serif"><div><div class="eyebrow">A slower rhythm starts here</div>{title}<p class="lead">Choose a moment to pause, reflect and plan your next chapter.</p><div class="line"><span>Personal consultation</span><span>30 min</span></div></div><div class="card light-card"><div class="row"><h2>Choose a time</h2><span>↗</span></div><p class="tiny">THURSDAY / SAMPLE AVAILABILITY</p><div class="schedule-grid">'+''.join(f'<div class="slot {"selected" if i==2 else ""}">{v}</div>' for i,v in enumerate(['09:00','10:00','11:30','13:00','14:00','15:00','16:00','17:00']))+'</div><div class="line"><span class="tiny">Online · Your local time</span></div><p class="tiny">Visual example only. No appointment is booked.</p></div></div>'
    else:
        body=f'<div class="workspace"><aside><b>Your workspace</b><p>◉ Profile</p><p>Appearance</p><p>Notifications</p><p>Team members</p><p>Billing</p></aside><article><small>PERSONAL SETTINGS</small>{title}<div class="row"><div class="row"><span class="avatar" style="width:52px;height:52px;font-size:18px">AR</span><div><b>Alex Rivera</b><p class="tiny">Sample profile · Studio workspace</p></div></div><span class="badge">Edit profile</span></div><div class="line"></div><small>DISPLAY NAME</small><div class="field">Alex Rivera</div><small>WORKSPACE NAME</small><div class="field">Creative Studio</div><div class="row"><div><b>Weekly inspiration</b><p class="tiny">A small collection of ideas in your inbox.</p></div><span class="toggle"></span></div></article></div>'
    return f'<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><style>:root{{--paper:{bg};--ink:{fg};--accent:{accent};--soft:{soft}}}{CSS}</style></head><body><nav><b>{brand}</b><span>Discover &nbsp; / &nbsp; Our story &nbsp; / &nbsp; Get in touch ↗</span></nav><main>{body}</main><footer>FÉNIX ORIGINAL TEMPLATE &nbsp; / &nbsp; FICTIONAL BRANDS & SAMPLE DATA · MARCAS FICTICIAS Y DATOS DE EJEMPLO</footer></body></html>'

DECKS = {
 'research-deck': ('ink','RESEARCH / FIELD NOTES',[
  ('Better questions.\nClearer evidence.','A practical framework for turning curiosity into useful discoveries.','cover'),
  ('Start with what we do not know.','What gets in the way of a better everyday experience?\nStudy the problem before choosing the solution.','statement'),
  ('A method worth sharing.','Observe people in context.\nCompare patterns across interviews.\nTest a focused prototype.','steps'),
  ('Evidence, with perspective.','Describe what changed, who participated and what remains uncertain.\nSeparate observations from interpretations.','split'),
  ('Make the next question count.','Publish the method. Invite a different perspective. Choose one useful next experiment.','cover')]),
 'pitch-deck': ('midnight','VENTURE / PROJECT PITCH',[
  ('Good ideas\ndeserve a beginning.','A focused proposal for a more connected way to work.','cover'),
  ('Work is scattered.\nAttention is finite.','Teams spend too much energy finding context.\nOur opportunity: make the next step clear.','statement'),
  ('One space.\nA clearer direction.','Bring decisions, projects and outcomes together.\nBuild around the way small teams actually work.','split'),
  ('Start small. Learn fast.','Understand the daily workflow.\nBuild one useful pilot.\nMeasure and refine together.','steps'),
  ('Let’s build\nwhat comes next.','The next step: agree on a pilot, a team and a clear definition of success.','cover')]),
 'quarterly-review': ('forest','BUSINESS / QUARTERLY REVIEW',[
  ('Progress,\nin perspective.','A thoughtful look at what moved forward — and where we go next.','cover'),
  ('Focus creates momentum.','A stronger rhythm of delivery.\nClearer ownership across the team.\nCloser conversations with customers.','split'),
  ('Keep the useful lessons.','Make decisions visible.\nProtect time for deep work.\nReview outcomes, not only activity.','steps'),
  ('Name the risks early.','Capacity, dependencies and changing assumptions deserve a place in the conversation.','statement'),
  ('A sharper next chapter.','Choose fewer priorities. Define the evidence. Make space to learn.','cover')]),
 'training-deck': ('coral','LEARNING / HANDS-ON WORKSHOP',[
  ('Less watching.\nMore making.','A workshop for turning a new idea into a practical skill.','cover'),
  ('Learn by trying.','Start with a small challenge.\nShare your approach before looking for the perfect answer.','split'),
  ('Your first experiment.','Choose a question.\nMake a rough first version.\nAsk someone to try it.','steps'),
  ('A good question\nopens a new door.','What surprised you? What would you change? What will you try next?','statement'),
  ('Take one thing\ninto tomorrow.','A small, repeatable practice is worth more than a perfect plan.','cover')]),
 'product-demo': ('cobalt','PRODUCT / A CLOSER LOOK',[
  ('Meet your\nnext possibility.','A guided introduction to a simpler, more focused workspace.','cover'),
  ('Built for the moment\nwork gets complicated.','Give your team one place to see what matters and move forward.','statement'),
  ('From idea to outcome.','Capture the context.\nShape the work together.\nReview what changed.','steps'),
  ('Clarity you can feel.','Less context switching. More shared understanding.\nDesigned around real work, not more administration.','split'),
  ('Try your next\nproject here.','Start with one team, one workflow and one useful outcome.','cover')]),
 'proposal-deck': ('sand','STUDIO / CLIENT PROPOSAL',[
  ('A shared vision.\nA considered plan.','Strategy and design for your next chapter.','cover'),
  ('Begin with understanding.','Align on the audience, the challenge and the change we want to make.','statement'),
  ('A clear path forward.','Discover the real need.\nDesign and test the direction.\nDeliver the work with care.','steps'),
  ('Built together,\nat every step.','Shared checkpoints, transparent decisions and room for useful feedback.','split'),
  ('Ready when\nyou are.','Confirm the scope. Choose the team. Set the first conversation.','cover')]),
 'conference-talk': ('plum','IDEAS / KEYNOTE SESSION',[
  ('The future starts\nwith a better question.','Three perspectives on making meaningful things.','cover'),
  ('Curiosity is\na working method.','Pay attention to the overlooked.\nFollow the question beyond the obvious answer.','statement'),
  ('Three ideas to keep.','Notice the small friction.\nMake the invisible visible.\nLeave room for a different voice.','steps'),
  ('Make space\nfor the unexpected.','The strongest outcome may begin with the detail everyone else walked past.','split'),
  ('What will\nyou ask next?','Take the conversation beyond this room.','cover')]),
 'team-onboarding': ('mint','PEOPLE / WELCOME TO THE TEAM',[
  ('New faces.\nShared possibilities.','Your first chapter with the team starts here.','cover'),
  ('Good work\nis a team sport.','Be curious. Share your context. Ask for help early.','statement'),
  ('A thoughtful first week.','Meet the people behind the work.\nExplore one real project.\nMake a small contribution.','steps'),
  ('How we work,\ntogether.','Write down decisions. Make time to focus.\nTreat feedback as a shared tool for getting better.','split'),
  ('Bring your\nown perspective.','You do not need all the answers. A good question is a great beginning.','cover')]),
}

DOCUMENTS = {
 'executive-report': ('sand','STRATEGY / BRIEFING NOTE','A clearer direction',[('The essential view','A focused pilot gives the team a practical way to test the opportunity before committing to a wider rollout.'),('What we are learning','Customers value a clear next step and a consistent experience. The next research cycle should test those needs with a broader group.'),('Recommended next move','Agree on one measurable outcome, assign a pilot owner and review the evidence after four weeks.')]),
 'research-paper': ('ink','RESEARCH / WORKING PAPER','From question to evidence',[('Abstract','This working paper outlines a repeatable approach to understanding how small teams share project context.'),('Method','Combine contextual interviews with a focused prototype study. Record the sampling approach, conditions and sources of uncertainty.'),('Discussion','Present observed patterns alongside alternative explanations. Report limitations before making broader claims.')]),
 'meeting-record': ('mint','TEAM / DECISION RECORD','A conversation that moves work forward',[('Purpose','Align the team on the first pilot, its scope and the decisions needed to begin.'),('Decisions','Keep the first release focused on one workflow. Review the prototype with a small group before extending the scope.'),('Next actions','The project owner prepares a brief. Design shares a first prototype. The team reviews open questions at the next checkpoint.')]),
 'project-brief': ('coral','STUDIO / PROJECT BRIEF','Make the next step clear',[('The opportunity','Create a welcoming first experience that helps new users understand where to begin.'),('The work','Map the current journey, explore two directions and test one interactive prototype with representative users.'),('What success looks like','Users can explain the purpose, identify their next step and complete the first task without assistance.')]),
 'proposal-document': ('sand','PARTNERSHIP / PROPOSAL','Thoughtful work. Shared ambition.',[('Our understanding','Your team needs a clear visual direction and a practical system that can grow with the organization.'),('Proposed approach','Begin with discovery, explore a focused design direction and translate the chosen concept into reusable foundations.'),('Working together','Agree on responsibilities, deliverables and review checkpoints before the project begins. Commercial terms remain to be defined.')]),
 'operating-procedure': ('ink','OPERATIONS / FIELD GUIDE','A reliable way to deliver',[('Before you begin','Confirm the request, the responsible owner and the expected outcome. Gather the current source materials.'),('The procedure','Prepare the draft, ask a reviewer to check the result and address any outstanding questions before delivery.'),('Verification','Check the final artifact against the agreed scope. Record the outcome and retain the supporting evidence.')]),
 'case-study': ('coral','PERSPECTIVE / CASE STUDY','Small changes. A better experience.',[('The challenge','A fragmented onboarding journey left new users unsure about the first useful action.'),('The approach','The team mapped the journey, simplified the starting screen and tested clearer language with a small group.'),('What comes next','Document the observed feedback and repeat the study with a broader audience. This example does not claim measured business results.')]),
 'user-guide': ('mint','PRODUCT / QUICKSTART','Find your rhythm',[('Welcome to your workspace','Begin with one project. Give it a clear name and describe the outcome you want to achieve.'),('Your first useful action','Add the context, invite a collaborator and choose the next small task. Keep decisions close to the work.'),('Keep it simple','Review progress regularly, archive completed material and use feedback to improve the way your team works.')]),
}


def image_design(slug):
    if slug=='event-poster':
        return '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 960 600"><rect width="960" height="600" fill="#1535cd"/><g stroke="#d4ff74" stroke-width="22" fill="none">'+''.join(f'<circle cx="755" cy="305" r="{r}"/>' for r in [210,170,130,90])+'</g><rect x="35" y="35" width="890" height="530" fill="none" stroke="#ffffff66"/><g fill="white" font-family="Arial,sans-serif"><text x="65" y="92" font-size="16" letter-spacing="4">OFFSCRIPT / CREATIVE GATHERING</text><text x="62" y="260" font-size="100" font-weight="700" letter-spacing="-6">Ideas in</text><text x="62" y="365" font-size="100" font-weight="700" letter-spacing="-6">motion.</text><text x="65" y="474" font-size="24">18—19 SEP / LIMA + ONLINE</text><text x="65" y="530" font-size="12" letter-spacing="2">FÉNIX ORIGINAL · SAMPLE EVENT</text></g></svg>'
    if slug=='quote-card':
        return '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 960 600"><rect width="960" height="600" fill="#482747"/><text x="650" y="420" font-family="Georgia,serif" font-size="490" fill="#ffab8e" opacity=".35">“</text><path d="M65 75H895M65 525H895" stroke="#ffab8e88"/><g fill="#fff0e7"><text x="68" y="116" font-family="Arial" font-size="13" letter-spacing="4">A THOUGHT TO KEEP</text><text x="65" y="270" font-family="Georgia" font-size="74">Make space</text><text x="65" y="360" font-family="Georgia" font-size="74">for curiosity.</text><text x="70" y="468" font-family="Arial" font-size="15" letter-spacing="2">FÉNIX STUDIO / ORIGINAL WORDS</text></g></svg>'
    if slug=='research-cover':
        return '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 960 600"><rect width="960" height="600" fill="#f4f2ed"/><rect x="570" width="390" height="600" fill="#e3e4eb"/><g stroke="#465cbb" fill="none">'+''.join(f'<ellipse cx="765" cy="300" rx="{r}" ry="215" stroke-width="3" transform="rotate({a} 765 300)"/>' for r,a in [(160,0),(130,25),(100,50),(70,75)])+'</g><circle cx="765" cy="300" r="28" fill="#222630"/><g fill="#222630"><text x="62" y="90" font-family="Arial" font-size="14" letter-spacing="3">HORIZON / RESEARCH SERIES</text><text x="58" y="240" font-family="Georgia" font-size="76">New</text><text x="58" y="330" font-family="Georgia" font-size="76">perspectives.</text><path d="M62 390H126" stroke="#465cbb" stroke-width="6"/><text x="62" y="445" font-family="Arial" font-size="20">Questions that shape tomorrow.</text><text x="62" y="540" font-family="Arial" font-size="12" letter-spacing="2">VOLUME 01 / ORIGINAL SAMPLE</text></g></svg>'
    return '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 960 600"><rect width="960" height="600" fill="#f0e8d8"/><g font-family="Arial" fill="#352b23"><text x="60" y="70" font-size="13" letter-spacing="3">FORMA / IDENTITY SYSTEM</text><text x="55" y="175" font-size="88" font-weight="700" letter-spacing="-7">forma.</text><text x="60" y="220" font-size="18">A considered identity.</text></g><rect x="555" y="55" width="345" height="200" rx="12" fill="#173d33"/><path d="M645 210V140a75 75 0 01150 0v70M675 210V145a45 45 0 0190 0v65" fill="none" stroke="#d5e8a0" stroke-width="20"/>'+''.join(f'<rect x="{60+i*216}" y="310" width="196" height="185" rx="10" fill="{c}"/><text x="{72+i*216}" y="530" font-family="Arial" font-size="12" fill="#352b23">{name}</text>' for i,(c,name) in enumerate([('#173d33','FOREST / 173D33'),('#d5e8a0','LIME / D5E8A0'),('#c34930','TERRA / C34930'),('#352b23','INK / 352B23')]))+'</svg>'


def enhance_templates(items):
    themes=list(PALETTES)
    for index,item in enumerate(items):
        slug=item['id'];kind=item['kind'];c=item['content']
        item['version']=2
        if kind=='web':
            c['html']=web_design(slug)
        elif kind=='presentation':
            theme,eyebrow,slides=DECKS[slug]
            item['content']={'theme':theme,'eyebrow':eyebrow,'display_title':item['title'],'slides':[{'title':title,'body':body,'layout':layout,'notes':'Original sample narrative. Replace with verified project information.'} for title,body,layout in slides]}
        elif kind=='document':
            theme,eyebrow,title,sections=DOCUMENTS[slug]
            item['content']={'theme':theme,'eyebrow':eyebrow,'text':title+'\n\n'+'\n\n'.join(h+'\n'+b for h,b in sections)}
        elif kind=='image':
            c['svg']=image_design(slug)
        elif kind in ['diagram','chart','spreadsheet']:
            c['theme']=themes[index%len(themes)]
            c['eyebrow']={'diagram':'SYSTEMS / CONNECT THE DOTS','chart':'INSIGHTS / THE BIGGER PICTURE','spreadsheet':'WORKSPACE / EVERY DETAIL COUNTS'}[kind]
            c['display_title']=item['title']
            if kind=='spreadsheet':
                c['caption']='Illustrative working data · Datos de trabajo ilustrativos. No formula calculation.'
        item['description']=f'Original {item["title"].lower()} composition with editable content and a considered visual system.'
        item['description_es']=f'Composición original de {item["title_es"].lower()}, con contenido editable e identidad visual.'
