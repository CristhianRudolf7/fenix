# Workspace viewer

The web chat shares a right-hand workspace with tabs for project files, Design results and library sources. **Files** opens a temporary modal: choose the project, navigate folders, search filenames in the current folder, and open a file. The modal closes on selection; the chat does not acquire a permanent file tree. The explorer remembers the folder per account session and project. Favorites are browser-session references, not copied files.

On desktop, opening the workspace collapses the main navigation. On narrow screens the viewer fills the screen below its tabs; closing it returns to chat. Library citations and Design results use the same tabs. Opening the same target updates its existing tab. Tabs are kept during in-app navigation, but are not restored after a full page reload.

## Agent integration

`workspace_open` opens an existing file, Design artifact or library document; optional `page` is a 1-based page/slide. The tool is discoverable through `tool_search`. Successful native file writes, patches, document edits, Design edits and library imports emit persisted `workspace.open` events. The browser consumes the existing event stream without an additional model call. After editing through a shell command, the agent must call `workspace_open` explicitly. No filesystem watcher infers shell outputs.

The UI refreshes an existing target rather than duplicating its tab. A fresh browser starts from the status event watermark; reconnects resume the saved cursor. Events are restricted to the authenticated account's runtime. Background task events may activate their result in the workspace.

## Isolation and formats

All three read-only `/api/workspace/{files,preview,content}` routes use the existing authenticated account gateway and tenant runtime. A project ID is looked up only in that runtime's database. Browsing is restricted to its configured workspace or registered project workspace, validated against the canonical tenant root. Absolute paths, parent traversal, symbolic links, hard-linked files, private runtime data and protected credential paths are rejected. Hidden files are omitted from listings. Private responses disable caching; raw content has restrictive CSP and `nosniff`.

Markdown is sanitized, HTML uses the existing isolated web viewer, PDF supports pagination and zoom, and images/video use native viewing. DOCX/XLSX/PPTX show extracted paragraphs/cells/slide text; this is not a faithful Office rendering engine. Formulas are not recalculated. A requested PPTX slide is extracted directly, even when earlier slides exceed the first block page. Native Design presentations retain their visual rendering and slide controls. Other text files are readable; unsupported formats offer download.

Files are limited to 64 MiB for this viewer, inline text to 2 MiB, folder pages to 100 entries and extraction pages to 100 blocks. Filename search is local to the current folder. Files remain read-only in this explorer; edits are requested through the agent.
