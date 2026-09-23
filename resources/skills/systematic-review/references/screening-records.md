# Screening records / Registro de selección

Keep a JSON object with `searches` and `records`. A search has unique `id`, `database`, exact `query` and ISO date `searched_on`. Each record has unique `id`, `search_id` and exactly one current `state`:

| State | Meaning |
| --- | --- |
| pending | Title/abstract screening has not finished |
| duplicate | Removed duplicate; requires `duplicate_of` pointing to a nonduplicate record and `reason` |
| screen_excluded | Excluded at title/abstract screening; requires `reason` |
| full_text_pending | Selected for retrieval or full-text review, still awaiting a decision |
| not_retrieved | Full text was sought but not obtained; requires `reason` |
| full_text_excluded | Full text assessed and excluded; requires `reason` |
| included | Report included; requires `study_id` to distinguish reports from underlying studies |

Retain original metadata and decision history in additional fields or the project record. Multiple reports from one study can be included with the same `study_id`; that is different from a duplicate retrieval record. Do not mark an unavailable full text as an eligibility exclusion.

```sh
python3 <skill-directory>/scripts/screening_counts.py screening.json
```

The helper recomputes counts, rejects dangling duplicate/search references and reports pending decisions. It does not deduplicate automatically, judge eligibility or certify PRISMA compliance. Use current [PRISMA reporting guidance](https://www.prisma-statement.org/prisma-2020) for the actual review/reporting requirements.

ES: registra el origen y la decisión de cada resultado, incluyendo motivos y pendientes. Varias publicaciones pueden pertenecer al mismo estudio. Los recuentos automáticos son contabilidad del registro, no una certificación metodológica.
