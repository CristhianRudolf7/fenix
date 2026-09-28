---
name: rag-evaluation
description: Design and evaluate retrieval-augmented workflows against grounded questions and access boundaries.
metadata:
  version: 1.1.0
  category: data
  tags: rag, evaluation
  aliases: RAG, recuperación, embeddings, búsqueda híbrida, reranking
  required_tools: file_read,file_write,command_run
  requires: Corpus, representative questions and available retrieval tools
  effects: local-write
---
# Rag Evaluation

## Scope

Design and evaluate retrieval-augmented workflows against grounded questions and access boundaries.

## Inputs

Question types, evidence requirements, access scope and latency/cost constraints. Prerequisite: Corpus, representative questions and available retrieval tools.

## Workflow

1. Establish a lexical baseline and a set of answerable/unanswerable questions with relevant passages. Do not assume embeddings are required.
2. Preserve document IDs, versions, section boundaries and permissions in indexing. Compare chunking and retrieval methods using the same corpus/questions.
3. Measure retrieval coverage/ranking separately from answer grounding; a fluent answer can hide retrieval failure.
4. Test stale documents, conflicting evidence, missing answers and cross-user leakage. Add reranking or semantic search only when observed gains justify cost.

For security claims, record the actual caller, server-side permission filter, retrieved document IDs and reranked/context results. Test through the user-reachable API; an administrator database query is a different threat surface. Cosine similarity neither guarantees top-k retrieval nor demonstrates embedding inversion or corpus membership. Use agent-security-evaluation for controlled poisoned-document and cross-tenant memory tests, with exposure confirmed before interpreting model outcomes.

## Verification

Report citation support, abstention, retrieval metrics, latency and total task cost on held-out questions.

## Limits

Do not infer relationships or truth from vector similarity or expose unauthorized documents during candidate retrieval.
