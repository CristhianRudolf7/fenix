# Conversational autonomy and evidence-based execution

[Español](../es/conversational-autonomy.md)

Autonomy governs implementation choices, not the user's requirements or runtime authorization. There is one conversation and one execution system, with no new permission-mode controls in the interface.

## Owner preferences

The assistant uses `autonomy_manage` in its existing model/tool loop when the owner explicitly changes how independently it should work. For example, “Handle this yourself” selects autonomous behavior for this task; “Always consult me before deciding” selects a durable assisted preference. The tool requires a verbatim quote from the current task's direct owner message or steering instruction. Retrieved documents, model summaries and worker instructions cannot establish that provenance. Only direct web, Telegram or Live owner turns can save preferences.

Preferences are stored in the account's existing database. Resolution order is task, assigned project, account default, then assisted. Durable project/global scope must be explicit; otherwise use task scope. Workers inherit the parent's task preference, then their own fixed project's preference and the account default. They cannot modify preferences. Each inference receives a fresh, compact runtime policy so changes apply at subsequent safe boundaries, without replaying effects. The interlocutor forwards preference changes as work/steering and must not announce persistence before it occurs.

## Questions and requirements

At intake, unresolved questions about the objective, intended behavior, scope or acceptance criteria must be asked before dependent changes, even in autonomous mode. First reuse the original request, previous answers and available evidence. `ask_user` defaults to `category=requirement`; an autonomous preference cannot suppress those questions. Answers still enter the existing task contract. New material requirement conflicts discovered later also require clarification; autonomy is not permission to invent requirements.

`category=implementation` is for reversible technical choices within the agreed task. In assisted mode the owner may be consulted; in autonomous mode the runtime returns guidance to resolve the choice from evidence and record significant assumptions. Missing installable dependencies and ordinary repair steps should not be recast as requirement questions. The existing model classifies semantic intent; the deterministic runtime enforces scope/provenance and the selected question policy. No extra classifier model call or broad automatic guessing is introduced.

Autonomy does **not** change command/file approval settings, grant requests, denials, connector policies, sandbox boundaries, project specifications, planning review, verification requirements or completion audits. An actual required approval remains required.

## Noninteractive evaluation

The evaluation runtime explicitly marks its isolated database as noninteractive; evaluation tasks and their workers run autonomously. `ask_user` cannot create a pending wait. Recoverable implementation choices return to work; a genuinely missing requirement or authorization produces a recorded `owner_input_unavailable` blocker. `task_finish` must cite that observed call. The runtime never fabricates an owner answer or grants permission. This profile is independent of token/time limits and is not enabled by arbitrary model settings in ordinary conversations.

## Improvements following tasks 4–6

- Tool discovery returns compact `already_available` names instead of repeating schemas already in the current tool list. File-merge skill metadata covers JSON/CSV/Parquet normalization, source priority and conflicts; this is distinct from structured model responses.
- When a master completion omits prose evidence, the engine can cite bounded successful execution receipts already recorded for that task. It still runs every existing completion gate: absent, failed or stale checks remain blocking. Receipts do not prove all requested behavior by themselves.
- Patch preflight errors identify the file, hunk, match count and candidate lines. Ambiguity is never resolved by guessing or weakening exact matching. Preflight still happens before any write, and read/patch/retry remains the normal recovery path.
- Failed shell output distinguishes missing executables/modules from permission denials. `dpkg-source: not found` remains a dependency issue even when APT also reports its `_apt` download-user warning. A real runtime denial has priority; diagnostic hints do not grant authorization.
- Build/Python skills inspect the source languages declared by the build, including Cython files where relevant, and exercise required native behavior rather than relying on import success. Reuse examples and existing tests; do not create unrelated coverage requirements. Interactive smoke checks should receive explicit input and bounded waits.

These changes do not establish a benchmark score improvement until measured in another controlled run. The prior diagnostic results remain unchanged. No production deployment is part of this change.

## Improvements following tasks 7–9

Differential checks use independent restored fixtures and exercise a successful state change. Matching unchanged outputs after two rejected operations does not establish the main behavior. The updated testing/development skills request a small representative check, not more tests indiscriminately.

In noninteractive evaluations, a requirement clarification must include `blocking_reason` before it can establish an unavailable-owner blocker. A question without that explanation returns to work: distinguish indispensable missing intent from unspecified edge behavior, complete independent requirements and document reversible implementation conventions. Interactive intake questions still work without this field. This is a provenance/policy guard with model-supplied rationale, not independent semantic proof that the question is necessary.

The completion gate recognizes simple inline executable examples with absolute artifact paths (common compilers, interpreters and absolute executable invocations with arguments). Each recognized path requires a successful command receipt after the latest recorded edit/environment revision. Missing dependencies, failed execution, echoing commands and masked failures cannot satisfy it. The recognizer never executes user text and is deliberately not a full shell parser; relative/complex examples remain under the existing explicit contract tools. It verifies observed execution, not arbitrary output semantics. File-tool edits invalidate receipts; arbitrary shell mutations are not comprehensively inferred. Required permissions and other completion/audit checks remain intact.

`command_run` preserves the process layer's missing-dependency diagnosis instead of overwriting it with a generic error. Persistent-service guidance also covers lifetime after the turn and real protocol checks. No production deployment is included.

## Improvements following tasks 10–12

Direct compiler-plus-executable examples now require one successful compile/run chain that writes and executes the same output. The recognizer supports simple compiler invocations and explicit `-o` paths; it is not a shell interpreter, binary attestation or a replacement for semantic tests. Other build systems continue using explicit task contracts.

Environment inspection now always reports its workspace, uid, PID 1, found/missing tool paths and a completion marker. Administrative executable locations are checked before suggesting installation; only complete probes are cached. Discovery excludes already loaded schemas from its not-yet-loaded catalog and reminds callers to invoke visible tools directly.

Build, Git and service skills require fresh build provenance, independent deployment fixtures with new content and successful pushes, and the actual running outcome when requested. Shell syntax checks and a setup README do not establish a working service. Initial owner clarification and permissions are unchanged. No extra model or automatic reviewer is added.

The Harbor adapter adds task-independent context explaining the isolated execution target. Comparative CLI adapters must use the identical context. This corrects evaluation ambiguity without revealing hidden tests or treating a production workspace as authorized infrastructure. The interpretation of a genuine blocking requirement remains a model decision, not a deterministic proof.
