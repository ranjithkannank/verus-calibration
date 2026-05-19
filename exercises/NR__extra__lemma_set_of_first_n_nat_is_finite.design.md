# Design: NR__extra__lemma_set_of_first_n_nat_is_finite

External-validity task from VeruSAGE-Bench (upstream prefix `NR` =
nrkernel). Operator-authored design note — neutral by construction;
this batch is a methodology probe and the design note deliberately
withholds Verus tooling hints, lemma names, and proof-structure
suggestions.

## Frozen obligation

```
pub proof fn lemma_set_of_first_n_nat_is_finite(n: nat)
    ensures Set::new(|i: nat| i < n).finite()
```

A pure ghost-level lemma stating that the set of naturals strictly
below `n` is finite. The single parameter `n` is itself a `nat`. No
exec code, no preconditions.

## Forbidden (audited at commit time)

- Adding `assume(`, `external_body`, `unreachable!()`, `panic!(`, or
  `assume_specification` anywhere in the file.
- Modifying the `ensures` clause.
- Removing `fn main() {}` or the `verus!{}` wrapper — these are
  upstream scaffolding, not part of the obligation, but the file
  must compile as a whole.

## Sub-tasks

1. Run `verus exercises/NR__extra__lemma_set_of_first_n_nat_is_finite.rs --crate-type=lib`
   on the unmodified scaffold. Capture the exact rejection messages.
2. Attempt the smallest possible body that could close the
   obligation. Run verus.
3. If step 2 fails, read the rejection and iterate. Standard
   per-attempt protocol from AGENTS.md applies (one attempt per
   iteration, log each attempt, escalate after 3 consecutive
   failures on the same obligation).
