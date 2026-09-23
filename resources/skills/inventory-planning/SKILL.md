---
name: inventory-planning
description: Plan stock replenishment by SKU and location using inventory position, demand uncertainty, lead times and ordering constraints. Planificar existencias y reposición.
metadata:
  version: 1.0.0
  category: business
  tags: inventory, replenishment, stock, demand
  aliases: reponer inventario, stock de seguridad, punto de pedido, faltantes SKU, inventory replenishment, reorder point, safety stock, stockout
  required_tools: file_read,file_write,command_run
  requires: Dated stock and demand records, units, lead times and the owner's replenishment policy
  effects: local-write
---
# Inventory Planning

## Scope

Produce an explainable replenishment proposal and exception list. Use time-series-forecasting to build demand models and procurement-and-vendors to compare supplier terms. A proposed order is not a placed purchase order.

## Inputs

SKU/location, snapshot date, units and pack conversions, usable on-hand stock, allocations/backorders, expected receipts, demand history, lead times, review cadence, service objective, minimum order quantities, shelf life, capacity and budget.

## Workflow

1. Reconcile stock sources and units before calculating. Separate saleable, reserved, damaged and quarantined stock. Inspect missing periods and stockouts: observed sales under a stockout are not unconstrained demand. Retain unknowns and disputed counts.
2. Define inventory position from the system's actual ledger: usable on-hand plus eligible confirmed receipts minus unmet commitments. Do not subtract reservations twice when usable stock already excludes them, or subtract backorders again when they are included in commitments. Time-phase receipts so a late delivery cannot cover an earlier shortage.
3. Choose a demand baseline for the replenishment horizon, including lead time and review interval. Preserve promotion effects, intermittency, seasonality and uncertainty. New-item analogues are assumptions, not measured history. Compare methods with time-ordered backtests; zero-demand periods make percentage-error metrics unreliable.
4. Use the agreed service objective and distinguish cycle service level from unit fill rate. Estimate demand during the relevant horizon from an appropriate distribution or empirical scenarios. Do not apply a universal normal-distribution safety-stock formula to sparse or changing demand, or invent a standard service percentage.
5. Compute the unconstrained replenishment need, then apply pack size, MOQ, supplier calendar, receipt timing, capacity, expiry and budget constraints. Report the adjustment and any excess or unmet need. An infeasible constraint set stays infeasible; do not produce a negative quantity or silently relax a requirement.
6. Deliver a dated SKU/location table with input provenance, projected shortage interval, proposed quantity/arrival, assumptions and exception reason. Show delayed-supplier and demand-change scenarios where they affect the decision. Keep recommendation separate from ERP writes, supplier messages and purchase commitments.

## Verification

Reconcile totals by unit/location and manually inspect representative rows: incoming stock, overlapping commitments, zero demand, MOQ rounding and an arrival after a projected stockout. If formulas or code were added, verify those boundary cases with synthetic data. Preserve the original snapshot for replay.

## Limits

A forecast cannot guarantee availability. Policy thresholds and lead-time assumptions must come from the owner or evidence. Do not change inventory records, place orders, mark down products or cancel receipts merely because the analysis suggests it.

## Español

Calcula reposición por producto y ubicación con unidades y fechas verificadas. Evita descontar reservas dos veces y contar entregas tardías como cobertura de faltantes anteriores. Separa demanda, incertidumbre, nivel de servicio y restricciones de compra; entrega propuestas explicables sin emitir pedidos automáticamente.
