# Remote job lifecycle / Ciclo de trabajos remotos

Before submission, establish the exact executable source/revision, dependency environment, dataset/model access, hardware, time limit, authorized budget, output destination and visibility. Use an existing HF CLI/SDK or connected tool and inspect its current interface. Fénix has no guaranteed `hf_jobs` tool; do not invent one or send unsupported arguments to another tool.

Validate data and a small training/evaluation step where practical before scaling. Persist checkpoints and final artifacts outside ephemeral job storage, using the requested private/public destination. Pass secrets through supported secret injection, not inline source, logged arguments or published configuration.

Submit once and retain the job ID, request parameters and timestamp in the task record. If submission times out, reconcile account job state before retrying. A queued/running job is not successful training. Inspect relevant logs and terminal status, then reopen the saved model/adapter and evaluation output. Preserve failed and timed-out jobs in reports.

Plan polling around expected progress; do not resubmit to obtain faster feedback. Cancel only the intended job within authorization. A continuation request uses the existing job/checkpoint before starting a new run. For Trackio, choose local logging or an explicitly authorized Space; public dashboards and webhooks are not implied by training.

Source: [HF Jobs documentation](https://huggingface.co/docs/hub/jobs).

ES: define recursos y persistencia antes de lanzar, conserva el ID y reconcilia respuestas ambiguas. No confundas trabajo enviado con resultado verificado ni presupongas publicación de modelos o métricas.
