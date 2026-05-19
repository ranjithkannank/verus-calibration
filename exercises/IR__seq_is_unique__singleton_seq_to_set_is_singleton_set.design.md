# Design: IR__seq_is_unique__singleton_seq_to_set_is_singleton_set

External-validity task from VeruSAGE-Bench (upstream prefix `IR` =
ironkv). Operator-authored design note — neutral by construction;
this batch is a methodology probe and the design note deliberately
withholds Verus tooling hints, lemma names, and proof-structure
suggestions.

## Frozen obligation

```
pub proof fn singleton_seq_to_set_is_singleton_set<T>(x: T)
    ensures
        seq![x].to_set() == set![x],
```

A pure ghost-level equality between two singleton constructions of
different shapes: a singleton sequence converted to a set, and a
singleton set built directly. No exec, no recursion, no parameters
besides `x`.

## Forbidden (audited at commit time)

- Adding `assume(`, `external_body`, `unreachable!()`, `panic!(`, or
  `assume_specification` anywhere in the file.
- Modifying the `ensures` clause.
- Removing `fn main() {}` or the `verus!{}` wrapper — these are
  upstream scaffolding, not part of the obligation, but the file
  must compile as a whole.

## Sub-tasks

1. Run `verus exercises/IR__seq_is_unique__singleton_seq_to_set_is_singleton_set.rs --crate-type=lib`
   on the unmodified scaffold. Capture the exact rejection messages.
2. Attempt the smallest possible body that could close the
   obligation. Run verus.
3. If step 2 fails, read the rejection and iterate. Standard
   per-attempt protocol from AGENTS.md applies (one attempt per
   iteration, log each attempt, escalate after 3 consecutive
   failures on the same obligation).
