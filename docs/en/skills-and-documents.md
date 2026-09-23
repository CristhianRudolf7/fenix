# Scalable skills and Fénix Documents

[Español](../es/skills-and-documents.md)

PDF/scanned-image reading uses the isolated `document_extract` tool and
`pdf-workflows` skill. See [CPU PDF/OCR](pdf-ocr.md) for setup and limits.
PDF manipulation and the integrated right-hand preview are documented in the [PDF toolkit](pdf-toolkit.md).

## Skill discovery

The master and each subagent receive only task-local candidate metadata (at most two suggestions), not the complete installed catalog. `skill_search` returns up to eight ranked candidates; `skill_list` browses at most twenty entries per page. `skill_load` reads an enabled skill's instructions or one relative UTF-8 `resource` such as `references/tables.md`. A resource never executes when loaded. Search/load do not grant tool permissions.

The Rust runtime maintains a per-user incremental lexical index in its existing SQLite store (`skill_index_v1`). An in-memory cache stores metadata and weighted term postings. File path, device/inode, size, nanosecond modification/change times invalidate entries; SHA-256 identifies indexed content after runtime redaction. Changed entrypoints are parsed again, deleted entries are removed, and enabled state is checked on each retrieval. Restarting reuses the persisted index. Disk/stat checks and ranking still scale with catalog size; this is not a constant-time search engine.

Names, descriptions, tags, aliases and entrypoint bodies contribute to BM25-style ranking. Byte-identical indexed entrypoints collapse to one search result. Empty/no-match queries return no candidates rather than implying relevance. Use explicit bilingual aliases for cross-language discovery; there is no semantic embedding search or model reranker. Descriptions and search results are reference data, not authority. Search references through the skill entrypoint; linked resource contents are not indexed automatically.

YAML frontmatter supports folded descriptions, lists and nested `metadata` fields. `version`, `required_tools`, `tags` and `aliases` may be top-level or inside `metadata`. Early entrypoints without frontmatter remain compatible. Each skill folder has one `SKILL.md`; folders are immediate children of the configured skill directory. Entrypoints and loaded references must be regular UTF-8 files up to 32,000 bytes. Malformed entrypoints are isolated, listed with errors, and excluded from selection. Disabled skills and resource paths escaping their folder cannot load.

Existing installations retain their configured skill directory and custom files. New bundled skills install on runtime startup without overwriting customized entrypoints. Web skill administration can still list the complete catalog; that list is not injected into model context. Catalog administration currently omits invalid entries; use `skill_list` for their diagnostics.

## Native document engine

`crates/fenix-documents` is an original Rust implementation of a bounded OOXML authoring/editing layer. It uses general-purpose ZIP, XML and serialization libraries. It has no Microsoft 365 account, Office CLI, cloud document provider or third-party office application dependency at runtime. It is a document engine for the agent, **not a complete office suite or visual web editor**.

Three bundled skills are included: `documents-word`, `documents-spreadsheets`, and `documents-presentations`. They introduce the tools and their format-specific schemas on demand:

- `document_create`: creates a new `.docx`, `.xlsx` or `.pptx`; refuses overwrite.
- `document_read`: paginated text/cell inspection with package part references and a whole-file SHA-256.
- `document_edit`: conditional unique text replacement inside a paragraph (DOCX/PPTX), or one A1 cell update (XLSX). Text matches may span formatting runs. Replacement inherits the first matched run's formatting; unaffected runs/parts remain present.

Creation supports Word paragraphs, headings, bold runs and tables; Excel sheets and typed cells with explicit formulas; and 16:9 PowerPoint slides with titles and bullet text. Existing cell styles and unrelated package entries are preserved on edits. An edit cannot replace an entire imported document with a simplified reconstruction. The target file is written through a synced temporary file and an atomic rename; creation uses no-clobber persistence. Hash checks detect stale reads. OS errors after commit can still be reported as failures; inspect the destination before retrying. This is not a backup/rollback system and does not lock out external applications editing the same file.

The agent tools use normal workspace confinement, task/project authorization, file approvals, mutation leases, cancellation checks, repository guidance and effect journals. Review agents cannot edit documents. ZIP/XML parse and structural checks run locally, without an LLM call. Tool results never prove visual layout correctness.

## Current boundaries

- Formula strings are exported, **not calculated**. On cell edits, cached formula results are cleared across worksheets and recalculation is requested on open. Shared/array/data-table formula workbooks are refused before editing.
- No full Excel calculation engine, charts/pivots API, tracked changes, custom presentation layouts, image insertion, PDF export or visual rendering.
- Legacy `.doc`, `.xls`, `.ppt`, encrypted documents and UTF-16 XML parts are unsupported. Only ordinary OOXML UTF-8 packages are handled; full compatibility with every office extension is not claimed.
- Compressed and expanded packages are limited to 64 MiB and 4,096 parts. The agent's existing tool-argument limit also applies to creation specs. Native text/cell inspection is paginated but may parse the whole package.
- ZIP members are not extracted to the filesystem. Embedded scripts, macros, formulas and external relationships are never executed or fetched by this engine.

## Validation

Run `cargo test -p fenix-documents` and the agent library tests for `skills::` and `documents::`. The [real skill library checks](skill-library.md) validate the bundled workflows, bounded bilingual discovery, disabled state and resource installation. They measure deterministic retrieval, not model task-completion accuracy.

`cargo run -p fenix-documents --example fixtures -- /tmp/fenix-documents-check` creates sample files for independent interoperability checks. LibreOffice and Python readers are test tools only, not product dependencies. See [OOXML package structure](https://learn.microsoft.com/en-us/office/open-xml/spreadsheet/structure-of-a-spreadsheetml-document) and [PresentationML](https://learn.microsoft.com/en-us/office/open-xml/presentation/structure-of-a-presentationml-document) for the format basis.

### Local verification (2026-09-19)


Generated DOCX/XLSX/PPTX fixtures were opened with independent `python-docx`, `openpyxl` and `python-pptx` readers. The DOCX table grid, sheet names, explicit formulas/literal strings and slide text/layout/master were checked. LibreOffice Writer also loaded a DOCX fixture; Calc and Impress are not installed, so native-application rendering of XLSX/PPTX was not checked. No provider API was needed for these checks.

The runtime integration suite also passed all 24 tests, including document approval and effect journaling; the document crate passed all seven tests. Clippy for both crates and the server compilation check passed.
