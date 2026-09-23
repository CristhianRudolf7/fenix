---
name: travel-planning
description: Build a feasible travel itinerary from current options, user constraints and explicit booking status.
metadata:
  version: 1.0.0
  category: personal
  tags: travel, planning
  aliases: viaje, itinerario, hotel, vuelos, transporte
  required_tools: file_read,file_write,tool_search
  requires: Current travel sources or supplied bookings; web access when researching
  effects: local-write
---
# Travel Planning

## Scope

Build a feasible travel itinerary from current options, user constraints and explicit booking status.

## Inputs

Dates, origin/destination, travelers, budget, preferences and existing reservations. Prerequisite: Current travel sources or supplied bookings; web access when researching.

## Workflow

1. Resolve dates and local timezones, including overnight travel. Check official entry/transport conditions when relevant and current.
2. Compare realistic transport and lodging options with total price, luggage, cancellation and transfer time, recording source/time.
3. Build an itinerary with opening hours, travel buffers and alternatives for uncertainty; preserve accessibility and user preferences.
4. Distinguish proposed options from confirmed bookings and list deadlines or missing information.

## Verification

Check chronological feasibility, location distances and current availability before presenting a bookable option.

## Limits

Do not book, pay or submit traveler documents without authorization, or guarantee visa eligibility from incomplete facts.

## Español

Planifica con horarios y costes vigentes, márgenes reales y reservas claramente diferenciadas de propuestas.
