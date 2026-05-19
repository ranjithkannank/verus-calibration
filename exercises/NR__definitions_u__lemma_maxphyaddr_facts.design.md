# Design: NR__definitions_u__lemma_maxphyaddr_facts

External-validity task from VeruSAGE-Bench (upstream prefix `NR` =
nrkernel). Operator-authored design note — neutral by construction;
this batch is a methodology probe and the design note deliberately
withholds Verus tooling hints, lemma names, and proof-structure
suggestions.

## Frozen obligation

```
pub proof fn lemma_maxphyaddr_facts()
    ensures 0xFFFFFFFF <= MAX_PHYADDR <= 0xFFFFFFFFFFFFF
```

where `MAX_PHYADDR` is an `exec const` declared earlier in the file
as `(1usize << MAX_PHYADDR_WIDTH) - 1usize`, and
`MAX_PHYADDR_WIDTH: usize` is an opaque constant constrained only by
`axiom_max_phyaddr_width_facts` to lie in `[32, 52]`.

A pure ghost-level lemma stating concrete numeric bounds on
`MAX_PHYADDR`, derived from the axiomatized bit-width range. The
file declares `MAX_PHYADDR_WIDTH`, the axiom, `MAX_PHYADDR_SPEC`,
`MAX_PHYADDR`, and the target lemma. No exec changes are required.

## Forbidden (audited at commit time)

- Adding any verification-bypass tokens beyond what the scaffold
  baseline already carries.
- Modifying the `ensures` clause of `lemma_maxphyaddr_facts`, the
  axiom declaration, or the `MAX_PHYADDR` / `MAX_PHYADDR_SPEC`
  definitions.
- Removing `fn main() {}`, `global size_of usize == 8;`, or the
  `verus!{}` wrapper.

## Sub-tasks

1. Run `verus exercises/NR__definitions_u__lemma_maxphyaddr_facts.rs --crate-type=lib`
   on the unmodified scaffold. Capture the exact rejection messages.
2. Attempt the smallest possible body that could close the
   obligation. Run verus.
3. If step 2 fails, read the rejection and iterate. Standard
   per-attempt protocol from AGENTS.md applies (one attempt per
   iteration, log each attempt, escalate after 3 consecutive
   failures on the same obligation).
