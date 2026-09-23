# Geometry checks / Comprobaciones geométricas

Use the installed CAD kernel's API and record its version. For build123d, create editable Python geometry, export through its STEP/STL exporters and reopen STEP using `import_step`. Kernel validity, solid count, dimensions and volume answer different questions. Compare against independently computed expectations such as a box minus cylindrical holes; do not derive the expected result by repeating the model builder.

```sh
python3 <skill-directory>/scripts/inspect_geometry.py part.step \
  --expect-size-mm 40 30 4 --expect-solids 1
python3 <skill-directory>/scripts/inspect_geometry.py part.stl --mesh-units mm
```

Dependencies are `build123d` for STEP and `trimesh`/`numpy` for meshes. Check their availability in the task environment; they are not bundled into Fénix. The inspector emits JSON with the file hash. Exit 0 means its limited checks passed, 2 means a geometric/expectation check failed, and 1 means input/dependency/reader failure. It does not modify the input. Imported STEP geometry is measured in the kernel's millimeter convention. For mesh formats, units must come from design context, never a guessed bounding-box size.

The sample `assets/mounting_plate.py` exports a 40 × 30 × 4 mm plate with two 5 mm through-holes. Its independent ideal volume is `40*30*4 - 2*pi*(2.5**2)*4`. Change parameters in a project copy, not the bundled asset. Mesh volume approximates the analytic solid according to tessellation tolerance.

For 3MF preserve package units/material metadata; for GLB distinguish visualization from manufacturing data. These formats are not supported by the bundled inspector. Use a real compatible reader when required. Assembly overlap, clearance, collision, self-intersection and strength need additional specific checks. No native interactive CAD viewer is installed by this skill.

ES: inspecciona el archivo exportado con su lector real, conserva unidades y calcula expectativas independientes. El ejemplo no sustituye el diseño pedido. Los formatos o verificaciones no soportados quedan explícitamente pendientes.
