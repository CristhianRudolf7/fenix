# PDF toolkit and preview

[Español](../es/pdf-toolkit.md) · [Repository review](../research/pdf-repositories-2026-09-20.md) · [OCR](pdf-ocr.md)

Fenix now combines the `pdf-workflows` skill, an original Python PDF helper and a React PDF viewer. The helper executes through existing tenant commands, so it uses normal task authorization and execution accounting. The skill teaches operation selection, verification and delivery; it is not a new permission boundary.

## Capabilities

The helper supports 14 operations: inspection; assembly (merge, extract, reorder, duplicate and delete by explicit page selection); splitting to ZIP; rotation; visible cropping; text watermarks; page numbers; document metadata; lossless compression; AcroForm text filling; PDF-to-PNG ZIP; PNG/JPEG-to-PDF; raster/grayscale reconstruction; and raster redaction. See the [complete request protocol](../../resources/skills/pdf-workflows/references/operations.md).

Input snapshots carry SHA-256 receipts. A subsequent request can require that hash. Paths are workspace-relative, traversed using directory descriptors without following symlinks. Inputs must be regular files with one link. Outputs are created privately and published atomically without replacement. The helper never overwrites an existing path, invokes an external program, downloads a model or uploads a document.

Redaction renders all pages, paints requested regions into the pixels and creates a fresh image-only PDF. It verifies no extractable text, annotations, form fields or attachments survive. This discards vectors, searchable text, outlines and digital signatures. It does **not** prove the requested regions cover every sensitive detail; visual review remains necessary. Originals and previously published revisions remain. Cropping and watermarking do not redact. Ordinary vector transformations can preserve active content and are not sanitizers.

## Right-hand preview

`design_import` already stores an account-scoped PDF snapshot. The document viewer now consumes that authenticated asset in a dedicated PDF.js worker and draws pages on a canvas. Controls provide only page navigation and fit-width/zoom in a compact toolbar. Password entry appears only when required to open a protected file. The PDF panel has no content editor, rotation, search, extracted-text panel or revision controls; document changes are requested through the agent in chat. The password is only supplied to the browser worker; it is not sent to the server or model.

The viewer has no PDF scripting manager, XFA execution, annotation links or interactive forms. Worker, CMaps, fonts, ICC profiles and image decoders are served by Fenix; no CDN is needed. The optional QuickJS scripting assets are excluded. The server serves `.mjs` as JavaScript and `.wasm` correctly. CSP allows WebAssembly compilation with `wasm-unsafe-eval`, while ordinary JavaScript `eval` stays prohibited.

The viewer limits files to 64 MiB, documents to 2,000 pages and canvas/image rendering to approximately 12 million pixels. Large embedded images may be omitted by the renderer; a preview is not a fidelity certification. Reading, searching, OCR and editing workflows belong to the agent tools. The preview does not extract text or create a searchable layer.

## Packaging

Build the lightweight image from an approved tenant image containing the current Fenix executable. Use a registry digest for deployment reproducibility; for local validation an existing local tag can be used after recording its image ID:

```sh
docker build -f packaging/linux/Dockerfile.pdf \
  --build-arg FENIX_TENANT_IMAGE=fenix-tenant:local \
  -t fenix-tenant:pdf .
```

`/opt/fenix-pdf/bin/python` contains pinned pypdf, PDFium, Pillow and ReportLab dependencies. Skill resources are distributed with the agent and mounted when `skill_load` selects the skill; its response now includes `directory`, `resource_path` and `isolated_execution`, preventing guessed installation paths. The bundled skill migration preserves user-customized instructions.

For OCR and editing together, `Dockerfile.ocr` also installs these PDF dependencies and provides `/opt/fenix-pdf`. The lightweight image alone does not supply PaddleOCR. Building images and frontend assets does not switch running account containers: install the updated server/agent/execution image and configure its immutable ID through the normal deployment process. No tenant service configuration is changed automatically by this implementation.

Read-only inspection can select input mounts and runs offline under the existing command policy. Mutating commands use the normal workspace write mount and its configured network policy; the helper itself has no networking. Unlike the fixed OCR tool, this helper is a selected skill script invoked through `command_run`, not a newly privileged broker operation. Do not parse untrusted PDFs on the controller if the image is unavailable.

## Deliberate limits

No arbitrary text/font editing, XFA or non-text field editing, encryption/decryption, digital signing, Office/HTML conversion, PDF/A/PDF/UA validation, guaranteed repair, table reconstruction or searchable OCR output is added by this helper. Production fidelity and adversarial-corpus benchmarks are not established by synthetic regression tests. These require separate implementations and validation, not stronger claims in a skill.

Tests: `tests/pdf/test_tool.py`, existing `tests/ocr/test_worker.py`, browser `apps/web/tests/pdf.spec.js`; [validation evidence](../research/pdf-toolkit-validation-2026-09-20.json) records results and coverage limits. Frontend fixtures contain synthetic text and an intentionally inactive PDF script/link.
