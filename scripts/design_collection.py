"""Third original collection: distinct compositions, authored locally for Fénix.

References inform broad art directions only. No upstream code, fonts or assets are used.
"""
from html import escape

DECK_COMPOSITIONS = dict(zip(
    ['research-deck','pitch-deck','quarterly-review','training-deck','product-demo','proposal-deck','conference-talk','team-onboarding'],
    ['research-atlas','venture-stack','business-ledger','workshop-notes','product-stage','studio-arch','type-poster','welcome-collage']))
DOCUMENT_COMPOSITIONS = dict(zip(
    ['executive-report','research-paper','meeting-record','project-brief','proposal-document','operating-procedure','case-study','user-guide'],
    ['executive-columns','academic-paper','meeting-ledger','project-poster','proposal-letter','procedure-steps','case-editorial','guide-chapters']))
CHART_COMPOSITIONS = dict(zip(
    ['weekly-sales','growth-trend','expense-breakdown','experiment-results','response-time','channel-mix','cash-flow','survey-results'],
    ['weekly-columns','growth-area','expense-ring','experiment-dots','latency-step','channel-stack','cash-balance','survey-rank']))
SHEET_COMPOSITIONS = dict(zip(
    ['budget-sheet','task-tracker','expense-register','inventory-sheet','research-evidence','content-planner','vendor-comparison','experiment-log'],
    ['budget-ledger','task-board','expense-receipt','inventory-label','evidence-notebook','content-calendar','vendor-matrix','experiment-terminal']))


def scene(kind):
    """Original SVG scenes, built from geometric primitives, with no stock imagery."""
    if kind == 'landscape':
        body='<rect width="1000" height="500" fill="#e8d7ba"/><circle cx="760" cy="125" r="77" fill="#b24b30"/><path d="M0 380L245 75L560 390L810 180L1000 335V500H0Z" fill="#b19a7b"/><path d="M0 415L340 235L650 435L850 330L1000 405V500H0Z" fill="#727b68"/><path d="M0 480Q450 310 1000 465V500H0Z" fill="#2e4b40"/>'
    elif kind == 'building':
        body='<rect width="1000" height="500" fill="#e1e4de"/><path d="M120 395L540 455L915 300L500 245Z" fill="#abb4a8"/><path d="M170 190L545 245V435L170 380Z" fill="#f8f8ef"/><path d="M545 245L835 125V320L545 435Z" fill="#a6b5a5"/><path d="M170 190L460 70L835 125L545 245Z" fill="#fffdf4"/><g stroke="#566b59" stroke-width="5">'+''.join(f'<path d="M{580+i*45} {245-i*18}V{394-i*18}"/>' for i in range(5))+'</g><path d="M235 223L470 258V365L235 330Z" fill="#364c40"/><path d="M352 240V348" stroke="#c7cdbb" stroke-width="5"/><path d="M235 291L470 324" stroke="#c7cdbb" stroke-width="5"/>'
    elif kind == 'orbit':
        body='<rect width="1000" height="500" fill="#102324"/><g fill="none" stroke="#91b8a1">'+''.join(f'<ellipse cx="500" cy="250" rx="{r}" ry="{r/3}" transform="rotate({a} 500 250)"/>' for r,a in [(410,-20),(330,25),(230,75)])+'</g><circle cx="500" cy="250" r="102" fill="#b8d09c"/><path d="M465 155Q615 240 465 345Q380 230 465 155Z" fill="#79967a"/><circle cx="820" cy="96" r="11" fill="#faf6d8"/><path d="M790 96H850M820 66V126" stroke="#faf6d8"/>'
    elif kind == 'garden':
        body='<rect width="1000" height="500" fill="#e3e8c9"/><path d="M0 100L1000 380M150 0L450 500M600 0L880 500" stroke="#faf8ed" stroke-width="65"/>'+''.join(f'<g transform="translate({x} {y})"><rect x="-30" y="-20" width="60" height="65" fill="#889b6d"/><circle cy="-35" r="57" fill="{c}"/><path d="M0 10V105" stroke="#465239" stroke-width="6"/></g>' for x,y,c in [(120,330,'#abc565'),(340,170,'#397153'),(590,360,'#7fa349'),(780,165,'#315943'),(925,380,'#abc565')])
    elif kind == 'plate':
        body='<rect width="1000" height="500" fill="#153c31"/><ellipse cx="500" cy="260" rx="360" ry="210" fill="#6c8d65"/><ellipse cx="500" cy="230" rx="310" ry="185" fill="#eee8d6"/><ellipse cx="500" cy="230" rx="235" ry="132" fill="#d1dba2"/>'+''.join(f'<ellipse cx="{x}" cy="{y}" rx="64" ry="26" transform="rotate({a} {x} {y})" fill="#305b3d"/>' for x,y,a in [(410,210,-35),(565,270,25),(535,165,-12)])+'<circle cx="430" cy="285" r="21" fill="#c5753b"/><circle cx="630" cy="220" r="18" fill="#c5753b"/>'
    elif kind == 'contour':
        body='<rect width="1000" height="500" fill="#f1f0e8"/><g fill="none" stroke="#2e5c59" stroke-width="2">'+''.join(f'<path d="M{-100+i*22} 500C{150+i*15} {-160+i*18},{550+i*16} {590-i*16},{780+i*14} 0"/>' for i in range(24))+'</g><circle cx="580" cy="180" r="13" fill="#b7573c"/><path d="M580 180L720 110H920" fill="none" stroke="#b7573c"/><text x="730" y="94" font-family="monospace" font-size="15" fill="#284442">FIELD STATION / 01</text>'
    else:
        body='<rect width="1000" height="500" fill="#211c29"/><g fill="none" stroke="#e0f768" stroke-width="14">'+''.join(f'<path d="M0 {80+i*40}Q250 {330-i*30} 500 {190+i*20}T1000 {65+i*45}"/>' for i in range(9))+'</g>'
    return '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1000 500" role="img" aria-label="Original Fénix vector illustration">'+body+'</svg>'

BASE = '''*{box-sizing:border-box}body{margin:0;background:var(--brand-bg,var(--paper));color:var(--brand-text,var(--ink));font:15px/1.5 var(--brand-font,Arial,sans-serif)}header{display:flex;justify-content:space-between;align-items:center;padding:22px 5%;border-bottom:1px solid currentColor;font-size:11px}header strong{font-size:18px;letter-spacing:-.05em}h1,h2,h3,p{margin:0}h1{font-size:clamp(38px,7.5vw,96px);line-height:1;letter-spacing:-.06em;text-wrap:balance}h2{font-size:28px;line-height:1.1;letter-spacing:-.035em}p{line-height:1.6}small{font:10px/1.5 monospace;letter-spacing:.13em;text-transform:uppercase}svg{display:block;width:100%;height:auto}main{padding:35px 5%}.button{display:inline-block;background:var(--brand-accent,var(--accent));color:var(--paper);padding:12px 20px;border-radius:3px;font-size:12px;font-weight:700;margin-top:20px}footer{padding:17px 5%;font:9px/1.5 monospace;border-top:1px solid currentColor}.serif{font-family:Georgia,serif;font-weight:400}.line{border-top:1px solid;padding-top:16px}.muted{opacity:.65}.columns{display:grid;grid-template-columns:repeat(3,1fr);gap:25px}.columns p{font-size:13px;margin-top:12px}.strip{display:flex;justify-content:space-between;gap:15px;padding:16px 0;border-top:1px solid} @media(max-width:600px){header{padding:18px 6%}header span{display:none}main{padding:26px 6%}.columns{grid-template-columns:1fr}h1{font-size:46px}.strip{flex-wrap:wrap}}'''


def web_page(slug):
    if slug=='research-lab':
        brand='HORIZON / FIELD INSTITUTE';colors=('#f1f0e8','#233d3b','#ad4d35')
        css='.mast{text-align:center;max-width:800px;margin:0 auto 28px}.mast h1{font:400 clamp(40px,7vw,86px)/1.05 Georgia,serif;margin:16px 0}.atlas{height:220px;overflow:hidden;border:1px solid #233d3b}.atlas svg{height:100%;object-fit:cover}.columns{margin-top:24px}'
        body='<div class="mast"><small>OBSERVATION · METHOD · DISCOVERY</small><h1>The next question<br>changes everything.</h1></div><div class="atlas">'+scene('contour')+'</div><div class="columns"><div><small>01 / FIELDWORK</small><p>Research begins with attention.</p></div><div><small>02 / OPEN METHODS</small><p>Evidence everyone can examine.</p></div><div><small>03 / SHARED KNOWLEDGE</small><p>Discovery grows through exchange.</p></div></div>'
    elif slug=='consulting':
        brand='FORMA / INDEPENDENT ADVISORY';colors=('#efeae0','#282b27','#ac4d31')
        css='.intro{display:grid;grid-template-columns:2fr 1fr;gap:30px;align-items:end;margin-bottom:32px}.intro p{font-size:14px;max-width:250px}.service{display:grid;grid-template-columns:80px 1fr 45px;align-items:center;border-top:1px solid;padding:20px 0}.service b{font:42px Georgia}.service h2{font:32px Georgia}.service em{font-size:28px;font-style:normal}.service:nth-child(3){background:#283b33;color:#eee9dc;padding-left:20px;padding-right:20px}@media(max-width:600px){.intro{grid-template-columns:1fr}.service{grid-template-columns:50px 1fr 24px}.service h2{font-size:26px}}'
        body='<div class="intro"><h1 class="serif">Clarity is a<br>competitive edge.</h1><p>Independent thinking.<br>Practical progress.<br>A partner for the decisions that matter.</p></div>'+''.join(f'<div class="service"><b>0{i+1}</b><h2>{s}</h2><em>↗</em></div>' for i,s in enumerate(['Find the right question','Set a clear direction','Turn decisions into progress']))
    elif slug=='restaurant':
        brand='SAVIA / SEASONAL KITCHEN';colors=('#153c31','#f3ead6','#d7dda2')
        css='.menu-title{display:flex;align-items:end;justify-content:space-between;gap:30px}.menu-title h1{font:400 clamp(65px,16vw,170px)/.85 Georgia,serif;letter-spacing:-.07em}.menu-title p{max-width:210px;font-size:14px}.table{display:grid;grid-template-columns:1fr 3fr 1fr;align-items:center;margin-top:15px}.table p{font:18px/1.7 Georgia,serif}.table svg{width:120%;margin-left:-10%}.table small{display:block;margin-bottom:18px}.table aside:last-child{text-align:right}@media(max-width:600px){.menu-title{display:block}.menu-title p{margin-top:20px}.table{grid-template-columns:1fr 1fr}.table svg{grid-column:1/-1;grid-row:1;width:100%;margin:0}.table p{font-size:15px}}'
        body='<div class="menu-title"><h1>Savia.</h1><p>From the earth.<br>For the table.<br>A seasonal kitchen in your neighborhood.</p></div><div class="table"><aside><small>FROM THE GARDEN</small><p>Fresh leaves<br>Roasted roots<br>Herbs & citrus</p></aside>'+scene('plate')+'<aside><small>STAY A WHILE</small><p>Long lunches<br>Good company<br>Slow Sundays</p></aside></div><div class="strip"><small>TUESDAY — SUNDAY / 12:00 — 21:00</small><span>Find a place at our table ↗</span></div>'
    elif slug=='conference':
        brand='OFFSCRIPT / LIMA 2026';colors=('#f4f260','#191e1d','#191e1d')
        css='main{display:grid;grid-template-columns:3fr 1fr;gap:35px}.poster h1{font-size:clamp(58px,11vw,136px);font-weight:900;text-transform:uppercase;letter-spacing:-.075em;line-height:.87}.poster>p{max-width:420px;margin-top:30px;font-size:19px}aside{border-left:2px solid;padding-left:25px;display:flex;flex-direction:column;justify-content:space-between}.session{padding:18px 0;border-top:1px solid}.session b{font-size:32px;display:block}.ticket{font:90px/.9 Georgia,serif}.stamp{margin-top:30px;background:#191e1d;color:#f4f260;padding:12px;font:12px monospace;transform:rotate(-3deg)}@media(max-width:600px){main{grid-template-columns:1fr}.poster h1{font-size:65px}aside{border-left:0;padding:0;display:grid;grid-template-columns:1fr 1fr;gap:20px}.ticket{font-size:60px}}'
        body='<div class="poster"><small>DESIGN / CULTURE / THE UNEXPECTED</small><h1 style="margin-top:25px">Beyond<br>the usual.</h1><p>A gathering for the ideas that do not fit inside the lines.</p><div class="stamp">18—19 SEPTEMBER / TWO DAYS, MANY PERSPECTIVES</div></div><aside><div class="ticket">02<br><small>DAYS OF POSSIBILITY</small></div><div><div class="session"><small>DAY ONE</small><b>Question.</b></div><div class="session"><small>DAY TWO</small><b>Make.</b></div></div></aside>'
    elif slug=='nonprofit':
        brand='COMMON GROUND';colors=('#fbf9ef','#294b34','#294b34')
        css='.map{position:relative}.map svg{height:330px;object-fit:cover}.map h1{position:absolute;left:4%;top:8%;max-width:470px;background:#fbf9ef;padding:22px;font-size:58px;box-shadow:7px 7px 0 #294b34}.join{display:grid;grid-template-columns:1fr 2fr;gap:40px;margin-top:26px}.join p{font-size:15px}@media(max-width:600px){.map h1{font-size:36px;max-width:85%}.join{grid-template-columns:1fr;gap:18px}}'
        body='<div class="map">'+scene('garden')+'<h1>A greener street.<br>A closer community.</h1></div><div class="join"><h2 class="serif">Change starts<br>close to home.</h2><div><p>We bring neighbors together to care for shared spaces. A garden, a conversation, a small act that makes a place feel like ours.</p><span class="button">Find your next small action ↗</span></div></div>'
    elif slug=='course-page':
        brand='OPEN SCHOOL / CREATIVE PRACTICE';colors=('#f8f4ed','#29242c','#573952')
        css='.school-title{display:grid;grid-template-columns:2fr 1fr;gap:30px;align-items:end;margin-bottom:30px}.school-title h1{font:400 clamp(45px,7vw,80px)/1 Georgia,serif}.classes{display:grid;grid-template-columns:1fr 1fr 1fr;gap:12px}.lesson{padding:25px 22px;background:#e6bdb1;min-height:230px;display:flex;flex-direction:column;justify-content:space-between}.lesson:nth-child(2){background:#d5daec;transform:translateY(24px)}.lesson:nth-child(3){background:#d8dfbe}.lesson b{font:68px/.9 Georgia,serif}.lesson h2{font-size:24px;margin:28px 0 12px}.lesson small{font-size:9px}@media(max-width:600px){.school-title{grid-template-columns:1fr}.classes{grid-template-columns:1fr}.lesson:nth-child(2){transform:none}.lesson{min-height:180px}}'
        body='<div class="school-title"><h1>Learn something.<br>Make it yours.</h1><p>Short workshops.<br>Real practice.<br>A different way to begin.</p></div><div class="classes">'+''.join(f'<article class="lesson"><b>0{i+1}</b><div><h2>{t}</h2><small>{d}</small></div></article>' for i,(t,d) in enumerate([('Think in systems','DESIGN / SIX PRACTICAL LESSONS'),('Find your voice','WRITING / FOUR STUDIO SESSIONS'),('Start with curiosity','CREATIVE PRACTICE / FIVE EXPERIMENTS')]))+'</div>'
    elif slug=='newsletter':
        brand='THE MARGIN / AN INDEPENDENT LETTER';colors=('#f2eddf','#32302a','#9c4931')
        css='.masthead{text-align:center;border-bottom:3px double;padding-bottom:20px}.masthead h1{font:400 clamp(56px,12vw,140px)/1 Georgia,serif}.newspaper{display:grid;grid-template-columns:1fr 2fr 1fr;gap:24px;margin-top:26px}.newspaper h2{font:28px/1.1 Georgia,serif;margin:12px 0}.newspaper p{font-size:13px}.feature{border-left:1px solid;border-right:1px solid;padding:0 24px}.feature svg{height:170px;object-fit:cover;margin:15px 0}.feature h2{font-size:36px}.issue{font:70px Georgia,serif}@media(max-width:600px){.newspaper{grid-template-columns:1fr}.feature{border:0;padding:18px 0;border-top:1px solid}.masthead h1{font-size:56px}}'
        body='<div class="masthead"><small>A LETTER ABOUT DESIGN, CULTURE & EVERYDAY LIFE</small><h1>The Margin</h1><div class="strip"><small>VOLUME 01</small><small>TAKE YOUR TIME</small><small>ISSUE 024</small></div></div><div class="newspaper"><aside><small>THE EDITOR’S NOTE</small><h2>Look a little closer.</h2><p>There is a story in the things we pass every day. This issue is an invitation to pay attention.</p></aside><article class="feature"><small>THIS WEEK / PERSPECTIVE</small><h2>Ideas worth making time for.</h2>'+scene('landscape')+'<p>A field guide to slowing down, looking carefully and finding something unexpected.</p></article><aside><div class="issue">24</div><small>IN THIS EDITION</small><p class="line">The art of attention</p><p class="line">Objects with a second life</p><p class="line">A few good things</p></aside></div>'
    elif slug=='architecture-studio':
        brand='UMBRAL / ARCHITECTURE';colors=('#f5f4e9','#263c32','#263c32')
        css='.architecture{display:grid;grid-template-columns:1fr 1.7fr;gap:30px}.architecture h1{font:400 clamp(45px,7vw,82px)/1.04 Georgia,serif;position:relative;z-index:1}.architecture p{margin-top:25px;max-width:260px;font-size:14px}.building{margin-top:20px;align-self:center}.building svg{width:120%;margin-left:-12%}.project{display:flex;justify-content:space-between;border-top:1px solid;padding-top:15px;margin-top:25px;font-size:11px}@media(max-width:600px){.architecture{grid-template-columns:1fr}.building svg{width:100%;margin:0}}'
        body='<div class="architecture"><div><small>SPACES FOR A MORE CONSIDERED LIFE</small><h1 style="margin-top:25px">Built around<br>the way<br>we live.</h1><p>Architecture that listens to the landscape, the light and the people who call it home.</p></div><div class="building">'+scene('building')+'<div class="project"><span>HOUSE 01 / COURTYARD STUDY</span><span>EXPLORE ↗</span></div></div></div>'
    elif slug=='orbital-mission':
        brand='APOGEE / EXPLORATION LAB';colors=('#102324','#e9f1dd','#c6d991')
        css='main{padding-top:20px}.mission{display:grid;grid-template-columns:1fr 1fr;align-items:end;gap:40px}.mission h1{font-size:76px}.mission p{max-width:280px;justify-self:end;font-size:14px}.orbit{position:relative}.orbit svg{height:310px;object-fit:contain}.coordinates{position:absolute;bottom:12px;left:0;display:flex;flex-direction:column;gap:8px}.mission-label{position:absolute;right:0;top:30px;border:1px solid #c6d991;padding:16px;font:12px monospace}@media(max-width:600px){.mission{grid-template-columns:1fr;gap:20px}.mission h1{font-size:48px}.mission p{justify-self:start}.orbit svg{height:260px}.mission-label{display:none}}'
        body='<div class="mission"><h1>Beyond the<br>known horizon.</h1><p>A research studio exploring the systems that connect us to the world beyond.</p></div><div class="orbit">'+scene('orbit')+'<div class="coordinates"><small>EXPLORATION SERIES / 01</small><small>ORIGINAL CONCEPT VISUALIZATION</small></div><div class="mission-label">MISSION<br>OPEN DISCOVERY ↗</div></div>'
    elif slug=='sound-festival':
        brand='FRECUENCIA / SOUND & CULTURE';colors=('#211c29','#e5f567','#e5f567')
        css='.sound{position:relative;min-height:410px}.sound svg{position:absolute;inset:10px 0;width:100%;height:390px;opacity:.4}.sound h1{position:relative;z-index:1;font-size:clamp(64px,14vw,155px);font-weight:900;line-height:.85;letter-spacing:-.08em;max-width:800px;padding-top:40px}.sound .strip{position:relative;z-index:1;margin-top:35px}.marquee{border-top:1px solid;border-bottom:1px solid;padding:12px;font:12px monospace;letter-spacing:.1em;background:#e5f567;color:#211c29;transform:rotate(-2deg)}@media(max-width:600px){.sound{min-height:350px}.sound h1{font-size:clamp(45px,14vw,68px)}.sound svg{height:300px}.marquee{font-size:10px}}'
        body='<div class="sound">'+scene('sound')+'<h1>FEEL<br>EVERY<br>FREQUENCY.</h1><div class="strip"><small>LIVE SOUND / NEW PERSPECTIVES</small><small>LIMA · OCTOBER 24</small></div></div><div class="marquee">INDEPENDENT MUSIC / EXPERIMENTAL SOUNDS / COLLECTIVE ENERGY</div>'
    else:
        brand='SENDA / SLOW TRAVEL JOURNAL';colors=('#f2eadc','#343d32','#ac5337')
        css='.travel{text-align:center}.travel h1{font:400 clamp(48px,8vw,100px)/1 Georgia,serif;margin:15px auto 25px;max-width:850px}.panorama{height:245px;overflow:hidden;position:relative}.panorama svg{height:100%;object-fit:cover}.panorama:after{content:"";position:absolute;inset:0;background:linear-gradient(90deg,transparent 32%,#f2eadc 32% 33%,transparent 33% 66%,#f2eadc 66% 67%,transparent 67%)}.travel-foot{display:grid;grid-template-columns:1fr 2fr;gap:35px;margin-top:23px}.travel-foot h2{font:30px Georgia,serif}.travel-foot p{font-size:14px}@media(max-width:600px){.travel-foot{grid-template-columns:1fr}.panorama{height:200px}}'
        body='<div class="travel"><small>ROUTES LESS HURRIED / JOURNAL NO. 01</small><h1>Somewhere worth<br>taking your time.</h1></div><div class="panorama">'+scene('landscape')+'</div><div class="travel-foot"><h2>A different<br>kind of distance.</h2><p>Quiet trails, open landscapes and stories gathered along the way. A journal for people who travel to pay attention.</p></div>'
    paper,ink,accent=colors
    return f'<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><style>:root{{--paper:{paper};--ink:{ink};--accent:{accent}}}{BASE}{css}</style></head><body data-composition="{slug}"><header><strong>{brand}</strong><span>EXPLORE / ABOUT / CONTACT ↗</span></header><main>{body}</main><footer>FÉNIX ORIGINAL / FICTIONAL BRAND · MARCA FICTICIA / LOCAL SVG ARTWORK</footer></body></html>'


def new_image(slug):
    if slug=='topographic-poster':
        artwork=scene('contour').replace('viewBox="0 0 1000 500"','x="360" y="0" width="600" height="600" viewBox="0 0 1000 500" preserveAspectRatio="xMidYMid slice"')
        body='<rect width="960" height="600" fill="#f1f0e8"/>'+artwork+'<rect width="400" height="600" fill="#f1f0e8"/><g fill="#233d3b"><text x="48" y="70" font-family="monospace" font-size="13">FIELD NOTES / 001</text><text x="42" y="228" font-family="Georgia" font-size="75">Every</text><text x="42" y="310" font-family="Georgia" font-size="75">line tells</text><text x="42" y="392" font-family="Georgia" font-size="75">a story.</text><text x="48" y="525" font-family="monospace" font-size="12">FÉNIX / ORIGINAL CARTOGRAPHIC STUDY</text></g>'
    else:
        body='<rect width="960" height="600" fill="#edebdf"/><rect x="660" width="300" height="600" fill="#df543c"/><g font-family="Arial" font-weight="900" font-size="137" letter-spacing="-10" fill="#222c30"><text x="38" y="170">MAKE</text><text x="38" y="310">ROOM</text><text x="38" y="450">FOR NEW.</text></g><path d="M730 80H875V225M870 85L715 240" stroke="#edebdf" stroke-width="25" fill="none"/><text x="45" y="540" font-family="monospace" font-size="15" fill="#222c30">PERSPECTIVES / FÉNIX ORIGINAL / 2026</text>'
    return '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 960 600">'+body+'</svg>'


def enhance_collection(items):
    replacements={'research-lab','consulting','restaurant','conference','nonprofit','course-page','newsletter'}
    for item in items:
        c=item['content'];slug=item['id'];kind=item['kind'];item['version']=3
        if kind=='web':
            c['composition']=slug
            if slug in replacements:c['html']=web_page(slug)
        elif kind=='presentation':c['composition']=DECK_COMPOSITIONS[slug]
        elif kind=='document':c['composition']=DOCUMENT_COMPOSITIONS[slug]
        elif kind=='chart':c['composition']=CHART_COMPOSITIONS[slug]
        elif kind=='spreadsheet':c['composition']=SHEET_COMPOSITIONS[slug]
        elif kind=='diagram':c['composition']=slug
        else:c['composition']=slug
    def add(slug,kind,title,title_es,content,tags):
        items.append(dict(id=slug,kind=kind,title=title,title_es=title_es,description='Original local composition with editable content.',description_es='Composición original local con contenido editable.',version=3,source='Fénix original',content=content,tags=tags))
    for slug,en,es in [('architecture-studio','Architecture studio','Estudio de arquitectura'),('orbital-mission','Orbital research','Investigación orbital'),('sound-festival','Sound festival','Festival de sonido'),('travel-journal','Travel journal','Diario de viajes')]:
        add(slug,'web',en,es,dict(html=web_page(slug),composition=slug),['web','original',slug])
    for slug,en,es,theme,composition,slides in [
      ('systems-deck','Systems blueprint','Arquitectura de sistemas','cobalt','systems-blueprint',[
       ('Make the system\nunderstandable.','A technical narrative for the people who build and operate it.','cover'),
       ('Start with the boundaries.','Name the users, inputs and responsibilities.\nKeep dependencies visible.','split'),
       ('Three decisions to make.','Define the source of truth.\nSet the failure boundary.\nAgree on recovery behavior.','steps'),
       ('Reliability is a design choice.','Trace the ordinary path and the unexpected one before shipping.','statement'),
       ('Build for the next person.','Document the contract, test the assumptions and leave a useful map.','cover')]),
      ('culture-deck','Culture journal','Diario cultural','sand','culture-journal',[
       ('A place. A story.\nA different perspective.','A visual journal about the relationship between people and place.','cover'),
       ('Look beyond the postcard.','Notice the routines, materials and details that make a place its own.','split'),
       ('A thoughtful way to explore.','Listen to the people who live here.\nRecord what you observe.\nMake room for more than one story.','steps'),
       ('The everyday is worth noticing.','A quiet detail can tell us more than a familiar landmark.','statement'),
       ('Leave with a better question.','Share the story with care, context and curiosity.','cover')])]:
        add(slug,'presentation',en,es,dict(theme=theme,composition=composition,eyebrow=en.upper()+' / FÉNIX STUDIO',display_title=en,slides=[dict(title=t,body=b,layout=l,notes='Original illustrative narrative.') for t,b,l in slides]),['slides','original',composition])
    for slug,en,es in [('topographic-poster','Topographic poster','Cartel topográfico'),('typographic-poster','Typographic manifesto','Manifiesto tipográfico')]:
        add(slug,'image',en,es,dict(svg=new_image(slug),composition=slug),['image','original','editorial'])

    # Curated additions lead each category without changing existing template IDs.
    order = {kind: i for i, kind in enumerate(["web", "diagram", "presentation", "chart", "document", "spreadsheet", "image", "video"])}
    additions = {"architecture-studio", "orbital-mission", "sound-festival", "travel-journal", "systems-deck", "culture-deck", "topographic-poster", "typographic-poster"}
    items.sort(key=lambda item: (order[item["kind"]], item["id"] not in additions))
