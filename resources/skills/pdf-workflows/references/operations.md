# Fenix PDF operation protocol

The helper is original Fenix code over pypdf, PDFium, Pillow and ReportLab. It has no network calls or runtime installers. Its safety boundary is the existing tenant execution service: CPU, RAM, process, duration and workspace budgets still apply. `command_run` uses the existing command authorization; a skill does not grant extra permissions.

Load `pdf-workflows` with `skill_load` and require its returned `isolated_execution:true`. Resolve the helper as `<returned directory>/scripts/pdf_tool.py`; do not guess an account directory. Load this reference with `resource:"references/operations.md"`. Loading the full script is only needed to inspect/debug its implementation; that response also gives its exact `resource_path`. Write a JSON request with `file_write`. Run:

```text
/opt/fenix-pdf/bin/python -I '<resolved skill script path>' --workspace '<task workspace>' --request 'pdf-request.json'
```

Use `cwd` equal to the task workspace and `timeout_seconds:300`. Quote shell arguments correctly. For `inspect`, `read_paths` may select the request and each input (read-only execution). For mutations omit `read_paths`, because the output must be writable. Never mark a write command as read-only to avoid authorization. Use `command_status`, `command_output` and `command_cancel` for lifecycle. Only exit 0 plus `status:"completed"` and a verified output establishes completion. No host fallback when the image/interpreter is missing.

```json
{
  "operation": "assemble",
  "inputs": [
    {"path": "first.pdf", "sha256": "<hash from inspect>", "pages": [3, 1, 1]},
    {"path": "second.pdf", "pages": [1, 2]}
  ],
  "output": "combined.pdf"
}
```

Every input is `{path,sha256?}`; only `assemble` accepts input `pages`. SHA-256 is optional initially, required by the workflow after inspection. All paths are workspace-relative, all page numbers are 1-based. Output directories must already exist. Unknown keys/options and existing output paths fail. Output receipts include hash and byte count.

Publish PDF outputs with `design_import`. ZIP outputs are downloadable archives, not PDF previews. To preview a selected split page, use `assemble` on that source page to create a separate PDF and import it; do not submit a ZIP to `design_import` as a PDF.

| Operation | Options | Result / semantics |
| --- | --- | --- |
| `inspect` | `pages:[1,2]` optional | JSON: total pages, selected dimensions/rotation, document metadata, up to 200 fields, XFA flag. No output path. Defaults to first 20 pages. |
| `assemble` | none | Merge inputs in order; input `pages` selects/reorders/duplicates pages. Omit unwanted pages to delete. Existing outlines are imported; form field names are prefixed per input to avoid collisions. |
| `split` | `pages` optional | ZIP with one PDF per selected page. Generated safe member names; never extracts an archive. |
| `rotate` | `degrees:90`, `pages` optional | Relative rotation, multiples of 90 up to ±270. Other pages retained. |
| `crop` | `box:[0.1,0.1,0.9,0.9]`, `pages` optional | Visible crop relative to current crop box after rotation normalization. Content outside remains in the file. |
| `watermark` | `text:"DRAFT"`, `pages` optional | Visible centered text overlay; preserves underlying content. |
| `number` | `start:1`, `pages` optional | Footer numbers in selected order. |
| `metadata` | `values:{"Title":"Report","Author":"Fenix"}` | Replaces document Info, removes document XMP; allowed keys Title/Author/Subject/Keywords/Creator. Empty values removes these metadata fields; not forensic sanitization. |
| `compress` | none | Deduplicates objects and compresses streams without rasterizing. Compare actual byte counts; never promise reduction. |
| `fill` | `fields:{"customer":"Ana"}` | Existing AcroForm text fields only; fails for missing keys, XFA or other field types. Reopens output to verify values; inspect appearance. |
| `render` | `pages:[1]`, `dpi:150` | ZIP with PNGs. |
| `images` | `dpi:150` | Creates PDF from input PNG/JPEG images in order; EXIF rotation, white transparency background. |
| `rasterize` | `pages`, `dpi:150`, `grayscale:false` | Image-only PDF; discards document structure and text. Use only when this loss is acceptable. |
| `redact` | `rectangles:{"1":[[0.1,0.2,0.6,0.3]]}`, `dpi:150` | Reconstructs **all** pages into a fresh PDF, blackening marked pixels before encoding. Rectangles use displayed page coordinates. No original text/objects are copied. |

Rectangles are `[left,top,right,bottom]`, normalized 0..1, with top-left origin. Do not guess redaction coordinates from native text alone; first render and inspect the page including rotation/crop. Select slightly larger regions and verify surrounding text. This does not certify that all sensitive content was found, and does not delete originals or previous published revisions.

Limits: 16 inputs, 64 MiB each, 128 MiB combined; 2,000 source pages; 100 selected/output pages for vector operations; 20 pages for raster operations; 12 million pixels per raster; DPI 72–300; 64 MiB output; 64 KiB request. The command timeout is 300 seconds. A PDF with more than 100 pages can be reduced with `assemble` using an explicit page list. Raster redaction needs the whole document within the 20-page limit: process explicit parts and verify final assembly if larger.

Unsupported: password decryption/encryption, cryptographic signing/validation, arbitrary existing text replacement, XFA, checkbox/radio form filling, Office/HTML conversion, tagged accessibility or PDF/A certification, table reconstruction, guaranteed repair, searchable OCR layer. Route extraction to `document_extract`; never invent those capabilities from the helper's name. Rendering/rewriting is not a malware certificate. General vector operations may preserve links/actions/attachments; only raster reconstruction discards original objects.

## Español

Carga la skill y su script, crea el JSON en el espacio de trabajo y ejecútalo con el intérprete de la imagen aislada. Usa hashes tras inspeccionar; no sobrescribas originales. `assemble` une, selecciona, elimina y reordena; `split` y `render` entregan ZIP; `fill` admite campos de texto AcroForm. Las coordenadas son proporciones desde la esquina superior izquierda. Valida páginas y apariencia antes de publicar con `design_import`. El visor solo permite ver páginas y ajustar el zoom; pide búsquedas, OCR y edición al agente. Las limitaciones de calidad y funciones no implementadas se deben comunicar al usuario cuando afecten al resultado.
