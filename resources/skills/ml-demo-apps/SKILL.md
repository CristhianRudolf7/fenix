---
name: ml-demo-apps
description: Build and verify Gradio machine-learning demos and Hugging Face Spaces, including LoRA and ZeroGPU deployment constraints. Demos Gradio y Spaces.
metadata:
  version: 1.0.0
  category: data
  tags: gradio, spaces, demo, lora, zerogpu
  aliases: Gradio, Hugging Face Spaces, demo modelo, playground LoRA, ZeroGPU, interfaz Python
  required_tools: file_read,file_write,command_run,tool_search,web_read
  requires: Compatible Python/Gradio environment and model access; account and hardware for hosted deployment
  effects: mixed
---
# ML Demo Apps

## Scope

Build an interface that exercises a model and optionally deploy it to the requested Space. Use frontend-design-and-accessibility for a general web product, huggingface-hub for repository operations and ml-inference-deployment for a production inference service.

## Inputs

Model/base revision, adapter if any, task and input types, output artifact type, runtime, expected concurrency and local or hosted destination. A LoRA adapter must match its base architecture, pipeline and task.

## Workflow

1. Verify the inference recipe against the actual model card and installed APIs. Build a small callable that handles a representative input and returns a real artifact before wiring the UI.
2. Choose Gradio components and event signatures for the installed version. Separate request state from global model state; bound input size, queue/concurrency and cancellation. Keep per-user files isolated and avoid logging private inputs by default.
3. Produce the app, dependency manifest and README metadata. Read [Spaces and ZeroGPU](references/spaces-and-zerogpu.md) for hosted builds, GPU boundaries and LoRA handling. A normal CPU import is not proof that GPU inference works.
4. Validate locally where supported, then perform the authorized deployment using the requested SDK, hardware and visibility. Check build/runtime logs and an actual application request. Keep platform-specific validation explicit when it cannot be reproduced locally.
5. Inspect output content and file type, not just a success-shaped JSON response. Verify subsequent requests and concurrent users do not inherit another user's inputs, seed or adapter settings.

## Verification

Exercise a normal request, invalid input and cancellation or an error path. Confirm real output bytes and the consumer experience. Distinguish a runnable local app, a Space build and successful hosted inference.

## Limits

No GPU entitlement, HF account or model is installed by this skill. Creating a demo does not authorize public sharing, paid hardware, persistent telemetry or external alerts. Preserve explicit authorization already provided.
