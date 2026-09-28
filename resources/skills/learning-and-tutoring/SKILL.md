---
name: learning-and-tutoring
description: Teach a topic with focused explanations, worked examples and feedback on demonstrated understanding.
metadata:
  version: 1.1.0
  category: personal
  tags: learning, and, tutoring
  aliases: aprender, tutor, estudiar, ejercicios, explicación, Anki, flashcards, tarjetas de repaso, repetición espaciada
  required_tools: file_read,file_write,tool_search
  requires: Topic and available learning materials
  effects: local-write
---
# Learning And Tutoring

## Scope

Teach a topic with focused explanations, worked examples and feedback on demonstrated understanding.

## Inputs

Learner goal, prior knowledge, constraints and preferred format. Prerequisite: Topic and available learning materials.

## Workflow

1. Identify the specific concept or skill blocking progress using the user’s example; do not assume a full curriculum is needed.
2. Explain with a concrete example and connect new ideas to known concepts, preserving technical accuracy.
3. Offer an appropriately challenging exercise or ask for the learner’s reasoning when useful; respond to the actual misconception.
4. Adapt the next step from demonstrated understanding and provide a concise reference or practice plan if requested.

## Verification

For study cards, Anki notes or spaced practice, read [retrieval practice](references/retrieval-practice.md). Use the learner's actual material and preserve the distinction between generating cards and changing a live deck.

Check whether the learner can apply the concept to a changed example, not merely repeat a definition.

## Limits

Do not invent learning diagnoses or force a quiz when the user asked for a direct answer.
