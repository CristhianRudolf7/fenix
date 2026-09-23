# Local evidence ledger / Registro local de evidencia

Use a JSON object with `manuscript`, `sources` and `claims`. `manuscript` contains a relative UTF-8 `path` and its lowercase `sha256`. Each source has `id`, `path`, `sha256` and may retain an origin URL/version. Each claim has `id`, exact `text`, manuscript `line_start`/`line_end` (one-based, inclusive), `status`, `reason` and an `evidence` array. Evidence items contain `source_id`, `line_start`, `line_end` and an exact `quote` within those lines.

Statuses are `supported`, `contradicted`, `overstated` and `unresolved`. Every resolved status needs inspected evidence; unresolved claims retain their reason. Quotes and claim text may be shorter than the bounded line span, but must occur verbatim in it. Convert a PDF to an inspected text artifact first and retain its original citation/page mapping separately; this tool does not parse PDFs.

Run with an explicit project root:

```sh
python3 <skill-directory>/scripts/audit_evidence.py ledger.json --root <project-directory>
```

The command reads only named local files, rejects paths outside the root, checks hashes and locators, and prints JSON. Exit 0 means the ledger is structurally consistent; exit 1 means invalid input or stale/missing evidence. Status counts are **supplied judgments**, not conclusions made by this program. A consistent fabricated source can pass. No network resolution, semantic entailment, authenticity check or automatic manuscript rewrite occurs.

Retrieve a bundled helper through `skill_load` with its relative `resource` when its installed path is unavailable; save the inspected content in the authorized workspace before invoking it. Avoid assuming another installation's home path.

ES: conserva textos y hashes exactos, localizadores por línea y razones de cada juicio. El código valida coherencia del registro; no decide si una cita respalda la afirmación. Cambiar una fuente requiere volver a revisarla, no limitarse a sustituir su hash.
