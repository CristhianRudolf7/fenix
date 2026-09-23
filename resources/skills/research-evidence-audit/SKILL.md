---
name: research-evidence-audit
description: Audit manuscript claims against cited passages, result files and experiment provenance, distinguishing source identity from actual support. Auditar afirmaciones, citas y evidencia científica.
metadata:
  version: 1.0.0
  category: research
  tags: research, evidence, claims, citations, provenance
  aliases: verificar afirmaciones, auditar evidencia, cita no respalda, claim source alignment, evidence ledger, resultados inventados
  required_tools: file_read,file_write,web_read,web_search,command_run
  requires: Manuscript claims and accessible source passages or research artifacts
  effects: local-write
---
# Research Evidence Audit

## Scope

Check whether particular claims are supported by the cited version of a source or by recorded research outputs. Use bibliography-management to repair citation metadata and peer-review for broader methodological critique. The audit writes a report; it does not silently alter the manuscript.

## Inputs

Current manuscript, claims to audit, references, source versions and experiment records. Identify the coverage denominator: all declared claims, a specified section or a sample. Never describe a sample as a complete audit.

## Workflow

1. Assign stable claim IDs and preserve the exact wording and manuscript location. Separate reported measurements, interpretations, literature statements and hypotheses. Include uncited quantitative claims and limitations that change the meaning of a result.
2. Resolve source identity and version, then inspect the actual supporting passage, table or output. A DOI resolving, a matching title or an abstract mentioning the topic is not sufficient support for a specific numerical or causal claim.
3. Compare population, intervention, comparator, units, denominators, uncertainty and direction. Check whether observational evidence was rewritten as causation, a conditional result became universal or a negative result disappeared. Preserve unresolved access and conflicting evidence.
4. For experiment-backed statements, match code/data revisions, configuration, run identity and result artifact. A consistent log does not authenticate raw data or establish that the experiment was conducted correctly. Reproduction is a separate computation.
5. Record supported, contradicted, overstated or unresolved judgments with their reasoning and exact locators. Recheck entries affected by source or manuscript edits; retain prior findings rather than overwrite their history.

## References

Read [the evidence ledger contract](references/evidence-ledger.md) when producing a reusable audit. `scripts/audit_evidence.py` checks local source hashes, line locators, quotations and referential integrity. It does **not** decide semantic support or verify a DOI over the network.

## Verification

Every verdict cites inspected evidence and states what remains unknown. Report coverage and stale sources explicitly. Keep structural validation separate from the agent's substantive judgment.

## Limits

Do not fabricate source text, infer misconduct from missing materials or claim independent verification from another persona using the same evidence. An audit request does not authorize uploading unpublished manuscripts to external services.

## Español

Comprueba cada afirmación con el pasaje o resultado concreto, su versión y alcance. Una referencia existente no garantiza que respalde lo escrito. Registra sobreafirmaciones, contradicciones y casos sin resolver. El validador comprueba hashes y localizadores, no la verdad de la conclusión ni la autenticidad de los experimentos.
