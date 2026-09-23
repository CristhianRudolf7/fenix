# Boundary cases

For authorization changes, inspect both entry-point checks and storage predicates. Trace actor identity through jobs and callbacks. Compare owner, tenant and sharing semantics independently.

For queued effects, check revocation after enqueue, replay/idempotency keys and partial success before an error response. Cancellation is not necessarily rollback.

For path or parser changes, examine normalization order, links, encoding and the actual sink. For cache changes, identify whether keys include every identity/permission dimension needed by the cached value.

Report the smallest counterexample. If a defense defeats the candidate, record that evidence rather than retaining a finding just because the diff looked risky.
