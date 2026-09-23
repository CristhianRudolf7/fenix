# Agent-authored diagrams

Use one diagram for one question. Keep labels readable; split large systems into separate named diagrams instead of shrinking everything. The viewer supports zoom, pan, fullscreen, SVG export, searching structured nodes and reading their details. It never edits semantic data from pointer gestures.

## Structured graph

Call `design_publish` with `design.title`, `design.kind: "diagram"` and this content:

```json
{
  "graph": {
    "version": 1,
    "direction": "LR",
    "groups": [{"id":"office","label":"Office"}],
    "nodes": [
      {"id":"request","label":"Purchase request","group":"office","detail":"The employee supplies the amount and purpose.","references":["User-provided purchasing process"]},
      {"id":"decision","label":"Over S/5,000?","shape":"diamond"},
      {"id":"manager","label":"Manager approval"},
      {"id":"finish","label":"Approved","shape":"circle"}
    ],
    "edges": [
      {"id":"submit","from":"request","to":"decision"},
      {"id":"escalate","from":"decision","to":"manager","label":"Yes"},
      {"id":"ordinary","from":"decision","to":"finish","label":"No"},
      {"id":"authorize","from":"manager","to":"finish"}
    ]
  }
}
```

The example threshold is illustrative; never adopt it without user/source evidence.

- Directions: LR, RL, TB, BT. Default LR.
- Optional node `fill`, `stroke` and `color` customize background, border and text with #RRGGBB colors. Preserve readable contrast.
- Node shapes: rounded (default), rectangle, diamond, circle, database.
- Edges: solid (default) or dashed; always directed. Labels are optional.
- IDs use ASCII letters/digits, `_` or `-`, up to 80 bytes; all node/edge/group IDs must be unique.
- Groups are optional, flat, nonempty containers. A node may belong to one group.
- Node `detail` and `references` are optional plain text; references are evidence descriptions, not executable links.
- The engine compiles `source` from `graph`; do not edit a stale source alongside a graph. Omit `graph` entirely when switching to pure Mermaid.
- At most 200 nodes, 600 edges, 50 groups per diagram. Split larger material by concern.

## Focused changes

Read `design_read(id)` and send `diagram_edit` with the returned `expected_revision`:

```json
{
  "id":"actual-design-id",
  "expected_revision":1,
  "operations":[
    {"op":"put_node","value":{"id":"manager","label":"Director approval","detail":"Use the updated policy confirmed by the user."}},
    {"op":"direction","value":"TB"}
  ]
}
```

`put_node`, `put_edge`, `put_group` replace the complete named element or insert it. Preserve existing fields that should remain. `remove_node`, `remove_edge`, `remove_group` take `id`. Remove or reconnect affected edges in the same batch before deleting a node; update members when deleting a group. The batch is validated and saved as one revision. Stale or invalid edits leave the published version unchanged. An optional top-level `title` renames the diagram.

For broad revisions, use `design_publish(id, design)` with the complete content and `design.expected_revision`. Project scope is enforced by the task.

## Other diagram types

Use `content.source` with Mermaid, for example:

```text
sequenceDiagram
  participant User
  participant Agent
  User->>Agent: Request an explanation
  Agent-->>User: Show a diagram
```

Mermaid configuration/frontmatter and click directives are disallowed. The viewer owns rendering settings. Do not generate arbitrary HTML or scripts for diagram navigation. A schema/relationship validation success is not a browser rendering or factual correctness claim. Read the source facts and inspect the delivered visual when browser evidence is needed. Renderer failures preserve the last successfully rendered view during an update.

## Español

El agente es el único editor. Para flujos y arquitecturas utiliza `graph`; para secuencias, estados y relaciones de entidades utiliza Mermaid. Lee la revisión actual, conserva identificadores y datos no afectados, y publica cambios mediante las herramientas. Los archivos JSON y Mermaid son copias privadas de cada revisión; editarlos directamente no modifica la vista publicada. La validación estructural no demuestra la veracidad de las relaciones ni sustituye una revisión visual cuando sea necesaria.
