# Design — `cross_module_counter`

First multi-module exercise in the harness. A `counter` module exports a
bounded counter abstraction with three exec methods (`new`, `incr`,
`get`) whose specs are stated in terms of *closed* spec functions —
their bodies are not visible outside the module. A `client` module
imports the counter and implements `count_up_to(target)`, a function
that creates a fresh counter of bound `target` and increments it to
`target`, returning the final reading.

The cross-module obligation: `count_up_to`'s correctness proof depends
on `counter`'s API specs alone, not on the counter's internal
representation. The implementer must respect that boundary — no peeking
into `counter`'s private fields, no `open spec fn` for what should be
`closed`.

---

## 1. Module layout

Single file `exercises/cross_module_counter.rs` with two nested `mod`
declarations inside one `verus! { }` block. This is the smallest
possible multi-module test: it stresses module-level visibility,
`closed spec fn` opacity, and cross-module imports, without introducing
new tooling (verus's single-file invocation handles nested mods cleanly,
verified on /tmp/verus_multimod_test.rs).

```text
verus! {
    mod counter {
        pub struct Counter { value: u32, bound: u32 }    // private fields
        impl Counter {
            pub closed spec fn value(&self) -> u32 { ... }
            pub closed spec fn bound(&self) -> u32 { ... }
            pub closed spec fn invariant(&self) -> bool { ... }
            pub fn new(bound: u32) -> Counter { ... }
            pub fn incr(&mut self) { ... }
            pub fn get(&self) -> u32 { ... }
        }
    }
    mod client {
        use super::counter::Counter;
        pub fn count_up_to(target: u32) -> u32 { ... }
    }
}
```

The `closed` keyword on the three spec fns is load-bearing. From inside
`counter`, the body is visible; from `client`, only the signature is
visible. The function postconditions in `counter` (e.g. `c.value() == 0`
in `new`) bridge from "what the body says" to "what the client can
prove" — without the postconditions, `client` would have no facts to
reason from.

---

## 2. Spec contracts

### counter::Counter

- `value(&self) -> u32` — closed spec fn returning the abstract value.
- `bound(&self) -> u32` — closed spec fn returning the bound.
- `invariant(&self) -> bool` — closed spec fn defined as `value <= bound`.

### counter::Counter::new

```rust
pub fn new(bound: u32) -> (c: Counter)
    ensures
        c.invariant(),
        c.value() == 0,
        c.bound() == bound,
```

### counter::Counter::incr

```rust
pub fn incr(&mut self)
    requires
        old(self).invariant(),
        old(self).value() < old(self).bound(),
    ensures
        final(self).invariant(),
        final(self).value() == old(self).value() + 1,
        final(self).bound() == old(self).bound(),
```

### counter::Counter::get

```rust
pub fn get(&self) -> (v: u32)
    requires self.invariant(),
    ensures v == self.value(),
```

### client::count_up_to

```rust
pub fn count_up_to(target: u32) -> (final_count: u32)
    ensures final_count == target,
```

---

## 3. Algorithmic sketch

### counter (the easy module)

Each method's body is one line. The verifier should accept all three
with no proof hints once the field-to-spec-fn lifting is correct.

- `new`: construct `Counter { value: 0, bound: bound }`.
- `incr`: `self.value = self.value + 1`. The precondition `value <
  bound` rules out overflow.
- `get`: return `self.value`.

The three `closed spec fn`s are bodies the implementer writes; e.g.
`value(&self) -> u32 { self.value }`. From within the module, the body
is transparent, so the implementer can prove `c.value() == 0` after
`Counter { value: 0, .. }` by unfolding the spec fn definition.

### client (the multi-module part)

```text
let mut c = Counter::new(target);
let mut i: u32 = 0;
while i < target
    invariant
        c.invariant(),
        c.value() == i,
        c.bound() == target,
        i <= target,
    decreases target - i,
{
    c.incr();
    i = i + 1;
}
c.get()
```

The loop invariant is what makes the cross-module reasoning visible:
all four invariant conjuncts are in `client`'s vocabulary
(`c.value()`, `c.bound()`, `c.invariant()`) which are spec fn calls
that crossing into `counter`'s API. `client` cannot see the underlying
fields; it must rely on the postconditions of `new` and `incr` to
re-establish the invariant after each loop iteration.

After the loop, `i == target` and `c.value() == i`, so
`c.value() == target`. `c.get()`'s ensures gives the result equals
`c.value()`, closing the postcondition.

---

## 4. Sub-tasks

1. **Counter module skeleton.** Define `pub struct Counter { value: u32, bound: u32 }` and the three `closed spec fn`s with the bodies above. Verify the module compiles. No exec methods yet.
2. **Counter::new.** Implement the body and verify its three ensures clauses fire.
3. **Counter::get.** Implement and verify. Single-line body; ensures should be immediate.
4. **Counter::incr.** Implement the body. The three ensures clauses need the unfolded `closed spec fn` bodies — Verus will do this automatically inside the module. Confirm with `verus`.
5. **Client::count_up_to.** Implement the loop with the four-conjunct invariant above. Run `verus`; the cross-module facts should follow from `incr`'s postcondition.
6. **End-to-end verify.** `verus exercises/cross_module_counter.rs --crate-type=lib` exits 0 with no warnings about missing invariants or unfired triggers.

---

## 5. Patterns from the playbook that should apply

- **`final(self)` syntax** for `&mut self` postconditions (bounded_log's pattern).
- **Defensive frame-property asserts** if Verus doesn't immediately
  thread `c.bound() == target` through the loop. Try a one-liner:
  `assert(c.bound() == old(c).bound());` after `c.incr()`.
- The architect playbook's `=~=` extensional equality and `choose`
  witnesses are unlikely to be needed — this exercise has no set
  reasoning.

---

## 6. Anti-patterns (what NOT to do)

- **Do not change `closed` to `open` on the three spec fns** to make
  cross-module reasoning easier. The whole point is that `client` works
  *without* seeing the bodies. If verus complains about something not
  being deducible in `client`, the fix is to strengthen the function's
  `ensures`, not to widen the spec-fn visibility.
- **Do not expose `Counter`'s fields as `pub`.** Same reason. The
  hook's cheat-token check does not flag this, but the reviewer will.
- **Do not call `Counter::new` with `target == u32::MAX`** anywhere in
  a way that could overflow `c.value() + 1` in `incr`. The invariant
  `value <= bound` plus the requires `value < bound` covers this; just
  don't break it.
