---
name: shell-automation
description: Write reliable shell automation with correct quoting, exit handling and explicit file effects.
metadata:
  version: 1.0.0
  category: engineering
  tags: shell, automation
  aliases: bash, shell, script, automatizar comandos, shellcheck
  required_tools: file_read,file_write,command_run
  requires: Target shell and required local commands
  effects: mixed
---
# Shell Automation

## Scope

Write reliable shell automation with correct quoting, exit handling and explicit file effects.

## Inputs

Inputs, filesystem scope, expected outputs and retry behavior. Prerequisite: Target shell and required local commands.

## Workflow

1. Identify the actual shell and portability requirement. Pass values as arguments; quote paths and use arrays where supported instead of constructing executable strings.
2. Handle expected nonzero statuses deliberately; errexit behavior depends on context and is not a substitute for error handling. Check pipeline stages when failures matter.
3. Use collision-safe temporary files and cleanup traps without deleting paths you did not create. Do not repurpose HOME or other system variables.
4. Make repeated runs inspect existing state; after uncertain side effects, verify before retrying. Use a dry-run mode when batch mutation benefits from preview.

## Verification

Exercise spaces, leading dashes, empty input, command failure and interruption; run the available shell linter.

## Limits

Never treat JSON serialization as shell escaping or evaluate untrusted text. Preserve user files during cleanup.
