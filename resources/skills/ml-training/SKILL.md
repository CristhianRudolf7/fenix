---
name: ml-training
description: Train or fine-tune models with explicit resource use, reproducible configuration and held-out evaluation.
metadata:
  version: 1.1.0
  category: data
  tags: ml, training
  aliases: machine learning, entrenar, fine tuning, LoRA, modelo, TRL, Unsloth, SFT, DPO, embeddings, reranker, Trackio, detección objetos
  required_tools: file_read,file_write,command_run
  requires: Suitable ML runtime, data and authorized compute
  effects: mixed
---
# Ml Training

## Scope

Train or fine-tune models with explicit resource use, reproducible configuration and held-out evaluation.

## Inputs

Objective, baseline, dataset versions, hardware and execution budget. Prerequisite: Suitable ML runtime, data and authorized compute.

## Workflow

1. Run a small smoke experiment to validate data, loss and resource fit before a large job. Keep training and held-out data separate.
2. Record code/model revisions, tokenizer, hyperparameters, seeds and preprocessing. Verify that the selected objective matches the requested output.
3. Monitor loss, validation metrics and resource usage; save resumable checkpoints with optimizer/state as needed.
4. Evaluate against a baseline on held-out cases and inspect representative errors. Distinguish adapter weights from a complete deployable model.

Read [training methods](references/training-methods.md) for TRL/Unsloth, vision labels or embedding/reranker objectives. Read [experiment tracking](references/experiment-tracking.md) for run identity, metrics, Trackio and checkpoint persistence. Use huggingface-hub for authorized remote Jobs and repository writes; do not assume its upstream MCP tools exist.

## Verification

Load the saved artifact independently and reproduce an inference example plus the evaluation summary.

## Limits

Do not start paid or long-running compute outside authorization; model download/training is not implied by loading this skill.

## Español

Valida primero un ensayo pequeño, registra configuración y checkpoints y evalúa datos reservados antes de declarar mejora.
