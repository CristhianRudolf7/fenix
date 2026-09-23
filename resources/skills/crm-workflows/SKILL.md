---
name: crm-workflows
description: Maintain CRM contacts, companies and deals with identity resolution and auditable field updates.
metadata:
  version: 1.1.0
  category: office
  tags: crm, workflows
  aliases: CRM, Salesforce, HubSpot, contactos, oportunidades, Attio, Composio CRM, custom attributes, campos personalizados
  required_tools: tool_search
  requires: Authorized CRM connector or supplied export
  effects: external-write
---
# Crm Workflows

## Scope

Maintain CRM contacts, companies and deals with identity resolution and auditable field updates.

## Inputs

Account/workspace, entity identity, requested fields and operation. Prerequisite: Authorized CRM connector or supplied export.

## Workflow

1. Discover the actual object schema and required/custom fields. Resolve records using stable identifiers and evidence, not similar names alone.
2. Check duplicates and relationships before creating or merging. Preserve source attribution for contact and organization facts.
3. Update only requested fields; distinguish absent values from instructions to clear a field. Keep deal stages tied to observed commitments.
4. Read back the result and report IDs plus conflicts that could not be resolved.

Confirm the connector's user, tenant/workspace and selected connected account before resolving records. Search names to find candidates, then use stable record IDs and the current object schema for writes. Inspect custom attribute types and relationship/cardinality constraints; list membership, an object record and a company association are different entities. After a write, use an authoritative record read if search indexes lag. A connection marked active does not prove it belongs to the requested account.

## Verification

Check uniqueness, association targets and changed values, including externally visible automation triggered by updates.

## Limits

Do not scrape private contact data, merge uncertain identities or launch outreach because a CRM record was created.

## Español

Conserva identidad y procedencia, cambia campos precisos y controla automatizaciones; una skill no instala el CRM.
