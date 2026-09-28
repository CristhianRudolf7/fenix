---
name: cad-drawing-and-fabrication
description: Prepare and inspect 2D DXF cutting drawings, fabrication layers and supplier-specific handoffs with explicit units and geometric checks. Planos DXF y corte láser.
metadata:
  version: 1.0.0
  category: engineering
  tags: cad, dxf, drawing, fabrication, laser
  aliases: DXF, dibujo 2D, corte láser, waterjet, gasket, flat pattern, SendCutSend, plano fabricación
  required_tools: file_read,file_write,command_run,web_read
  requires: DXF tooling such as ezdxf; actual material and process requirements for fabrication verdicts
  effects: local-write
---
# CAD Drawing and Fabrication

## Scope

Create or inspect a dimensioned 2D fabrication artifact. Use parametric-cad for its source solid. A flat view of a bent part is not an unfolded flat pattern; bend allowance needs material, tooling and a defined calculation.

## Inputs

Geometry, units, datum, thickness, process, cut/engrave/bend operations and selected supplier if any. Preserve the editable source and requested DXF version.

## Workflow

1. Generate planar geometry at the intended scale. Keep operation layers separate, with text converted to outlines only when the target CAM consumer requires it. Do not silently project nonplanar geometry onto XY.
2. Reopen the exact DXF and check declared units, extents, entity types and layers. Inspect open profiles, duplicate edges, zero-length segments, self-intersections, overlapping cuts and unsupported annotations with the appropriate geometry tools.
3. For a projection from a 3D model, record the plane and transform. For a flat pattern, retain bend calculation inputs. Reconcile holes and external dimensions against the source model.
4. When supplier readiness is requested, fetch current official requirements for the selected process, material and thickness. Compare measured features with those requirements and mark missing rules as unknown. A generic minimum-feature table does not establish acceptance for every supplier.
5. Provide the drawing, source, inspection results and unresolved fabrication checks. Uploading a drawing or placing an order is a separate authorized action.

## References

Use [DXF inspection and fabrication checks](references/drawing-checks.md). `scripts/inspect_drawing.py` provides read-only units, bounds and entity diagnostics through ezdxf; it explicitly leaves contour connectivity and manufacturing fitness unverified.

## Verification

Check both the actual geometry and its intended consuming application when available. Do not claim a cutting-ready result from a successful parse or a screenshot alone.

## Limits

No vendor account, ordering integration, CNC controller or cutter is installed by this skill. Treat unknown units, unsupported entities and incomplete contour checks as unresolved rather than passing.
