# Design: `bounded_log`

## Spec recap

```rust
pub struct Log {
    cap: usize,
    msgs: Vec<Message>,           // Message = u64
}

closed spec fn capacity(&self) -> nat       { self.cap as nat }
closed spec fn view(&self)     -> Seq<Message> { self.msgs@ }
closed spec fn well_formed(&self) -> bool   { self.msgs.len() <= self.cap }

fn new(capacity: usize) -> Self      // empty, capacity preserved, well-formed
fn len(&self) -> usize               // == view().len()
fn get(&self, index: usize) -> Option<Message>   // bounds-checked
fn append(&mut self, msg: Message) -> Result<(), ()>
   // success: len+1, last element == msg, frame: prefix unchanged
   // failure: at capacity, view unchanged
```

The frozen postconditions on `append` include the frame property
`forall|i: int| 0 <= i < old.len() ==> self.view()[i] == old.view()[i]`,
which is the centerpiece of this exercise.

## 1. Representation choice

Stay with the struct as given. `cap: usize`, `msgs: Vec<Message>`. No
auxiliary ghost variables, no parallel `Seq`. The whole point of the
exercise is to lean on `Vec`/`Seq` axioms for the frame property; adding
ghost state would defeat that.

The three `closed spec fn`s are defined *in this module*, so within the
`impl Log` block their bodies are transparent — `self.view()` unfolds to
`self.msgs@` for the SMT solver. (`closed` only hides bodies from
**outside** the module.)

The four exec functions need **no loops**. Every postcondition is
discharged by a single `Vec` primitive plus the built-in `Seq` axioms
that come with it. (The `// TODO(loop)` markers in the file are tags
identifying this as a "loop exercise" category, not a directive to add
loops where none are needed.) If verus surprises us by demanding manual
indexing, we revisit; first pass is loop-free.

## 2. Algorithmic sketch

```rust
fn new(capacity: usize) -> Self {
    Log { cap: capacity, msgs: Vec::new() }
}

fn len(&self) -> usize {
    self.msgs.len()
}

fn get(&self, index: usize) -> Option<Message> {
    if index < self.msgs.len() {
        Some(self.msgs[index])
    } else {
        None
    }
}

fn append(&mut self, msg: Message) -> Result<(), ()> {
    if self.msgs.len() < self.cap {
        self.msgs.push(msg);
        Ok(())
    } else {
        Err(())
    }
}
```

That's the entirety of the executable code. Whatever proof noise is
needed goes inside `append`.

## 3. Key invariants

Struct-level invariant is exactly `well_formed`:

```
self.msgs.len() <= self.cap
```

Preserved by:
- `new`: `msgs.len() == 0 <= cap` trivially.
- `append` success branch: we entered the branch under
  `self.msgs.len() < self.cap`, so after `push` we have
  `self.msgs.len() == old.msgs.len() + 1 <= self.cap`.
- `append` failure branch: no mutation.

No function-level invariant beyond what's in `well_formed` and the spec.

## 4. Loop invariant sketches

None. There are no loops in the planned implementation.

If the implementer is forced to introduce one (e.g. if `Vec::new()` or
`Vec::push` turn out to be unavailable in the current vstd surface — they
should be available), revisit and add an invariant block here.

## 5. Helper lemmas predicted

None expected. The needed facts are:

- `Vec::new()` spec: `result@.len() == 0`. (Built-in.)
- `Vec::push(x)` spec:
  - `self.len() == old(self).len() + 1`
  - `self@ == old(self)@.push(x)`
  - (Built-in via vstd.)
- `Seq::push(x)` axioms:
  - `s.push(x).len() == s.len() + 1`
  - `s.push(x)[s.len() as int] == x`
  - `forall|i: int| 0 <= i < s.len() ==> s.push(x)[i] == s[i]`
  - (Built-in axioms in vstd's `Seq`.)

If the frame `forall` in the `append` postcondition fails to close, the
fallback is to write the trivial proof inline as an
`assert(forall ... by { ... })` block, **not** a separate lemma.

## 6. SMT trouble spots

1. **Frame property in `append`.** After `self.msgs.push(msg)`, the solver
   must instantiate the `Seq::push` index axiom for every `i < old.len()`.
   Usually this is automatic. If not, the canonical nudge is:
   ```rust
   assert(self.msgs@ == old(self).msgs@.push(msg));
   assert(forall|i: int| 0 <= i < old(self).msgs@.len()
          ==> self.msgs@[i] == old(self).msgs@[i]);
   ```
   placed *between* the `push` and the `Ok(())`.

2. **Linking `view()` to `msgs@` across the mutation.** Because `view()`
   is `closed`, the solver inside this module *does* see the body, but
   I have occasionally seen Verus need a one-line assert tying `self.view()`
   to `self.msgs@` after `&mut self` mutation. Cheap insurance:
   ```rust
   assert(self.view() == self.msgs@);
   assert(old(self).view() == old(self).msgs@);
   ```

3. **`well_formed` preservation under `push`.** Needs
   `self.msgs.len() <= self.cap` after `push`. Have
   `old.msgs.len() < self.cap` (branch condition) and
   `self.msgs.len() == old.msgs.len() + 1`. Arithmetic should be trivial.
   If it isn't, assert `old(self).msgs.len() + 1 <= self.cap` before the
   push or `self.msgs.len() == old(self).msgs.len() + 1` after.

4. **Failure branch — `self.view() == old(self).view()`.** No mutation
   happened, so this is reflexive. Verus generally accepts this without
   help; if not, an explicit `assert(self.view() == old(self).view())`
   forces it.

5. **`capacity()` preservation.** `self.cap` is not touched in `append`,
   so `self.capacity() == old(self).capacity()` should be automatic. If
   the closed-spec unfolding wobbles, `assert(self.cap == old(self).cap)`
   then `assert(self.capacity() == old(self).capacity())`.

6. **`new` and `get`.** Expected to verify with zero proof annotation.
   The only surprise risk is `Vec::new()`'s view spec — confirm it
   gives `result@ == Seq::empty()` (it does in vstd).

## 7. Suggested order of operations

Easiest postcondition first, frame property last.

1. **`new`** — fill in `Log { cap: capacity, msgs: Vec::new() }`. Verify.
   Three postconditions, all immediate.
2. **`len`** — `self.msgs.len()`. Verify. One postcondition, immediate.
3. **`get`** — bounds check + index/return None. Verify. The `Some` branch
   uses `self.msgs[index]` whose spec gives `self.msgs@[index as int]`,
   which equals `self.view()[index as int]` by the (transparent-in-module)
   `view()` definition.
4. **`append` failure branch first** — write the whole function but focus
   on getting the `Err` arm to verify. The two failure obligations
   (`old.len() == old.cap`, `self.view() == old.view()`) are direct.
5. **`append` success branch, without frame** — verify `self.well_formed()`,
   `capacity` preserved, `len+1`, and `view()[old.len()] == msg`. These
   come from the `Vec::push` spec and the last-element axiom of `Seq::push`.
6. **Frame property** — usually goes through with no extra annotation;
   if it doesn't, drop in the assert from §6 #1. If even that doesn't
   work, escalate (do not introduce a lemma yet — write the inline
   `assert forall ... by { ... }` instead, since the body is one line).

Expected total body: ~25-35 lines including any defensive asserts. If
the implementation balloons past ~60 lines of proof, something is
wrong — back off and reconsider.

## Summary: Loop-free `Vec`-backed Log; only `append`'s frame property is non-trivial, and a single `assert` tying `self.msgs@` to `old(self).msgs@.push(msg)` should suffice if the built-in `Seq::push` axioms don't auto-fire.
