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
- "Halfway to a Verified Byzantine-Tolerant Sensor Poll"
- "When Composition Means Re-Implementation"

The writing agent picks. The most accurate title hints at the
partial scope ("halfway", "honestly", "without imports"). A
title that overclaims composition would be misleading; the post
itself spends meaningful space being clear about the gap.

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
honest one-sentence summary: we built a verified end-to-end
function whose correctness theorem spans the seam between two
BFT-shaped primitives, in one autonomous run, and the result is
narrower than "compose the existing primitives into a small
system" would suggest if read literally. The composition regime
is demonstrated; the literal composition is not.

This post is about the gap between those two things.

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
not over-claiming.

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
exercises. The `sensor_poll` exercise *ports* the primitives: it
re-implements them as sibling modules inside its own crate. From
a verification standpoint the proofs are real; from a "reuse the
verified artifacts" standpoint we did not reuse them as
artifacts, we copied their source.

**We did not use the full `quorum_cert`.** The original
`quorum_cert` exercise exports two pieces: a structural check
(`verify_qc_structure`, which validates that a quorum certificate
has the right shape — distinct voters in range, threshold met)
and a safety lemma (`lemma_qc_has_honest_voter`, which proves
that any quorum certificate over a Byzantine-bounded network
includes at least one honest voter). It also carries a real
trust boundary: `signature_valid(pk, msg, sig)` is an
uninterpreted spec predicate, with a real deployment expected to
connect it to an external crypto library.

The `auth` module in `sensor_poll` ports only the bitmap-distinct
half of this. It does not carry `signature_valid`, does not
prove the honest-voter guarantee, does not reason about a
signature trust boundary. "Distinct sensor IDs" is a much weaker
property than "valid signatures over signed reports." A real
Byzantine-tolerant sensor poll would need the latter; we have
the former.

**We did not use `ft_midpoint`.** The composition picks marzullo
(intervals) over ft_midpoint (scalar readings). One choice; we
made it for tractability. The verified `ft_midpoint` sits unused.

**This is not "a small system."** It is a verified end-to-end
function. A system would have multiple flows, a configuration
surface, integration points beyond a single composition call,
probably real I/O. What we built is one function whose
postcondition depends on two other functions' postconditions via
a projection lemma. That is composition reasoning, demonstrated;
it is not a system, claimed.

## What this does and does not show

The methodology question — does the autonomous loop handle
integration reasoning across primitives stated in different
domains — gets a yes. The composition seam closed on a one-line
extensional-equality identity. The agent produced this in one
attempt. The pattern is templatable for future composition
exercises.

The roadmap question — do we have a verified Byzantine-tolerant
sensor poll that reuses the three primitives as imported
components — gets a no. We have a verified function that touches
the composition regime using ported simplified versions of the
primitives. The pieces needed to make this real are concrete and
named:

1. Restructure existing exercises as importable Verus crates
   so a downstream exercise can declare them as dependencies
   rather than re-implement.
2. Add the signature-verification half — bring
   `verify_qc_structure` and the `signature_valid` trust
   boundary into the composition so the auth side covers signed
   reports, not just distinct sensor IDs.
3. Use both `ft_midpoint` and `marzullo`, or build a poll that
   selects between them based on the input shape.

Each is its own piece of work. They are now in the repo's
`BACKLOG.md` under the partial-completion entry for option 1.

## Where this fits

This work moved the methodology from "demonstrated on twelve
exercises, no composition" to "demonstrated on twelve exercises
plus one composition demonstration." That is progress on the
methodology front. It is partial progress on the
verified-BFT-systems goal: we have a composition regime confirmed
working, and a concrete punch list of three things that still
have to happen for a real end-to-end Byzantine-tolerant
verified system.

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
`main` branch, in `exercises/sensor_poll/`. The witness used to
confirm the spec admits a model is in
`exercises/sensor_poll_witness/`. The design note is in
`exercises/sensor_poll/design.md`. The agent's per-attempt
commits and the reviewer's audit are in the git history.

Re-run:

```bash
./scripts/install-hooks.sh
./ralph/check-spec.sh sensor_poll     # confirm spec admits a model
git tag spec-frozen-sensor_poll       # already exists; force-move if re-doing
./ralph/run-exercise.sh sensor_poll   # start the loop
```

## Commits supporting these claims

- `89fcee5` — `sensor_poll` scaffold + tooling
- `6d9ff8a` — implementer attempt-1, all three modules
- `64fc4a3` — reviewer's APPROVE note
- `2a2036b` — `sensor_poll: DONE`
- `ca061ca` — `BACKLOG.md` updated with the honest scope of what
  option 1 did and did not do

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
