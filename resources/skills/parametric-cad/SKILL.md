---
name: parametric-cad
description: Create, edit and inspect dimensioned 3D CAD parts or assemblies with editable parameters and verified STEP or mesh exports. Modelado CAD paramétrico y piezas 3D.
metadata:
  version: 1.0.0
  category: engineering
  tags: cad, parametric, geometry, step, build123d
  aliases: CAD, pieza 3D, modelo paramétrico, STEP, STL, build123d, bracket, soporte mecánico, ensamblaje
  required_tools: file_read,file_write,command_run,tool_search
  requires: A compatible CAD kernel such as build123d and dimensions or declared assumptions
  effects: local-write
---
# Parametric CAD

## Scope

Produce editable geometry and inspect saved CAD artifacts. Use cad-drawing-and-fabrication for 2D cut profiles, robot-description for kinematics and 3d-printing-preparation for slicing. This skill does not install a kernel or a CAD viewer implicitly.

## Inputs

Dimensions, units, coordinate datum, mating interfaces, material/process constraints and required output formats. Recover existing design intent and source parameters before changing geometry. An image alone does not establish physical scale.

## Workflow

1. Check the project interpreter and kernel API. Keep source, inputs and outputs distinct; reuse the existing modeling system. Express important dimensions and tolerances as named parameters and document unspecified dimensions.
2. Build the simplest valid feature sequence; check intermediate solids before adding fillets, booleans or assemblies. Derive placements and clearances mathematically. Preserve component names and separate purchased parts from fabricated geometry.
3. When a catalog component is required, verify its exact model, manufacturer dimensions, source and file checksum where supplied. Distinguish unavailable network access from an unsuccessful search. Mark simplified envelopes explicitly.
4. Regenerate from source and reopen the exported file with its real reader. Check topology, expected solids, dimensions, volume and interface clearances. Selection indices can change after rebuilding; reselect geometry by verified features.
5. Inspect meaningful views using an available viewer or local rendering tool and deliver source, parameters, exports and checks. Report unavailable visual inspection; never invent a live preview URL.

## References

Read [geometry and export checks](references/geometry-checks.md) for format boundaries. `scripts/inspect_geometry.py` measures STEP through build123d and mesh files through trimesh. `assets/mounting_plate.py` is a small editable build123d example, not a universal part generator.

For off-the-shelf geometry, read [catalog parts](references/catalog-parts.md) to search, validate identity and preserve download provenance.

## Verification

Compare requested dimensions with the saved artifact, not only source constants or an attractive rendering. Rebuild after parameter changes and inspect the new export. Units are explicit, especially for STL, which carries no reliable unit declaration.

## Limits

Kernel validity is not mechanical certification. Do not infer strength, tolerance fit, manufacturing readiness or safety from positive volume. A model request does not authorize manufacturing or operating machinery.
