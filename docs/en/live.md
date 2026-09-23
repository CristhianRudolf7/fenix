# Browser voice and Live

The web application builds with Vite, React, TypeScript, Tailwind and pnpm. Axum embeds `apps/web/dist` at compile time; production does not run Node or a Vite server. Run `pnpm --dir apps/web install --frozen-lockfile` and `pnpm --dir apps/web build` before Cargo. For development, `pnpm --dir apps/web dev` proxies `/api` to the loopback Rust server at port 4173. The proxy is a local development endpoint, not a public deployment gateway.

React owns the application shell, chat layout and Live controls. Existing administration and browser interaction controllers have been ported to TypeScript and retained within isolated DOM boundaries, preserving their forms and behaviors. They are not a full rewrite into declarative React components; type tightening and further component extraction can happen independently of voice. The strict compiler enables an incremental exception for implicit parameter types in these existing controllers.

## Two entrances, one conversation

The microphone dictates into the draft; the owner reviews and sends it to the master normally. Live opens a separate conversational connection configured under the gear next to Live. Its default model is Nemotron on a RunInfra-compatible endpoint, with a separate key stored by the backend. The connection is verified before saving. It does not inherit or replace the master's provider or model.

The observer receives recent shared conversation, project/task bindings, subagent states and observable progress. It gets no execution tools, raw provider reasoning or file contents. A user turn can ask it to forward the user's **exact text** to the master's normal queue/control APIs or steer a named task. Ambiguous requests should be clarified. This intent decision still depends on the model; existing master approvals continue to apply. Automatic observation requests are denied if their model output attempts delegation or steering.

Durable activity events mark the observer dirty; updates are grouped rather than spoken per tool. A 20-second client interval and an 18-second server debounce avoid per-event inference. Unchanged snapshots produce no model request. Narration can remain silent when no useful change occurred. User messages take priority over pending automatic narration. Speech and replies also appear in the same visible chat; user conversation is available to subsequent master context. Status narration is not promoted to trusted task evidence.

## Voice lifecycle and limitations

Only browser Web Speech APIs are used for STT/TTS, with no audio API key. Availability varies by browser and operating system. Browser recognition may send audio to its vendor; it is not guaranteed offline. Unsupported recognition leaves text chat enabled and shows an explanation. Permission or recognition errors are visible, without an endless permission retry loop.

The call requests an echo-cancelled microphone stream. Recognition pauses during synthesized speech; an audio energy detector can interrupt synthesis and resume listening, and an explicit interrupt control is always available. This is a best-effort browser implementation: speaker feedback, recognition startup, missed initial syllables, device suspension and audio routing require testing on actual devices. Headphones can help. It is not guaranteed full-duplex telephony or a browser-independent low-latency voice service.

Hanging up aborts pending voice inference, stops recognition and synthesis, closes the audio context, releases microphone tracks and closes only the Live session. The master, plans, commands and subagents keep running. Navigation within the app keeps the call active. Closing the page ends voice; tasks stay on the server. A new call replaces an older local voice session, not the underlying work.

## Verification

Tests use local model mocks and browser speech/media doubles: no provider charges and no real microphone recordings. They check exact/idempotent delegation, shared history, cancellation isolation, dictation without auto-send, navigation and unsupported recognition. Human speech quality, real Nemotron latency, echo cancellation and device-specific recognition require real-browser/manual testing; mock tests do not certify them.

## Shared interlocutor for every channel

Web text chat, Telegram, and Live use the same conversational connection (Nemotron by default), configured under conversation/Live settings. The master connection remains independent: use Astra low with Flex for task execution. There is no silent fallback from the interlocutor to the master when conversational credentials are missing.

The interlocutor answers conversation and observed status, forwarding the user's exact instructions to the master for execution. Textual approval and task controls retain their deterministic handlers. Telegram replies use the originating account's delivery address; delegated tasks retain their channel and explicit task references. Closing Live does not cancel text conversation or background work. Master task results remain available in shared history.

OpenAI execution settings support `flex_processing: true`, which explicitly sends `service_tier: "flex"` on Responses requests, including subagents. Standard pricing is not used as an automatic fallback.
