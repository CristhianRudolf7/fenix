# Parallel coding worktrees

Fénix provisions an engine-owned Git worktree before an execution subagent starts
in a Git repository. Each worker keeps its own `fenix/worker-<task_id>` branch,
checkout, command working directory and file-effect lock. The master remains
available and workers can edit/build/test concurrently without sharing files.
Plan reviewers and completion auditors continue reading the project evidence;
they do not receive an editable checkout.

## Lifecycle

1. Use a project workspace pointing at the repository root, with an initial
   commit, a named target branch and no uncommitted files or unfinished Git
   operation. Fénix will not silently stash, commit or discard the user's drafts.
2. Persist the task, source checkout, original branch, base commit and worktree
   location before provisioning. Git worktrees live under
   `<user-root>/projects/.fenix-worktrees/<repository-key>/<task-id>` in hosted
   mode, or `.fenix-worktrees/<repository-key>/<task-id>` beside the source
   repository in standalone mode. User/project bindings stay unchanged.
3. Redirect worker file tools, repository instructions, LSP and relative command
   paths to that checkout. Independent worktrees have independent file locks;
   shared services retain their existing synchronization. No extra model call
   selects or provisions a worktree.
4. Before successful completion, stop the worker's managed processes, wait for
   file effects and request integration approval when the configured file-write
   policy is `ask`. Commit its changes locally with an English commit message.
5. Serialize integration, recheck the plan assignment, source branch and clean
   source checkout, then check for conflicts with `git merge-tree`. Merge a
   compatible result locally. Only then publish a completed worker report, so
   subsequent plan milestones start from integrated predecessor changes.
6. Return `blocked` if integration fails. Preserve the worker branch/files and
   report the reason. A detected merge conflict does not modify the source index
   or working files. A new plan revision can reconcile blocked planned work;
   the master can retry a settled standalone worker with `worktree_manage`.

`worktree_manage(action="get", subagent_id=...)` reports assignment, integration
state, working changes and a bounded diff summary. Workers may inspect their own
assignment; integration is restricted to master execution tasks. Results and
subagent status snapshots retain worktree metadata. Repeated integration of an
already merged commit does not create another merge commit.

## Boundaries

- Non-Git folders keep the previous execution behavior. A broken Git checkout
  or a project scoped below the repository root is reported instead of silently
  executing against a shared Git directory. Initialize/commit a repository before
  requesting parallel coding isolation.
- Worktrees isolate file edits, not operating-system authority. Existing tenant
  and execution sandboxes remain responsible for security. Canonical repository
  and common Git paths must stay within the current user's project/workspace
  roots. Worktree commands must start inside their assigned checkout.
- The execution broker mounts the worker checkout, not shared Git metadata.
  Consequently raw Git commands may be unavailable in isolated command
  containers. Use the native inspection tool; commits and merges belong to the
  engine. Standalone mode still uses its existing trusted-process boundaries.
- Do not switch worker branches, edit Git internals, push or merge manually from
  a worker. The engine checks repository identity, branch identity and base
  ancestry before integration. Git hooks, signing and fsmonitor are disabled for
  engine-managed Git operations. Repositories defining custom Git filter or merge
  programs are rejected; those programs are not executed by the controller. Git
  2.38 or newer is required for merge conflict preflight.
- Integration is serialized within this runtime. Other applications can still
  change the source checkout; reserve it for integration during parallel work.
  Semantic compatibility still requires tests and the existing selective auditor;
  a clean Git merge is not proof of correctness.
- Cancellation, errors and restart preserve the worktree. Restart does not
  automatically resume execution. An interrupted provisioning step blocks rather
  than overwriting a possibly occupied directory. No automatic branch/worktree
  deletion or remote push is performed.

Declared requirement amendments also receive a [project contract](project-contracts.md)
preflight before integration. This catches stale revisions and overlapping IDs, not
undeclared or arbitrary semantic contradictions.

See also [execution harness](execution-harness.md) and [completion audits](completion-audits.md).
