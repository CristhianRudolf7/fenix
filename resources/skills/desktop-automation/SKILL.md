---
name: desktop-automation
description: Operate native desktop applications through available accessibility or screen-control tools with fresh window evidence and verified saved outputs. Automatización de escritorio.
metadata:
  version: 1.0.0
  category: operations
  tags: desktop, gui, window, accessibility
  aliases: aplicación de escritorio, ventana activa, ratón y teclado, native desktop app, GUI automation
  required_tools: tool_search,command_run,file_read,files_list
  requires: Accessible graphical session and an authorized OS-compatible desktop adapter
  effects: mixed
---
# Desktop Automation

## Scope

Use for native application controls or dialogs. Prefer an existing document/API operation when it directly performs the requested edit. Browser pages belong to browser-navigation.

## Inputs

Application, target document/window, requested actions, output path and the available desktop session.

## Workflow

1. Discover an existing desktop adapter and inspect its capabilities before issuing commands. A headless shell does not imply a graphical desktop. X11 utilities do not establish Wayland support; macOS accessibility and screen recording permissions are separate capabilities.
2. Identify the application and window using current title/process/document evidence. Prefer accessibility roles and stable control identity over coordinates. For screenshots, establish monitor, scale and coordinate origin before targeting.
3. Observe, act on one meaningful transition, then verify. Re-observe after moving/resizing windows, navigation, dialogs or focus changes. Before typing or pasting, confirm the intended field and focus; do not send a blind sequence of clicks and delays.
4. Distinguish a closed dialog from a saved artifact. Resolve overwrite choices within the user's intended file scope, then inspect the actual file or application state. Preserve unsaved work in unrelated windows.
5. If controls are unavailable, the screen is locked or a permission decision is required, report the precise limitation and complete independent file preparation where possible. Keep retries bounded by new observations.

## Verification

Record the relevant window, observed transition and saved output. A successful input-injection call does not prove the application accepted or completed the operation.

## Limits

This skill adds no desktop driver, privileged daemon or persistent watcher. Do not auto-dismiss permission prompts, close unrelated apps or bypass owner access settings. Keep screenshot/clipboard scope limited to the task.
