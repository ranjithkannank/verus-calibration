# Design: MA__bin_sizes__shift_is_div

External-validity task from VeruSAGE-Bench (upstream prefix `MA` =
memory-allocator). Operator-authored design note — neutral by
construction; this batch is a methodology probe and the design note
deliberately withholds Verus tooling hints, lemma names, and
proof-structure suggestions.

## Frozen obligation

```
proof fn shift_is_div(x: u64, shift: u64)
    requires 0 <= shift < 64,
    ensures x >> shift == x as nat / pow2(shift as int),
```

where `pow2(i: int) -> nat` is defined in the scaffold as `1` for
`i <= 0` and `pow2(i-1) * 2` otherwise.

A pure ghost-level lemma relating a `u64` bit-shift to division by a
power of two over `nat`. The shift width is bounded by `64`. The
file's only frozen definitions are `pow2` and the obligation itself;
any helper proof functions the proof needs may be added.

## Forbidden (audited at commit time)

- Adding `assume(`, `external_body`, `unreachable!()`, `panic!(`, or
  `assume_specification` anywhere in the file.
- Modifying the `requires` or `ensures` clause, or the body of
  `pow2`.
- Removing `fn main() {}` or the `verus!{}` wrapper.

## Sub-tasks

1. Run `verus exercises/MA__bin_sizes__shift_is_div.rs --crate-type=lib`
   on the unmodified scaffold. Capture the exact rejection messages.
2. Attempt the smallest possible body that could close the
   obligation. Run verus.
3. If step 2 fails, read the rejection and iterate. Standard
   per-attempt protocol from AGENTS.md applies (one attempt per
   iteration, log each attempt, escalate after 3 consecutive
   failures on the same obligation).
