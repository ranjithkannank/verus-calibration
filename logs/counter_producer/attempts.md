## Attempt 1 — 2026-05-17

**Sub-task:** All three sub-tasks 1+2+3 in one shot (counter.rs port,
producer.rs loop, main.rs pipeline) — design.md predicted the playbook
should generalise cleanly, so I executed the full port in one attempt.

**Approach:**
- `counter.rs`: direct byte-port of the three exec bodies from
  `counter_multifile/counter.rs` (`Counter { value: 0, bound }`,
  `self.value = self.value + 1`, `self.value`).
- `producer.rs`: `let start = c.get();` snapshot, then a `while i < n`
  loop calling `c.incr(); i = i + 1;`. Six-conjunct loop invariant:
  `c.invariant()`, `c.value() == start + i`, `c.bound() == old(c).bound()`,
  `i <= n`, `start == old(c).value()`, `start + n <= c.bound()`.
  `decreases n - i`.
- `main.rs`: three-call pipeline — `Counter::new(target)`, then
  `producer::produce(&mut c, target)`, then `c.get()`.

**Verifier output:** `verification results:: 6 verified, 0 errors`

**Next idea:** Done — hand to reviewer.
