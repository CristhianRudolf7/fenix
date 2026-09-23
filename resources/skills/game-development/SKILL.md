---
name: game-development
description: Build or repair playable games with explicit simulation timing, input, collision, pause/restart and target-platform checks. Desarrollo de videojuegos.
metadata:
  version: 1.0.0
  category: engineering
  tags: game, gameplay, physics, controls
  aliases: videojuego, Godot, Unity, game loop, colisiones, juego Canvas, playable game
  required_tools: file_read,file_write,command_run,tool_search
  requires: Requested engine/runtime, editable project and available execution target
  effects: local-write
---
# Game Development

## Scope

Use for interactive gameplay or game-specific engine work. Use simulation-modeling for a scientific model and web-animation for interface motion without gameplay.

## Inputs

Core mechanic, engine/platform, player controls, win/loss conditions, asset constraints and existing project behavior.

## Workflow

1. Reuse the requested or existing engine. Verify its installed version, export target and available testing path. Prototype the smallest playable loop with suitable placeholder assets when helpful; do not force an unrelated engine, personal asset server or production release.
2. Separate input collection, simulation state and rendering. Use the engine's physics tick or a bounded fixed-step accumulator where stable physics requires it. Cap catch-up work after a stall; do not advance movement by a fixed amount per rendered frame. Follow the [actual engine's timing contract](https://docs.godotengine.org/en/stable/tutorials/physics/physics_introduction.html) when using Godot.
3. Define coordinate spaces, units, collision layers and authority. Check fast-moving bodies, edge contacts and repeat triggers. For multiplayer, distinguish local prediction from server-authoritative outcomes; connecting two clients is not a synchronization test.
4. Make menu, playing, paused, won/lost and restart states explicit as needed. Reset timers, listeners, pooled objects and audio on restart/disposal. Handle lost focus, held input, resize and device changes without duplicate loops or stuck controls.
5. Test a complete play loop on the target: start, controls, progress, failure/success and restart. Check more than one frame rate or a simulated stall, touch/keyboard behavior and browser audio activation where relevant. Profile actual target performance before claiming a frame-rate budget.
6. Deliver source, run/export instructions and the verified build. Preserve asset provenance and distribution rights. A screenshot or compiling project does not establish playability.

## Verification

Record the engine/version, target and played/tested paths. Inspect gameplay outcomes and state reset, not only file existence or a blank successful launch.

## Limits

No game engine, asset collection or publishing account is installed. Deployment, store submission, monetization and live server administration require their own requested scope.

## Español

Respeta el motor elegido y verifica un ciclo jugable completo. Separa simulación de renderizado, comprueba colisiones y evita que pausa o reinicio acumulen eventos. Compilar o mostrar una captura no demuestra que el juego funcione.
