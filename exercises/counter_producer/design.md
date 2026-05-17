# Design — `counter_producer`

Second multi-file exercise. Three modules:

- `counter` — same bounded `Counter` as `counter_multifile`. Closed
  spec fns `value()`, `bound()`, `invariant()`. Exec methods `new`,
  `incr`, `get`.
- `producer` — a thin module that bulk-increments a counter by `n`.
  Function `produce(c: &mut Counter, n: u32)`. Precondition: the
  counter's value plus `n` does not exceed its bound. Postcondition:
  the counter's value increased by exactly `n` and the bound is
  unchanged.
- `main` — entry. Declares the two modules. Defines a small
  end-to-end function `pipeline(target)` that creates a fresh counter
  of bound `target`, calls `produce(target)`, and returns the final
  reading.

The new regime this exercise stresses, beyond `counter_multifile`:
the *producer* maintains a loop invariant that the *counter* cannot
expose. Specifically, `producer::produce`'s body is a loop that calls
`c.incr()` `n` times. Each call increments `c.value()` by 1
(counter's postcondition). The producer must compose that single-step
fact over `n` iterations into the multi-step claim
`c.value() == old(c).value() + n` — a fact module `counter` does not
state and cannot derive on its own.

If the agent handles this cleanly in one attempt, the data point is
"playbook generalises to genuine cross-module composition." If they
get stuck threading the bound check through the loop, the
sub-iteration data shows exactly where multi-module reasoning needs
new pattern entries.

---

## 1. Layout

```text
exercises/counter_producer/
    main.rs        # entry: mod counter; mod producer; pipeline(target)
    counter.rs     # Counter struct + closed spec fns + exec methods
    producer.rs    # produce(c: &mut Counter, n: u32)
    design.md      # this file
```

The witness mirrors:

```text
exercises/counter_producer_witness/
    main.rs
    counter.rs
    producer.rs
```

`producer.rs` imports the counter via `use crate::counter::Counter;`
(not `use super::counter::Counter;` — both modules are at crate root,
declared in `main.rs`). Verus on `main.rs` walks all three files.
Tested on `/tmp/producer_test/` — 6 verified, 0 errors.

---

## 2. Module contracts

### `counter::Counter`

Byte-identical to `counter_multifile/counter.rs`. The closed spec fns
are visible to `producer` (they are `pub`); only their bodies are
hidden.

### `producer::produce`

```rust
pub fn produce(c: &mut Counter, n: u32)
    requires
        old(c).invariant(),
        old(c).value() + n <= old(c).bound(),
    ensures
        final(c).invariant(),
        final(c).value() == old(c).value() + n,
        final(c).bound() == old(c).bound(),
```

The precondition `old(c).value() + n <= old(c).bound()` rules out
overflow and ensures every `incr` call inside the loop satisfies
*incr's* precondition `value < bound`. This is the load-bearing
arithmetic the agent must thread through the loop invariant.

### `main::pipeline`

```rust
pub fn pipeline(target: u32) -> (r: u32)
    ensures r == target,
```

Body builds a fresh counter, calls produce with `n = target`, returns
`c.get()`. Three function calls. Verification rests entirely on the
postconditions of `Counter::new`, `producer::produce`, and
`Counter::get`.

---

## 3. Sub-tasks

1. **Counter file.** Fill `exercises/counter_producer/counter.rs` with
   the three exec method bodies. Direct port from
   `counter_multifile/counter.rs`. No new patterns.
2. **Producer body.** Fill `producer::produce` with a loop that calls
   `c.incr()` `n` times. The loop invariant must include:
   - `c.invariant()` — so the next `incr` precondition is satisfied
   - `c.value() == start + i` — the composed step fact, where `start
     == old(c).value()` is captured before the loop
   - `c.bound() == old(c).bound()` — frame fact, threaded by `incr`'s
     postcondition
   - `i <= n` — loop counter bound
   - `start + n <= c.bound()` — the function's precondition surviving
     into the loop body. This is needed because `incr` requires
     `value() < bound()`, and at iteration `i` the value is
     `start + i`. We have `start + i < start + n <= c.bound()`, so
     `incr`'s precondition holds.
   - `start == old(c).value()` — anchors the composed claim
3. **Pipeline body.** Fill `main::pipeline` with the three-call
   sequence. No loop, no invariant. The hardest part is making sure
   the precondition for `produce` (`old(c).value() + target <=
   old(c).bound()`) is discharged after `Counter::new(target)`. From
   `new`'s ensures: `c.value() == 0` and `c.bound() == target`, so
   `0 + target <= target` holds trivially.
4. **End-to-end verify.** `verus exercises/counter_producer/main.rs
   --crate-type=lib` exits 0. Witness gives 6 verified, 0 errors.

---

## 4. Patterns from the playbook that should apply

- **Cross-module `closed spec fn` + postcondition bridge** (from
  `cross_module_counter`). Same shape, but now applied across three
  modules. The producer's loop invariant uses
  `c.value()`, `c.bound()`, `c.invariant()` — all closed spec fn
  calls — and Verus re-establishes each conjunct after `c.incr()`
  from incr's ensures alone.
- **`final(self)` syntax** for `&mut self` postconditions (bounded_log,
  cross_module_counter).
- **Loop invariant captures a "start" snapshot.** Capture
  `let start = c.get();` before the loop, then the invariant says
  `c.value() == start + i` and `start == old(c).value()`. The
  `start` ghost variable is real exec (we use `get`, which is fine
  for u32), letting the invariant compose without needing `ghost`
  syntax.

---

## 5. Anti-patterns

- **Do not change `closed` to `open` on the counter's spec fns** to
  let the producer see internal fields. The producer must work
  through the closed spec fn API.
- **Do not omit `start + n <= c.bound()` from the loop invariant.**
  Without it, Verus cannot discharge `incr`'s precondition on
  iteration `i`.
- **Do not weaken `produce`'s precondition** to make verification
  easier. The precondition `old(c).value() + n <= old(c).bound()` is
  what makes the loop bound-safe.
