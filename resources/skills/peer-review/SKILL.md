---
name: peer-review
description: Assess a manuscript’s claims, methodology and evidence with constructive, bounded findings.
metadata:
  version: 1.1.0
  category: research
  tags: peer, review
  aliases: revisión por pares, evaluar paper, rigor, manuscrito
  required_tools: file_read,file_write,tool_search
  requires: Manuscript and authorized review scope
  effects: local-write
---
# Peer Review

## Scope

Assess a manuscript’s claims, methodology and evidence with constructive, bounded findings.

## Inputs

Research claim, target venue/criteria, supplementary evidence and review task. Prerequisite: Manuscript and authorized review scope.

## Workflow

1. Identify the central claim and which experiment or argument supports it. Read methods and limitations alongside results.
2. Check controls, sampling, leakage, statistical assumptions and alternative explanations relevant to that claim.
3. Separate decisive validity concerns from presentation suggestions. Explain what evidence would resolve each concern without demanding unrelated studies.
4. Provide a balanced review with locations, limitations of your expertise/access and unresolved questions.

For venue-bound criteria, re-review or a requested reviewer calibration, read [criteria and revision checks](references/criteria-and-revision.md). Use research-evidence-audit when the requested deliverable is a claim-by-claim evidence report. Review perspectives do not imply independent measurements.

## Verification

Ensure each criticism follows from the supplied text and does not misrepresent absent information as proven misconduct.

## Limits

Respect confidentiality and venue AI-review rules; do not upload unpublished material elsewhere or contact authors without authorization.

## Español

Evalúa evidencia y metodología con hallazgos localizados; diferencia problemas de validez y mejoras de presentación.
