# CI trust boundaries

Map event, checked-out revision, runner, secrets, token permissions, caches/artifacts and downstream consumers for each job. A fork PR's code, dependency install hooks, generated reports and cache entries remain untrusted when a later privileged workflow consumes them. Privileged `pull_request_target` or `workflow_run` jobs must not execute untrusted head code/artifacts just because an earlier job completed. Review self-hosted runner persistence and credentials separately from hosted disposable runners.

Give each job the permissions it actually needs; bind OIDC trust to expected repository, workflow/ref, environment and audience as supported. An `environment:` name does not prove protection rules are configured. Verify the platform settings that enforce release gates. Pass untrusted text as data via arguments/environment and quote it correctly; neither YAML nor secret masking prevents script interpolation.

Pin reviewed external actions/build inputs and maintain updates rather than freezing known-vulnerable revisions. Separate an unprivileged build from privileged signing/publishing when the build consumes untrusted inputs. Authenticate artifact origin, producing run and digest before promotion; a matching filename or successful download is insufficient. Treat caches as untrusted and prevent lower-trust jobs from poisoning privileged consumers.

For a change, test the normal branch and the relevant untrusted-input path without real secrets. Confirm intended checks still run, inspect token scopes and simulate an untrusted artifact being rejected. Keep publication authorization from the current task; this reference does not add mandatory approvals to every build.

## Español

Traza evento, revisión, runner, permisos y artefactos entre jobs. No ejecutes contenido no confiable en un job privilegiado por provenir de otro workflow. Verifica reglas reales del entorno, permisos OIDC y digest del artefacto. Los nombres de entorno y el enmascarado de secretos no aplican esos controles por sí solos.
