# Recovery evidence

Define snapshot ID/time, included paths, exclusions, database consistency point and intended RPO/RTO before verifying a restore. Compare restored files to a manifest from that same snapshot, not to a changing live directory. Use relative paths rooted at each dataset, record unreadable/missing entries explicitly and keep manifests outside the trees being compared. Matching error strings are not matching hashes. Preserve metadata/ACLs and links in a separate check where they affect restoration.

For an offline comparison of SHA-256 file manifests, use [compare_manifests.py](../scripts/compare_manifests.py):

```bash
python3 scripts/compare_manifests.py baseline.json restored.json
```

Both inputs use this Fénix-specific format (the digest below is the actual SHA-256 of an empty file):

```json
{
  "schema_version": 1,
  "snapshot_id": "snapshot-2026-09-20-001",
  "algorithm": "sha256",
  "complete": true,
  "errors": [],
  "files": {
    "empty.txt": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
  }
}
```

Generate these records from trusted acquisition of regular files in the declared scope. The helper reads the two JSON files only; it does not traverse paths, generate hashes, restore data or verify the truth of the manifests. Use the same snapshot ID for its baseline and restored copy. Empty inventories require explicit investigation and are inconclusive. Inputs are bounded to 8 MiB each; duplicate JSON keys, invalid digests and ambiguous paths are errors. Extra restored files fail this exact-scope comparison. Set `complete: false` and preserve `errors` whenever collection skipped or could not read data. Exit 0 means complete matching manifests, 1 means a demonstrated mismatch, and 2 means invalid or inconclusive evidence. Completeness problems remain in the output even when a mismatch is known.

After comparing bytes, restore the application/database in a separate environment and run a representative read/write operation within scope. A listing of database objects, successful decompression or a scanner with no detections is not recovery evidence. Encrypted/compressed legitimate files have high entropy; extension/entropy heuristics cannot certify the absence of ransomware. Choose a recovery point using the incident timeline and preserve uncertainty about when compromise began.

For immutable storage, verify the actual provider/tool combination supports required locks, retention, versioning and backup maintenance. Test permitted deletion resistance on disposable objects; do not discover an irreversible retention policy on the only production backup. Separate backup writer, restore and retention-administration identities and retain an independently recoverable key path. Document how prune/lock operations behave under retention and whether recovery works when the primary identity is unavailable.

## Español

Compara contra el mismo snapshot con rutas relativas y errores explícitos. El auxiliar compara manifiestos, no certifica archivos ni ausencia de malware; entradas incompletas nunca pasan. Verifica además la aplicación restaurada, permisos, claves y tiempos. Comprueba retención y mantenimiento con objetos desechables antes de imponer bloqueos irreversibles.
