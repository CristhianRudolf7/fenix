# Rust investigation cases

At FFI boundaries, inspect callback lifetime, allocator pairing, ownership transfer and unwind behavior. For raw buffers, reconcile length/capacity with initialized elements and pointer validity. Test feature flags and platform assumptions actually used by the target.

For async work, model claim → execute → persist → deliver. Locate crash or cancellation windows and examine whether a retry repeats an external effect. Check that actor/project context stays attached to jobs and streams after reconnect.

For filesystem access, verify descriptor lifetime and symlink behavior at use time. A canonical path checked before an attacker-controlled replacement may not protect a later open. Explain exploit prerequisites rather than inferring a reachable race from two calls alone.

For resource exhaustion, quantify attacker-controlled growth, retention and concurrency. Use bounded fixtures; ordinary load tests do not establish authorization to exhaust a live service.
