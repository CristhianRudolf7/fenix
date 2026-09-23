# Specification consistency / Coherencia de especificaciones

Use for software with several requirements, shared contracts or consequential acceptance uncertainty. Review only the changed behavior and affected dependencies; do not invent a full specification for a clear local fix.

## Keep one authoritative record

Recover the original request, accepted amendments and relevant project constraints. In project-scoped work, discover `project_spec` if a persistent project contract is relevant, then read the current version and all relevant pages. Reuse its stable requirement IDs and revisions. A proposal is not an applied amendment. Follow the native proposal/application workflow within existing authorization; do not create a project for an unrelated general-chat task.

Use `plan_manage` for execution milestones and `plan_evidence` for observed claims when a plan exists. Discover their current schemas rather than guessing arguments. Requirements describe intended behavior; evidence records what was observed. Export a specification document only for a useful handoff or requested deliverable, recording its source/version so it cannot silently become a competing authority. Preserve existing IDs; do not rename them just to introduce an FR/SC naming convention.

## Check whether the requirement can be decided and tested

For each material changed outcome, identify actor, trigger, observable result, scope and the relevant failure behavior. Separate actual requirements, assumptions and design choices. A useful acceptance example describes initial state, action and expected observation independently of the implementation.

- Replace consequential vague terms with an agreed observable criterion. If a threshold is missing, label a proposed target as an assumption or ask a focused question; never invent a number and present it as the user's requirement.
- Check only relevant boundaries: invalid or absent data, authorization, duplicate requests, partial failures and recovery. Missing documentation alone does not justify adding product features.
- Separate outcomes testable before delivery from outcomes requiring later observation. For example, response latency needs a workload/environment and measurement; reduced customer churn needs a population and observation period. Instrumentation or a passing test does not establish the business outcome. Record its measurement owner or unresolved dependency and avoid an endless implementation loop for a future metric.

## Compare artifacts before executing

Use a compact mapping in the existing review or plan when the relationships are not already clear:

`requirement ID/source + revision → acceptance observation → interface/artifact → milestone/check`

Read both sides of each affected boundary. Compare field names/types, required and null semantics, validation limits, units, state transitions, authorization, errors and compatibility where relevant. Trace dependencies in execution order. Flag contradictions, requirements without planned coverage, obsolete decisions, and tasks with no requirement or justified enabling role. Build/setup tasks may support requirements indirectly; existing unrelated code is not automatically scope creep.

Record findings as source references, discrepancy, consequence and a concrete next action. Resolve implementation choices autonomously within authorization. For a material conflict in user intent, recover the original source and ask only if it remains unresolved; continue independent work. Preserve completed effects and review the affected dependencies after an amendment. Do not treat an earlier review as approval of changed requirements.

The result is an implementable current plan with explicit unresolved decisions. No forced phase sequence, branch, commit, approval round or checklist artifact is needed. Consistency is an assessment, not proof that the eventual implementation works.

## Español

Aplica esta revisión a software con varios requisitos o contratos y solo a los límites afectados. Recupera la solicitud y cambios aceptados; reutiliza los IDs y revisiones del contrato vigente mediante `project_spec` cuando corresponda. Usa el plan y sus evidencias, sin crear otra fuente de verdad. Una propuesta no equivale a un cambio aplicado.

Relaciona requisito, observación de aceptación, interfaz y tarea. Identifica ambigüedades y compara tipos, límites, permisos, errores y compatibilidad entre productor y consumidor. No inventes métricas ni funciones: separa requisitos, supuestos y decisiones. Distingue resultados comprobables al entregar de métricas comerciales que necesitan observación posterior.

Registra cada contradicción u omisión con fuente, consecuencia y acción. Conserva trabajo válido, revisa dependencias afectadas y pregunta solo ante incertidumbre material de intención. Una tarea auxiliar puede estar justificada aunque no sea una función solicitada. No impongas documentos, ramas ni aprobaciones rutinarias; la coherencia del plan no demuestra funcionamiento.
