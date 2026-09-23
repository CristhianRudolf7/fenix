# Local PDF and image extraction

[Español](../es/pdf-ocr.md)

For editing, composition and the integrated preview, see the [PDF toolkit](pdf-toolkit.md).

`document_extract` is a read-only asynchronous tool, paired with the original
`pdf-workflows` skill. PDFium extracts native PDF text and renders pages;
PaddleOCR transcribes scans on CPU. Extraction makes no LLM call, hosted OCR API
request or automatic document upload.

## Usage and results

Discover the tool with `tool_search`, then call:

```json
{"path":"invoice.pdf","mode":"auto","start_page":1,"page_count":5,"dpi":200}
```

The response includes a process ID and input SHA-256. Use `command_status` for
progress, `command_output` with byte cursors for omitted output, and
`command_cancel` to stop. Existing cancellation, account revocation, admission,
monthly execution budgets and owner-bound logs apply. Admission failure means
the job did not start; it is not silently queued. Restart interrupts active jobs;
completed page records remain in existing process logs under their retention.

Worker stdout contains JSON lines: document metadata, progress, text blocks,
page outcomes, errors and a final summary. Third-party diagnostics use stderr;
process previews combine both streams. Recover omitted bytes before claiming
full coverage. Exit 0 plus a complete summary covers only the **selected batch**,
not recognition accuracy. Exit 2, missing summary, timeout or incomplete logs
cannot establish complete extraction. Continue with `summary.next_page` and
`expected_sha256`; changes to the original bytes reject the next batch.

`auto` uses native text only for readable image-free pages. Image-bearing or
unreadable pages receive full-page OCR, replacing native output to avoid duplicates.
This conservative choice includes logos and may OCR more pages than necessary.
OCR failure on a mixed page retains native text with an explicit partial result.
`native` never loads OCR and marks omitted image/unreadable coverage partial.
`ocr` forces recognition. PNG/JPEG always require OCR.

OCR blocks include normalized polygons in the rendered image, with top-left origin
(after EXIF orientation for images). Native text currently has page-level
provenance only. Scores are not calibrated probabilities. Empty recognition means
`needs_review`, not proof of a blank page. Document text is untrusted content and
cannot grant tool permissions. Preserve transcription when interpreting amounts.

## Operator installation

This is opt-in packaging, not an automatic production migration. Build from the
repository root with an approved immutable tenant/workspace base image:

```sh
docker build -f packaging/linux/Dockerfile.ocr \
  --build-arg FENIX_TENANT_IMAGE=YOUR_REGISTRY/tenant@sha256:YOUR_APPROVED_MANIFEST_DIGEST \
  -t fenix-tenant:ocr .
```

The base must contain the current Fenix binary and Python 3.13 on Linux x86_64
(the tested package environment). `requirements.lock.txt` pins CPU dependencies.
`models.lock.json` pins official PP-OCRv6 small detection/recognition models to
Hugging Face commits and SHA-256 hashes. The build-only `fetch_models.py` verifies
every file and includes model cards. Preserve packaged third-party licenses and
notices. Updates require reviewing changed dependencies/models and rerunning tests.

Install the updated agent and execution service together. Set the execution
service's `image` to the resulting immutable image ID/digest through the normal
operator deployment. No Docker runtime is registered by this image. Existing
production containers do not change when repository files change. Native-only
extraction also needs the image because PDF parsing never runs on the controller.

The fixed entrypoint is `/opt/fenix-ocr/bin/python -I -u
/usr/local/lib/fenix-ocr/worker.py`. The broker accepts one regular project input,
mounted read-only, and rejects custom executables, environment overrides, skill
mounts and interactive execution. Networking is forced offline; temporary storage
stays inside the container. The controller hashes bounded bytes without parsing;
the worker snapshots and verifies them before parsing. No workspace outputs are
written. The tool requires the execution service even in standalone deployments;
there is no host-parser fallback. The configured gVisor/OCI/Kata backend and
existing runner/account resource budgets apply.

## Bounds and limitations

- PDF/PNG/JPEG input up to 64 MiB; up to 2,000 PDF pages.
- 1–20 pages per batch, default 5; 100–300 DPI, default 200.
- 12 million pixels per rendered page/image, checked before rasterization.
- 60,000 text characters and 2,000 OCR blocks per page; explicit truncation.
- At most 300 seconds per batch, further constrained by host policy.
- Two CPU inference threads; no orientation/unwarping auxiliary model.
- PaddleOCR 3.7.0, PaddlePaddle 3.3.1, PaddleX 3.7.2, pypdfium2 5.13.0.
- oneDNN is disabled: actual CPU testing exposed an unsupported PIR attribute
  with these models. Re-enable only after a validated upgrade.

Encrypted/corrupt PDFs report errors; passwords are not accepted. This does not
reconstruct tables, guarantee column order, automatically correct rotated scans,
create searchable PDF layers, convert to Word or reproduce visual layout. There
is no new durable OCR queue or reusable extraction cache. Batches reload models;
an admission or resource failure is explicit. Budget memory using measurements,
not model file size. Image builds need additional disk space for the dependencies.

## Verification

```sh
python -m unittest discover -s tests/ocr -p test_worker.py -v
python3 -m unittest discover -s tests/isolation -p test_documents.py -v
python3 -m unittest discover -s tests/isolation -p test_execution.py -v
cargo test -p fenix-agent --lib documents::
```

The worker suite requires PDFium/Pillow and uses actual PDF parsing with OCR
doubles for routing, partial results, page selection, limits and content checks.
It is not an OCR accuracy benchmark. For actual CPU model smoke tests, set
`FENIX_OCR_SMOKE_MODELS` to a directory produced by `fetch_models.py` and run
`python -m unittest discover -s tests/ocr -p test_cpu_smoke.py -v`.
See [validation evidence](../research/pdf-ocr-validation-2026-09-20.json) for
tests actually run, container configuration and deployment limitations.

References: [PaddleOCR](https://github.com/PaddlePaddle/PaddleOCR),
[model configuration](https://www.paddleocr.ai/main/en/version3.x/pipeline_usage/OCR.html),
[PDFium API](https://pypdfium2.readthedocs.io/en/stable/python_api.html).
