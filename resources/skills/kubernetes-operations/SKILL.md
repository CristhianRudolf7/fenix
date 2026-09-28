---
name: kubernetes-operations
description: Inspect and change Kubernetes workloads with explicit context, rollout and recovery evidence.
metadata:
  version: 1.1.0
  category: operations
  tags: kubernetes, operations
  aliases: kubernetes, k8s, pods, deployment, helm, RBAC, pod security, network policies, seguridad Kubernetes
  required_tools: file_read,file_write,command_run
  requires: Authorized cluster access and available kubectl/Helm tooling
  effects: mixed
---
# Kubernetes Operations

## Scope

Inspect and change Kubernetes workloads with explicit context, rollout and recovery evidence.

## Inputs

Cluster/context, namespace, workload and desired operation. Prerequisite: Authorized cluster access and available kubectl/Helm tooling.

## Workflow

1. Confirm context and namespace before commands. Read events, pod conditions, deployment state and relevant logs to separate scheduling, image, readiness and application failures.
2. Prepare narrow declarative changes and inspect the diff. Preserve selectors, storage claims, service accounts and compatibility with deployed versions.
3. Apply within authorization and monitor rollout status plus user-facing health. Distinguish readiness from liveness; a restart loop may be caused by the probe itself.
4. On failure, inspect current revisions and data effects before rollback. Never assume deleting a pod repairs persistent storage or database changes.

For Kubernetes security review, load [cluster security](references/cluster-security.md). Evaluate effective RBAC, admission and network enforcement together, including service accounts and ephemeral containers.

## Verification

Check desired/ready replicas, endpoints and an actual request; report degraded dependencies.

## Limits

Do not grant cluster-admin or disable isolation to make a deployment work. This skill does not provision cluster access.
