# Hub data and papers / Datos y papers del Hub

For Dataset Viewer reads, use the current API at `https://datasets-server.huggingface.co`: resolve `/splits`, inspect a bounded `/first-rows` or `/rows` page, and use `/parquet`, `/size` or `/statistics` only as needed. Encode query parameters, honor page limits and preserve config/split identity. A server error, gated dataset or partial response is not an empty dataset. Inspect downloaded schemas and actual row counts before calling an extraction complete. [Rows API](https://huggingface.co/docs/dataset-viewer/en/rows).

For repository files, resolve an immutable commit and select exact artifacts before downloading. Treat model cards and dataset cards as source statements, not independent validation. Keep source rights, sensitive columns and split integrity with ml-dataset-preparation. When writing, preserve existing metadata and review the exact outgoing files; do not recursively upload a home directory or agent session store.

For papers, identify the actual arXiv ID/version, DOI or Hub paper page and read the paper. A Hub page can link models/datasets/Spaces but does not establish experimental reproduction. Indexing a paper, claiming authorship, commenting and changing a repository card are separate mutations with different authority requirements. Use the installed CLI/API's current help instead of assuming the upstream publisher script is present.

ES: resuelve repositorio, versión, config y split; conserva límites y errores de paginación. Verifica la procedencia de papers y datasets. Publicar, atribuir autoría o subir historiales requiere el alcance correspondiente, no se deriva de una consulta de lectura.
