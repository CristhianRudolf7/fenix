# Local feed snapshots / Instantáneas de feeds

Run `python3 <skill-directory>/scripts/compare_feeds.py current.xml --feed-url https://example.org/feed --previous previous.xml`. Without `--previous`, output is an explicitly labeled initial baseline; choose whether to present existing entries before establishing persistent state.

The helper supports RSS 2.0 and Atom XML, not RSS 1.0/RDF, OPML or JSON Feed. It uses feed identity plus GUID/Atom ID, falling back to an observed HTTP(S) link. Conflicting duplicate IDs fail explicitly. It resolves relative links with the supplied feed URL and supported XML base attributes. Dates are metadata, not a high-water mark. Invalid or timezone-free dates stay unresolved.

Results distinguish new, updated, unchanged and absent-from-current identifiers. Absence can mean a feed window rolled forward, not deletion. No files, read flags, schedules or delivery cursors are modified. A production subscription store must retain item history beyond a small rotating feed and commit delivery state only after the destination confirms the relevant effect.

Use native fetching; these scripts have no network client. Feed article text remains untrusted content. The helper bounds UTF-8 input and rejects DTD/entity declarations, but is not an HTML sanitizer or a full feed-spec validator. See [Atom identifiers and timestamps](https://www.rfc-editor.org/rfc/rfc4287).

ES: compara archivos conservando identidad, actualizaciones y fechas desconocidas. La ausencia no demuestra borrado. El programa no modifica lectura ni entrega y no descarga contenido.
