## Attempt 1 — 2026-05-15T00:00:00Z
**Approach:** Implemented all four exec bodies following the design doc: `new` returns `Log { cap: capacity, msgs: Vec::new() }`, `len` returns `self.msgs.len()`, `get` returns `Some(self.msgs[index])` or `None` based on bounds check, and `append` uses a capacity check + `Vec::push` + two frame-property asserts to help the SMT solver.
**Verifier output:** verification results:: 4 verified, 0 errors
**Next idea:** Success — no further attempts needed.
