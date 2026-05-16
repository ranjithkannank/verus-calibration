# Design — `bounded_log`

## Representation

The struct is already fixed by the frozen spec:

```rust
pub struct Log {
    cap: usize,
    msgs: Vec<Message>,
}
```

We do **not** introduce ghost fields. The `view()` is `self.msgs@` and the
`capacity()` is `self.cap as nat`; both projections are total functions of
the existing concrete state, so no auxiliary state is needed.

Why `Vec` (not just `Seq`)? The exercise is exec code; `Seq` is ghost-only.
`Vec` gives us `push`, `len`, and indexing in exec mode, with vstd already
exposing the `Vec::view`-based specs we need.

## Algorithmic sketch

All four functions are loop-free. The capacity check in `append` is a
single comparison.

```text
new(capacity):
    return Log { cap: capacity, msgs: Vec::new() }

len(&self):
    return self.msgs.len()

get(&self, index):
    if index < self.msgs.len() { Some(self.msgs[index]) } else { None }

append(&mut self, msg):
    if self.msgs.len() == self.cap { return Err(()) }
    self.msgs.push(msg)
    return Ok(())
```

No loops anywhere. The interesting proof obligation is the frame property
on `append`'s success branch.

## Key invariants

### Struct-level (already encoded as `well_formed`)

| English | Verus |
|---------|-------|
| Length never exceeds capacity. | `self.msgs.len() <= self.cap` |

That is the entire well-formedness condition. There is nothing else to add
because `view()` is just `self.msgs@` and `capacity()` is just `self.cap as nat`.

### Local (per-function) facts the verifier needs

- `new`: `Vec::new()` returns a vector with `@.len() == 0`. vstd provides this.
- `len`: `Vec::len()` returns `self.msgs@.len() as usize`. vstd provides this; the cast `as nat` lines up trivially because we already know `msgs.len() <= cap <= usize::MAX`.
- `get`: `Vec::index` requires `index < self.msgs.len()`, which is exactly the guard. The result equals `self.msgs@[index as int]`.
- `append` success: `Vec::push` ensures `self.msgs@ == old(self).msgs@.push(msg)`. From this, `Seq::push` axioms give:
  - new length = old length + 1
  - last element = `msg`
  - all earlier indices unchanged

## Loop invariants

**None.** No function in this exercise has a loop. Skip this section.

## Predicted helper lemmas

None required. The vstd built-in specs for `Vec::new`, `Vec::len`,
`Vec::index`, and `Vec::push` cover everything. If the implementer feels
the urge to write a helper lemma here, they should resist and instead
sprinkle the asserts described below.

## SMT trouble spots

1. **Frame property in `append` (the canonical one).** After `self.msgs.push(msg)`,
   the postcondition

   ```rust
   forall|i: int| 0 <= i < old(self).view().len() ==>
       final(self).view()[i] == old(self).view()[i]
   ```

   does not always close on its own because the solver needs to chain
   `self.msgs@ == old(self).msgs@.push(msg)` with the `Seq::push` axiom that
   says `s.push(x)[i] == s[i]` for `i < s.len()`. The reliable nudge is a
   two-step assert immediately after the `push`:

   ```rust
   assert(self.msgs@ == old(self).msgs@.push(msg));
   assert(forall|i: int| 0 <= i < old(self).msgs@.len()
       ==> self.msgs@[i] == old(self).msgs@[i]);
   ```

   The first assert reminds the solver of the post-state of `push`; the second
   triggers the `Seq::push` index axiom on every `i` in range.

2. **`final(self)` syntax.** Verus 0.2026.05.13 (per the spec's header
   comment) requires `final(self)` rather than bare `self` in postconditions
   of `&mut self` functions. The spec already uses `final(self)` and
   `old(self)`; no implementer action needed in the *spec*, but be aware
   that any *asserts* you write inside `append` referencing the post-state
   should use plain `self` (since you are inside the body, post-state is
   the current state).

3. **`view()` vs `msgs@`.** `view()` is `closed`, so inside the body you can
   freely use `self.msgs@` interchangeably; outside (in callers' proofs)
   only `view()` is visible. For asserts inside the bodies of these four
   methods, use `self.msgs@` directly — it's shorter and triggers the same
   axioms.

4. **`capacity()` cast.** `capacity()` returns `nat`, the spec compares
   `old(self).view().len() == old(self).capacity()` and `result.is_err()`
   requires this. The exec-level guard is `self.msgs.len() == self.cap`.
   The implicit `usize -> nat` casts on both sides should line up; if they
   don't, an explicit `assert(old(self).msgs@.len() == old(self).cap as nat)`
   inside the `Err` branch will close it.

5. **`well_formed` preservation in success branch.** After `push`, the
   length grows by one. We need `self.msgs.len() <= self.cap`. The guard
   already established `self.msgs.len() < self.cap` (since `==` was false
   and `<=` was the precondition's invariant). Verus knows this, but if it
   complains, add `assert(old(self).msgs.len() < old(self).cap)` before
   `push`.

## Suggested order of operations

1. **`new`.** Trivially: `Log { cap: capacity, msgs: Vec::new() }`. Verify
   first to confirm the toolchain works.
2. **`len`.** One-liner: `self.msgs.len()`.
3. **`get`.** Two-line if/else returning `Some(self.msgs[index])` or `None`.
   The `index < self.msgs.len()` guard discharges the indexing
   precondition.
4. **`append` — Err branch first.** Write the capacity check. The `Err`
   postcondition is

   ```rust
   old(self).view().len() == old(self).capacity()
       && final(self).view() == old(self).view()
   ```

   Returning `Err(())` without mutating gives `final(self).view() == old(self).view()`
   for free; the equality `len == cap` follows from `len <= cap` (well_formed)
   and `len == cap` (the exec guard).
5. **`append` — Ok branch last.** This is the only place that needs proof
   nudges. Push, then add the two asserts from trouble spot #1, then
   `Ok(())`. If verification fails, try one assert at a time to localize.

If anything fails after one round of nudges, do **not** start adding helper
lemmas — re-read the verifier output, the missing fact is almost certainly
a `Seq::push` axiom that needs one more `assert(self.msgs@.len() == ...)`
to fire.

## Summary: A loop-free, Vec-backed implementation where the only real proof obligation is `append`'s frame property, dischargeable with two asserts after `Vec::push`.
