# Design and visual results

[Español](../es/design.md)

Design is a local visual workspace connected to the continuous conversation. The desktop preview opens on the right and automatically collapses the navigation menu, restoring it on close; at widths up to 900px it fills the screen and provides Back/Escape navigation. Closing a preview never cancels an agent task.

## Library and workflow

The catalog contains **72 original editable examples**: 20 web interfaces, 10 Mermaid diagrams, 10 presentations, 8 charts, 8 documents, 8 spreadsheets, 6 SVG images and 2 short silent WebM examples. Sample content and values are illustrative. No Open Design application, upstream templates or paid design service is bundled.

- **Templates:** search, category filters, paginated gallery and previews.
- **Favorites:** template choices stored in the authenticated user's database.
- **My designs:** saved results, optional project filtering and file import.
- **Brand identity:** name, logo, colors and generic font family. Apply colors/type explicitly to a saved web design or presentation; existing versions remain unchanged. The uploaded logo is available as a private asset and in the identity preview; templates do not automatically insert it.

Select a template and its destination project to make a private editable copy. The preview has content inspection/editing, download, revision selection and restoration as a new revision. Continue in chat inserts an explicit design/project reference into the draft; it does not submit a message. The source editor edits structured JSON; conversational edits are the primary route for nontechnical users.

## Viewer contracts

| Kind | Input | Viewer and boundaries |
| --- | --- | --- |
| web | `content.html` | Standalone HTML in an opaque sandbox, scripts/forms/remote resources disabled. Visual preview, not a running backend application. Relative assets must be embedded as data URLs for this version. |
| diagram | `content.source` | Mermaid, loaded lazily; strict rendering and sanitized SVG displayed as an image. Read-only zoom/pan/fullscreen navigation; content changes are made by the agent. Structured `content.graph` adds searchable nodes and references. |
| presentation | `content.slides` | `{title,body,notes?}` slides, navigation and notes. |
| chart | `content.chart_type`, `labels`, `values` | Bar, line or pie; values and labels remain available as a table. Includes negative bar values. |
| document | `content.text` or private PDF asset | Text document or browser PDF viewer. Native PDF availability depends on the browser. |
| spreadsheet | `content.columns`, `rows` | Filterable, sortable table, 50 rows per visible page. No formula engine. |
| image | `content.svg` or private image asset | SVG, PNG, JPEG, WebP or GIF display. |
| video | private asset or bundled sample | MP4/WebM with native controls and HTTP byte ranges; codec support depends on the browser. |

Import also accepts HTML, Mermaid (`.mmd`/`.mermaid`), TXT/MD, CSV and a JSON object with `title`, `kind`, `content`. DOCX/XLSX/PPTX imports show **extracted text/cells/slides**, not the original Office layout, embedded images or recalculated formulas. Extraction is bounded and a partial preview is labeled. Existing native document tools remain available to create/edit Office files. Downloads preserve supported source formats; structured slides/charts export JSON, not an implied PDF/PPTX conversion.

## Agent tools

Discover these tools through `tool_search`:

- `design_templates`: metadata pages, selected template body and saved brand metadata.
- `design_present`: show 1–8 real template choices in chat, in the order used for textual selection.
- `design_list` / `design_read`: inspect scoped results and revisions.
- `design_publish`: create a typed visual result or conditionally update an existing ID.
- `diagram_edit`: atomic, revision-checked changes to structured diagram nodes, edges and groups.
- `design_import`: snapshot an allowed workspace file into the visual workspace.

Task project scope is derived by the runtime. A model cannot retarget a saved design or read another project's result by guessing its ID. Publication returns the design identity; the web polls metadata to display new task results. The conversation model receives recent result/choice metadata as observable state. Plan review workers cannot publish effects.

## Persistence and limits

Migration `008_designs.sql` adds designs and immutable revisions to each user's SQLite database. Updates require `expected_revision`; concurrent stale edits fail without replacing current content. Files are snapshotted with generated IDs in `<data-dir>/design-assets`, with owner-only file permissions. Asset routes use the existing authenticated tenant boundary and return private/no-store responses. Projects and tasks are recorded with results; built-in templates contain no user data.

Files are limited to 64 MiB, structured content to 1 MB, standalone HTML to 500 KB and Mermaid source to 128 KB. These are format/resource limits rather than model token budgets. SVG/HTML content never executes in the host application. Diagram previews and their source are read-only in the UI; edits and restores go through the agent. Other formats retain explicit editing controls.

## Maintaining the catalog

Source: `resources/designs/catalog.json`. Run `python3 scripts/build-design-catalog.py` to reproduce the original text templates. `resources/designs/media` contains the two original motion samples. No third-party design assets are included; dependencies retain their upstream package licenses.

Real preview thumbnails live in `apps/web/public/assets/design-thumbnails`. After building the frontend and the Rust server, regenerate them with:

```bash
cd apps/web
UPDATE_DESIGN_THUMBNAILS=1 pnpm test tests/application.spec.js --grep 'welcome validates|Design catalog'
```

The test uses a local fake model solely for setup, checks every template and takes screenshots. New kinds require coordinated input validation, viewer, tool documentation and tests; new templates use existing kinds without adding another skill or tool.

## Original template collection, second edition

The second edition introduced original visual compositions for 64 entries: distinct website identities,
eight presentation themes with cover/split/steps/statement layouts, editorial documents,
framed data/diagram canvases, four vector graphics, and two motion pieces. Inspiration
came from the visual-first template browsing patterns of [Pitch](https://pitch.com/)
and [Canva](https://www.canva.com/templates/); no third-party template assets or code
were copied. Artwork is local SVG/CSS, with no remote image/font dependency.

The gallery, navigation, filters and editing controls are unchanged. Additional renderer
styles are scoped to content with an explicit `theme`; existing unthemed user designs
retain their previous rendering. Supported themes are `midnight`, `cobalt`, `coral`,
`forest`, `plum`, `sand`, `mint` and `ink`. A slide can optionally select `cover`, `split`,
`steps` or `statement` with `layout`. Content remains structured/editable; document text
is the source of truth for both the preview and export. Existing saved template copies
are not automatically overwritten.

Regenerate the content with `python3 scripts/build-design-catalog.py`; its visual
compositions live in `scripts/design_template_styles.py`. Regenerate the original motion
assets with `node scripts/build-design-motion.mjs`. Thumbnails are screenshots of the
production template renderer at a consistent 960×600 canvas, not generic placeholders.
Use the catalog Playwright test documented above after rebuilding the server.
After regenerating thumbnails, rebuild the frontend and server once more so the
embedded gallery includes the new images. The `Template gallery` browser test checks
the shipped covers and records desktop/mobile review screenshots.
For a partial thumbnail refresh, set `DESIGN_TEMPLATE_KINDS` to a comma-separated
list of kinds; leave it unset for the complete 72-template validation.

## Third collection: composition diversity and white data canvases

The catalog now contains **72 locally authored templates**: 20 websites, 10 decks,
10 diagrams, 8 charts, 8 documents, 8 spreadsheets, 6 vector images and 2 motion pieces.
The eight additions are architecture, orbital research, sound festival and travel
websites; systems and culture decks; and topographic and typographic posters.

`composition` is optional structured metadata. It selects an actual arrangement of
content, typography and locally drawn SVG scenes; it is independent of `theme`.
The ten decks have different cover structures and retain editable slide text. Documents
use columns, ledgers, numbered procedures, letter layouts and chapter cards. Spreadsheet
compositions differ by their working purpose. Existing saved copies are never overwritten.

Charts and diagrams use white paper in both the viewer and the catalog covers. The eight
chart compositions draw directly from editable labels and values: columns, area, ring,
lollipop, step line, proportional segments, diverging bars and horizontal rankings.
Thumbnail capture fits the full content and waits for layout before taking the image.
Chart covers focus on the chart artwork; the editable data table remains in the viewer.

### Reference research and asset provenance

The following Open Design examples were inspected as references for broad composition
ideas, not imported as templates or dependencies:

| Reference | Principle considered | Original Fénix direction |
| --- | --- | --- |
| [Editorial](https://github.com/nexu-io/open-design/tree/main/design-templates/html-ppt-taste-editorial) | Type scale and asymmetric reading order | Culture journal and client proposal |
| [Brutalist](https://github.com/nexu-io/open-design/tree/main/design-templates/html-ppt-taste-brutalist) | Strong typographic structure | Keynote poster and conference website |
| [Blueprint](https://github.com/nexu-io/open-design/tree/main/design-templates/html-ppt-knowledge-arch-blueprint) | Technical visual language | Original isometric systems scene |
| [Retro zine](https://github.com/nexu-io/open-design/tree/main/design-templates/html-ppt-zhangzara-retro-zine) | Collage and layered paper | Workshop notes and welcome booklet |
| [Cartesian](https://github.com/nexu-io/open-design/tree/main/design-templates/html-ppt-zhangzara-cartesian) | Measured data layout | White chart canvases and research atlas |
| [Signal](https://github.com/nexu-io/open-design/tree/main/design-templates/html-ppt-zhangzara-signal) | Contrast in decision documents | Business ledger |
| [Magazine poster](https://github.com/nexu-io/open-design/tree/main/design-templates/magazine-poster) | Display typography | Original typographic manifesto |
| [Finance report](https://github.com/nexu-io/open-design/tree/main/design-templates/finance-report) | Dense information hierarchy | Budget ledger and executive columns |
| [Dashboard](https://github.com/nexu-io/open-design/tree/main/design-templates/dashboard) | Legible data displays | Directly rendered, editable charts |
| [Kami landing](https://github.com/nexu-io/open-design/tree/main/design-templates/kami-landing) | Editorial pacing | Architecture and travel websites |

[Slidev themes](https://github.com/slidevjs/themes) and
[Presenton](https://github.com/presenton/presenton) were also reviewed for the separation
of content and layout. No source files, images, fonts, logos or brand systems from these
projects are bundled. Template composition code and artwork were written here, using
CSS and SVG. Existing open-source viewer dependencies keep their own licenses.
No template requires an external design provider, API, CDN, font service or image host.

The third collection is authored in `scripts/design_collection.py` and
`apps/web/src/design/template-compositions.{tsx,css}`. Run
`python3 scripts/build-design-catalog.py` to regenerate the catalog, then use the
browser checks above to verify the real renderer and refresh the covers.


[Agent-authored diagrams: navigation, storage and revisions](diagrams.md).

## Choosing a design from chat

New visual creation requests in the web chat open **Choose a design** in the existing
right workspace pane before they are submitted to the agent. Common Spanish and English
creation phrases select the relevant category. The composer also has a manual **Choose
a design** entry point for other wording or browsing all categories. Editing requests,
explanatory questions and explicit “without a template / from scratch” requests continue
through the ordinary chat path.

The picker uses the actual Design catalog and its local thumbnail images, loads every
catalog page for the selected category, and supports category switching and search. The
original brief remains editable. Selecting a card does not send anything: **Generate with
this template** submits the choice; **Generate without a template** submits a custom
creation request. Closing the pane leaves the draft in the composer and allows reopening
it during the same page session. On phones the picker uses the existing full-width
workspace pane with keyboard focus containment and Escape to return to chat.

The original document IDs and reply-to-task reference travel with the request. The server
accepts an optional `design_choice` in the existing `/api/messages` and `/api/chat` payload:
`{"mode":"template","template_id":"research-deck"}` or `{"mode":"custom"}`. It validates
catalog IDs before starting the turn, then appends the selected direction to the user's
brief. The agent must load the chosen template, adapt its content, preserve its visual
composition, carry the choice into delegated work, and publish the requested result in
the normally resolved project. No unchanged sample artifact is created by the picker.
Generation still uses the configured agent/model; template selection itself makes no
model request. Published results continue to open through existing workspace events.

Loading and submission errors retain the brief. A catalog outage still allows custom
creation. In-flight submissions disable generation buttons; successful submission clears
the draft without retrying an accepted message if a later history refresh fails.

Validation: `pnpm --dir apps/web run build`, `pnpm --dir apps/web test tests/design-creation.spec.js tests/workspace.spec.js`, and `cargo test --locked -p fenix-server --lib design_choice`.
