---
name: payment-and-billing
description: Implement payment and subscription workflows with verified events, idempotency and reconciled financial
  state.
metadata:
  version: 1.0.0
  category: engineering
  tags: payment, and, billing
  aliases: pagos, facturación, suscripción, stripe, webhook pago
  required_tools: file_read,file_write,command_run
  requires: Authorized payment sandbox/connector and current provider documentation
  effects: mixed
---
# Payment And Billing

## Scope

Implement payment and subscription workflows with verified events, idempotency and reconciled financial state.

## Inputs

Checkout/subscription flow, currency, tax source, ledger and failure requirements. Prerequisite: Authorized payment sandbox/connector and current provider documentation.

## Workflow

1. Use provider-supported tokenization/hosted collection; keep sensitive payment details out of application logs and ordinary storage.
2. Create operations with stable idempotency keys and reconcile provider IDs with internal orders. A browser redirect alone does not prove payment.
3. Verify webhook signatures and handle repeated/out-of-order events with explicit state transitions. Separate authorization, capture, settlement, refund and dispute.
4. Test failures, duplicate submissions, subscription changes and reconciliation in the provider sandbox before authorized live changes.

## Verification

Check amounts in the correct currency units, ledger consistency and exactly which provider state supports fulfillment.

## Limits

Do not charge/refund real accounts during an implementation test or assume cancellation automatically refunds a payment.

## Español

Valida eventos y estados financieros, controla duplicados y prueba en sandbox sin efectuar cargos reales por inferencia.
