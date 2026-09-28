# Native Gaia2, TerminalWorld and Terminal-Bench 4.0 evaluation

[Español](three-benchmarks-luna-low-2026-09-26.es.md) · [Machine-readable evidence](three-benchmarks-luna-low-2026-09-26.json)

One selected task from each benchmark, using the real Fenix Rust runtime with
**GPT-6 Luna, low reasoning, standard processing, no Flex and no Jev**. Each task
had one agent execution. This is an integration and failure audit, not an estimate
of benchmark-wide accuracy.

| Benchmark and task | Outcome | Native runtime | Estimated API cost |
| --- | --- | ---: | ---: |
| Gaia2-CLI execution — `scenario_universe_21_2m6akl` | Passed after repairing judge transport and regrading the unchanged trace; 6/6 effects matched | 14m 04s | $0.0591 including judge |
| TerminalWorld Verified — `tw_100459` | Official reward 1; functional outcome partial | 1m 45s | $0.0098 |
| Terminal-Bench 4.0 — `atrx-vep-crispr` | Official reward 0; required report missing | 24m 49s | $0.0806 |

Times include API waits, and Gaia2 includes its final judge wait. They exclude
cold image preparation. The tasks shared this two-CPU host and recording gateway;
these are not isolated hardware performance measurements. Aggregate quota waits
were 629s, 45s and 663s respectively. There were no HTTP errors in agent calls.
Harbor agent phases were 122.50s for TerminalWorld and 1,517.10s for Terminal-Bench;
their full trials, including verifier preparation/cleanup, took 159.26s and
1,926.20s. Gaia2's controller took 896.16s before offline regrading.

## Task audit

### Gaia2

Find a news article, identify two contacts from several clues, send the article,
move it, delete selected family pictures, and count other files. Fenix initially
searched inefficiently and tried to ask for help, then continued investigating.
It eventually performed both sends, the move, both deletions and the correct
four-file count. The upstream judge matched all six expected effects.

The first judgment was **invalid as a model-quality result**: upstream LiteLLM
rejected its fixed temperature 0 before any judge API call. The compatibility
wrapper retained the configured temperature 1 and low reasoning. The same
read-only saved trace was judged once successfully, without rerunning the agent
or replaying actions. Scenario and event hashes are recorded. A subsequent report
export error was recovered from the judge's durable log, with no additional LLM
call. The original failed judgment remains intact.

This uses the official **Gaia2-CLI variant**, with Luna low as configurable judge,
not the original ARE Python-tool protocol or calibrated leaderboard judge.

### TerminalWorld

Run SporeStack with a dummy provider and capture its complete output. Fenix found
the CLI, consulted its help, ran the requested command and saved the diagnostics.
The command reported a missing token. Fenix checked the available tokens, then
reported a blocker instead of funding an account or claiming a server existed.

The verifier checks the output file and launch-attempt message; it does **not**
establish that provisioning succeeded. Its reward 1 therefore has a narrower
meaning than successful end-to-end provisioning. The requested four CPUs could
not be allocated on this two-CPU server; an explicit Harbor override used two.
Task instructions, deadline and verifier were unchanged.

### Terminal-Bench 4.0

Reconstruct ATRX coding variants, annotate them using offline VEP, identify an
NMD-escaping variant in a protein domain, and select a nearby CRISPR target.
Fenix recovered from early reading/coordinate errors and premature completion,
reconstructed a transcript and ran VEP and its NMD plugin. It ultimately treated
its inferred NMD/domain mismatch as a requirements conflict and requested an
owner decision. It produced no `mutation.report.json`; the separate official
verifier returned 0. Missing-artifact collection was also recorded, but the
verifier still ran. No context-window exhaustion or provider rejection occurred.

The observed weakness is escalation from a scientific interpretation to a claimed
requirements contradiction without resolving the interpretation independently.
This audit does not establish that the benchmark itself is inconsistent. A useful
next improvement is checking parsers, coordinate conventions and interpretation
before escalating such a conflict, while preserving explicitly incomplete work.
No task-specific solution or hidden checker was added to Fenix.

## Accounting and reproducibility

| Benchmark | API requests | Input tokens | Cached input | Output tokens |
| --- | ---: | ---: | ---: | ---: |
| Gaia2 agent | 32 | 1,702,562 | 1,366,826 | 7,733 |
| Gaia2 judge | 1 | 2,649 | 0 | 51 |
| TerminalWorld | 14 | 226,921 | 166,791 | 1,613 |
| Terminal-Bench 4.0 | 53 | 2,756,576 | 2,402,420 | 29,901 |

Input totals accumulate across requests; the largest individual request was
93,802 tokens. Output includes reasoning tokens. Costs use recorded provider
usage, including cache writes, connection checks and caching, with the recorded
September 26 standard short-context prices: $0.10/M input, $0.01/M cached input,
$0.125/M cache writes and $0.50/M output. They exclude infrastructure and taxes and
are estimates, not invoices. Separate judge setup probes are in the JSON.

Selection is pinned and deterministic: first Verified JSONL row for TerminalWorld
(`dda7c099cc076735aef28c03bf8d3624dc0564e1`); first sorted task directory for
Terminal-Bench 4.0 (`452bf305c6daa62fc59061d22133a7cbc7c1572e`); first sorted scenario
ID in Gaia2-CLI's execution split (`240ac47bfc62b24c934f95d014ffa5a3cab8b04c`).
ARE runtime: `87ebd38f31aafae0f11e14f55617903196236cfb`. Harbor: 0.23.0.

All three used frozen binary SHA-256
`aef326d43d7509898cc76cf1596107af43ca8fddcb34d42a5e2244b733c5d8c1`.
The working tree was dirty; HEAD alone does not identify the evaluated code.
Raw traces, frozen bundles, manifests and original results are retained outside
the repository at the evidence location in the JSON.

## Integration changes and checks

- Pinned dataset preparation, Harbor one-task runner, and official Gaia2-CLI
  transport into `fenix eval`; Python does not implement another agent loop.
- An opt-in interactive session preserves runtime state, deduplicates deliveries,
  forwards clarifications and keeps notifications separate from user answers.
- Agent and Gaia2 private state/judge logs use separate Unix users. The evaluated
  agent's inability to read private files was checked.
- After the frozen run, fixed the legacy origin-based clarification denial,
  terminal-error delivery, judge startup validation and judge report serialization.
  Those changes were not retroactively applied to the agent results.
- Final local validation: 5 evaluation tests, 7 autonomy regressions, 8 transport
  and quota tests, and 2 dataset-preparation tests passed. Rust server built;
  Python syntax and diff checks passed. The actual judge compatibility probe passed.

The updated binary is separately hashed in the JSON and packaged as
`fenix-eval-gaia2:87ebd38-fixed`. Nothing was deployed to production.
