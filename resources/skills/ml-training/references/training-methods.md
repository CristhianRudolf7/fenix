# Training methods / Métodos de entrenamiento

Choose the objective from the requested outcome and available labels. For language models, SFT uses supervised examples; preference objectives such as DPO require correctly paired chosen/rejected responses; reward-based optimization needs a reward whose behavior is independently tested. Check installed TRL/Unsloth APIs and sequence-length settings rather than copying an old configuration. For vision-language models, truncation can remove image tokens. Keep adapters distinct from merged/full weights.

For vision, document image decoding, coordinate convention and transforms. Detection boxes may be xyxy, xywh or normalized; preserve image dimensions and transform labels together. Keep classification label maps stable and segmentation masks aligned. Evaluate with the metric appropriate to the task, such as classification accuracy, detection AP at declared thresholds or segmentation overlap, including class/sample coverage.

For embeddings and rerankers, distinguish bi-encoder dense embeddings, cross-encoder pair scores, sparse retrieval and late-interaction representations. Choose loss and negative mining consistent with labels and the deployment retriever. Prevent near-duplicate entities or mined evaluation answers leaking into training. Evaluate retrieval ranking with a held-out query/document set and compare against the original model.

Save code, environment, tokenizer/processor, split IDs, seeds and configuration with the artifact. Reload the saved model/adapter independently and test the actual consumer. Cloud execution and publishing use huggingface-hub's job lifecycle when requested; selecting a trainer does not force a paid remote job.

ES: diferencia objetivos, formatos de etiquetas y arquitecturas. Comprueba truncación, cajas/máscaras, negativos de recuperación y separación de evaluación. Conserva versiones y verifica el modelo guardado antes de atribuir una mejora al entrenamiento.
