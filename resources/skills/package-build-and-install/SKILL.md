---
name: package-build-and-install
description: Build packages, wheels and indexes; verify a fresh consumer installation. Paquetes instalación distribución.
metadata:
  version: 1.7.0
  category: engineering
  tags: package, build, and, install
  aliases: paquete, distribución, wheel, instalación limpia
  required_tools: command_run,command_status,file_read,file_write
  requires: Available task-authorized Fénix tools
  effects: mixed
---
# Package Build And Install

## Scope

Build packages, wheels and indexes; verify a fresh consumer installation. Paquetes instalación distribución.

## Inputs

Package source, declared build backend, artifact and consumer environment.

## Workflow

Keep diagnosis, dependency repair, source changes, installation and consumer verification in one end-to-end execution assignment when they share one environment and form a serial feedback loop. A plan can track these as checklist items without creating a new worker and audit for each step. Delegate separately only when work is genuinely independent or a distinct deliverable warrants it. Preserve the final completion audit for substantial work.

Inspect the packaging layout and build metadata. Build the distributable using the declared backend. Inspect the produced artifact. If serving an index, verify the actual package-specific endpoint and links using the relevant packaging standard, not just the root listing. Keep the service supervised with persistent=true. Test installation from a separate clean environment using the requested source; a message such as Requirement already satisfied does not exercise that source. Import and use the installed package from outside the source checkout. Record the artifact, source URL and fresh-install result. Do not hard-code benchmark package names or hidden expected output.

After a concrete compiler/import error is understood, read only its owning code and necessary call-site or build metadata, apply a focused repair, and rerun that check. Do not inspect all compatibility sites before testing the first repair. Group independent targeted reads in one response. Complete the minimum consumer path early: repair → build → install into the requested interpreter → run the requested example from outside the checkout. Then expand regression coverage. Preserve pinned dependencies and do not equate an in-place build with a global installation.

## Completion and recovery

Keep the requested consumer environment authoritative. If the user requires a global installation or a fixed dependency version, verify that exact interpreter/environment and preserve that version; an isolated environment is only an additional diagnostic, not a substitute. Inspect build and runtime dependencies before choosing installation flags. Using --no-deps does not make missing runtime dependencies optional. A wheel built in isolation may differ from an in-place build; verify the installed artifact really contains and loads the requested native modules.

Treat missing build/runtime dependencies and ordinary compiler/import failures as actionable diagnostics, not terminal blockers. Within the authorized installation scope, identify and install the missing compatible dependency, retry the failed check, and follow the next observed error. Do not reinstall an unchanged artifact as a substitute for resolving a missing dependency. Do not change pinned versions or broaden permissions to make a test pass. If authorization or a real external prerequisite is missing, ask a concrete question instead of ending with unfinished implementation advice.

Keep a compact ledger of the target interpreter, preserved versions, completed build/install commands, latest failing check, and next repair. Recover these facts after compaction; do not repeat completed installs or reload the same procedure merely because the transcript was summarized. For builds/installations that are still running, use command_status with a meaningful wait rather than repeated one-second polls.

## Build provenance

For direct compiled programs, compile to an explicit output path and execute that exact path only after compilation succeeds (use &&). Do not validate an older executable left in a different directory. After a source edit, rebuild before rerunning. Start difficult cross-language syntax work with a minimal compiling skeleton in every required compiler, then add behavior. Build systems may use their own explicit targets and existing tests; do not impose extra broad suites for a small change.

### Cross-target builds

Before porting a large program to an unfamiliar emulator or ABI, inspect the supplied loader/runtime contract and prove a tiny executable can enter, emit output and perform the file operation the task actually needs. Compilation alone does not establish compatible startup, libraries or system calls. Add features to that working path incrementally. Treat a supplied acceptance runner as authoritative unless changing it is explicitly part of the deliverable; diagnose the artifact before modifying the runner. Never substitute a no-op for unsupported behavior just to suppress its error. Preserve the original runner for comparison and justify any permitted runtime change with the relevant behavioral check.

For native extensions, derive targeted source searches from the build declarations, including .pyx/.pxd/.c/.h when present rather than restricting compatibility fixes to .py. A successful import establishes loading, not native behavior: run a small representative operation for each extension explicitly required by the task, reusing the requested example or existing suite where applicable. Do not generate an unrelated test battery.

A missing dpkg-source, compiler or test runner is an environment prerequisite. Inspect the actual error and install the necessary declared tools within existing authorization, then resume extraction/build/checks. APT may warn that _apt cannot read a downloaded archive and still download it as root; that warning alone is not a permission denial. Do not request permission preemptively when policy already allows the required work. Never change credentials or bypass a real denial. Use bounded noninteractive smoke checks: supply debugger input and EOF explicitly instead of leaving a help command waiting.

## Verification

From outside the source checkout, import and exercise the installed package in the requested consumer environment. Run the user's example and applicable existing tests, respecting explicit exclusions. Verify both native extension loading and actual Python-facing behavior. A successful build/install log does not establish that the requested use case works. Continue fixing actionable failures before finalizing; report the exact checks and unresolved limitations without claiming completion of unfinished work.

## Limits

Use only tools authorized for this task and respect the configured user/project workspace. A skill does not expand permissions. Preserve customized user data and disclose unavailable capabilities. Do not claim success or automatic rollback without observed evidence.
