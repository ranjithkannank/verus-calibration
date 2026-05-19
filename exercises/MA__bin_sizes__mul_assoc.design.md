# Design: MA__bin_sizes__mul_assoc

External-validity task from VeruSAGE-Bench (upstream prefix `MA` =
memory-allocator). Operator-authored design note — the agent's THINK
phase is skipped because the task arrives spec-only.

## Frozen obligation

```
proof fn mul_assoc(x: nat, y: nat, z: nat)
    ensures (x * y) * z == y * (x * z)
{
    // body to fill
}
```

A pure ghost-level arithmetic fact over `nat`. No exec code. No
representation choices. No loops. No helper data structures.

## Why this can be hard for verus

`(x * y) * z == y * (x * z)` mixes associativity and commutativity in
one obligation. Verus's default SMT encoding splits multiplication
into linear and nonlinear fragments; a goal that depends on both
fragments often does not close from baseline axioms alone. The
solver needs a nudge toward nonlinear reasoning.

## Suggested order of operations

1. Try the obligation with no body. Some verus configurations close
   single-equality nonlinear goals over `nat` automatically; if so,
   you are done. Do not declare success without seeing `0 errors`.
2. If verus rejects, the next escalation is to invoke nonlinear
   arithmetic explicitly. Verus has dedicated proof-mode tooling for
   this — search the standard library and recent exercises in this
   repo for the canonical pattern.
3. If that still fails, decompose by hand: prove `(x*y)*z == x*(y*z)`
   via `lemma_mul_associative` from `vstd::arithmetic::mul`, then
   `x*(y*z) == y*(x*z)` via `lemma_mul_commutative` applied to the
   inner pair `(x, y*z)` plus `lemma_mul_associative` again. Three or
   four `assert(... by { ... })` blocks; each calls a vstd lemma.

The simpler approach is overwhelmingly likely to succeed; only fall
through to the manual decomposition if the SMT solver flatly refuses.

## Forbidden (audited at commit time)

- Adding `assume(`, `external_body`, `unreachable!()`, `panic!(`, or
  `assume_specification` anywhere in the file.
- Modifying the `ensures` clause.
- Removing `fn main() {}` or the `verus!{}` wrapper — these are
  upstream scaffolding, not part of the obligation, but the file must
  compile as a whole.

## Sub-tasks

1. Try the empty-body or one-line proof. Run `verus exercises/MA__bin_sizes__mul_assoc.rs --crate-type=lib`. If verification passes, log attempt-1 and stop.
2. (Only if step 1 fails.) Add the manual three-lemma chain from `vstd::arithmetic::mul`.
