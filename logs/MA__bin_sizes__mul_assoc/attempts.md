## Attempt 1 — 2026-05-18

**Sub-task:** Design sub-task 1 — "Try the empty-body or one-line proof."
First confirmed the empty body fails (postcondition not satisfied),
then applied the one-line `assert(...) by (nonlinear_arith)` proof
recommended by the design's step 2 / AGENTS.md ground-truth note.

**Approach:** Inserted a single
`assert((x * y) * z == y * (x * z)) by (nonlinear_arith);`
inside the proof fn body.

**Verifier output:** `verification results:: 2 verified, 0 errors` —
verus exited 0.

**Next idea:** Done. Hand to reviewer.
