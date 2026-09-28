---
name: simulation-modeling
description: Build and validate simulations with explicit assumptions, random inputs and sensitivity analysis.
metadata:
  version: 1.0.0
  category: research
  tags: simulation, modeling
  aliases: simulación, simpy, Monte Carlo, eventos discretos
  required_tools: file_read,file_write,tool_search
  requires: Simulation runtime and source assumptions/data
  effects: local-write
---
# Simulation Modeling

## Scope

Build and validate simulations with explicit assumptions, random inputs and sensitivity analysis.

## Inputs

System boundary, entities/resources, question, horizon and outputs. Prerequisite: Simulation runtime and source assumptions/data.

## Workflow

1. Define the conceptual model and time units before code. Record omitted mechanisms and the source of distributions.
2. Validate event ordering, resource release and conservation invariants on a small deterministic scenario.
3. Use independent replications and controlled seeds; separate warm-up from steady-state measurement when relevant.
4. Compare outputs to known cases or observed data and vary uncertain assumptions instead of reporting one trajectory as truth.

## Verification

Check invariants, sensitivity and confidence across replications, with a reproducible configuration.

## Limits

A simulation predicts under assumptions; it does not establish real-world causality or validate its own input distributions.
