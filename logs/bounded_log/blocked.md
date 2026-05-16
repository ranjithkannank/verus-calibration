# Blocked: bounded_log — irreconcilable spec-freeze / Verus-version conflict

## Summary

Attempt 2 tried to restore the original frozen spec text (bare `self` in the
`ensures` clause of `append`) and confirmed that **Verus 0.2026.05.13 refuses
to compile it**. The compiler error is:

```
error: to dereference a mutable reference parameter in a postcondition,
       disambiguate by wrapping it in either `old` or `final`
  --> exercises/bounded_log.rs:67:13
   |
67 |             self.well_formed(),
   |             ^^^^
   |
   = help: For information on the new mutable reference support, see:
           https://github.com/verus-lang/verus/blob/main/source/docs/migration-mut-ref.md
```

The relevant migration guide explains that the current Verus version
**requires** every bare `self` in a `&mut self` postcondition to be wrapped in
either `old(self)` (pre-state) or `final(self)` (post-state). There is no
flag or workaround to suppress this.

## Conflict

| Constraint | What it requires |
|---|---|
| Frozen spec (reviewer rule 1) | `self.well_formed()`, `self.capacity()`, `self.view()` (bare `self`) |
| Verus 0.2026.05.13 syntax | `final(self).well_formed()`, `final(self).capacity()`, `final(self).view()` |

These two constraints are mutually exclusive. No implementer-level change can
satisfy both simultaneously.

## Attempts

1. **Attempt 1** — Used `final(self)` everywhere in `ensures`. Verifies (exit 0),
   but REJECTED by reviewer because it differs from frozen spec byte-for-byte.
2. **Attempt 2** — Restored bare `self` to match frozen spec exactly. Rejected by
   Verus 0.2026.05.13 with the syntax error above (exit 1). Verification fails.

## Resolution required

Only the **architect** (per AGENTS.md) is empowered to re-freeze the spec.
The correct fix is:

1. The architect updates the frozen spec to use `final(self)` (semantically
   identical post-state reference, required by current Verus).
2. The `spec-frozen-bounded_log` tag is force-moved to the new commit.
3. The reviewer re-audits against the new frozen baseline.

The implementation body (two `assert` nudges for the frame property) is
correct and verifies cleanly with `final(self)`. Only the spec text needs the
architect's re-freeze decision.

## Current file state

`exercises/bounded_log.rs` is left with `final(self)` in the `ensures` clause
(the version that verifies with Verus 0.2026.05.13). Raw verifier output for
attempt 2 (bare `self`) is in `logs/bounded_log/raw/attempt-2.txt`.
