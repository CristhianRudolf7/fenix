---
name: load-and-resilience-testing
description: Evaluate service capacity and failure recovery with representative workloads and controlled fault injection.
metadata:
  version: 1.0.0
  category: operations
  tags: load, and, resilience, testing
  aliases: prueba carga, estrés, resiliencia, concurrencia, throughput
  required_tools: file_read,file_write,command_run
  requires: Authorized test environment and load generator
  effects: mixed
---
# Load And Resilience Testing

## Scope

Evaluate service capacity and failure recovery with representative workloads and controlled fault injection.

## Inputs

Workload distribution, success metric, target capacity and permitted failure scope. Prerequisite: Authorized test environment and load generator.

## Workflow

1. Build a workload from real operation mixes and think times; avoid a synthetic endpoint that misses the expensive path.
2. Ramp load while tracking throughput, errors, queueing and latency distribution. Ensure the generator itself is not the bottleneck.
3. Inject only authorized bounded failures and observe recovery, retry amplification and duplicate effects.
4. Return to baseline and clean test data. Compare configurations under the same workload and resource conditions.

## Verification

Report achieved load, percentiles, errors, saturation point and recovery evidence; include incomplete trials.

## Limits

Do not stress a public or production service without authorization or turn timeout removal into a performance fix.
