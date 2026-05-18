# Design — `swap_multiset`

Invention test for the methodology. The proof obligation includes
a `final(v)@.to_multiset() == old(v)@.to_multiset()` postcondition;
the playbook in `AGENTS.md` has no entry mentioning `to_multiset()`
or multiset arithmetic.

This is the third attempt at staging this measurement. Two earlier
attempts (`vec_swap`, `vec_swap_v2`) were invalidated — the first
by an over-permissive tool whitelist that let the implementer
read the witness file, the second by an operator copy-paste error
that put the agent's prior solution into the v2 scaffold. Both
are kept on disk as evidence. This exercise uses a freshly-typed
unimplemented scaffold and runs under the hardened whitelist that
denies witness reads.

The methodology question: can the agent assemble a proof in a
family the playbook does not document? Prior discovery tests
(`sensor_poll_honest`, `counter_filler`) showed pattern
recognition + adaptation from playbook entries that named the
relevant family. There is no entry to recognise here. The agent
has to find vstd's `to_multiset` broadcast lemmas in
`vstd::seq_lib`, the multiset arithmetic axioms in
`vstd::multiset`, and assemble a count-by-element argument
without an anchor.

If the agent verifies in one attempt, that is a positive
invention data point. If multi-attempt, the per-attempt log is
the data on the cost of invention vs recognition. If escalation
fires or the cap hits, that is the negative data point. Each is
useful.

---

## 1. Layout

Single file: `exercises/swap_multiset.rs`. Witness mirrors at
`exercises/swap_multiset_witness.rs`. No mod declarations, no
sibling modules.

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

The first four ensures are positional facts. The fifth is the
multiset clause. The case `i == j` is allowed by the requires and
behaves as a no-op.

---

## 3. Sub-tasks

1. **Exec body.** Two reads, two writes.
2. **Multiset clause.** Discharge
   `final(v)@.to_multiset() == old(v)@.to_multiset()`.
3. **End-to-end verify.** `verus exercises/swap_multiset.rs --crate-type=lib`
   exits 0.

The design note deliberately does not name lemmas, broadcast
groups, or proof structure for sub-task 2.

---

## 4. Anti-patterns

- **Do not weaken the multiset postcondition.**
- **Do not introduce `assume(...)` or `external_body`** to dodge
  the multiset arithmetic.
- **Do not attempt to read `exercises/swap_multiset_witness.rs`.**
  The tool whitelist denies it; respect the spirit of the test.
