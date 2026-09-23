#!/usr/bin/env python3
"""Estimate model weights and homogeneous dense/GQA/MQA KV cache; not a fit guarantee."""

import argparse
import json
import math
import sys

GIB = 1024 ** 3


def positive(value, name, integer=False):
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value) or value <= 0:
        raise ValueError(f"{name} must be positive and finite")
    if integer and type(value) is not int:
        raise ValueError(f"{name} must be an integer")
    return value


def estimate(parameters, weight_bits, kv=None, overhead_gib=None, available_gib=None):
    positive(parameters, "parameters", integer=True)
    positive(weight_bits, "weight_bits")
    weights = math.ceil(parameters * weight_bits / 8)
    cache = None
    missing = ["Weight storage is approximate: tensor metadata and mixed precision may add bytes."]
    if kv is not None:
        if set(kv) != {"layers", "kv_heads", "head_dim", "context", "batch", "kv_bits"}:
            raise ValueError("Provide exactly all six KV dimensions")
        for name, value in kv.items():
            positive(value, name, integer=True)
        cache = math.ceil(2 * kv["layers"] * kv["kv_heads"] * kv["head_dim"] * kv["context"] * kv["batch"] * kv["kv_bits"] / 8)
    else:
        missing.append("KV cache dimensions were not supplied.")
    overhead = None
    if overhead_gib is not None:
        positive(overhead_gib, "overhead_gib")
        overhead = math.ceil(overhead_gib * GIB)
    else:
        missing.append("Runtime activations, workspaces and allocator overhead were not supplied.")
    accounted = weights + (cache or 0) + (overhead or 0)
    assessment = "capacity_not_supplied"
    if available_gib is not None:
        positive(available_gib, "available_gib")
        assessment = "insufficient_under_estimate" if accounted > available_gib * GIB else "requires_runtime_measurement"
    return {
        "estimated_weight_bytes": weights, "estimated_kv_bytes": cache,
        "assumed_runtime_overhead_bytes": overhead, "accounted_bytes": accounted,
        "accounted_gib": accounted / GIB, "available_gib": available_gib,
        "capacity_assessment": assessment, "fit_verified": False,
        "limitations": missing + ["KV formula assumes homogeneous dense/GQA/MQA attention; MLA, hybrid or sliding-window layouts need their actual cache implementation."],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--parameters", required=True, type=int)
    parser.add_argument("--weight-bits", required=True, type=float)
    for name in ("layers", "kv-heads", "head-dim", "context", "batch", "kv-bits"):
        parser.add_argument("--" + name, type=int)
    parser.add_argument("--overhead-gib", type=float)
    parser.add_argument("--available-gib", type=float)
    args = parser.parse_args()
    values = {name: getattr(args, name) for name in ("layers", "kv_heads", "head_dim", "context", "batch", "kv_bits")}
    try:
        supplied = [v is not None for v in values.values()]
        if any(supplied) and not all(supplied):
            raise ValueError("Supply all KV dimensions or omit all of them")
        result = estimate(args.parameters, args.weight_bits, values if all(supplied) else None, args.overhead_gib, args.available_gib)
    except (ValueError, OverflowError) as error:
        print(json.dumps({"error": str(error)}))
        return 1
    print(json.dumps(result, indent=2, allow_nan=False))
    return 0


if __name__ == "__main__":
    sys.exit(main())
