---
name: mathematical-computation
description: Solve symbolic or numerical calculations with explicit domains, units and independent checks.
metadata:
  version: 1.0.0
  category: research
  tags: mathematical, computation
  aliases: matemáticas, álgebra, cálculo, sympy, ecuaciones
  required_tools: file_read,file_write,tool_search
  requires: Appropriate exact/numerical computation tools
  effects: local-write
---
# Mathematical Computation

## Scope

Solve symbolic or numerical calculations with explicit domains, units and independent checks.

## Inputs

Problem statement, domain assumptions, units and required precision. Prerequisite: Appropriate exact/numerical computation tools.

## Workflow

1. Write the mathematical model and assumptions explicitly; distinguish exact constants from floating approximations.
2. Choose symbolic reasoning or a numerical method suited to conditioning and domain. Track dimensions and uncertainty through transformations.
3. Check candidate solutions in the original expression, including excluded denominators, branches and boundary cases.
4. Report precision and residual/error evidence, with derivation sufficient to audit the result.

## Verification

Use substitution, an independent identity or a second numerical resolution where it meaningfully checks correctness.

## Limits

Do not discard complex roots or singular cases without a domain reason, or present extra digits as accuracy.
