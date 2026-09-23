#!/usr/bin/env python3
"""Read DXF units, bounds and entity diagnostics; do not certify fabrication readiness."""

import argparse
from collections import Counter
import json
from pathlib import Path
import sys


def inspect_drawing(path):
    import ezdxf
    from ezdxf import bbox, units

    document = ezdxf.readfile(path)
    space = document.modelspace()
    entities = list(space)
    if not entities:
        raise ValueError("Drawing has no model-space entities")
    types = Counter(entity.dxftype() for entity in entities)
    unit = int(document.header.get("$INSUNITS", 0))
    try:
        scale = units.conversion_factor(unit, units.MM) if unit else None
    except (ValueError, TypeError):
        scale = None
    box = bbox.extents(entities)
    bounds = None
    if box.has_data and scale is not None:
        bounds = [[float(v) * scale for v in corner] for corner in (box.extmin, box.extmax)]
    unsupported = sorted(set(types) - {"LINE", "ARC", "CIRCLE", "LWPOLYLINE", "POLYLINE", "SPLINE", "ELLIPSE"})
    polylines = [entity for entity in entities if entity.dxftype() == "LWPOLYLINE"]
    return {
        "units_code": unit, "units_known": scale is not None, "bounds_mm": bounds,
        "entity_count": len(entities), "entity_types": dict(sorted(types.items())),
        "layers": sorted({entity.dxf.layer for entity in entities}),
        "open_lwpolylines": sum(not entity.closed for entity in polylines),
        "unsupported_entity_types": unsupported,
        "manufacturing_ready_verified": False,
        "unchecked": ["Contour connectivity and self-intersections", "Duplicate/overlapping cuts", "Bend calculations and supplier-specific limits", "Complete bounding-box coverage for unsupported entities"],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path)
    args = parser.parse_args()
    try:
        result = inspect_drawing(args.input)
        print(json.dumps(result, indent=2, allow_nan=False))
        return 0
    except Exception as error:
        print(json.dumps({"error": str(error)}))
        return 1


if __name__ == "__main__":
    sys.exit(main())
