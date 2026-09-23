# Memory accounting / Cálculo de memoria

For an explicitly supplied parameter count, nominal weight bytes are `parameters × bits / 8`. This omits tensor headers, block scales, nonquantized layers and runtime buffers. Prefer actual artifact sizes when available, distinguishing the selected quantization from alternative files in the same repository.

For homogeneous dense/GQA/MQA attention, KV bytes are `2 × layers × kv_heads × head_dim × allocated_tokens × batch × bytes_per_cache_element`. Use KV heads, not necessarily query heads. Allocate for the actual context/concurrency target and cache dtype. MLA, recurrent/hybrid blocks, sliding windows, cache quantization metadata, offloading and tensor parallel placement need their specific implementation.

```sh
python3 <skill-directory>/scripts/estimate_memory.py \
  --parameters 8000000000 --weight-bits 4 \
  --layers 32 --kv-heads 8 --head-dim 128 --context 8192 --batch 2 --kv-bits 16 \
  --available-gib 16
```

The example uses hypothetical configuration values, not a named model recommendation. It yields approximately 3.73 GiB of nominal weights plus 2 GiB of KV cache, before unspecified overhead. Available GiB means memory available to this workload, not the GPU's marketed capacity. The helper never emits a verified-fit claim. An incomplete estimate may rule a configuration out, but cannot establish a successful load.

ES: diferencia GB decimales de GiB, pesos de caché y capacidad total de memoria realmente disponible. Introduce datos comprobados de la arquitectura. Las cifras son estimaciones; confirma la capacidad mediante una carga e inferencia representativas.
