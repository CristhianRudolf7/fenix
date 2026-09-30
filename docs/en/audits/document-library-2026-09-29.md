# Document library validation — September 29, 2026

[Español](../../es/audits/document-library-2026-09-29.md)

## Scope and evidence

Twelve public-source files were loaded through the production library API: native PDF (including all 114 pages of RFC 2616), DOCX, XLSX, PPTX, Markdown, TXT, CSV, JSON, and a public OCR scan converted to PNG, JPEG and scanned PDF. Source URLs, hashes, individual results and coverage limits are in the [machine-readable report](../../research/document-library-validation-2026-09-29.json). Test documents remain labeled “Library audit” in the owner's library.

Validated original-byte downloads, fragment hashes and pagination, all 151 Iris CSV records, lexical retrieval, duplicate detection, version conflicts, old-version citations, metadata/title reindexing, collection filtering, disabling/re-enabling sources, explicit retry, corrupt-file recovery and deletion. The public frontend opened and rendered a real PDF on desktop and a 390-pixel mobile viewport without JavaScript errors.

Real OpenAI requests verified hybrid retrieval on five documents, persisted 1,536-dimensional vectors and reuse of ten unchanged embeddings after a new revision. PNG, JPEG and scanned PDF recovery retained OCR polygons and searchable text. Office and OCR coverage warnings remain visible; “partial” does not mean that no text was extracted.

## Findings and changes

- Semantic indexing was enabled without an available library credential. The previously configured OpenAI key returned `429 credit_balance_exhausted / insufficient_quota`. A second distinct authorized key returned HTTP 200 and was configured specifically for the library, independently of the conversational model roles.
- Re-extracting a partial document regenerated fragment identifiers and broke existing citations. Storage now preserves identifiers for unchanged text, location and ordinal within the same revision. The new regression also verifies embedding reuse. This source change is **not deployed**.
- The first PNG extraction failed in production. An explicit retry succeeded, as did a separate offline gVisor run using the production compatibility gate. The initial failure's root cause is unconfirmed; isolation was not weakened. Diagnostic runs without that gate were not production-equivalent and are excluded from conclusions.
- A DOCX search assertion initially requested a word absent from the actual file. Independent XML inspection corrected that test oracle; lexical retrieval returned both actual paragraphs.

## Verification and limits

Local checks passed: **9 library tests, 1 HTTP integration test and 3 document isolation policy tests**. They cover recovery after reopening storage, project/account separation, unchanged citations after retry, provider protocol validation and transient HTTP retries in addition to the production checks above. Protocol/error injection tests use doubles; real retrieval uses the paid embedding endpoint.

This is a functional audit, not a retrieval-quality benchmark or proof of support for every document. No 1,000-page/load test or end-to-end conversational-answer accuracy benchmark was run. Formula recalculation, Office layout fidelity, exact OCR accuracy and table reconstruction remain outside the guarantees. No new code was deployed and production services were not restarted.
