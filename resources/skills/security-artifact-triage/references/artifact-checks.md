# Artifact checks

For Office files, distinguish OLE and OOXML from the actual container signature. Inspect VBA/XLM, auto-execution entrypoints, embedded objects, DDE and external relationships such as remote templates. A DOCX without VBA can still reference external content. Record stream names and relationships; extracted macro source may disagree with compiled VBA (stomping). Treat deobfuscation as a hypothesis and preserve raw and transformed text. Do not invoke Office, LibreOffice or a script interpreter to see what a suspicious macro does.

For PDF, inspect the object graph, incremental revisions, compressed object streams, actions and embedded files. Link `/OpenAction`, `/AA`, `/JS` or `/Launch` to actual referenced objects rather than counting words in bytes. Record parser disagreement and encrypted/unparsed streams. Extract JavaScript as text with automatic emulation disabled. Benign forms can contain JavaScript; an indicator's presence alone is insufficient for attribution.

For archives, list members under resource limits before extraction. Check misleading names, nested containers, path traversal, links and declared versus observed expanded size. Encrypted entries remain unknown without an authorized password. Never assume an archive is safe because the top-level format parsed or one visible member is benign.

For executables/scripts, record architecture, imports, signatures and relevant strings without loading the sample as a module. Distinguish a valid signer from a trusted publisher and expected release. A string mentioning a network host does not prove contact; refer deeper control/data-flow questions to reverse-engineering.

Defang indicators in human-facing reports while preserving original values in the controlled evidence copy. Keep extracted payloads away from auto-indexers and previewers. Public reputation lookups and sample uploads are separate disclosures with different scope.

## Español

Office requiere revisar macros, objetos y relaciones externas; PDF requiere seguir referencias y revisiones, no contar palabras. En archivos comprimidos limita expansión y rechaza rutas peligrosas. Conserva código como texto y trata fallos de parsing, cifrado y resultados parciales como límites. Firma válida, entropía alta o una URL aislada no determinan malicia.
