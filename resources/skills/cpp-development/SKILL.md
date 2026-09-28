---
name: cpp-development
description: Implement C or C++ changes involving memory ownership, lifetime, ABI and native build behavior.
metadata:
  version: 1.0.0
  category: engineering
  tags: cpp, development
  aliases: c++, cpp, memoria, punteros, cmake, compilador
  required_tools: file_read,file_write,command_run
  requires: Declared compiler, build system and relevant native diagnostics
  effects: mixed
---
# Cpp Development

## Scope

Implement C or C++ changes involving memory ownership, lifetime, ABI and native build behavior.

## Inputs

Target platform, language standard, ABI and failure scenario. Prerequisite: Declared compiler, build system and relevant native diagnostics.

## Workflow

1. Inspect compile commands, flags and ownership conventions. Trace allocation/deallocation pairs and object lifetime across callbacks or FFI.
2. Prefer scoped resource ownership for new C++ code while preserving caller contracts. Check overflow, bounds, alignment and iterator/reference invalidation.
3. For concurrent code, identify synchronization and publication guarantees; volatile is not a general synchronization mechanism.
4. Run the configured build and targeted tests; use available address/undefined-behavior/thread diagnostics appropriate to the suspected fault.

## Verification

Reproduce the original input under relevant debug/release configurations and verify binary compatibility when required.

## Limits

Do not change the language standard or ABI casually. Sanitizer startup failure is not evidence the program is correct.
