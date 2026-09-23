# YouTube summaries from captions

[Español](../es/youtube-summary.md)

Fénix's original `youtube-summary` skill accepts a video link, obtains an available caption track, and directs the agent to summarize the evidence in the user's language with timestamp links. Example: “Summarize this YouTube video in Spanish and highlight its main conclusions: <URL>”. The helper retrieves evidence; the existing agent model writes the summary.

## Design and source review

We inspected [pytube](https://github.com/pytube/pytube), including its README and caption implementation, at commit `a32fff39058a6f7e5e59ecd06a7467b71197ce35`. It supports caption formats as part of a broader video downloader. We chose the narrower [youtube-transcript-api](https://github.com/jdepoix/youtube-transcript-api) dependency, version 1.2.4, after reviewing its API, network flow, track selection, XML parser and error types at commit `8f150ba8836da30a36bcd40e8fca226ed179ba72`. This was a focused integration review, not a complete security audit of either repository.

The skill, helper, URL validation, bounded HTTP session, language policy, evidence format and tests are Fénix code. The MIT-licensed adapter remains an external dependency; its source was not copied into the skill. Existing audio transcription and video editing skills do not provide this link-to-caption workflow, so a separate discoverable skill avoids expanding their scope.

## Behavior and limits

- Accepts watch, short, Shorts, live and embed video links. A playlist parameter does not expand the task into multiple videos.
- Defaults to Spanish, then English source captions; prefers manual captions within the preferred language. Automatic captions and fallback to another language are explicitly labeled. `--languages` and `--strict-language` control selection.
- Preserves timestamps, repeated speech and overlapping segments. Records retrieval time, adapter version and a content hash. The agent reads the complete fetched track, or declares partial coverage.
- Treats caption content as source data, not executable instructions. Allows only the expected HTTPS YouTube endpoints for the same video, refuses redirects, and limits request count, time and size. It does not import personal cookies, environment proxies or `.netrc` credentials.
- Uses no GPU, video download, transcription service or separate subscription. Ordinary model inference and infrastructure costs still apply.

Captions can contain mistakes or omit speech. Visual demonstrations are not inspected. YouTube can block hosting-provider IPs, require login, provide no subtitles or change its endpoints. Failed extraction must not become an invented summary based on the title. The skill can use an already authorized visible transcript or user-supplied SRT/VTT/text. It does not bypass restrictions or silently download audio.

## Packaging and operation

The skill and its resources are bundled through the existing Fénix skill installer. Its helper runs inside the user's configured execution environment through `command_run`, using the resource path returned by `skill_load`. Instructions and the JSON/error contract are in the [skill](../../resources/skills/youtube-summary/SKILL.md) and [caption protocol](../../resources/skills/youtube-summary/references/caption-protocol.md).

`Dockerfile.workspace` includes the pinned Python environment. To add it to an existing tenant/workspace image without rebuilding the entire office toolchain, run from the repository root:

```bash
docker build -f packaging/linux/Dockerfile.youtube \
  --build-arg FENIX_TENANT_IMAGE=fenix-tenant:local \
  -t fenix-youtube:local .
docker image inspect --format '{{.Id}}' fenix-youtube:local
```

Use your actual base tag, or a named registry digest such as `registry/image@sha256:...`; a bare local `sha256:...` image ID is not a valid BuildKit `FROM` reference. Pin the resulting immutable image identity in the existing execution-service configuration. The base must provide `python3` and `python3-venv`. The dependency environment is `/opt/fenix-youtube`; all requirements are pinned. Do not install these dependencies on the controller as a fallback.

Normal application/image rollout is still required to activate this source change in a running deployment. This task does not change the production service or its runtime isolation policy.

## Validation

The offline suite exercises URL boundaries, language selection, real adapter parsing with synthetic responses, output provenance, malformed inputs, HTTP budgets and safe error handling. CI installs the pinned dependencies and runs `python -m unittest discover -s tests/youtube -v`. Native skill tests check English/Spanish discovery, neighboring skill separation, resource bundling and disabled-state preservation.

Live extraction succeeded for two public videos on 2026-09-21: six English segments and 316 Spanish segments. These observations apply to this network at that time, not every video or future hosting IP. Captions themselves are not committed. Exact executed checks and source hashes are recorded in the [validation record](../research/youtube-summary-validation-2026-09-21.json). This does not measure end-to-end model summary quality or certify production gVisor behavior.

Validated: **16 Python tests**, repeated successfully in the restricted container, and **33 native skill tests** passed. New discovery coverage includes eight YouTube requests and four neighboring tasks.
