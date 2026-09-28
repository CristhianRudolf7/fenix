---
name: patent-research
description: Research patent publications, families and official record status with reproducible searches and precise identifiers. Investigar patentes y antecedentes técnicos.
metadata:
  version: 1.0.0
  category: research
  tags: patents, prior-art, intellectual-property, research
  aliases: búsqueda de patentes, antecedentes de patente, estado patente, patent landscape, prior art search, USPTO, EPO, PATENTSCOPE
  required_tools: web_search,web_read,file_read,file_write,tool_search
  requires: Public official records or supplied patent documents; authorized access for restricted APIs
  effects: local-write
---
# Patent Research

## Scope

Find and compare patent documents or verify reported bibliographic/status information. Use literature-review for scholarly papers. Research findings do not establish patentability, infringement, freedom to operate or current legal ownership.

## Inputs

Technical features, jurisdictions, relevant date window, known publication/application identifiers and intended research question. A confidential unpublished invention requires an agreed disclosure boundary before submitting its details to public search services.

## Workflow

1. Separate an exact record lookup, technical prior-art search and landscape/count analysis. Record search date and scope. Load [record handling](references/record-handling.md) for source choices and the evidence table.
2. Start with technical synonyms and classifications, then expand from relevant documents, cited material and assignee name variants. Retain queries, filters, result counts and inaccessible pages. Search results alone do not prove an exhaustive search or absence of prior art.
3. Normalize publication, application and grant identifiers with jurisdiction and kind code intact. Keep priority, filing, publication and grant dates separate. Group related family members for analysis without discarding their different jurisdictions, texts or statuses.
4. Read the relevant document passages, drawings and claims when available. Compare specific technical features with located evidence; an abstract or keyword match is only a discovery lead. Distinguish quoted document statements from the researcher's interpretation.
5. Verify material status or assignment information through the appropriate official record at the observation date. A recorded assignment, publication, expiry label or family member does not settle a legal conclusion about another record.
6. Deliver a source-linked table, feature comparison, search log and explicit gaps. Use current official documentation for API fields/authentication; an unavailable key can justify public-browser or supplied-document work, not an invented endpoint.

## Verification

Check document identity, dates and cited passages against the source. Count documents and families separately; flag unverified translations and incomplete coverage. Keep legal conclusions for qualified review with the complete records.

## Limits

No filing, fee payment or legal opinion is provided by this skill. Do not infer worldwide rights from a single publication or promise that a search found every relevant document.
