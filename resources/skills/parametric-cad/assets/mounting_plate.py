#!/usr/bin/env python3
"""Editable example in millimeters; generate STEP/STL into a new output directory."""

import argparse
import math
from pathlib import Path


def make_plate(width=40.0, depth=30.0, thickness=4.0, hole_diameter=5.0, hole_pitch=24.0):
    import build123d as cad

    dimensions = (width, depth, thickness, hole_diameter, hole_pitch)
    if not all(math.isfinite(v) and v > 0 for v in dimensions):
        raise ValueError("Dimensions must be positive and finite")
    if hole_pitch <= hole_diameter or hole_pitch + hole_diameter >= width or hole_diameter >= depth:
        raise ValueError("Holes must remain separate and inside the plate")
    plate = cad.Box(width, depth, thickness)
    for x in (-hole_pitch / 2, hole_pitch / 2):
        plate = plate - cad.Pos(x, 0, 0) * cad.Cylinder(hole_diameter / 2, thickness * 2)
    plate.label = "mounting_plate"
    return plate


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    # A new directory avoids replacing an existing design with example geometry.
    args.output.mkdir(parents=True, exist_ok=False)
    import build123d as cad

    plate = make_plate()
    cad.export_step(plate, args.output / "mounting_plate.step")
    cad.export_stl(plate, args.output / "mounting_plate.stl")


if __name__ == "__main__":
    main()
