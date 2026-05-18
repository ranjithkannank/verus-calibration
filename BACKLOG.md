# Backlog

Work intentionally deferred. Each entry says what it is, why it was deferred,
what would need to be true to take it on, and where it slots back into the
roadmap.

## Verified Byzantine agreement (Lamport-Shostak-Pease, one-shot)

**What it is.** The textbook Byzantine agreement primitive: `n` nodes, up
to `f` Byzantine where `n > 3f`, all correct nodes agree on a single value
proposed by a designated source, even when the source itself may be
Byzantine. The classical 1982 result. Multi-round messaging, with each
node sending its received value to every other node in subsequent rounds.

**Why deferred.** Two reasons.

1. *It does not connect directly to the aerospace-safety problem we are
   solving.* Real avionics does not use textbook one-shot Byzantine
   agreement. It uses specialized protocols (SAFEbus, TTP, ARINC 664)
   tuned to flight-control timing and fault models. Verifying LSP is
   academically valuable but not on the direct path to lowering the cost
   of building safer avionics.
2. *Multi-round messaging is a substantial new verification regime.* Our
   harness has been stressed on single-function code (binary_search,
   quorum_count) and single-module code with structural reasoning and
   safety lemmas (bounded_log, quorum_cert). It has not been stressed on
   temporal reasoning, message-ordering invariants, view-change
   correctness, or any of the multi-round shapes Byzantine agreement
   needs. That is its own multi-month methodology question and worth
   tackling on its own, not folded into the BFT-primitive path.

**What would need to be true to take it on.**

- Sensor fusion (both `ft_midpoint` and `marzullo`) verified, so the
  single-round agreement / pigeonhole pattern is fully exercised through
  the harness.
- Hardware-deployed sensor-fusion demo in progress or done, so we have a
  concrete real-world setting to anchor the more abstract messaging
  reasoning.
- A specific aerospace problem that genuinely needs LSP-style agreement
  (rather than the narrower agreement primitives modern avionics
  actually uses). If no such problem is identified, this stays deferred
  indefinitely.

**Where it slots in.** After the sensor-fusion track lands and before
any work on full BFT consensus protocols (PBFT, HotStuff). It is the
natural bridge between primitive-level verification and protocol-level
verification, but only worth taking on once we know what protocol
shape an actual deployment would benefit from.

**Estimated cost.** Unknown. The verification effort for LSP in Verus
alone is plausibly comparable to the entire calibration plus
quorum_cert combined, because of the message-round inductive
reasoning. The harness changes to support multi-round protocols are
their own undertaking on top.

## VeruSAGE-Bench external-validity test

**What it is.** 849-task benchmark across real distributed systems,
OS kernels, and storage code, used by AutoVerus and VeruSAGE in their
published evaluations (Microsoft Research). Pull a small set (3-5
tasks initially), scaffold each in our spec-frozen / witness / tag
pattern, run them through the autonomous loop, and report
attempts-to-verify against AutoVerus's and VeruSAGE's numbers on the
same tasks.

**Why this matters.** Every exercise to date was either designed by
us or ported from a textbook. The methodology's external validity —
does it survive on tasks we didn't design? — is the load-bearing
assumption under every other forward move on the methodology track.
A positive result strengthens every prior claim; a negative result
tells us what to fix before bigger investments like LSP.

**What would need to be true to take it on.**

- The harness needs to accept externally-supplied specs. Currently
  every exercise has its design.md authored by us. A VeruSAGE-Bench
  task arrives as just a spec; we'd write a design note from the
  signature, then run the loop. The operator-authored witness step
  applies the same way (write a reference impl, run check-spec.sh).
- The task is small enough to fit the iteration cap. Some
  VeruSAGE-Bench tasks may be larger than our typical exercise; we'd
  pick smaller ones first and grow.
- We're willing to publish negative results. If our methodology does
  worse than AutoVerus on the same tasks, the right move is to
  report that honestly, not to retry until it works.

**Where it slots in.** Most natural next move on the methodology
track. Should *precede* LSP and the larger composition work — if
external validity doesn't hold on small VeruSAGE-Bench tasks, scaling
to LSP-sized verification is premature.

**Estimated cost.** Low for the first 3-5 tasks (a day of setup per
task plus loop runtime). A full benchmark sweep would be larger, but
the first tasks settle the methodology-survives-or-not question
inexpensively.

## Distributed-systems simulation + algorithm-variant tuning

**What it is.** Take a verified primitive (e.g., `marzullo`,
`sensor_poll_honest`), instantiate it as runnable code, run it inside
a distributed simulator (`madsim`, `dslabs`, or a custom
deterministic event simulator), inject realistic fault patterns
(network partitions, asymmetric latency, Byzantine messages near
the `n = 2f+1` boundary), measure behavior. Use the results to
explore algorithm variants that are still provably correct but
perform better under specific fault profiles.

**Why deferred.** Orthogonal to the verification track.
Verified-correct ≠ fault-tolerant-in-practice — a verified algorithm
can satisfy `n >= 2f+1` and still degrade badly when `f` is exactly
at threshold, when message latency varies asymmetrically across the
quorum, or when adversarial scheduling exploits message ordering.
Empirical simulation surfaces those gaps. But this is a different
*kind* of work than verification: it's about runtime behavior, not
spec satisfaction. Closer in spirit to the separate `bft_autotune`
project than to `verus-calibration`.

**What would need to be true to take it on.**

- A distributed simulator picked (a deterministic event simulator
  gives the cleanest scientific story; `madsim` is a candidate).
- The verified primitives lifted into runnable code. Verus
  generates Rust binaries from verified source; the lift is mostly
  packaging + a small driver per primitive.
- A scenario catalog (which fault patterns to test, what
  behavioral metrics to measure). Behavioral metrics matter most:
  agreement latency, message complexity, witness-quality under
  partial faults.
- A way to feed back into the verification track. If simulation
  surfaces a variant that performs better, we'd want to re-verify
  that variant — closing the empirical-to-verified loop is what
  makes this directly relevant to the project's main goal.

**Where it slots in.** Separate track. Possibly merged with
`bft_autotune` rather than `verus-calibration` proper. Should not
precede external-validity validation (VeruSAGE-Bench) on the
methodology track — empirical work that depends on a methodology
whose external validity is unproven is hard to scope.

**Estimated cost.** Weeks to months for the simulator + first
primitive instrumented. Per-experiment cost low once the harness
exists.

## "What's next" options as of 2026-05-18

Originally captured 2026-05-17 as a check-in after the multi-module
work landed; updated 2026-05-18 to reflect intervening completions
and two new candidates.

**Completed since 2026-05-17:**

- Option 1(b) — `sensor_poll_signed`: signature trust boundary
  threaded into the composition at the spec layer (see exercise on
  `main`).
- Option 4 — less-guided cross-module exercise: extended into two
  deliberate discovery tests (`sensor_poll_honest`,
  `counter_filler`), both 1-attempt successes, both audit-confirmed
  under hardened whitelist on 2026-05-18.
- *Bonus, not on the original list:* first invention test
  (`swap_multiset`), on a proof family the playbook did not
  document; 1-attempt success after two invalidated prior attempts
  (witness leak + operator copy-paste error) that drove the
  witness-deny ACL hardening.

**Open from the 2026-05-17 list:**

1. **Compose the existing BFT primitives into a small system.** —
   *option 1(b) DONE 2026-05-17 as `sensor_poll_signed`. Options
   1(a) and 1(c) still open.* The `sensor_poll` exercise has a
   verified end-to-end function (`poll`) whose correctness theorem
   spans the seam between two BFT-shaped primitives, via a
   projection lemma. The composition regime is demonstrated.

   What this *did not* do:
   - Did not import `quorum_cert` or `marzullo` as crate dependencies.
     Each existing exercise compiles as its own Verus crate; there
     is no way (in the current repo layout) to depend on one from
     another. `sensor_poll` *ports* the primitives — re-implements
     them as siblings inside its own crate.
   - Did not use the full `quorum_cert`. The `auth` module is a
     simplified variant — distinct-sensor structural check only. The
     signature-verification half (the `signature_valid` uninterp
     predicate, the `lemma_qc_has_honest_voter` honest-voter
     guarantee) was dropped.
   - Did not use `ft_midpoint`. `sensor_poll` composes `auth` +
     `marzullo`; `ft_midpoint` is unused.
   - Not really "a small system" — it's a verified end-to-end
     *function*, not a system. No multiple flows, no configuration
     surface, no integration points beyond one composition call.

   The bigger version of option 1, still open:
   a. Restructure existing exercises as importable Verus crates,
      so a downstream exercise can `use quorum_cert::*;` rather
      than re-implement. Possibly large repo refactor.
   b. Add the signature-verification half — bring `quorum_cert`'s
      `verify_qc_structure` properly into the composition so the
      trust boundary covers signed reports, not just distinct
      sensor IDs.
   c. Use both `ft_midpoint` and `marzullo`, or build a poll that
      selects between them.

   Each piece is its own work. Reasonable next exercise on this
   axis: "sensor_poll_signed" that does (b) — adds the
   signature-verification trust boundary — without yet attempting
   (a).
2. **Verified Byzantine agreement.** See the entry above. Multi-round
   messaging, larger scope, comparable to `quorum_cert` or
   `ft_midpoint` in complexity, distinct verification regime.
3. **Hardware deployment.** Take the existing sensor-fusion algorithms
   and run them on dissimilar redundant boards (Pi + BeagleBone +
   STM32) with live fault injection. Multi-week. Real-time and
   certification considerations dominate. Off-topic for the
   autonomous-loop methodology work; it's an avionics-engineering
   project.
4. **Less-guided cross-module exercise.** — *DONE 2026-05-17 as
   `counter_filler`; audit-confirmed 2026-05-18 under hardened
   whitelist.* Extended also into `sensor_poll_honest` (different
   proof family). Both 1-attempt successes. The discovery-vs-
   execution caveat that motivated this option has two data points
   on two proof families now.
5. **Stop and ship.** *Partially in progress.* Both writeup drafts
   (`composition-post.md`, `methodology-updates.md`) reframed
   2026-05-18 as revision inputs for the existing May 10 and May 17
   draft posts; the blog-writing agent prompt is queued.

**New candidates added 2026-05-18:**

6. **VeruSAGE-Bench external-validity test.** See the dedicated
   section above. Smallest commitment of the three new candidates;
   most informative result for the methodology claim. Recommended
   next move on the methodology track.
7. **Distributed-systems simulation + algorithm-variant tuning.**
   See the dedicated section above. Separate track from
   `verus-calibration`; closer in spirit to `bft_autotune`. Should
   not precede external-validity validation.

When picking back up: re-read this list, the writeup drafts in
`writeup/`, and the latest commit messages on `main`. The choice
depends on what the goal in the moment is (advance the BFT problem
verification → 2 or 1(a)/(c); validate the methodology externally
→ 6; explore the empirical/performance angle → 7; engage an
audience → 5 followed by selective publishing; build a physical
demo → 3).
