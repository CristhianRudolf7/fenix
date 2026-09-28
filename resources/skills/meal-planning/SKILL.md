---
name: meal-planning
description: Plan ordinary meals and shopping quantities from preferences, pantry, time and dietary constraints.
metadata:
  version: 1.0.0
  category: personal
  tags: meal, planning
  aliases: menú, comidas, recetas, lista compras, cocina
  required_tools: file_read,file_write,tool_search
  requires: User preferences and reliable recipe/food-safety references when needed
  effects: local-write
---
# Meal Planning

## Scope

Plan ordinary meals and shopping quantities from preferences, pantry, time and dietary constraints.

## Inputs

People, days, budget, available ingredients, allergies and cooking equipment. Prerequisite: User preferences and reliable recipe/food-safety references when needed.

## Workflow

1. Reuse known preferences and clarify allergies or constraints when absent and relevant. Do not infer a medical diet.
2. Choose feasible meals with overlapping ingredients, preparation time and storage needs; account for leftovers explicitly.
3. Scale quantities consistently and separate pantry items from purchases. Label nutrition values as estimates with their source when included.
4. Provide preparation/storage notes from reliable guidance where safety matters and a consolidated shopping list.

## Verification

Check ingredient conflicts, serving arithmetic and realistic storage/preparation timing.

## Limits

Do not prescribe treatment, extreme restriction or guarantees about allergen safety; verify labels and professional guidance for medical needs.
