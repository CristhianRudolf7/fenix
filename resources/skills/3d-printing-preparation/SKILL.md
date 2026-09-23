---
name: 3d-printing-preparation
description: Inspect mesh printability and prepare slicer-generated G-code using an actual printer, material and process profile. Preparar impresión 3D y revisar G-code.
metadata:
  version: 1.0.0
  category: engineering
  tags: printing, mesh, slicer, gcode, fabrication
  aliases: impresión 3D, imprimible, G-code, slicer, laminar, FDM, Bambu, OrcaSlicer, PrusaSlicer, DfAM
  required_tools: file_read,file_write,command_run,tool_search
  requires: Mesh with known units and a compatible installed slicer plus actual printer profile
  effects: local-write
---
# 3D Printing Preparation

## Scope

Inspect geometry for additive manufacturing and prepare local print artifacts. Preparing or slicing does not start a printer. Use parametric-cad for geometry changes and cad-drawing-and-fabrication for sheet cutting.

## Inputs

Exact mesh, units, target process, printer, nozzle, plate, material and approved native slicer profile. Recover these from the project; do not fabricate a profile for a real machine.

## Workflow

1. Inspect the saved mesh for finite coordinates, degenerate faces, scale, disconnected bodies, winding and closure. Use geometry tools for measurements; a render does not measure wall thickness or internal cavities.
2. Compare available measurements with current material/process guidance. State unmeasured wall thickness and trapped volume explicitly. Overhang support rules for FDM do not directly apply to powder processes; sampled estimates are not exact support volumes.
3. Discover the installed slicer's actual CLI and profile format. Prepare the precise input/output/profile command and inspect its effects. Run slicing within the existing authorization, saving stdout, exit status and the profile identity.
4. Inspect the produced G-code with the slicer's preview and appropriate validator. Check units, coordinate/extrusion modes, machine bounds, temperatures, start/end scripts and unexpected macros against that machine's contract. Unknown commands remain unresolved.
5. Hand off the file with profile, material and validation evidence. For an explicitly requested live print, use an existing authorized printer adapter, verify machine status and exact job identity, and confirm observed acceptance. If no adapter exists, report that boundary rather than inventing MQTT/FTPS commands.

## References

Read [printability and machine handoff](references/print-handoff.md) for evidence requirements. Geometry measurements are available through parametric-cad's inspector; slicer execution still needs a real installed slicer.

## Verification

Confirm actual output, extrusion/motion preview and profile identity. Slicing success is not proof of safe or successful physical manufacture. Distinguish prepared, uploaded, accepted and printed states.

## Limits

This skill adds no slicer, printer connection or manufacturer credentials. It does not package generic G-code as a vendor-specific job archive or start paid fabrication by implication.

## Español

Comprueba malla, escala y requisitos del proceso. Genera G-code con un laminador real y el perfil correcto; revisa trayectorias, temperaturas y comandos. Distingue preparación, subida, aceptación e impresión. No inventes perfiles ni conexiones a máquinas.
