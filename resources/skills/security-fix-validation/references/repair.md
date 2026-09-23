# Repair evidence

For each finding, retain the base revision, candidate revision, original reproduction, observed before/after result, legitimate-path test and remaining gaps. Distinguish fixed, partially mitigated, not reproduced and not retested.

Check similar routes or entrypoints sharing the same failed invariant. Verify that the repair cannot be bypassed by an alternate identity source, batch operation, asynchronous continuation or cached result.

Do not suppress a scanner rule solely to obtain a clean report. If a finding was false, document its refutation. Do not broaden a patch into unrelated refactoring or remove features to make tests pass.

Publishing a repaired skill remains a server-controlled workflow; this worker can provide evidence but cannot grant its own approval.

Ensure a reproduction reaches the relevant operation before asserting the outcome on both revisions. Import errors, missing fixtures and setup failures cannot stand in for a demonstrated pre-fix defect. Use the same behavioral assertion for both revisions; do not let the assertion branch on revision labels.
