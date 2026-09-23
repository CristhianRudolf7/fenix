---
name: ml-dataset-preparation
description: Prepare training and evaluation datasets with provenance, deduplication and split integrity.
metadata:
  version: 1.0.0
  category: data
  tags: ml, dataset, preparation
  aliases: dataset, entrenamiento, etiquetas, train test, fuga datos
  required_tools: file_read,file_write,command_run
  requires: Authorized data and available local processing tools
  effects: local-write
---
# Ml Dataset Preparation

## Scope

Prepare training and evaluation datasets with provenance, deduplication and split integrity.

## Inputs

Task, labeling definition, source rights, sampling and deployment population. Prerequisite: Authorized data and available local processing tools.

## Workflow

1. Record source versions, collection conditions and allowed uses. Define labels and ambiguous cases before annotation.
2. Deduplicate at the appropriate entity/content level and split by subject/time/group when random rows would leak information.
3. Fit preprocessing on training data only. Keep held-out labels and benchmark answers out of generated training examples.
4. Export versioned data and a manifest with counts, exclusions and known biases; protect sensitive fields.

## Verification

Check overlap across splits, label distributions, schema and reproducibility of transformations.

## Limits

Synthetic examples must be labeled as synthetic. Dataset availability does not imply permission to train or publish it.

## Español

Conserva procedencia y separa conjuntos sin fuga por entidad o tiempo; documenta exclusiones y datos sintéticos.
