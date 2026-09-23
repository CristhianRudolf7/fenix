# Print preparation and handoff / Preparación y entrega de impresión

Record input hash, declared units, machine/profile identity, material, layer/nozzle settings, slicer version and generated artifact. Check the native profile instead of copying approximate bed/temperature values from an example. Discover the installed CLI with its help; backend flags and project/archive formats are not interchangeable.

Closure, consistent winding and positive mesh volume are useful preliminary checks, but do not establish wall thickness, trapped material, support requirements or strength. Thickness sampling can miss local weak features. Keep unknown measurements visible and use the intended manufacturing process's current guidance. Do not silently scale a mesh because its dimensions seem unusual.

Preview start/end code and the complete toolpath with the slicer. Static G-code parsing requires modal state, coordinate/extrusion modes, offsets, arcs, homing, machine-specific macros and temperature interpretation. A checker that only recognizes G0/G1 cannot certify arbitrary output. When validation cannot interpret a command, inspect its machine-specific definition or keep the handoff unresolved.

There is no bundled Bambu or other live-printer driver. If the task explicitly includes printing and a supported adapter exists, use its exact current protocol and credentials within the existing authorization. Verify the selected machine, profile, job and status; reconcile an ambiguous start response before retrying to avoid duplicate physical actions. Preparing, slicing or uploading alone does not authorize starting the machine.

ES: conserva hashes y perfiles reales, revisa trayectorias y distingue estados de preparación e impresión. Los comandos modales o específicos de fabricante requieren su intérprete. Una comprobación parcial no demuestra seguridad ni éxito físico.
