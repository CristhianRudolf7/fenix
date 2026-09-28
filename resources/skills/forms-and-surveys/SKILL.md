---
name: forms-and-surveys
description: Design and process forms or surveys with clear questions, valid branching and interpretable responses.
metadata:
  version: 1.1.0
  category: office
  tags: forms, and, surveys
  aliases: formulario, encuesta, Google Forms, respuestas, cuestionario
  required_tools: tool_search
  requires: Form connector or local/exported form data
  effects: external-write
---
# Forms And Surveys

## Scope

Design and process forms or surveys with clear questions, valid branching and interpretable responses.

## Inputs

Purpose, audience, required fields, privacy and analysis plan. Prerequisite: Form connector or local/exported form data.

## Workflow

1. Define the decision each question supports. Avoid leading, double-barreled questions and incompatible answer scales.
2. Use required fields and branching only when necessary; ensure every path reaches a valid submission and accessible labels.
3. For collected data, preserve question/version identifiers and distinguish skipped, not applicable and missing responses.
4. Create or revise through an available tool, or deliver a specification when no connector exists; distribute only when authorized.

## Native Google Forms

Use `connector_call`, select an account with `forms` enabled, and inspect `workspace_schema` before `forms.forms.get`, `forms.forms.batchUpdate` or `forms.forms.responses.list`. Invoke `request` with `/forms/FORM_ID` or `/forms/FORM_ID/responses`; responses support bounded `page_all`. Form structure and response access have distinct scopes. Connection checks need a specific form ID, so authorized does not mean this form is readable. Preserve question IDs and revision controls when updating a live form; do not change its publication or sharing as a side effect of response analysis.

## Verification

Test representative branches and exports; inspect duplicate responses and sample coverage before drawing conclusions.

## Limits

Do not collect sensitive data without a task-specific need or present convenience samples as representative populations.
