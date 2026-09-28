---
name: robot-description
description: Author or validate URDF robot structure, SRDF planning semantics and SDF simulation descriptions with explicit frames, units and inertials. Robots, articulaciones y Gazebo.
metadata:
  version: 1.0.0
  category: engineering
  tags: robot, urdf, srdf, sdf, kinematics
  aliases: URDF, SRDF, SDF, Gazebo, MoveIt, ROS, robot, articulación, cinemática, inercia
  required_tools: file_read,file_write,command_run
  requires: Robot dimensions and target ROS, MoveIt or simulator contracts; optional native validators
  effects: local-write
---
# Robot Description

## Scope

Describe robot structure and simulation/planning semantics. URDF owns links, joints, meshes, inertia and physical limits. SRDF references that structure for planning groups, poses and collision exclusions. SDF describes simulator models, worlds, sensors, frames and plugins. SDF here is SDFormat, not a signed-distance field.

## Inputs

Target consumer/version, link/joint table, frames, coordinate conventions, mesh units and available mass properties. Keep visual and collision geometry distinct; document approximations.

## Workflow

1. Extract actual names and topology before editing. Derive transforms and axes from measurements or source CAD, using SI units and radians unless the consumer explicitly says otherwise. Mesh millimeters need an explicit conversion to meters.
2. Check that URDF forms a connected rooted tree, joint references exist and limits are consistent. Derive inertials from known shape/material or measured properties; do not invent mass to silence a parser.
3. Validate SRDF names against the paired URDF. Derive planning groups and end effectors from topology. A collision exclusion needs a reason grounded in geometry or sampled analysis, not a broad disable-all shortcut.
4. For SDF, resolve pose and axis frames against the actual format version; verify includes and resources. Identify plugins before loading a simulator because they are executable code.
5. Run available native validators and a bounded simulator/planner smoke check when authorized and installed. Inspect joint motion, resource loading and collisions, and separate static validity from runtime behavior.

## References

Read [format and validation boundaries](references/robot-formats.md). `scripts/check_urdf.py` performs a limited offline URDF topology, limits and inertia check. It does not expand xacro, resolve ROS packages, validate SDF/SRDF or prove collision-free planning.

## Verification

Report the target version, checks run, unresolved resources and physical assumptions. A valid XML document is insufficient evidence of correct kinematics or dynamics.

## Limits

No hardware controller is invoked by these checks. Publishing to a live robot, disabling protections or executing motion requires its own task scope and operational checks.
