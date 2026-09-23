---
name: cache-design
description: Design or diagnose caches with explicit keys, freshness, invalidation and stampede behavior.
metadata:
  version: 1.1.0
  category: engineering
  tags: cache, design
  aliases: cache, redis, TTL, invalidación, datos obsoletos, content hash cache, hash de contenido, invalidación por versión
  required_tools: file_read,file_write,command_run
  requires: Existing storage/cache mechanism and workload evidence
  effects: mixed
---
# Cache Design

## Scope

Design or diagnose caches with explicit keys, freshness, invalidation and stampede behavior.

## Inputs

Cached entity, consistency requirements, tenant scope and failure behavior. Prerequisite: Existing storage/cache mechanism and workload evidence.

## Workflow

1. Identify the source of truth and what staleness is acceptable. Include tenant, authorization-sensitive dimensions and version in keys when needed.
2. Choose invalidation and expiry from the data lifecycle; TTL alone may not handle a permission revocation.
3. Handle concurrent misses, negative results and dependency failure without unbounded stampedes or caching errors as valid data.
4. Measure hit ratio together with latency, memory and origin load. Test cold-cache and invalidation behavior, not just warmed requests.

For cached file or model processing, include content identity, parser/model revision, preprocessing/options and authorized scope in the key. A filename or content hash alone does not capture changed processing semantics or access rights. Publish cache entries only after validated complete processing; interrupted writes and error responses must not become reusable success entries. Test same bytes with changed options/version and same path with changed bytes.

## Verification

Check cross-user isolation, stale updates, simultaneous misses and recovery after cache loss.

## Limits

Do not add a cache before locating the bottleneck or treat cached authorization as permanently valid.

## Español

Define vigencia y claves por ámbito; prueba invalidación, fallos y estampidas de misses.

En procesamiento de archivos, la clave incluye contenido, versión del procesador, opciones y ámbito. Publica solo resultados completos y prueba cambios de contenido y de configuración.
