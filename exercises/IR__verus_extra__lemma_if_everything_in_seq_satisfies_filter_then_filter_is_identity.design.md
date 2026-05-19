# Design: IR__verus_extra__lemma_if_everything_in_seq_satisfies_filter_then_filter_is_identity

External-validity task from VeruSAGE-Bench (upstream prefix `IR` =
ironkv). Operator-authored design note — neutral by construction;
this batch is a methodology probe and the design note deliberately
withholds Verus tooling hints, lemma names, and proof-structure
suggestions.

## Frozen obligation

```
pub proof fn lemma_if_everything_in_seq_satisfies_filter_then_filter_is_identity<A>(
    s: Seq<A>,
    pred: spec_fn(A) -> bool,
)
    requires
        forall|i: int| 0 <= i && i < s.len() ==> pred(s[i]),
    ensures
        s.filter(pred) == s,
```

A pure ghost-level lemma: if every element of `s` satisfies the
predicate `pred`, then filtering `s` by `pred` leaves it unchanged.
No exec, no extra parameters beyond `s` and `pred`.

## Forbidden (audited at commit time)

- Adding `assume(`, `external_body`, `unreachable!()`, `panic!(`, or
  `assume_specification` anywhere in the file.
- Modifying the `requires` or `ensures` clauses.
- Removing `fn main() {}` or the `verus!{}` wrapper.

## Sub-tasks

1. Run `verus exercises/IR__verus_extra__lemma_if_everything_in_seq_satisfies_filter_then_filter_is_identity.rs --crate-type=lib`
   on the unmodified scaffold. Capture the exact rejection messages.
2. Attempt the smallest possible body that could close the
   obligation. Run verus.
3. If step 2 fails, read the rejection and iterate. Standard
   per-attempt protocol from AGENTS.md applies (one attempt per
   iteration, log each attempt, escalate after 3 consecutive
   failures on the same obligation).
