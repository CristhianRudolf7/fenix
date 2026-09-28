---
name: procurement-and-vendors
description: Compare suppliers and renewal decisions using requirements, total cost and contract evidence.
metadata:
  version: 1.0.0
  category: business
  tags: procurement, and, vendors
  aliases: compras empresa, proveedor, renovación, procurement, licencias
  required_tools: file_read,file_write,tool_search
  requires: Requirements, quotes/contracts and authorized usage evidence
  effects: local-write
---
# Procurement And Vendors

## Scope

Compare suppliers and renewal decisions using requirements, total cost and contract evidence.

## Inputs

Required capability, volume, budget, deadlines and existing commitments. Prerequisite: Requirements, quotes/contracts and authorized usage evidence.

## Workflow

1. Build a requirements matrix from actual needs and separate mandatory from preferred criteria.
2. Normalize quotes by units, term, currency, implementation, support, exit and renewal conditions. Identify unused licenses using usage evidence.
3. Compare reliability, support, integration and data portability alongside price; retain uncertain or unverified vendor claims.
4. Deliver a recommendation and negotiation questions with evidence; record notice periods and decisions needing an owner.

## Verification

Reconcile total cost for the same term and workload and check renewal dates against the contract.

## Limits

Do not cancel subscriptions, sign contracts or purchase licenses merely because the analysis recommends it.
