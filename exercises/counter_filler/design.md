# Design — `counter_filler`

Third multi-file composition exercise, set up as the **second
deliberate discovery test** for the methodology. Same proof family
as `counter_producer` (cross-module composition over a closed-spec
counter API), but a structurally different loop shape so the
playbook entry for `counter_producer` does not transfer by literal
copy.

**The methodology question this exercise tests.** `sensor_poll_honest`
gave one data point that the methodology can recognise and reuse
a proof family from one exercise's playbook entry on a new
obligation in a different exercise — specifically the
inclusion-exclusion family from `ft_midpoint`. That was a yes on
one family. This exercise tests the same question on a different
family: the snapshot-and-preservation pattern that
`counter_producer` documents in its playbook entry. If the agent
constructs the right invariant for `fill_to` from the contract
alone — without the design note naming the invariant's shape — we
have a second data point on a different family. If not, the
sub-iteration history shows where the cross-family transfer
breaks down.

The design note below states the obligation and the informal why,
nothing more. The agent has the same playbook access it has had on
every prior exercise.

---

## 1. Layout

```text
exercises/counter_filler/
    main.rs        # entry: mod counter; mod filler; pipeline(target)
    counter.rs     # Counter struct + closed spec fns + exec methods
    filler.rs      # fill_to(c: &mut Counter, target: u32)
    design.md      # this file
```

The witness mirrors:

```text
exercises/counter_filler_witness/
    main.rs
    counter.rs
    filler.rs
```

`filler.rs` imports the counter via `use crate::counter::Counter;`
(both modules are at crate root, declared in `main.rs`). Verus on
`main.rs` walks all three files.

---

## 2. Module contracts

### `counter::Counter`

Byte-identical to `counter_producer/counter.rs`. The same closed
`value()` / `bound()` / `invariant()` spec fns, the same
`new(bound)` / `incr(&mut self)` / `get(&self)` exec methods with
the same requires/ensures. The implementer ports the three exec
bodies; nothing about the spec changes.

### `filler::fill_to`

```rust
pub fn fill_to(c: &mut Counter, target: u32)
    requires
        old(c).invariant(),
        old(c).value() <= target,
        target <= old(c).bound(),
    ensures
        final(c).invariant(),
        final(c).value() == target,
        final(c).bound() == old(c).bound(),
```

The function advances `c.value()` from wherever it starts up to
exactly `target`. Bound check rules out overflow and ensures every
`incr` call inside the body satisfies `incr`'s precondition.

What this is *not*: a copy of `counter_producer`'s `produce(c, n)`.
There is no separate `i: u32` loop counter; `c.value()` itself is
what advances. There is no `n` parameter; the stopping condition
is `c.value() == target`. The loop shape is target-bounded, not
counter-bounded.

### `main::pipeline`

```rust
pub fn pipeline(target: u32) -> (r: u32)
    ensures r == target,
```

Body builds a fresh counter, calls `fill_to(&mut c, target)`,
returns `c.get()`. Three function calls. Verification rests
entirely on `Counter::new`'s, `fill_to`'s, and `Counter::get`'s
postconditions.

---

## 3. Sub-tasks

1. **Counter file.** Fill `exercises/counter_filler/counter.rs`
   with the three exec method bodies. Same shape as
   `counter_multifile` / `counter_producer`. No new patterns.
2. **Filler body.** Implement `fill_to`. The body has to advance
   `c.value()` to `target` without violating `incr`'s precondition
   and without losing the bound's frame. The specific shape of
   loop, invariant, and decreases clause is the implementer's
   design choice — not pre-specified here.
3. **Pipeline body.** Fill `main::pipeline` with the three-call
   sequence. No loop, no invariant. Discharge `fill_to`'s
   precondition (`0 <= target && target <= target`) from `new`'s
   ensures.
4. **End-to-end verify.** `verus exercises/counter_filler/main.rs
   --crate-type=lib` exits 0.

---

## 4. Patterns from the playbook that may apply

The playbook (in `AGENTS.md`, accumulated across previous
exercises) contains patterns for cross-module composition over a
closed-spec counter API. Whether and how those patterns apply to
this exercise is left to the agent.

---

## 5. Anti-patterns

- **Do not change `closed` to `open`** on the counter's spec
  fns to let the filler see the internal `value` field.
- **Do not weaken `fill_to`'s precondition.** The
  `target <= old(c).bound()` clause is what keeps every internal
  `incr` call safe.
- **Do not copy `counter_producer`'s loop invariant verbatim.**
  This exercise has a different loop shape; an invariant designed
  for `produce(c, n)` will not discharge `fill_to`'s obligations.
  The shared part is the proof family, not the specific conjuncts.
