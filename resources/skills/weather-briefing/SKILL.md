---
name: weather-briefing
description: Retrieve and explain current weather, hourly forecasts and official alerts for a resolved location and time window. Pronóstico meteorológico y condiciones locales.
metadata:
  version: 1.0.0
  category: personal
  tags: weather, forecast, rain, wind
  aliases: tiempo meteorológico, pronóstico, lluvia, viento, weather forecast, alertas meteorológicas
  required_tools: web_search,web_read,tool_search
  requires: Current accessible weather source and resolved location/timezone
  effects: read-only
---
# Weather Briefing

## Scope

Use for weather observations, forecasts and alerts. Travel planning may consume this result; a weather question does not need a full itinerary or recurring notification.

## Inputs

Place or coordinates, intended dates/activity, timezone and preferred units. Use relevant owner-provided location; do not assume a default city from someone else's examples.

## Workflow

1. Resolve ambiguous place names and the actual forecast timezone. Convert today/tomorrow/weekend to local calendar dates. Preserve the requested location rather than selecting the first geocoding match silently.
2. Read current authoritative or documented provider data through available tools. Fetch only the needed variables and horizon. Distinguish station observations, modeled current conditions and future forecasts; retain observation/issue/valid times and retrieval time.
3. Check unit metadata and align timestamps with values. Separate sustained wind from gusts, precipitation amount from probability and hourly periods from daily extrema. Null data is unknown, not zero.
4. Explain likely conditions for the relevant hours, including uncertainty and meaningful changes. A displayed zero or rounded probability is not a guarantee. Forecasts outside the provider's horizon remain unavailable rather than extrapolated weather facts.
5. For severe-weather questions, consult the relevant official alert source and check geographic area, validity and cancellations. Routine forecast output is not an official aviation/marine briefing or an all-clear when alerts could not be retrieved.

## Verification

State location, local dates, units, source and freshness. Check that recommendations follow the requested activity and the available evidence. Do not turn a download timestamp into the forecast's issue time.

## Limits

This skill adds no weather provider. Consult current [Open-Meteo field/timezone documentation](https://open-meteo.com/en/docs) when that source is used; do not hardcode historical limits, pricing or station coverage.
