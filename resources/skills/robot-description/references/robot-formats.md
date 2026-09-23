# Robot format boundaries / Límites de formatos robóticos

Run `python3 <skill-directory>/scripts/check_urdf.py robot.urdf` for an offline structural check. The helper rejects duplicate/unknown links, disconnected trees, joint/mimic cycles, reversed limits, nonfinite numeric values and nonpositive-definite supplied inertias. It reports missing inertials and unverified mesh references. It does not modify files, resolve network URIs or execute plugins.

Expand xacro using the project's actual ROS toolchain before this check. Use native `check_urdf` and simulator/MoveIt validators when available; distinguish their coverage from this helper. For SRDF, compare robot/link/joint names, group chains and end-effectors against the exact URDF. Preserve reasons for disabled collisions and check representative configurations with the actual planner. For SDF, use the target libsdformat/Gazebo version, resolve `relative_to`/`expressed_in`, inspect resource/plugin paths, and run its native validation before simulation.

A visual mesh may be too detailed for collision or have the wrong scale. A positive inertia tensor can still be physically wrong; verify its frame, mass, center of mass and principal-moment constraints against the source body. A successful XML parse or static report cannot establish those facts.

Sources: [URDF](https://docs.ros.org/en/rolling/Tutorials/Intermediate/URDF/URDF-Main.html), [SDFormat specification](http://sdformat.org/spec), [MoveIt URDF/SRDF](https://moveit.picknik.ai/main/doc/examples/urdf_srdf/urdf_srdf_tutorial.html).

ES: distingue topología, semántica de planificación y simulación. El helper cubre invariantes concretos de URDF; no reemplaza validadores nativos ni comprueba recursos, colisiones o propiedades físicas reales.
