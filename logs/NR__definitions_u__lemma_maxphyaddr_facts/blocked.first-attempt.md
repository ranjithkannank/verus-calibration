# Blocked: NR__definitions_u__lemma_maxphyaddr_facts

**Reason:** harness limitation — pre-commit hook rejects the scaffold commit.

**Status:** blocked before any agent attempt. Iteration count: 0.

## What happened

The upstream task file (copied byte-for-byte from
`microsoft/verus-proof-synthesis/benchmarks/VeruSAGE-Bench/tasks/NR__definitions_u__lemma_maxphyaddr_facts.rs`)
declares

```rust
#[verifier(external_body)]
pub const MAX_PHYADDR_WIDTH: usize = 52;
```

The `external_body` marker is the upstream's way of declaring an
opaque trust-boundary constant whose actual value is constrained only
by an accompanying `pub axiom fn` declaration. It is part of the
task's frozen spec context.

`scripts/git-hooks/pre-commit`'s cheat-token check inspects the diff
for added `external_body` occurrences. On the initial scaffold commit
for a new exercise file, **every** line is an added line (the file
did not exist before). The hook therefore reports the baseline
`external_body` as agent-introduced and rejects the commit.

`ralph/check-spec.sh` has the same gap for the witness file (it does a
straight grep, not a diff, so any occurrence of `external_body` in
the witness — including comments referencing the policy — is flagged).

## What this is a data point on

The harness's no-cheat boundary was designed assuming the spec is
authored by the operator (us) and would never legitimately contain
`external_body`. External tasks from VeruSAGE-Bench do legitimately
contain `external_body` in their baseline scaffold, as the upstream's
way of declaring opaque trust-boundary constants alongside an axiom.

This is a real external-validity finding distinct from the proof
content itself: the methodology probe surfaces a harness adaptation
need before the agent loop can start.

## Fix sketch (not done tonight)

The pre-commit hook should treat the scaffold commit as a baseline
event. Cleanest implementation: skip cheat-token detection when the
spec-frozen tag for the exercise does not yet exist (i.e., when the
file is brand-new). Subsequent commits diff against the spec-frozen
tag and any newly-introduced `external_body` would still be caught.

`ralph/check-spec.sh`'s cheat check should mirror the same logic:
allow `external_body` in the witness iff it appears in the exercise
file (byte-aligned) too.

## What this does NOT block

The methodology probe continues with tasks 7 and 8. This task is
specifically blocked on harness, not on the methodology under test.
