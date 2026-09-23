#!/usr/bin/env python3
"""Check a limited set of URDF structural and numeric invariants offline."""

import argparse
import json
import math
from pathlib import Path
import sys
import xml.etree.ElementTree as ET


def finite(value, label):
    try:
        number = float(value)
    except (ValueError, TypeError) as error:
        raise ValueError(f"Missing or invalid {label}") from error
    if not math.isfinite(number):
        raise ValueError(f"Nonfinite {label}")
    return number


def vector(value, label):
    values = str(value).split()
    if len(values) != 3:
        raise ValueError(f"{label} needs three coordinates")
    return [finite(v, label) for v in values]


def index(elements, label):
    result = {}
    for element in elements:
        name = element.get("name")
        if not name or name in result:
            raise ValueError(f"Missing or duplicate {label} name")
        result[name] = element
    return result


def check_urdf(data):
    if len(data) > 8 * 1024 * 1024:
        raise ValueError("URDF exceeds 8 MiB")
    upper = data.upper()
    if b"<!DOCTYPE" in upper or b"<!ENTITY" in upper:
        raise ValueError("DTD/entity declarations are not supported")
    root = ET.fromstring(data)
    if root.tag != "robot" or not root.get("name"):
        raise ValueError("Expected a named URDF robot")
    if any("}" in element.tag for element in root.iter()):
        raise ValueError("Expand xacro/namespaced extensions with their actual tool before this check")
    links = index(root.findall("link"), "link")
    joints = index(root.findall("joint"), "joint")
    if not links:
        raise ValueError("Robot has no links")
    parents = {}
    children = {name: [] for name in links}
    mimic = {}
    for name, joint in joints.items():
        parent, child = joint.find("parent"), joint.find("child")
        if parent is None or child is None:
            raise ValueError(f"Joint {name} needs a parent and child")
        a, b = parent.get("link"), child.get("link")
        if a not in links or b not in links or a == b or b in parents:
            raise ValueError(f"Invalid parent/child topology at {name}")
        parents[b] = a
        children[a].append(b)
        kind = joint.get("type")
        if kind not in {"fixed", "revolute", "continuous", "prismatic", "floating", "planar"}:
            raise ValueError(f"Unknown joint type at {name}")
        axis = joint.find("axis")
        if axis is not None:
            xyz = vector(axis.get("xyz"), "joint axis")
            if sum(v * v for v in xyz) <= 0:
                raise ValueError(f"Zero joint axis at {name}")
        if kind in {"revolute", "continuous", "prismatic"}:
            limit = joint.find("limit")
            if limit is None:
                raise ValueError(f"Missing joint limits at {name}")
            if finite(limit.get("effort"), "effort") <= 0 or finite(limit.get("velocity"), "velocity") <= 0:
                raise ValueError(f"Nonpositive effort/velocity at {name}")
            if kind != "continuous" and finite(limit.get("lower"), "lower limit") > finite(limit.get("upper"), "upper limit"):
                raise ValueError(f"Reversed joint limits at {name}")
        related = joint.find("mimic")
        if related is not None:
            target = related.get("joint")
            if target not in joints or target == name:
                raise ValueError(f"Unknown or self-referential mimic joint at {name}")
            mimic[name] = target
            for attr in ("multiplier", "offset"):
                if attr in related.attrib:
                    finite(related.get(attr), attr)
    roots = sorted(set(links) - set(parents))
    if len(roots) != 1:
        raise ValueError("Robot must have exactly one root")
    seen, pending = set(), [roots[0]]
    while pending:
        current = pending.pop()
        if current in seen:
            raise ValueError("Robot link graph contains a cycle")
        seen.add(current)
        pending.extend(children[current])
    if seen != set(links):
        raise ValueError("Robot link graph is disconnected or contains a cycle")
    for name in mimic:
        visited = set()
        current = name
        while current in mimic:
            if current in visited:
                raise ValueError("Mimic joints contain a cycle")
            visited.add(current)
            current = mimic[current]
    missing_inertials = []
    for name, link in links.items():
        inertial = link.find("inertial")
        if inertial is None:
            missing_inertials.append(name)
            continue
        mass, inertia = inertial.find("mass"), inertial.find("inertia")
        if mass is None or inertia is None or finite(mass.get("value"), "mass") <= 0:
            raise ValueError(f"Invalid mass properties at {name}")
        xx, yy, zz, xy, xz, yz = [finite(inertia.get(k), k) for k in ("ixx", "iyy", "izz", "ixy", "ixz", "iyz")]
        determinant = xx * yy * zz + 2 * xy * xz * yz - xx * yz ** 2 - yy * xz ** 2 - zz * xy ** 2
        minor = xx * yy - xy ** 2
        if not math.isfinite(minor) or not math.isfinite(determinant) or xx <= 0 or minor <= 0 or determinant <= 0:
            raise ValueError(f"Inertia tensor is not positive definite at {name}")
    for origin in root.iter("origin"):
        for field in ("xyz", "rpy"):
            if field in origin.attrib:
                vector(origin.get(field), f"origin {field}")
    return {
        "static_status": "valid_for_checked_invariants", "robot": root.get("name"),
        "root_link": roots[0], "links": len(links), "joints": len(joints),
        "links_without_inertials": sorted(missing_inertials),
        "mesh_references_not_resolved": [m.get("filename") for m in root.iter("mesh")],
        "unchecked": ["Physical accuracy and inertia triangle inequalities", "Meshes, geometry dimensions and collisions", "Transmissions, ros2_control and simulator/planner execution", "SRDF, SDF and xacro semantics"],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path)
    args = parser.parse_args()
    try:
        with args.input.open("rb") as stream:
            result = check_urdf(stream.read(8 * 1024 * 1024 + 1))
    except (ValueError, OSError, OverflowError, ET.ParseError) as error:
        print(json.dumps({"static_status": "invalid", "error": str(error)}))
        return 1
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
