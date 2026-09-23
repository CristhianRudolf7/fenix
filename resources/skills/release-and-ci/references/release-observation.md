# Release observation

Load when a task includes verifying an actual deployment or observing a canary. Release preparation alone does not start production monitoring.

## Establish what was deployed

Record the intended environment, application, artifact digest or revision and deployment ID. Resolve account/region and actual runtime identity from the authorized platform's evidence. A green build, listening port or HTTP 200 may still serve the old artifact or an error/login page. Verify the application-specific readiness and version signal when available; otherwise leave version confirmation unresolved.

Use the existing deploy mechanism and permissions. On a timeout, inspect the remote operation before retrying. Distinguish failed, still progressing and completed without an observed response; repeated deployments can have duplicate effects.

## Compare equivalent observations

Use a known pre-deployment baseline with matching routes, identity, fixtures, viewport and measurements. A reference captured after deployment is only a monitoring-start reference; do not label it pre-deployment or use it to assert no release regression. Preserve the older baseline and failed evidence.

Choose observations that expose this change's failure modes: relevant user flows, response failures, error identities, latency distribution, queue growth or affected business outcomes. Compare errors by signature as well as counts; three different errors can replace three old ones. Treat an inaccessible probe as unknown, not healthy. Redact private page content and request tokens in reports.

## Bound observation and response

State a finite observation window and sampling cadence. Use an existing scheduled-workflow mechanism only when ongoing observation is requested and configured; do not claim background monitoring after the task ends. Avoid overlapping rounds, report actual timing and preserve both first occurrence and recovery.

Apply the project's SLO or agreed rollout criteria. Investigate isolated transient failures proportionately, but surface a demonstrated data-loss or cross-tenant failure immediately rather than waiting for it to repeat. Do not import a universal two-sample, percentage or ten-minute policy from another system.

Rollback is a separate mutation and must fit existing authorization. Check whether migrations, queued work and external effects are reversible before executing the documented recovery. A rollback command succeeding does not establish restored data or healthy traffic; verify the old artifact and affected flow.

## Deliver evidence

Report artifact/environment, observation interval, tested flows, baseline provenance, failures and recovery, untested areas and current state. A healthy sampled window is bounded evidence, not proof that all users or future traffic are unaffected. Keep merge, deploy and verification states separate.

## Español

Comprueba entorno, identidad y artefacto servido, además del resultado del despliegue. Compara estados equivalentes y no llames previa a una referencia capturada después. Acota observación y cadencia, conserva errores y recuperación y distingue falta de datos de salud. Revertir requiere autorización y comprobar efectos ya persistidos; verifica la recuperación real y no prometas vigilancia continua al finalizar.
