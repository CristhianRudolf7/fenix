---
name: mobile-app-testing
description: Verify mobile app flows on actual devices or emulators with lifecycle, permissions and input behavior.
metadata:
  version: 1.0.0
  category: engineering
  tags: mobile, app, testing
  aliases: QA móvil, android, iOS, emulador, teclado, APK
  required_tools: file_read,file_write,command_run
  requires: Authorized test build and available device/emulator tools
  effects: mixed
---
# Mobile App Testing

## Scope

Verify mobile app flows on actual devices or emulators with lifecycle, permissions and input behavior.

## Inputs

Platform/build, device configuration, test data and reproduction steps. Prerequisite: Authorized test build and available device/emulator tools.

## Workflow

1. Identify the installed build and runtime device; distinguish a native app from a responsive website.
2. Reproduce with clean and existing state as relevant. Exercise keyboard, orientation, safe areas, navigation/back behavior and permission denial.
3. Check background/foreground transitions, network loss and restored state. Capture screenshots/logs with timestamps while excluding private accounts.
4. Compare observed output to the requested user flow and retest after changes on the same configuration.

## Verification

Report device/OS/build, reproducible steps, evidence and which other platforms were not tested.

## Limits

Do not claim iOS coverage from Android or uninstall a user’s real app/data as routine cleanup.

## Español

Identifica dispositivo y build; prueba teclado, permisos y ciclo de vida con evidencia y cobertura explícita.
