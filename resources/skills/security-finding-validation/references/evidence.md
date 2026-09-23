# Evidence criteria

Injection requires attacker influence over executable syntax and a reachable interpreter, not merely string concatenation. Authorization defects require an actual protected action/data exposure with a principal lacking that permission. A UUID alone neither proves nor disproves authorization.

Concurrency claims need a feasible ordering, shared state and an observable policy violation. A safe Rust program may still contain a logical race. A panic needs reachability and service impact; process-local or request-local failure matters.

A defensive test should demonstrate intended denial and continued legitimate access. Retain contradictory evidence. When reviewers disagree, compare preconditions and reproduce the disputed path; never resolve a technical disagreement by majority vote alone. Ask the master for independent review if the uncertainty materially changes a decision.
