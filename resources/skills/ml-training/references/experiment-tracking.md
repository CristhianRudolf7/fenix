# Experiment tracking / Seguimiento de experimentos

Retain a run ID, configuration, dataset/model/code revisions, timestamps, metrics, checkpoint paths and terminal status. Use a local log when that meets the task; Trackio or another existing tracker can provide structured logging and retrieval. Confirm installed APIs and local versus remote storage. A dashboard is not proof that checkpoints persisted.

Keep per-step and per-epoch metrics distinct; record sample counts and evaluation settings. Flag nonfinite losses/gradients and investigate sudden zero loss before interpreting it as success. Alert thresholds should follow the actual task, not an arbitrary universal loss cutoff. Record interrupted and failed runs alongside successful ones.

Trackio Space synchronization can publish data; choose visibility and outgoing fields deliberately within the user's authorization. Do not log raw examples, credentials or full prompts merely because a tracker accepts them. External webhooks send messages and require that scope. Continue an existing run/job by identity after resumption and avoid unbounded automatic retraining in response to every alert.

ES: registra configuración, métricas, artefactos y estado terminal de cada ejecución. Conserva fallos y distingue almacenamiento local de sincronización externa. No publiques datos ni envíes alertas externas por defecto.
