---
name: huggingface-hub
description: Work with Hugging Face models, datasets, paper pages, repository files and authorized Jobs using the installed CLI or supported APIs. Hub, datasets y trabajos de Hugging Face.
metadata:
  version: 1.1.0
  category: data
  tags: huggingface, hub, datasets, jobs, repositories
  aliases: Hugging Face, HuggingFace, HF Hub, hf CLI, datasets-server, HF Jobs, subir modelo, dataset remoto
  required_tools: web_search,web_read,file_read,file_write,command_run,tool_search
  requires: Public Hub access or authorized credentials; compatible CLI/SDK for remote mutations
  effects: mixed
---
# Hugging Face Hub

## Scope

Search and inspect Hub artifacts, retrieve dataset slices and manage explicitly requested repository or job operations. Use model-selection for recommendations, ml-training for training methods and ml-demo-apps for Gradio/Spaces applications.

## Inputs

Repository ID and type, revision, relevant files, intended read or write operation and access scope. Do not assume a model repository when the user supplied a dataset or Space.

## Workflow

1. Check installed `hf --help` and the selected subcommand, or the available SDK/API documentation. Prefer current supported interfaces over pasted historical commands. Fénix does not necessarily have the upstream repository's `hf_jobs` MCP tool.
2. Resolve repository identity, immutable revision and artifact metadata. Inspect cards, license, architecture and file sizes before downloads. Fetch selected files or bounded dataset pages instead of whole weights/corpora when metadata answers the question.
3. For Dataset Viewer operations, resolve config and split before rows; honor pagination, totals and partial/error states. A preview is not the full dataset. Read [Hub data and paper operations](references/hub-data.md) when those workflows apply.
4. For writes, use the requested namespace, visibility and exact file set. Review the outgoing diff and protect existing card metadata. An analysis request does not authorize uploading agent history, changing visibility or claiming authorship.
5. For authorized remote compute, read [job lifecycle](references/jobs.md), establish resources and budget, submit once, preserve the returned job ID and inspect terminal status and persisted artifacts. An ambiguous submission response needs reconciliation before resubmission.

When the user supplies a repository URL or ID, inspect its card and file metadata directly instead of first doing a general web search. Do not install a CLI merely to read a public file that available HTTP tools can retrieve. Before filtering, inspect the actual categorical values, schema, configuration and split; prose labels may differ from stored labels. Load each dataset/tokenizer once per computation, then batch filtering, independent subtotals, result writing and readback. Preserve exact tokenizer revision and special-token settings when counts depend on them.

## Verification

Record resolved revisions, retrieved coverage and output paths. Verify remote changes by their commit/object identity. Distinguish local preparation from uploaded content and queued jobs from successful output.

## Limits

This skill installs no account or provider. Use existing credential mechanisms without printing tokens or reading unrelated credential files. Repository code, pickled weights and `trust_remote_code` require code review before execution; a download is not permission to execute it.
