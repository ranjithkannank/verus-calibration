# Source material — blog post on the multi-module generalisation

This file is raw input for the blog-writing agent. Goal: a self-contained
follow-up to <https://ranjithkannan.com/2026/05/10/verus-calibration-formal-verifier-loop/>
covering the multi-module and multi-file work that came after the BFT
exercises. Assumes the reader hasn't read the prior posts; opens with
enough context to stand alone.

Tone target: same as the existing posts (technical-practical, "we" voice,
minimal em-dashes, no folksy aphorisms, no time-of-day framing, no cost
numbers, no over-claiming). Be honest about what the result does and
doesn't show.

## Title candidates

- "Does a Verified-Coding Loop Generalise to Multi-Module Verus?"
- "Three Cross-Module Exercises, Three First-Attempt Verifications, One Caveat"
- "Multi-Module Verus Through the Autonomous Loop"
- "Beyond One File: Stressing the Harness on Cross-Module Reasoning"

The writing agent picks. The most accurate framing is "generalisation
checkpoint, not destination" — a title that hints at that ("does it
generalise", "one caveat") is closer to the truth than a triumphal one
("three first-attempt verifications").

## Opening

Earlier posts in this series described an autonomous coding loop wired
into the Verus formal verifier and used it to produce three verified
calibration exercises and three verified Byzantine fault-tolerant
primitives — a quorum certificate, a fault-tolerant midpoint, and
Marzullo's interval-agreement algorithm. Every exercise lived in a
single Rust source file with a single `verus! { }` block. The
verifier was invoked as `verus <file>.rs --crate-type=lib`. Whatever
trust we'd built in the methodology was trust under a single-file
assumption.

This post is the multi-module checkpoint. We took the same harness,
extended it to handle Verus code split across multiple modules and
multiple files, and ran three new exercises through it. All three
verified on the first attempt with reviewer approval. That's the
headline, and we'll get to the caveat before claiming anything from
it.

## Why multi-module is the next regime to stress

Real Verus codebases are not single files. They are crates of modules
with `pub` boundaries, `closed spec fn` definitions that hide
implementation details from callers, and `use` declarations that
import types and functions across module lines. Anything we want the
methodology to help produce — a verified consensus engine, a verified
sensor-fusion pipeline running on real hardware — is multi-module
from the outset.

So the question is mechanical: does the loop still work when the
exercise spans more than one module, more than one file, with a
trust boundary at the module line? The methodology had three sandbox
layers we depended on:

1. A pre-commit hook that whitelists paths, blocks verification-bypass
   tokens (`assume(...)`, `external_body`, `unreachable!()`,
   `panic!(...)`), and refuses any commit that modifies the frozen
   `requires` / `ensures` clauses of an exercise's spec.
2. A Claude Code tool whitelist scoping the agent's filesystem and
   shell access per role (architect / implementer / reviewer).
3. A second-agent audit on a different model that runs after `verus`
   exits zero and applies a five-point checklist against the diff
   between the working tree and the frozen tag.

Layers 2 and 3 are independent of file layout. Layer 1 was written
under the single-file assumption. Specifically: the path whitelist
matched `exercises/*.rs` and `exercises/*.design.md`, and the spec
preservation check derived the exercise name as
`basename "$f" .rs`, then looked for the tag
`spec-frozen-<name>`. None of that handles
`exercises/<name>/main.rs` cleanly.

Multi-module also opens a new failure surface in the spec itself. A
spec that uses `closed spec fn` to hide a representation, and a
function whose pre- and postconditions are stated in terms of that
closed spec, is harder to author correctly than a single-file
exercise. Two of the six exercises in the original calibration
required mid-run spec re-freezes — one for a Verus syntax
migration, one for a missing precondition. We expected the
multi-module exercises to surface similar operator-side mistakes.

## What we built

Three exercises, in increasing order of complexity:

### `cross_module_counter` — multi-module in one file

A `counter` module exports a bounded counter abstraction. Internal
fields `value: u32` and `bound: u32` are private. The public API is
three `closed spec fn`s — `value()`, `bound()`, `invariant()` — and
three exec methods: `new(bound)`, `incr(&mut self)`, `get(&self)`. A
`client` module imports `Counter` and implements `count_up_to(target)`,
which creates a fresh counter of bound `target` and increments it to
`target`.

The whole thing fits in one file with two nested `mod { }` blocks
inside a single `verus! { }`. No new tooling needed; the existing
single-file invocation walks the nested modules.

Important property: the `client` module *cannot* see `Counter`'s
private fields. The four-conjunct loop invariant inside
`count_up_to` (`c.invariant() && c.value() == i && c.bound() ==
target && i <= target`) is stated entirely in `closed spec fn`
vocabulary. The verifier re-establishes each conjunct after
`c.incr()` from `incr`'s postcondition alone.

The agent verified this in one attempt. The reviewer approved.

### `counter_multifile` — same algorithm, real directory layout

We then split `cross_module_counter` into a directory:

```
exercises/counter_multifile/
    main.rs           # mod counter; pub fn count_up_to(...)
    counter.rs        # struct Counter + closed spec fns + methods
```

The verifier is invoked as `verus
exercises/counter_multifile/main.rs --crate-type=lib`. The
declaration `mod counter;` in `main.rs` resolves to the sibling
`counter.rs` file via standard Rust module rules. Verus walks both
files in one invocation. We tested this on a throwaway prototype
before committing to it; the verifier handled it without complaint.

The harness extensions this required were modest:

- The pre-commit hook's path whitelist gained an
  `exercises/*/*.rs` clause and an `exercises/*/*.md` clause.
- The hook's cheat-token check, which previously matched
  `exercises/*.rs`, now also matches `exercises/*/*.rs`.
- The hook's spec-preservation step, which derived the exercise name
  from the filename, now derives it from the directory when the path
  matches `exercises/<dir>/<file>.rs`, and skips paths under
  `<name>_witness/` directories (those have no frozen tag).
- The pre-spec verification tool, which previously expected a
  single-file witness at `exercises/<name>_witness.rs`, now detects
  whether the layout is single-file or multi-file and runs `verus`
  on either the file or the directory's `main.rs` entry point.
- The orchestrator script, which previously hard-coded
  `EXFILE="exercises/${EXERCISE}.rs"`, now detects layout at script
  entry and threads a separate "edit scope" variable through the
  per-role prompts.

The agent verified this exercise on the first attempt. The reviewer
approved.

### `counter_producer` — cross-module composition

The third exercise adds a third module:

```
exercises/counter_producer/
    main.rs        # mod counter; mod producer; pipeline(target)
    counter.rs     # Counter (unchanged)
    producer.rs    # produce(c: &mut Counter, n: u32)
```

The `producer` module's job is to bulk-increment a counter by `n`. Its
contract:

```rust
pub fn produce(c: &mut Counter, n: u32)
    requires
        old(c).invariant(),
        old(c).value() + n <= old(c).bound(),
    ensures
        final(c).invariant(),
        final(c).value() == old(c).value() + n,
        final(c).bound() == old(c).bound(),
```

The body is a loop that calls `c.incr()` `n` times. The loop
invariant has six conjuncts:

```
c.invariant(),
c.value() == start + i,
c.bound() == old(c).bound(),
i <= n,
start + n <= c.bound(),
start == old(c).value(),
```

The proof rests on composing `incr`'s single-step postcondition
(`value goes up by 1, bound unchanged, invariant preserved`) into a
multi-step claim (`value goes up by exactly n, bound unchanged,
invariant preserved`). The `start + n <= c.bound()` conjunct is what
keeps `incr`'s precondition (`value < bound`) satisfied across all
`n` iterations.

This is the first exercise in the series where one module's loop
invariant carries a fact the other module cannot expose. `counter`
does not know about `start` or `i` or `n`; those are
`producer`-internal variables. The composition happens in
`producer`'s loop, not in `counter`'s API.

The agent verified this exercise on the first attempt. The reviewer
approved.

## What the witness file caught (and didn't)

Before tagging the frozen spec for each exercise, we ran the
operator-time pre-spec verification check that we'd added in the
previous methodology refinement. The check requires an
operator-authored reference implementation at
`exercises/<name>_witness.rs` (or
`exercises/<name>_witness/main.rs` for multi-file) and runs `verus`
on it. If the witness verifies, the spec is provably satisfiable;
if it doesn't, the spec is either unprovable or the witness is
wrong, and the operator fixes one before the agent loop ever
starts.

For all three multi-module exercises, the witness verified cleanly.
That confirmed the spec admitted a model and let us freeze it with
confidence. The check didn't catch anything because we didn't make
the kind of mistake the check is built to catch — but the
counterfactual is real: if either of the multi-module exercises had
been written with a syntax error or a missing precondition, the
witness check would have surfaced it at operator time, not at agent
attempt 6.

We separately re-ran the empirical negative test from the
previous post (which strips a precondition from a copy of the
`marzullo` witness and confirms `verus` rejects it). It still
passes. The witness check remains the load-bearing operator-time
safety net.

## What we learned

Three things, all of them narrow.

**The tooling generalises.** Verus's single-file invocation handles
multi-module and multi-file via the standard Rust module mechanism.
No `cargo-verus` or build-system orchestration is required for the
shape of exercise we're doing. The hook and the pre-spec
verification tool needed small extensions — path whitelist clauses,
a directory-aware exercise-name derivation, a layout switch in the
witness check — and after those extensions the harness handled all
three multi-module exercises with no friction. The hook still
caught every cheat-token attempt and still locked the frozen
clauses byte-for-byte.

**The playbook executes cleanly across module boundaries.** The
proof patterns the architect's playbook collected over the first
six exercises — `final(self)` syntax for `&mut self`
postconditions, four-conjunct loop invariants in closed-spec-fn
vocabulary, composing single-step postconditions over a loop — all
applied to the multi-module exercises without modification. The
implementer's first attempt produced the loop invariant the
architect's design note predicted, in each of the three cases.

**One regime didn't get tested.** This is the caveat. In all three
multi-module exercises, the architect's design note pre-named the
loop invariant. The implementer's job was to execute that design,
not to derive it. So we've shown the methodology produces a working
proof when the design is right; we have not shown the methodology
produces the design itself for genuinely new multi-module problems.

The hard skill in multi-module Verus is choosing the loop invariant
that bridges the module boundary. We don't have data on whether
the architect role finds that invariant from scratch, because the
design notes we wrote already contained it. A more honest test would
be a design note that gives the proof obligation and a short prose
hint, and lets the implementer derive the invariant. We deferred
that test.

## What this doesn't prove

It does not prove the methodology handles multi-module problems of
arbitrary shape. The three exercises are all variants of the same
underlying abstraction — a bounded counter with an external user.
Real cross-module verification problems have multiple state
variables, multiple distinct transitions, and invariants relating
state in one module to state in another. We touched the multi-module
regime three times with progressively complex tooling, but the
algorithmic shape stayed constant.

It does not prove the harness handles the next layer up either:
genuinely multi-file Verus crates with a `lib.rs`, a build system,
external dependencies under `assume_specification`, and downstream
consumers. We exercised the smallest possible multi-file shape, in
which `main.rs` declares two or three sibling modules and `verus`
walks them as one crate. Real systems use more structure than that.

And it does not change the overall trajectory. The reason any of
this matters is that we want verified Byzantine-tolerant systems
running on real hardware. That goal still requires the same two
unsolved pieces we named at the end of the BFT post: a verified
multi-round Byzantine agreement primitive, and a hardware deployment
of the sensor-fusion algorithms on dissimilar redundant boards with
live fault injection. Neither was advanced by the multi-module
checkpoint.

## Where this fits

This work moved the methodology from "demonstrated on six
single-file exercises" to "demonstrated on nine exercises across
single-file, single-file multi-module, multi-file, and multi-file
with cross-module composition." That's progress on the methodology
front. It is not progress on the BFT-for-safety-critical-systems
problem the methodology is meant to serve.

The next concrete step on the problem itself is still the verified
Byzantine agreement primitive (Lamport-Shostak-Pease-style,
multi-round), then a hardware-deployed demonstration of the
existing sensor-fusion algorithms running on multiple boards under
live fault injection. The multi-module checkpoint was a precondition
for those — it would have been awkward to attempt a multi-round
agreement protocol in a single file — but it isn't a substitute for
them.

If a reader is here looking for "is this approach mature enough to
build a verified avionics stack with," the honest answer remains
"not yet, but the gaps are concrete and named." If a reader is here
looking for "is this approach mature enough to be a useful proof
companion on a small verified Rust library," the honest answer is
"yes, for the regimes we've covered, and we know which regimes we
haven't covered."

## Reproducing

Everything in this post is in
<https://github.com/ranjithkannank/verus-calibration> on the `main`
branch.

- `exercises/cross_module_counter.rs` — single-file multi-module
- `exercises/counter_multifile/` — first multi-file exercise
- `exercises/counter_producer/` — multi-file with composition

Per-exercise design notes are in each exercise file or directory.
Per-attempt commits show the actual agent diffs. The reviewer's
audit notes are in `logs/<exercise>/review.md` for each run. The
harness extensions are in `scripts/git-hooks/pre-commit`,
`ralph/check-spec.sh`, and `ralph/run-exercise.sh`.

To re-run any single exercise:

```bash
./scripts/install-hooks.sh
./ralph/check-spec.sh <exercise>     # confirm spec admits a model
git tag spec-frozen-<exercise>       # freeze
./ralph/run-exercise.sh <exercise>   # start the loop
```

The exercise file or directory must already exist; the witness
file or directory must already exist; everything else the loop
produces.

## Commits supporting these claims

For the writing agent's reference:

- `dd88922` — `cross_module_counter: DONE`
- `42370be` — multi-file scaffold + tooling
- `d2e9e96` — `run-exercise.sh` learns multi-file layout
- `f5bea5d` — `counter_multifile: DONE`
- `2fe1a4b` — `counter_producer` scaffold
- `e0ab586` — `counter_producer: DONE`

All on `main` at <https://github.com/ranjithkannank/verus-calibration>.
