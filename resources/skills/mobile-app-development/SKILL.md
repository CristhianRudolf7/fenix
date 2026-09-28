---
name: mobile-app-development
description: Implement native or cross-platform mobile features with platform lifecycle and state restoration constraints.
metadata:
  version: 1.0.0
  category: engineering
  tags: mobile, app, development
  aliases: desarrollo móvil, swift, kotlin, flutter, react native
  required_tools: file_read,file_write,command_run
  requires: Existing mobile SDK and project stack
  effects: mixed
---
# Mobile App Development

## Scope

Implement native or cross-platform mobile features with platform lifecycle and state restoration constraints.

## Inputs

Target platform, feature, navigation/state ownership and supported devices. Prerequisite: Existing mobile SDK and project stack.

## Workflow

1. Inspect the current native, Flutter or React Native architecture and platform targets. Preserve established state/navigation patterns.
2. Keep UI updates on the appropriate execution context; bind async work to lifecycle and cancel obsolete requests.
3. Implement permission denial, interruption and state restoration paths alongside the happy path. Avoid storing secrets in ordinary application preferences.
4. Build with the project toolchain and exercise the feature on an available emulator/device; use platform-specific behavior rather than desktop assumptions.

## Verification

Check launch, navigation, rotation or resizing, background return and accessible controls.

## Limits

Do not migrate the mobile stack or promise background execution that the target OS does not permit.
