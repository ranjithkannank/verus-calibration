# Source material — blog post on composing two verified primitives

This file is raw input for the blog-writing agent. Goal: a
self-contained follow-up to the prior posts in the series. Assumes
the reader hasn't read them; opens with enough context to stand
alone. The piece's centerpiece is an honest accounting of what
"composing existing primitives" did and did not mean here.

Tone target: same as the existing posts (technical-practical, "we"
voice, minimal em-dashes, no folksy aphorisms, no time-of-day
framing, no cost numbers, no over-claiming). Be honest about what
the result does and doesn't show, including the partial scope of
the goal.

## Title candidates

- "What 'Composing Two Verified Primitives' Actually Means"
- "Composition Without Imports: A Verified Sensor Poll, Honestly"
- "From Composition Demonstration to Honest-Voter Guarantee, in Three Exercises"
- "Closing the Loop on Designed vs Discovered Proofs"

The writing agent picks. The first two are the original framing
(one composition exercise, partial scope). The third reflects the
extended arc the post now covers (three composition exercises
landing two of the three "what this did not do" items from the
first run). The fourth foregrounds the deliberate discovery test
that motivated the third exercise. A title that overclaims
composition would be misleading; the post itself spends meaningful
space being clear about the remaining gaps.

## Opening

Earlier posts in this series described an autonomous coding loop
wired into the Verus formal verifier, used it to produce six
calibration exercises, three Byzantine fault-tolerant primitives
(a quorum certificate, a fault-tolerant midpoint, Marzullo's
interval-agreement algorithm), and three multi-module exercises
that stressed the harness on cross-module reasoning. Each
artifact lived in its own Verus crate. They were verified but
uncomposed.

This post is what happened when we tried to compose them. The
honest one-sentence summary: we built three composition
exercises, each adding one piece of what a real Byzantine-tolerant
verified sensor poll would need. The first demonstrated the
composition regime. The second threaded the cryptographic trust
boundary through the contract. The third strengthened the
postcondition with an honest-voter guarantee and ran as a
deliberate discovery test for the methodology itself. All three
verified in one attempt each. The result is meaningfully closer
to "compose the existing primitives into a small system" than the
first exercise alone suggested, and it remains short of that
literal goal in ways the post is explicit about.

This post is about both halves.

## What we set out to do

The previous post noted that the three verified BFT primitives —
`quorum_cert`, `ft_midpoint`, `marzullo` — were uncomposed. A
verified Byzantine-tolerant sensor poll that authenticates signed
sensor reports via the quorum-style check and combines the
authenticated readings via the fault-tolerant midpoint or
Marzullo would be the first end-to-end system on the path. We
listed it as the smallest concrete next move on the
verified-BFT-systems trajectory.

The methodology question riding along: does the autonomous loop
handle integration reasoning, where the correctness statement
spans the seam between two primitives stated in different
domains?

Both questions had clear yes/no shapes. Both deserve clear yes/no
answers.

## What we built

A new exercise, `sensor_poll`, in a multi-file directory layout:

```text
exercises/sensor_poll/
    main.rs      # mod fusion; mod auth; poll(reports, n, f)
    fusion.rs    # marzullo
    auth.rs      # SensorReport + distinct_sensors + check_distinct
    design.md
```

The `fusion` module is a verbatim port of the verified `marzullo`
exercise. Same types, same uninterp `correct_at` trust boundary,
same five `open spec fn` definitions, same algorithm, same proof
helpers. The agent ported it from the source exercise in one
attempt.

The `auth` module defines a `SensorReport` struct carrying a
`sensor_id: u32` and an `interval`, plus a `distinct_sensors`
predicate and a `check_distinct` exec function that decides it.
The implementation is the bitmap-backed single-pass pattern from
`quorum_cert::verify_qc_structure`, simplified to one vector and
one threshold.

The `main` module defines `poll(reports, n, f) -> Option<Interval>`.
The body: call `check_distinct`; if false, return `None`. Otherwise
project each `SensorReport`'s `interval` field into a fresh
`Vec<Interval>`. Call `marzullo` on that. Use `choose` to extract
the witness point from `marzullo`'s existential, then a one-line
projection lemma to bridge the frame.

The projection lemma is the load-bearing piece. Marzullo's
postcondition is stated in terms of `intervals_containing(intervals@, p)`
— a set of indices into the projected `Seq<Interval>`. The caller
of `poll` wants a fact about `reports_containing(reports@, p)` —
a set of indices into the original `Seq<SensorReport>`. These two
sets are extensionally equal because the projection just pulls
out the `interval` field; the membership predicates align after
substitution. Verus closes the equality with `=~=` alone.

```rust
proof fn lemma_reports_eq_intervals_containing(
    reports: Seq<SensorReport>, p: Reading)
    ensures
        reports_containing(reports, p)
            =~= intervals_containing(project_intervals(reports), p),
{
    // body intentionally empty
}
```

That empty-body lemma is the entire composition seam. Everything
else is plumbing: the structural check, the port, the
`choose`-and-instantiate dance in `poll`'s body.

## The result

The agent verified the exercise on the first attempt. `verus
exercises/sensor_poll/main.rs --crate-type=lib` reports 16
verified, 0 errors. The reviewer's five-point audit returned
APPROVE. Two commits in the chain after the scaffold: the
implementer's attempt-1 and the reviewer's APPROVE. Total
wall-clock for the loop: a few minutes per role, single iteration.

The agent's solution and the operator-authored witness (which we
wrote before freezing the spec to confirm the spec admits a
model) differ in detail: the agent's `marzullo` port is tighter,
the witness has more intermediate lemmas. Both pass. The design
note pre-named the projection lemma and the `choose`/instantiate
pattern, so the agent's job was execution, not discovery — same
caveat as the previous multi-module exercises. Worth noting and
not over-claiming. We come back to this caveat below; it is what
the third exercise in this post was built to test.

## Closing the signature gap — `sensor_poll_signed`

The first exercise's `auth` module ported only the
bitmap-distinct half of `quorum_cert`. "Distinct sensor IDs" is
a much weaker property than "valid signatures over signed
reports." The second exercise — `sensor_poll_signed` — closed
that gap at the contract layer.

`fusion.rs` and the structural half of `auth.rs` are unchanged.
`auth.rs` gains the cryptographic trust boundary lifted from
`quorum_cert`: `Hash`, `PubKey`, `Signature` type aliases; three
uninterpreted spec predicates (`pk_of`, `signature_valid`,
`report_msg`); two open spec predicates
(`all_signatures_valid(reports)` and a `valid_report_bundle`
conjunction of distinct-and-signed); and a `sig: Signature`
field on `SensorReport`. `poll`'s precondition gains
`all_signatures_valid(reports@)`. Its `Some`-branch ensures
gains `valid_report_bundle(reports@)`.

What the exec layer does not gain: a signature-verification
function. `signature_valid` stays opaque, the same way
`quorum_cert.rs` left it. A real deployment would connect it to
a vetted external crypto library via a thin exec wrapper outside
the repo. The trust boundary in this exercise lives entirely at
the spec layer — the caller of `poll` is responsible for having
verified signatures upstream, the same way a real Byzantine
protocol receives already-signed messages from its network
stack. The composition theorem now states that `poll` only
returns `Some` when the inputs constitute a valid signed bundle,
which is a real strengthening even though no exec-layer
signature check happens inside the exercise.

The implementer's new work was a one-line conjunction:
`assert(valid_report_bundle(reports@));` after `check_distinct`
returns true. `distinct_sensors(reports@)` is in scope from
`check_distinct`'s ensures, and `all_signatures_valid(reports@)`
is in scope from the precondition, and Verus joins them. The
rest of `poll`'s body is byte-equivalent to the first
exercise's. One attempt, 16 verified, 0 errors, reviewer
APPROVE.

This is the second of the three items the first post's "what
this did not do" section named. It moves the trust-boundary axis
from "structural distinct only" to "structurally distinct
and signed at the spec layer." It does not move the
"exec-layer signature verification" axis. That is the next step
on this axis and still in BACKLOG.

## The honest-voter guarantee — `sensor_poll_honest`

The third exercise pushed harder. The first two exercises both
carried the same caveat: the design note pre-named the
load-bearing proof construct, so the agent executed a designed
proof rather than discovering one. Every 1-attempt success since
`marzullo` has carried this caveat. It is worth taking
seriously — methodology that handles execution of designed
proofs is a narrower claim than methodology that supports
discovery.

`sensor_poll_honest` was set up specifically to test the
discovery half. Its `fusion.rs` and `auth.rs` are byte-identical
to `sensor_poll_signed`. Its `main.rs` adds one conjunct to
`poll`'s `Some`-branch ensures:

```rust
&&& exists|p: Reading, k: int|
    interval.lo <= p && p <= interval.hi
    && 0 <= k < reports.len()
    && correct_at(k)
    && point_in_interval(p, reports[k].interval)
```

In words: there exists a point `p` in the returned interval AND
an index `k` such that sensor `k` is honest (not Byzantine) AND
its reported interval contains `p`. This is the BFT-meaningful
strengthening: the signature trust boundary is now load-bearing
in the proof, not just threaded through the contract. The
returned interval is now provably backed by at least one honest
sensor's report.

The design note for this exercise was deliberately incomplete.
It stated the obligation. It stated the informal mathematical
content — that `n - f` supporters and `n - f` correct sensors,
both subsets of an `n`-element universe, must overlap, and that
with `n >= 2f + 1` the overlap has at least one element. It did
not name the supporting lemmas. It did not name the helper-set
constructions. It did not name the trigger annotations. It did
not name the sub-proof structure. The architect's playbook
(accumulated in `AGENTS.md` from earlier exercises) does name
those constructs, including the inclusion-exclusion identity
`lemma_set_intersect_union_lens` and the universe-finite bridge
via `lemma_int_range` and `lemma_len_subset`. But those entries
sit under `ft_midpoint` — a different exercise with a different
proof obligation in the same proof family.

The agent verified in one attempt. Its proof introduced a new
helper lemma `lemma_honest_supporter_exists(reports, p, f)`:

```rust
proof fn lemma_honest_supporter_exists(
    reports: Seq<SensorReport>, p: Reading, f: nat)
    requires
        reports.len() >= 2 * f + 1,
        correct_indices(reports.len()).len() >= reports.len() - f,
        reports_containing(reports, p).len() >= reports.len() - f,
    ensures
        exists|k: int|
            0 <= k < reports.len()
            && correct_at(k)
            && point_in_interval(p, reports[k].interval),
{
    // ... establishes both sets are subsets of [0, n) via
    // lemma_int_range and lemma_len_subset, applies
    // lemma_set_intersect_union_lens to get
    // |s ∪ c| + |s ∩ c| == |s| + |c|, bounds |s ∪ c| <= n,
    // concludes |s ∩ c| >= 2(n − f) − n >= 1, then uses
    // axiom_is_empty_len0 / axiom_is_empty to extract the
    // witness from the non-empty intersection.
}
```

The lemma's name, its signature, its proof structure, its choice
of helper-set construction, and its specific use of the
inclusion-exclusion identity were all the agent's. The design
note named none of them. The agent recognised that the proof
family from `ft_midpoint`'s playbook entry applied to a new
situation in a different exercise on a different obligation, and
reused it.

This is one data point on one proof family. It moves the
designed-vs-discovered axis from "untested, plausible caveat" to
"tested once, supports discovery within an established family."
It does not yet tell us whether the methodology supports
discovery on a proof family the playbook does not already
document. That is a separate test on a different exercise, and
the next move on the methodology axis.

## What this did not do

The framing "compose the existing primitives into a small
system" reads literally as "import `quorum_cert` and `marzullo`
and `ft_midpoint` from their existing crates, use them as
building blocks, prove an end-to-end fact about the combined
behavior." That is not what we did. The accurate accounting:

**We did not import the existing primitives as crate
dependencies.** Each existing exercise compiles as its own Verus
crate via `verus <file>.rs --crate-type=lib`. There is no
`Cargo.toml`, no workspace, no dependency arrows between
exercises. All three composition exercises *port* the primitives:
they re-implement them as sibling modules inside their own
crate. From a verification standpoint the proofs are real; from
a "reuse the verified artifacts" standpoint we did not reuse
them as artifacts, we copied their source. Restructuring the
existing exercises as importable Verus crates is still open in
BACKLOG.

**We did not push signature verification into the exec layer.**
`sensor_poll_signed` and `sensor_poll_honest` both carry the
`signature_valid` uninterp predicate and reason about it at the
spec layer. Neither exercise has an exec function that walks
each report and calls a cryptographic verifier; that step is
left to the caller (and ultimately to an external library
connected via an `assume_specification` outside the repo, the
way `quorum_cert.rs` is structured). The exec-layer trust
boundary is the same as `quorum_cert`'s — and the same
limitation. Adding it is a step on this axis still open in
BACKLOG.

**We did not use `ft_midpoint`.** All three exercises pick
marzullo (intervals) over ft_midpoint (scalar readings). One
choice; we made it for tractability. The verified `ft_midpoint`
sits unused.

**These are not "a small system."** They are three verified
end-to-end functions, each building on the last. A system would
have multiple flows, a configuration surface, integration points
beyond a single composition call, probably real I/O. What we
have is a chain of three functions whose postconditions depend on
two other functions' postconditions via a projection lemma and,
in the third exercise, an inclusion-exclusion lemma. That is
composition reasoning, demonstrated; it is not a system, claimed.

## What this does and does not show

The composition-regime question — does the autonomous loop
handle integration reasoning across primitives stated in
different domains — gets a yes from all three exercises. The
first exercise's composition seam closed on a one-line
extensional-equality identity. The second exercise threaded an
opaque trust-boundary precondition through the contract without
disturbing the exec layer. The third exercise produced a new
inclusion-exclusion lemma to bridge two large index subsets in
a finite universe. All three landed in one attempt.

The designed-vs-discovered question — does the methodology
support discovery, not just execution of pre-named proof
constructs — gets a tentative yes, on one proof family. The
third exercise was the first deliberate test. The agent
recognised the inclusion-exclusion family from the `ft_midpoint`
playbook entry and applied it to a new obligation in a new
exercise. This is suggestive on one data point. A second
discovery test on a different proof family is the natural next
move.

The roadmap question — do we have a verified Byzantine-tolerant
sensor poll that reuses the three primitives as imported
components — still gets a no. We have three verified functions
that exercise the composition regime, the spec-layer trust
boundary, and the honest-voter guarantee, using ported (not
imported) primitives without an exec-layer cryptographic check
and without using `ft_midpoint`. The pieces still missing for a
real end-to-end verified Byzantine-tolerant sensor poll are
concrete and named:

1. Restructure existing exercises as importable Verus crates
   so a downstream exercise can declare them as dependencies
   rather than re-implement. (Still open.)
2. Add the exec-layer signature-verification step. The spec
   layer is now wired through; the exec wrapper around
   `signature_valid` (almost certainly an
   `assume_specification` outside the repo connected to a
   vetted crypto library) is the missing piece. (Half done.)
3. Use both `ft_midpoint` and `marzullo`, or build a poll that
   selects between them based on the input shape. (Still open.)

Each is its own piece of work. They are in the repo's
`BACKLOG.md` under the partial-completion entry for option 1.

## Where this fits

This work moved the methodology from "demonstrated on twelve
exercises, no composition" to "demonstrated on twelve exercises
plus three composition exercises plus one deliberate discovery
test." That is progress on the methodology front, with one
explicit caveat (the discovery test is one data point on one
proof family) and one explicit non-caveat (the spec-layer trust
boundary is now wired through to an honest-voter guarantee).
It is partial progress on the verified-BFT-systems goal: we
have a composition regime confirmed working at three levels of
sophistication, and a shorter punch list of things that still
have to happen for a real end-to-end Byzantine-tolerant verified
system.

The choice of what to do next on that punch list depends on
which gap matters more. The signature-verification half is the
most direct on the trust-boundary axis — it is what an avionics
reviewer would ask about first. The repo restructure is the
most direct on the reusable-artifacts axis — it is what makes
"compose verified components" a real practice. Neither is hard;
both are work; neither is needed before the other.

Or the next move is hardware deployment of the existing
primitives, which is a wholly different track and is captured
elsewhere in the BACKLOG. The trajectory is unchanged: get to a
verified Byzantine-tolerant sensor poll running on dissimilar
hardware under live fault injection.

## Reproducing

Everything in this post is in
<https://github.com/ranjithkannank/verus-calibration> on the
`main` branch, in `exercises/sensor_poll/`,
`exercises/sensor_poll_signed/`, and
`exercises/sensor_poll_honest/`. The witnesses used to confirm
each spec admits a model live under
`exercises/<name>_witness/`. The design notes are in
`exercises/<name>/design.md`. The agent's per-attempt commits
and the reviewer's audits are in the git history.

Re-run any of them:

```bash
./scripts/install-hooks.sh
./ralph/check-spec.sh <exercise>      # confirm spec admits a model
git tag spec-frozen-<exercise>        # already exists; force-move if re-doing
./ralph/run-exercise.sh <exercise>    # start the loop
```

with `<exercise>` one of `sensor_poll`, `sensor_poll_signed`,
`sensor_poll_honest`.

## Commits supporting these claims

`sensor_poll`:

- `89fcee5` — scaffold + tooling
- `6d9ff8a` — implementer attempt-1, all three modules
- `64fc4a3` — reviewer's APPROVE note
- `2a2036b` — `sensor_poll: DONE`
- `ca061ca` — `BACKLOG.md` updated with the honest scope

`sensor_poll_signed`:

- `dacd129` — scaffold + witness (signature trust boundary
  added at the spec layer)
- `75e54f0` — `sensor_poll_signed: DONE`

`sensor_poll_honest`:

- `f85bca5` — scaffold + witness (design note omits lemma
  names by construction)
- `bbb8e69` — implementer attempt-1, introduces
  `lemma_honest_supporter_exists` via inclusion-exclusion
  recognised from the `ft_midpoint` playbook entry
- `ad91c63` — `sensor_poll_honest: DONE`

All on `main` at
<https://github.com/ranjithkannank/verus-calibration>.

## A note on tone for the writing agent

The hardest line to keep in this piece is the line between
"we did real composition work" (true, and worth saying) and "we
composed the existing primitives" (overclaim, and worth not
saying). The post is most useful if it lands the first without
implying the second. The "What this did not do" section exists
precisely to keep the honest read accessible to a reader who
skims. Resist the urge to soften it or to bury it after the
methodology-win section. The honesty is the point.

The discovery-test result in the third exercise is the strongest
methodology claim in this post. It is also a one-data-point
claim. Land it accurately: the agent recognised and reused a
pattern from the playbook in a different exercise with a
different obligation, in one attempt. It does not show that the
methodology can invent patterns the playbook does not document.
The second discovery test, on a different proof family, is the
test that earns the stronger claim. That is the next move on the
methodology axis.
