# Stateful testing

Define actors, objects, permitted operations and an independent expected-policy function. Generate short sequences covering creation, sharing, revocation, enqueue, retry, cancellation and delivery. Compare persisted state and emitted effects, not only returned status codes.

Observe effects using recording substitutes for external services. Exercise the real application entrypoint and identity binding; calling a mocked policy helper alone cannot validate request routing.

Use controlled synchronization to reproduce races instead of hoping sleep durations trigger them. Keep fuzz corpus and minimized failures as task artifacts. Mutation testing may help determine whether an assertion detects a deliberately broken guard, but a surviving mutation requires interpretation and should not automatically become a vulnerability.
