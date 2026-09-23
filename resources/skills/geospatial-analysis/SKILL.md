---
name: geospatial-analysis
description: Analyze geographic data with correct coordinate systems, spatial joins and location privacy.
metadata:
  version: 1.0.0
  category: data
  tags: geospatial, analysis
  aliases: geoespacial, mapas, coordenadas, GIS, geopandas
  required_tools: file_read,file_write,command_run
  requires: Spatial data and available GIS tooling
  effects: local-write
---
# Geospatial Analysis

## Scope

Analyze geographic data with correct coordinate systems, spatial joins and location privacy.

## Inputs

Geometry types, CRS, units, target operation and location sensitivity. Prerequisite: Spatial data and available GIS tooling.

## Workflow

1. Inspect coordinate reference systems and geometry validity. Do not compute distance/area in angular coordinates as if they were meters.
2. Reproject deliberately and document transformations. Check axis order, antimeridian behavior and boundary predicates.
3. Validate spatial join multiplicity and unmatched features; overlapping polygons may produce multiple matches.
4. Aggregate or generalize sensitive locations appropriately for the requested output and preserve source attribution.

## Verification

Check known control points, units, extent and counts before/after joins.

## Limits

Do not geocode private addresses through an external service without authorization or infer identity from location traces.

## Español

Comprueba CRS, unidades y multiplicidad de cruces; protege ubicaciones precisas y valida puntos conocidos.
