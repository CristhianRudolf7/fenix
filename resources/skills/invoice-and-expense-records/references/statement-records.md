# Statement reconciliation / Conciliación de movimientos

Preserve the original export and its locale/sign convention before normalizing. Posted and pending rows can represent the same transaction at different stages; use provider identity and explicit replacement evidence, not equal amount/date alone. Distinguish transactions from invoices and one payment from an invoice settlement: partial, grouped and refunded payments require links and residual balances.

Keep account and currency separate. Internal transfers affect account balances but are not automatically expenses or income in a combined budget. Category guesses remain suggestions until supported. A complete arithmetic balance does not prove all transactions are authentic, correctly classified or tax-ready.

The local helper takes JSON with `accounts` and `transactions` arrays:

```json
{
  "accounts": [{"id": "checking", "currency": "USD", "opening": "100.00", "closing": "90.00"}],
  "transactions": [{"id": "bank-001", "account": "checking", "currency": "USD", "amount": "-10.00", "status": "posted"}]
}
```

Run `python3 <skill-directory>/scripts/reconcile_statement.py normalized.json`. Amounts must be signed decimal strings without grouping separators or exponents; normalization is explicit, not guessed. Identical duplicate IDs within an account are counted once and reported; conflicting duplicates fail. Pending rows are excluded and counted. A closing discrepancy returns exit 2; invalid input returns 1. Output does not infer categories, delete rows, link invoices or perform payments.

ES: normaliza signos y formatos de forma explícita, separa cuentas/monedas y conserva pagos parciales, reembolsos y transferencias. Cuadrar un saldo es una comprobación aritmética, no una certificación contable.
