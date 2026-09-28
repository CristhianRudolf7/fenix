---
name: smart-home-control
description: Inspect or control connected lights, thermostats and home scenes using exact device identities and observed state changes. Domótica y dispositivos del hogar.
metadata:
  version: 1.0.0
  category: personal
  tags: home, devices, scenes, thermostat
  aliases: Home Assistant, Homey, domótica, bombillas inteligentes, termostato, smart lights
  required_tools: tool_search,command_run,web_read
  requires: An existing authorized home controller connection and reachable devices
  effects: mixed
---
# Smart Home Control

## Scope

Use for a requested device state or scene. Configuring a recurring automation is a separate outcome. Printer jobs use 3d-printing-preparation; robot motion uses its actual controller workflow.

## Inputs

Home/controller, room, device identity, desired capability/value and whether the request includes a group or scene.

## Workflow

1. Discover the existing connection and inspect devices, rooms and supported capabilities. Resolve ambiguous names before mutation. Fuzzy search can suggest candidates; write using a verified unique ID, not the first substring match.
2. Read current state, availability, units and supported ranges. Brightness scales, temperature units and read-only sensor values differ by controller. Unknown/unavailable state is not off or zero.
3. Choose an explicit desired-state action when supported. Toggle, scene and script commands may not be idempotent. Inspect a scene's known scope before applying it; changing lights does not imply operating locks, cameras or heating in other rooms.
4. Call the real device service. In Home Assistant, writing `/api/states` changes the recorded representation and does not communicate with the device; use the supported service action for control. See the [official REST API](https://developers.home-assistant.io/docs/api/rest/).
5. Read back state or a relevant event with a bounded wait. Separate accepted, pending, unavailable and observed success. If the response is ambiguous, inspect current state before retrying, particularly for toggles or scenes.

## Verification

Report exact targets, requested values, observed results and partial failures. A controller acknowledgement or optimistic state is not independent physical confirmation.

## Limits

No credentials, controller or hardware driver is installed. Preserve established action permissions; a status query does not authorize a change. Do not weaken interlocks or expose device credentials in command arguments or logs.
