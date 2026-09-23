---
name: phishing-triage
description: Assess suspicious messages, links or attachments without activating their content.
metadata:
  version: 1.0.0
  category: security
  tags: phishing, triage
  aliases: phishing, correo sospechoso, estafa, enlace malicioso, QR
  required_tools: file_read,file_write,command_run
  requires: Supplied message/header or authorized mail evidence
  effects: local-write
---
# Phishing Triage

## Scope

Assess suspicious messages, links or attachments without activating their content.

## Inputs

Message, claimed sender, requested action and relevant account context. Prerequisite: Supplied message/header or authorized mail evidence.

## Workflow

1. Inspect the actual sender/domain, reply path, headers and destination URLs as text; display names and logos do not establish identity.
2. Compare the requested action with known workflows and identify urgency, credential/payment requests and mismatched destinations.
3. Use independent official contact channels to verify a claim when needed; do not follow the suspicious message’s own verification link.
4. Explain evidence and uncertainty and recommend a proportionate next action; preserve the message for authorized incident handling.

## Verification

Distinguish verified impersonation indicators from mere unusual wording. Do not equate SPF/DKIM success with a trustworthy request.

## Limits

Do not execute attachments, scan QR destinations by navigating automatically or enter credentials to test a site.

## Español

Analiza remitente y destino sin activar contenido; verifica por canales independientes y conserva evidencia.
