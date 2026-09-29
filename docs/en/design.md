# Design and visual results

[Español](../es/design.md)

Design is the local visual engine behind the continuous conversation. There is no separate Design page: design proposals are chosen in chat, and finished results open in the chat's right-hand panel. On desktop that panel collapses the navigation menu and restores it on close; at widths up to 900px it fills the screen and provides Back/Escape navigation. Closing a preview never cancels an agent task.

## Library and workflow

The catalog contains **72 original editable examples**: 20 web interfaces, 10 Mermaid diagrams, 10 presentations, 8 charts, 8 documents, 8 spreadsheets, 6 SVG images and 2 short silent WebM examples. Sample content and values are illustrative. No Open Design application, upstream templates or paid design service is bundled.

- **Proposals:** the agent presents catalog templates as a carousel of real covers inside its chat reply; each opens in a centered preview modal.
- **Favorites:** the star on a proposal card is stored in the authenticated user's database, and the agent sees favorite IDs when it looks for templates.
- **Results:** saved results appear as chips above the composer and open in the right-hand panel, with download, revisions and scoped refresh.
- **Brand identity:** a saved name, logo, colors and font family remain available to the agent through `design_templates`. The former Design page editor for brand identity and file import was removed with the page; the stored data and APIs are unchanged.

Conversational edits are the primary route for changing a result. Diagrams stay read-only in the viewer and change through the agent.

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

- `design_templates`: metadata pages (kind, tags, palette `theme`, `composition`), a selected template body, favorite template IDs and saved brand metadata (`brand_saved` distinguishes a saved brand from the default).
- `design_present`: show 1–8 real template proposals as an in-chat carousel, in the order used for textual selection.
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
UPDATE_DESIGN_THUMBNAILS=1 pnpm test tests/application.spec.js --grep 'managed welcome|Design catalog'
```

The test uses a local fake model solely for setup, opens every template in the chat preview modal (as a saved template link does) and takes screenshots. New kinds require coordinated input validation, viewer, tool documentation and tests; new templates use existing kinds without adding another skill or tool.

## Original template collection, second edition

The second edition introduced original visual compositions for 64 entries: distinct website identities,
eight presentation themes with cover/split/steps/statement layouts, editorial documents,
framed data/diagram canvases, four vector graphics, and two motion pieces. Inspiration
came from the visual-first template browsing patterns of [Pitch](https://pitch.com/)
and [Canva](https://www.canva.com/templates/); no third-party template assets or code
were copied. Artwork is local SVG/CSS, with no remote image/font dependency.

Additional renderer
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
embedded proposal carousel includes the new images. `tests/design-creation.spec.js`
checks that the carousel loads the shipped 960×600 covers and records desktop/mobile
screenshots.
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

For a new web page or slide presentation, the agent first reuses what it already knows:
the request, earlier conversation, memory, favorite templates and a saved brand. If a
meaningful visual preference is still missing (purpose or audience, color palette, style
or tone), it asks one short question about only those points and waits for the reply.
It does not ask again for preferences already given, does not invent answers, and skips
the question when the user asks it to decide, gives a complete brief or wants a custom
design. The guidance lives in `resources/agent-system.md`.

With the preferences known, the agent searches `design_templates` and calls
`design_present` with the best-fitting templates. The web chat shows them inside that
reply as a compact, horizontally scrollable carousel of real 960×600 covers, numbered in
order so the user can also answer in text ("the second one"). Previous/next buttons appear
when the row overflows; cards are keyboard-focusable buttons and each carries a favorite
star. Clicking a card opens the existing centered preview modal (full screen on phones,
focus contained, Escape or outside click to close). The right-hand panel is never used to
choose a proposal; it remains for finished results, files and library sources.

**Choose this design** in the modal sends a short chat message with
`design_choice: {"mode":"template","template_id":"…"}`. The server validates the catalog
ID and appends the selected template link, so the agent continues with the requirements
already in the conversation, loads that template, adapts its content while preserving the
composition, and publishes the real result. If the message cannot be sent (for example,
another message is still being sent or the service fails), the choice stays attached
above the composer and travels with the next message; it can be removed there.

The server still accepts `{"mode":"custom"}` for creation without a catalog template. No
unchanged sample artifact is created by choosing. Offers load with the existing design
poll; covers fall back to a drawn placeholder, preview loading errors appear in the modal,
and failed favorite changes roll back with a visible message.

Old links keep working: `/design` redirects to the chat, and `/design?template=<id>` (the
link stored with each chosen template) opens that template's preview modal over the chat,
both on page load and when clicked in the conversation.

Validation: `pnpm --dir apps/web run build`, `pnpm --dir apps/web test tests/design-creation.spec.js tests/workspace.spec.js`, `cargo test --locked -p fenix-agent --lib catalog_revisions_scope_and_file_imports_are_real` and `cargo test --locked -p fenix-server --lib design_choice`.
