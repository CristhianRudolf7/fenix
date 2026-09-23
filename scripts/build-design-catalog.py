#!/usr/bin/env python3
"""Build original Fénix example templates. No downloads or provider dependencies."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
items = []
def add(slug, kind, en, es, content, tags):
    items.append(dict(id=slug, kind=kind, title=en, title_es=es, description=f'Editable {en.lower()} example with sample content.', description_es=f'Ejemplo editable de {es.lower()} con contenido de muestra.', tags=tags, version=1, source='Fénix original', content=content))

webs = [
 ('saas-launch','SaaS launch','Lanzamiento SaaS','Build less. Accomplish more.','hero'),
 ('research-lab','Research laboratory','Laboratorio de investigación','Research with an open horizon.','editorial'),
 ('consulting','Consulting studio','Estudio de consultoría','A clear path to your next chapter.','split'),
 ('portfolio','Creative portfolio','Portafolio creativo','Selected work, considered details.','gallery'),
 ('restaurant','Restaurant','Restaurante','Seasonal food. A place to gather.','menu'),
 ('product-store','Product storefront','Tienda de productos','Everyday objects, thoughtfully made.','gallery'),
 ('conference','Conference','Conferencia','Ideas worth sharing.','schedule'),
 ('nonprofit','Community organization','Organización comunitaria','Small actions. Lasting change.','hero'),
 ('documentation','Documentation portal','Portal de documentación','Everything you need to get started.','docs'),
 ('analytics-dashboard','Analytics dashboard','Dashboard de analítica','Your business at a glance.','dashboard'),
 ('project-dashboard','Project workspace','Panel de proyectos','Move work forward together.','kanban'),
 ('pricing-page','Pricing comparison','Comparación de planes','Choose the plan that fits.','pricing'),
 ('course-page','Course catalog','Catálogo de cursos','Make room for something new.','gallery'),
 ('newsletter','Editorial newsletter','Boletín editorial','Notes from the field.','editorial'),
 ('appointment','Booking interface','Interfaz de reservas','Make time for what matters.','booking'),
 ('account-settings','Account settings','Configuración de cuenta','Your workspace, your preferences.','settings'),
]
for slug,en,es,headline,layout in webs:
    add(slug,'web',en,es,{'html':''},[layout,'web',es.lower()])

diagrams=[
 ('decision-flow','Decision flow','Flujo de decisiones','flowchart TD\n A[New request] --> B{Complete information?}\n B -->|Yes| C[Process request]\n B -->|No| D[Ask for details]\n D --> B\n C --> E[Deliver result]'),
 ('login-sequence','Login sequence','Secuencia de inicio de sesión','sequenceDiagram\n participant U as User\n participant W as Web\n participant A as Auth\n U->>W: Submit credentials\n W->>A: Validate\n alt Valid\n A-->>W: Session\n W-->>U: Welcome\n else Invalid\n A-->>W: Error\n W-->>U: Try again\n end'),
 ('service-architecture','Service architecture','Arquitectura de servicios','flowchart LR\n U[Client] --> G[Gateway]\n G --> A[Application]\n A --> Q[Job queue]\n Q --> W[Worker]\n A --> D[(Database)]\n W --> D'),
 ('project-entities','Project entities','Entidades de proyecto','erDiagram\n USER ||--o{ PROJECT : owns\n PROJECT ||--o{ TASK : contains\n TASK ||--o{ RESULT : produces'),
 ('task-states','Task lifecycle','Ciclo de vida de tareas','stateDiagram-v2\n [*] --> Queued\n Queued --> Running\n Running --> Waiting\n Waiting --> Running\n Running --> Completed\n Running --> Failed\n Completed --> [*]'),
 ('delivery-timeline','Delivery timeline','Cronograma de entrega','gantt\n title Sample project\n dateFormat YYYY-MM-DD\n section Discovery\n Research :a, 2026-01-01, 5d\n section Delivery\n Prototype :after a, 7d\n Review :2026-01-13, 3d'),
 ('team-structure','Team structure','Estructura de equipo','flowchart TD\n L[Project lead] --> D[Design]\n L --> E[Engineering]\n L --> R[Research]\n E --> F[Frontend]\n E --> B[Backend]'),
 ('research-map','Research map','Mapa de investigación','mindmap\n root((Research question))\n  Evidence\n   Literature\n   Experiments\n  Method\n   Dataset\n   Evaluation\n  Findings\n   Limitations\n   Next steps'),
 ('approval-flow','Review workflow','Flujo de revisión','flowchart LR\n D[Draft] --> R[Review]\n R --> C{Changes needed?}\n C -->|Yes| D\n C -->|No| P[Publish]'),
 ('customer-journey','Customer journey','Recorrido del cliente','journey\n title Sample customer journey\n section Discover\n Find product: 4: Customer\n Read guide: 3: Customer\n section Use\n Start trial: 5: Customer\n Complete task: 5: Customer'),
]
for slug,en,es,source in diagrams:add(slug,'diagram',en,es,{'source':source},['diagram',es.lower()])
for slug,en,es,sections in [
 ('research-deck','Research findings','Resultados de investigación',['Question','Method','Evidence','Limitations','Next steps']),
 ('pitch-deck','Project pitch','Presentación de proyecto',['Problem','Opportunity','Approach','Plan','Request']),
 ('quarterly-review','Quarterly review','Revisión trimestral',['Overview','Results','Learnings','Risks','Priorities']),
 ('training-deck','Training workshop','Taller de capacitación',['Objective','Example','Practice','Discussion','Takeaway']),
 ('product-demo','Product demonstration','Demostración de producto',['Audience','Challenge','Walkthrough','Benefits','Next steps']),
 ('proposal-deck','Client proposal','Propuesta para cliente',['Context','Scope','Deliverables','Timeline','Decision']),
 ('conference-talk','Conference talk','Charla de conferencia',['Opening','Context','Three ideas','Example','Conclusion']),
 ('team-onboarding','Team onboarding','Bienvenida al equipo',['Welcome','People','Ways of working','First week','Resources']),
]:add(slug,'presentation',en,es,{'slides':[]},['slides',es.lower()])
for slug,en,es,style,labels,values in [
 ('weekly-sales','Weekly sales','Ventas semanales','bar',['Mon','Tue','Wed','Thu','Fri'],[12,19,16,24,21]),
 ('growth-trend','Growth trend','Tendencia de crecimiento','line',['Jan','Feb','Mar','Apr','May'],[20,24,22,35,42]),
 ('expense-breakdown','Expense breakdown','Distribución de gastos','pie',['Operations','Tools','Research','Other'],[40,25,20,15]),
 ('experiment-results','Experiment comparison','Comparación experimental','bar',['Baseline','Method A','Method B'],[62,74,79]),
 ('response-time','Response time','Tiempo de respuesta','line',['1','2','3','4','5'],[120,95,110,80,85]),
 ('channel-mix','Acquisition channels','Canales de adquisición','pie',['Search','Direct','Referrals'],[45,35,20]),
 ('cash-flow','Cash flow','Flujo de caja','bar',['Jan','Feb','Mar','Apr'],[12,-4,8,20]),
 ('survey-results','Survey responses','Respuestas de encuesta','bar',['Useful','Neutral','Not useful'],[38,12,5]),
]:add(slug,'chart',en,es,{'chart_type':style,'labels':labels,'values':values,'unit':'sample units','caption':'Illustrative data, not measured results.'},['statistics',style,es.lower()])
for slug,en,es,heads in [
 ('executive-report','Executive report','Informe ejecutivo',['Summary','Findings','Recommendations']),
 ('research-paper','Research paper outline','Esquema de artículo',['Abstract','Methods','Results','Limitations']),
 ('meeting-record','Meeting record','Acta de reunión',['Participants','Decisions','Action items']),
 ('project-brief','Project brief','Ficha de proyecto',['Objective','Scope','Deliverables','Success criteria']),
 ('proposal-document','Business proposal','Propuesta comercial',['Context','Proposed work','Timeline','Terms']),
 ('operating-procedure','Operating procedure','Procedimiento operativo',['Purpose','Prerequisites','Procedure','Verification']),
 ('case-study','Case study','Caso de estudio',['Challenge','Approach','Evidence','Lessons']),
 ('user-guide','User guide','Guía de usuario',['Overview','Getting started','Common tasks','Troubleshooting']),
]:add(slug,'document',en,es,{'text':''},['document',es.lower()])
for slug,en,es,cols,rows in [
 ('budget-sheet','Project budget','Presupuesto de proyecto',['Item','Quantity','Unit cost','Total'],[['Research',2,120,240],['Design',3,100,300],['Review',1,80,80]]),
 ('task-tracker','Task tracker','Seguimiento de tareas',['Task','Owner','Status'],[['Define scope','Alex','Completed'],['Prototype','Sam','In progress'],['Review','Lee','Planned']]),
 ('expense-register','Expense register','Registro de gastos',['Date','Category','Amount'],[['2026-01-05','Travel',45],['2026-01-06','Tools',29]]),
 ('inventory-sheet','Inventory','Inventario',['SKU','Item','Stock','Reorder at'],[['A-01','Notebook',42,10],['A-02','Pen',18,20]]),
 ('research-evidence','Evidence matrix','Matriz de evidencia',['Source','Method','Finding','Limitation'],[['Study A','Experiment','Example finding','Small sample'],['Study B','Survey','Example finding','Selection bias']]),
 ('content-planner','Content planner','Calendario de contenidos',['Date','Channel','Topic','Status'],[['2026-01-10','Blog','Introduction','Draft'],['2026-01-15','Newsletter','Update','Planned']]),
 ('vendor-comparison','Vendor comparison','Comparación de proveedores',['Vendor','Cost','Delivery','Notes'],[['Vendor A',100,'5 days','Sample'],['Vendor B',120,'3 days','Sample']]),
 ('experiment-log','Experiment log','Registro de experimentos',['Run','Configuration','Score','Seconds'],[['R1','Baseline',0.62,12],['R2','Variant',0.74,15]]),
]:add(slug,'spreadsheet',en,es,{'columns':cols,'rows':rows,'caption':'Example values. Formulas are not recalculated.'},['table',es.lower()])
for slug,en,es,headline in [('event-poster','Event poster','Cartel de evento','Ideas in motion'),('quote-card','Quote card','Tarjeta de cita','Make space for curiosity.'),('research-cover','Research cover','Portada de investigación','New perspectives'),('brand-board','Brand board','Panel de marca','A considered identity')]:
 add(slug,'image',en,es,{'svg':''},['image',es.lower()])
add('motion-intro','video','Motion introduction','Introducción animada',{'sample':'motion-intro','caption':'Original silent motion example, 4 seconds.'},['video','motion'])
add('data-motion','video','Data animation','Animación de datos',{'sample':'data-motion','caption':'Original silent data animation, 4 seconds.'},['video','statistics'])
from design_template_styles import enhance_templates
enhance_templates(items)
from design_collection import enhance_collection
enhance_collection(items)
assert len(items)==72, len(items)
target = ROOT/'resources/designs/catalog.json'
serialized = json.dumps(items,ensure_ascii=False,indent=2)+'\n'
if not target.exists() or target.read_text() != serialized:
    target.write_text(serialized)
print(f'Wrote {len(items)} original templates')
