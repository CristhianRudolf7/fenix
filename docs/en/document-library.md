# Document library

Fénix keeps private, versioned sources separately from its curated memory. Open **Library** (`/documents`) to upload files, filter your account’s documents by name and file type, inspect citations, or select sources for the existing chat. Name and type filters apply before pagination. New uploads enter the general scope; per-document metadata and existing project associations are preserved. Content retrieval remains available to the agent and in document conversations. The agent can summarize, compare, extract facts, and create derived documents or diagrams using these sources and its existing tools.

## Upload and processing

Supported files: UTF-8 TXT, Markdown, CSV and JSON; DOCX, XLSX and PPTX; PDF, PNG and JPEG. Originals are stored under the account runtime's `documents/<document-id>/v<revision>.<extension>`. Account data directories and databases remain separate. Each file has a SHA-256 checksum. Uploading identical bytes with the same format in the same project returns the existing document; revisions use optimistic concurrency and retain previous originals and citations.

Extraction runs in a persistent background queue. Restarting requeues interrupted extraction. States distinguish queued, processing, ready, partial and failed; embedding status is independent. A failed or unavailable embedding service does not remove lexical search. Embedding requests retry transport errors, HTTP 429 and server errors up to three attempts. Retrying an exhausted embedding request or a failed extraction is explicit. Retrying a partial extraction reprocesses the source; retrying a ready document retries embeddings. Re-extraction preserves citation identifiers when the fragment text, location and ordinal are unchanged within the same revision.

Office files use the native Rust extractor. Spreadsheet cells are grouped by row with original part/cell references; CSV respects quoted fields and multiline records. The library does not assume the first row is a header. Formula results are cached source values, **not recalculated results**. Office images, drawing objects and visual layout are not transcribed; the interface discloses this limitation. Use exported originals with appropriate document tools for calculations and editing.

PDFs and images require the existing isolated [OCR execution service](pdf-ocr.md), including its OCR image and a configured workspace. There is no unrestricted host fallback. PDFs process in page batches, keeping page references and OCR polygons where available. Missing pages, OCR warnings and partial extraction are visible. OCR is not a guarantee of table reconstruction or factual accuracy. The original can be downloaded even when extraction fails.

Processing bounds: 64 MiB per upload, 20,000 indexed fragments, 20-page PDF batches with a continuation ceiling of 2,000 pages. These bound processing resources, not agent task budgets. Split oversized sources. Extraction does not execute document macros or embedded instructions.

## Embeddings and retrieval

The fixed model is OpenAI `text-embedding-3-small`, at 1,536 dimensions. The library reuses the configured official OpenAI API connection, including organization/project headers, or uses a separate account-local credential provisioned by the service operator. The customer interface does not request or display provider keys. Keys configured for custom proxies are not forwarded to OpenAI. Enabling embeddings sends extracted text and search queries to OpenAI; originals and vectors remain in the account's local storage.

Only text requiring indexing is sent, in batches of at most 16 fragments. Unchanged fragment text in later revisions reuses its embedding. Queries also incur embedding input tokens. The integration uses the normal embeddings endpoint, not Batch or Flex. The reference price is USD 0.02 per million input tokens; check [OpenAI pricing](https://platform.openai.com/docs/pricing) for current rates. Actual reported tokens are recorded locally in `library_usage`. Storage, OCR execution and agent replies are separate costs. No LLM reranker runs on every search.

Retrieval combines FTS5/BM25 text matches and cosine similarity using reciprocal rank fusion. It filters to the selected project, enabled current revision, document selection and collection **before ranking**. General scope does not silently search other projects. If embeddings fail, results explicitly disclose lexical fallback. Vector scoring currently scans the selected scope in SQLite; this is suitable for an initial personal library, not a tested large-scale ANN index.

## Chat and citations

Use **Documents** in the chat composer or **Ask Fénix** from Library. Selected sources must share a project. The existing conversation receives source identities and exact version links; the master agent retrieves evidence on demand. Sources are untrusted data, never agent instructions. A search result is not proof that the whole document was inspected. Full comparisons and totals require complete relevant coverage, paginated reading or analysis of the exported original.

Citations open a source panel with the version, source location and extracted fragment. PDFs support page navigation and zoom, with OCR region highlighting when geometry is available. Office sources show extracted text and offer the original download; this is not a full Office renderer. The sidebar collapses on desktop; the panel covers the screen on mobile and can be closed to return to chat. Old citations continue opening their original revision until that document is deleted.

Agent tools:

- `library_list`: discover source metadata within authorized task scope.
- `library_search`: retrieve scoped evidence, citations and fallback status.
- `library_read`: paginate a specific source version.
- `library_export`: copy an original into the confined task workspace without overwriting a file, through the existing file approval/journaling path.
- `library_import`: register a workspace document or publish a new revision with the expected previous version.

The agent edits working copies through existing file/Office tools and imports a revision. It cannot silently change an original already cited. Disabling a document removes it from search while retaining history. Deletion removes its indexed content and originals; previously shared links then stop resolving.

## Validation

Targeted tests cover duplicate detection, version conflicts, lexical/vector scope filtering, reuse of unchanged embeddings, quoted CSV rows, provider request/response validation and transient retries. HTTP tests cover upload limits, the extraction queue, exact-version citations, account separation, settings redaction and deletion. These use temporary data and mocked provider responses; they are not a paid OpenAI quality benchmark. Production deployment and OCR infrastructure installation are separate operations.

The [September 29 real-document audit](audits/document-library-2026-09-29.md) additionally exercises public documents, production extraction and real OpenAI embeddings. Its report distinguishes source fixes from deployed behavior and records failures and coverage limits.
