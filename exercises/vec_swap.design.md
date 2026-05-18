# Design — `vec_swap`

First **invention test** for the methodology. The proof obligation
includes a `final(v)@.to_multiset() == old(v)@.to_multiset()`
postcondition; the playbook in `AGENTS.md` has no entry mentioning
`to_multiset()` or any multiset-arithmetic pattern. This exercise
exists to probe whether the methodology supports finding a
proof family that the playbook does not already document.

Prior discovery tests (`sensor_poll_honest`, `counter_filler`)
showed the methodology can *recognise* a proof family the
playbook names and apply it to a new obligation. They did not
test whether the methodology can *invent* — find an unfamiliar
proof family and assemble it from vstd primitives without a
playbook entry to anchor on.

The exec body is small (a few `Vec::set` calls). The proof half
is the work: discharge the multiset clause from `to_multiset`'s
broadcast lemmas in `vstd::seq_lib` and the multiset arithmetic
in `vstd::multiset`. The agent has the same vstd access it has
had on every prior exercise; what's new is that no prior
exercise sits adjacent in the playbook.

If the agent verifies in one attempt, that's a positive invention
data point. If not, the per-attempt log shows which sub-step is
the obstacle — locating the right vstd lemma, threading
intermediate count facts, or something else. Either outcome is
useful.

---

## 1. Layout

Single file: `exercises/vec_swap.rs`. Witness mirrors at
`exercises/vec_swap_witness.rs`. No mod declarations, no sibling
modules. This is a single-function exercise.

---

## 2. The contract

```rust
pub fn swap(v: &mut Vec<u32>, i: usize, j: usize)
    requires
        i < old(v)@.len(),
        j < old(v)@.len(),
    ensures
        final(v)@.len() == old(v)@.len(),
        final(v)@[i as int] == old(v)@[j as int],
        final(v)@[j as int] == old(v)@[i as int],
        forall|k: int|
            0 <= k < final(v)@.len() && k != i as int && k != j as int ==>
                final(v)@[k] == old(v)@[k],
        final(v)@.to_multiset() == old(v)@.to_multiset(),
```

The first four ensures are the positional facts — they fall out
of `Vec::set`'s built-in spec almost directly. The fifth is the
multiset clause.

The case `i == j` is allowed by the requires and behaves as a
no-op. The witness handles it as a separate case in the proof.

---

## 3. Sub-tasks

1. **Exec body.** Two reads and two writes:
   `let tmp = v[i]; let other = v[j]; v.set(i, other); v.set(j, tmp);`
   The positional ensures follow from `Vec::set`'s view spec.
2. **Multiset clause.** Discharge
   `final(v)@.to_multiset() == old(v)@.to_multiset()`. The exec
   body's effect on the view is two `update` operations.
3. **End-to-end verify.** `verus exercises/vec_swap.rs --crate-type=lib`
   exits 0.

The design note deliberately does not name lemmas, broadcast
groups, or proof structure for sub-task 2. Finding them is the
test.

---

## 4. Anti-patterns

- **Do not weaken the multiset postcondition** to `len()`
  equality or some other weaker invariant. The multiset clause
  is the substance of this exercise.
- **Do not assert a body for any uninterp spec fn** (there are
  none in this exercise, but the rule stays).
- **Do not introduce `assume(...)` or `external_body`** to dodge
  the multiset arithmetic.
