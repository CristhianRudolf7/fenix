#!/usr/bin/env python3
"""Measure saved STEP or mesh geometry, without repairing or exporting the input."""

import argparse
import hashlib
import json
import math
from pathlib import Path
import sys

UNIT_SCALE = {"mm": 1.0, "cm": 10.0, "m": 1000.0, "in": 25.4}


def inspect_geometry(path, mesh_units=None):
    path = Path(path)
    if path.suffix.lower() in {".step", ".stp"}:
        import build123d as cad

        shape = cad.import_step(path)
        solids = list(shape.solids())
        if not solids:
            raise ValueError("No solids found; this inspector expects physical solid parts")
        box = shape.bounding_box()
        size = [float(box.size.X), float(box.size.Y), float(box.size.Z)]
        volume = sum(float(solid.volume) for solid in solids)
        result = {
            "format": "STEP", "units": "mm", "solid_count": len(solids),
            "size_mm": size, "volume_mm3": volume,
            "valid_geometry": bool(shape.is_valid) and all(s.is_valid for s in solids) and volume > 0,
            "limitations": ["Assembly volumes are summed; overlaps are not subtracted.", "No strength, tolerance-fit or manufacturability certification."],
        }
    elif path.suffix.lower() in {".stl", ".obj", ".ply"}:
        import numpy as np
        import trimesh

        if mesh_units not in UNIT_SCALE:
            raise ValueError("Mesh inspection requires explicit --mesh-units")
        mesh = trimesh.load_mesh(path, process=False)
        if not isinstance(mesh, trimesh.Trimesh) or len(mesh.faces) == 0:
            raise ValueError("Expected a nonempty triangle mesh")
        if not np.isfinite(mesh.vertices).all():
            raise ValueError("Mesh contains nonfinite coordinates")
        mesh.apply_scale(UNIT_SCALE[mesh_units])
        degenerate = int(np.count_nonzero(mesh.area_faces <= 1e-12))
        # STL duplicates per-face vertices. Weld only an inspection copy; do not repair the file.
        topology = mesh.copy()
        topology.merge_vertices()
        closed = bool(topology.is_watertight)
        winding = bool(topology.is_winding_consistent)
        volume = float(topology.volume) if closed and winding else None
        result = {
            "format": path.suffix.lower()[1:], "declared_input_units": mesh_units, "units": "mm",
            "face_count": len(mesh.faces), "size_mm": mesh.extents.tolist(),
            "volume_mm3": volume, "watertight": closed, "winding_consistent": winding,
            "degenerate_faces": degenerate,
            "valid_geometry": closed and winding and degenerate == 0 and volume is not None and volume > 0,
            "limitations": ["Topology uses a welded inspection copy at trimesh's default precision.", "Self-intersections, minimum wall thickness, supports and physical printability are not checked."],
        }
    else:
        raise ValueError("Supported inputs: STEP/STP, STL, OBJ, PLY")
    if not all(math.isfinite(value) and value >= 0 for value in result["size_mm"]):
        raise ValueError("Invalid geometry bounds")
    if result["volume_mm3"] is not None and not math.isfinite(result["volume_mm3"]):
        raise ValueError("Invalid geometry volume")
    with path.open("rb") as stream:
        result["sha256"] = hashlib.file_digest(stream, "sha256").hexdigest()
    return result


def check_expectations(result, expected_size=None, expected_solids=None, tolerance=0.001):
    if not math.isfinite(tolerance) or tolerance < 0:
        raise ValueError("Tolerance must be finite and nonnegative")
    failures = []
    if not result["valid_geometry"]:
        failures.append("Geometry did not pass the inspector's limited validity checks")
    if expected_size is not None:
        if len(expected_size) != 3 or not all(math.isfinite(v) and v > 0 for v in expected_size):
            raise ValueError("Expected dimensions must be three positive finite values")
        if any(abs(a - b) > tolerance for a, b in zip(result["size_mm"], expected_size)):
            failures.append("Saved dimensions differ from expected dimensions")
    if expected_solids is not None:
        if type(expected_solids) is not int or expected_solids < 1 or "solid_count" not in result:
            raise ValueError("Expected solid count requires STEP and a positive integer")
        if result["solid_count"] != expected_solids:
            failures.append("Saved solid count differs from expected count")
    return failures


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path)
    parser.add_argument("--mesh-units", choices=sorted(UNIT_SCALE))
    parser.add_argument("--expect-size-mm", nargs=3, type=float)
    parser.add_argument("--expect-solids", type=int)
    parser.add_argument("--tolerance-mm", type=float, default=0.001)
    args = parser.parse_args()
    try:
        result = inspect_geometry(args.input, args.mesh_units)
        result["failed_checks"] = check_expectations(result, args.expect_size_mm, args.expect_solids, args.tolerance_mm)
        print(json.dumps(result, indent=2, allow_nan=False))
        return 2 if result["failed_checks"] else 0
    except (OSError, ValueError, RuntimeError, ImportError) as error:
        print(json.dumps({"error": str(error)}))
        return 1


if __name__ == "__main__":
    sys.exit(main())
