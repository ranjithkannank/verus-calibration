# Design: AL__a_submap_of_a_finite_map_is_finite

External-validity task from VeruSAGE-Bench (upstream prefix `AL` =
anvil-library). Operator-authored design note — neutral by
construction; this batch is a methodology probe and the design note
deliberately withholds Verus tooling hints, lemma names, and
proof-structure suggestions.

## Frozen obligation

```
pub proof fn a_submap_of_a_finite_map_is_finite<K, V>(m1: Map<K, V>, m2: Map<K, V>)
    requires
        m1.submap_of(m2),
        m2.dom().finite(),
    ensures
        m1.dom().finite(),
```

A pure ghost-level lemma: if `m1` is a submap of `m2` (every key in
`m1` is in `m2` and maps to the same value), and `m2`'s domain is
finite, then `m1`'s domain is also finite.

## Forbidden (audited at commit time)

- Adding `assume(`, `external_body`, `unreachable!()`, `panic!(`, or
  `assume_specification` anywhere in the file.
- Modifying the `requires` or `ensures` clauses.
- Removing `fn main() {}` or the `verus!{}` wrapper — these are
  upstream scaffolding, not part of the obligation, but the file
  must compile as a whole.

## Sub-tasks

1. Run `verus exercises/AL__a_submap_of_a_finite_map_is_finite.rs --crate-type=lib`
   on the unmodified scaffold. Capture the exact rejection messages.
2. Attempt the smallest possible body that could close the
   obligation. Run verus.
3. If step 2 fails, read the rejection and iterate. Standard
   per-attempt protocol from AGENTS.md applies (one attempt per
   iteration, log each attempt, escalate after 3 consecutive
   failures on the same obligation).
