# PDF repository review and Fenix implementation

Reviewed on 2026-09-20. This is a targeted review of feature inventories, PDF interfaces and selected implementations, not an audit of every file or a claim of feature parity. No implementation from these three products was copied into Fenix. Our orchestration, request protocol, filesystem handling, transformations, skill and UI are original; PDF parsing/rendering uses maintained general-purpose libraries.

## Source snapshots and scope

| Repository | Observed HEAD | Inspected areas |
| --- | --- | --- |
| [BentoPDF](https://github.com/alam00000/bentopdf/tree/72d20bba092dbf4c293767086196c77f5f4519f0) | `72d20bba092dbf4c293767086196c77f5f4519f0` | README, package dependencies, PDF tool configuration; merge, compression, form-filler and redaction paths. |
| [ImageToolbox](https://github.com/T8RIN/ImageToolbox/tree/34b679472db93a36a994cf4d5d84cfd39daf577d) | `34b679472db93a36a994cf4d5d84cfd39daf577d` | Feature tree and README; PdfManager contract; AndroidPdfManager methods for searchable PDF, page operations, flattening and sanitizing; hOCR model. |
| [Gotenberg](https://github.com/gotenberg/gotenberg/tree/39e43205b17a21f48af75e13b6f0bd5597c50e9f) | `39e43205b17a21f48af75e13b6f0bd5597c50e9f` | README; PDF engine route inventory and merge pipeline; Chromium routes and file URL rejection; LibreOffice routes/protection detection; API context limits and cleanup. |

### BentoPDF

The browser-first product covers page organization, annotations/forms, conversion, OCR, compression and security-related workflows. Its UI separates tasks and uses workers for expensive operations. The merge interface supports explicit ordering, while compression distinguishes structural/image optimization from raster reconstruction. Those distinctions informed Fenix's explicit operation modes and loss disclosures. [Feature inventory](https://github.com/alam00000/bentopdf/blob/72d20bba092dbf4c293767086196c77f5f4519f0/README.md), [compression implementation](https://github.com/alam00000/bentopdf/blob/72d20bba092dbf4c293767086196c77f5f4519f0/src/js/logic/compress-pdf-page.ts).

The inspected `src/js/logic/redact.ts` function draws black rectangles and saves the document. That specific path alone does not demonstrate removal of underlying text. This is not a conclusion about every redaction path in the project. Fenix therefore implements its own pixel reconstruction, with text/object checks and an explicit loss of document structure. [Inspected redaction path](https://github.com/alam00000/bentopdf/blob/72d20bba092dbf4c293767086196c77f5f4519f0/src/js/logic/redact.ts).

Its form-filler delegates viewer actions through an iframe. Fenix uses a direct React/canvas viewer and a separately verified form transformation, avoiding reliance on another application's DOM controls. No BentoPDF modules, WASM products, interface assets or AGPL implementation were imported. [Form filler](https://github.com/alam00000/bentopdf/blob/72d20bba092dbf4c293767086196c77f5f4519f0/src/js/logic/form-filler-page.ts).

### ImageToolbox

This is an Android image editor with a substantial PDF subsystem, not a drop-in server library. Its PDF interface includes image conversion, contact sheets, searchable OCR, merging/splitting, removal/reordering/rotation, numbers/watermarks/signature appearance, password protection, compression, grayscale/dark mode, metadata, crop, flatten, image extraction, comparison, resize and sanitization. Mobile scanning and image preparation are useful adjacent capabilities, but Fenix does not need to port the Android UI or hundreds of filters. [PDF interface](https://github.com/T8RIN/ImageToolbox/blob/34b679472db93a36a994cf4d5d84cfd39daf577d/feature/pdf-tools/src/main/java/com/t8rin/imagetoolbox/feature/pdf_tools/domain/PdfManager.kt).

The implementation distinguishes visual flattening from structural modifications. Searchable output positions invisible text using OCR geometry; catalog/page cleanup addresses several classes of metadata, scripts and attachments. Lessons: bound raster resolution, preserve page dimensions, describe visual-versus-structural changes, and require real word geometry before promising searchable OCR. Fenix implements images, rendering and raster reconstruction now; it does not fabricate an OCR text layer or claim complete structural sanitization. [Android implementation](https://github.com/T8RIN/ImageToolbox/blob/34b679472db93a36a994cf4d5d84cfd39daf577d/feature/pdf-tools/src/main/java/com/t8rin/imagetoolbox/feature/pdf_tools/data/AndroidPdfManager.kt).

### Gotenberg

Gotenberg separates HTTP conversion orchestration from Chromium, LibreOffice and PDF engines. It exposes web/Office conversion, PDF composition, metadata/bookmarks, encryption, stamps, flattening and archival/accessibility conversion. It would add another service and several heavy engines if adopted whole. Fenix already has per-user execution and lifecycle management, so our helper reuses that boundary. [Overview](https://github.com/gotenberg/gotenberg/blob/39e43205b17a21f48af75e13b6f0bd5597c50e9f/README.md).

The inspected PDF routes validate incompatible options and order transformations intentionally; some metadata/bookmarks must be applied after conversions that discard them. The API context establishes a deadline, bounds request bytes, creates a private work directory and removes temporary files on cancellation. These informed Fenix's strict operation schemas, bounded inputs/outputs and atomic final publication. [PDF routes](https://github.com/gotenberg/gotenberg/blob/39e43205b17a21f48af75e13b6f0bd5597c50e9f/pkg/modules/pdfengines/routes.go), [request context](https://github.com/gotenberg/gotenberg/blob/39e43205b17a21f48af75e13b6f0bd5597c50e9f/pkg/modules/api/context.go).

Chromium's URL routes reject `file://`; uploaded HTML has a different local-resource context. This illustrates why generic URL-to-PDF is not simply another filename operation. Fenix's helper accepts workspace files only and adds no remote URL converter. Office conversion, font fidelity and archive standards need separate tests and confinement work. [Chromium routes](https://github.com/gotenberg/gotenberg/blob/39e43205b17a21f48af75e13b6f0bd5597c50e9f/pkg/modules/chromium/routes.go).

## Implemented versus deferred

| Capability | Fenix status |
| --- | --- |
| Native extraction / scanned document OCR | Existing isolated CPU tool retained and regression tested. |
| Merge / extract / reorder / duplicate / delete | Own `assemble` operation using explicit input page lists. |
| Split / render | Own bounded ZIP output with generated filenames. |
| Rotate / crop / numbers / watermarks | Own implementation with separate semantics. |
| Metadata / compression | Info/XMP updates and lossless stream/object optimization; no guaranteed size reduction or sanitization. |
| Forms | Inspect and fill existing AcroForm text fields, reopen to verify values. |
| Image conversion / grayscale / flattening | PNG/JPEG input; raster output with explicit structural loss. |
| Redaction | All-page pixel reconstruction; verify no original text/forms/attachments. Visual coverage still requires review. |
| Right panel | Own React viewer over PDF.js: navigation and zoom only, with a password prompt when needed. Subsequent UI simplification keeps all document operations in the agent. |
| OCR searchable layer / deskew / document scanning | Not added; extraction is available, geometry/fidelity work remains. |
| Arbitrary text editing / advanced annotations / XFA | Not implemented. |
| HTML/Office-to-PDF / PDF-to-Office/table conversion | Not added by this change. |
| Encryption / digital signatures / PDF/A/PDF/UA | Not implemented or certified. |
| Repair / comparison / booklet / N-up / workflow canvas | Potential future work, not advertised as available. |

## Ownership and dependencies

The three reference repositories remain external research sources. BentoPDF's repository uses AGPL licensing, ImageToolbox Apache-2.0, and Gotenberg MIT; dependencies can have separate terms. Fenix adds no dependency on those applications. It directly packages pinned pypdf, pypdfium2/PDFium, Pillow, ReportLab and PDF.js with their upstream notices. These provide parsing primitives; original code does not mean reinventing the PDF specification or copying a product under a new name. [BentoPDF license](https://github.com/alam00000/bentopdf/blob/main/LICENSE), [ImageToolbox license](https://github.com/T8RIN/ImageToolbox/blob/master/LICENSE), [Gotenberg license](https://github.com/gotenberg/gotenberg/blob/main/LICENSE).

## Español

La revisión se concentró en funciones PDF y rutas relevantes de los tres proyectos, con commits identificados; no es una auditoría de todo su código. De BentoPDF tomamos la separación de operaciones y calidad; de ImageToolbox, la relación entre imágenes, OCR y estructura; de Gotenberg, validación, límites y ciclo de vida. Creamos código propio sobre bibliotecas generales, sin incorporar esos productos. La tabla distingue lo implementado de lo pendiente. No hay base para prometer manipulación perfecta de cualquier PDF: firmas, formularios complejos, fuentes, accesibilidad y conversiones necesitan validaciones específicas.
