#!/usr/bin/env python3
"""Check explicitly normalized statement balances without categorizing or paying."""

import argparse
from decimal import Decimal, localcontext
import json
from pathlib import Path
import re
import sys


def require(condition, message):
    if not condition:
        raise ValueError(message)


def money(value):
    require(isinstance(value, str) and len(value) <= 50 and
            re.fullmatch(r"-?\d{1,30}(?:\.\d{1,12})?", value), "Amounts must be bounded plain decimal strings")
    return Decimal(value)


def identifier(value):
    require(isinstance(value, str) and bool(value.strip()) and len(value) <= 256, "Invalid identifier")
    return value


def reconcile(document):
    require(isinstance(document, dict), "Expected a JSON object")
    accounts, transactions = document.get("accounts"), document.get("transactions")
    require(isinstance(accounts, list) and 0 < len(accounts) <= 10000, "Expected a bounded accounts array")
    require(isinstance(transactions, list) and len(transactions) <= 100000, "Expected a bounded transactions array")
    balances = {}
    for row in accounts:
        require(isinstance(row, dict), "Invalid account")
        key, currency = identifier(row.get("id")), row.get("currency")
        require(isinstance(currency, str) and bool(re.fullmatch(r"[A-Z]{3}", currency)), "Use a three-letter currency code")
        require(key not in balances, "Duplicate account ID")
        balances[key] = {"currency": currency, "opening": money(row.get("opening")),
                         "closing": money(row.get("closing")), "net": Decimal(0), "posted": 0, "pending": 0}
    seen, duplicates = {}, []
    # Bounds above make this precision exact even when every input has 30 integer digits.
    with localcontext() as context:
        context.prec = 64
        for row in transactions:
            require(isinstance(row, dict), "Invalid transaction")
            account, key = identifier(row.get("account")), identifier(row.get("id"))
            require(account in balances, "Unknown account")
            balance = balances[account]
            require(row.get("currency") == balance["currency"], "Transaction/account currency mismatch")
            status = row.get("status")
            require(status in ("posted", "pending"), "Status must be posted or pending")
            amount = money(row.get("amount"))
            identity = (account, key)
            if identity in seen:
                require(seen[identity] == row, "Conflicting duplicate transaction ID")
                duplicates.append({"account": account, "id": key})
                continue
            seen[identity] = row
            balance[status] += 1
            if status == "posted":
                balance["net"] += amount
        results = []
        for key, row in balances.items():
            calculated = row["opening"] + row["net"]
            difference = row["closing"] - calculated
            results.append({"account": key, "currency": row["currency"], "posted": row["posted"],
                            "pending_excluded": row["pending"], "calculated_closing": str(calculated),
                            "declared_closing": str(row["closing"]), "difference": str(difference),
                            "balanced": difference == 0})
    return {"accounts": results, "duplicate_rows": duplicates,
            "balanced": all(row["balanced"] for row in results),
            "classification_verified": False, "source_authenticity_verified": False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path)
    args = parser.parse_args()
    try:
        with args.input.open("rb") as stream:
            data = stream.read(16 * 1024 * 1024 + 1)
        require(len(data) <= 16 * 1024 * 1024, "Input exceeds 16 MiB")
        result = reconcile(json.loads(data))
    except (ValueError, OSError, TypeError) as error:
        print(json.dumps({"error": str(error)}))
        return 1
    print(json.dumps(result, indent=2))
    return 0 if result["balanced"] else 2


if __name__ == "__main__":
    sys.exit(main())
