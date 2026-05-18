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

## "What's next" options as of 2026-05-17

Set of five directions the methodology could go from here, captured
during a check-in after the multi-module work landed. Currently
pursuing option 1; others kept here for later.

1. **Compose the existing BFT primitives into a small system.**
   `quorum_cert`, `ft_midpoint`, and `marzullo` are verified but
   uncomposed. A verified Byzantine-tolerant sensor poll that uses
   the quorum-style structural check to authenticate signed sensor
   reports and then runs `ft_midpoint` or `marzullo` on the
   authenticated values would be the first end-to-end *system* on
   the path. Multi-module by necessity, with a composition theorem
   that spans the seam between two primitives. Bounded scope, novel
   regime (system-level integration), directly advances the original
   goal.
2. **Verified Byzantine agreement.** See the entry above. Multi-round
   messaging, larger scope, comparable to `quorum_cert` or
   `ft_midpoint` in complexity, distinct verification regime.
3. **Hardware deployment.** Take the existing sensor-fusion algorithms
   and run them on dissimilar redundant boards (Pi + BeagleBone +
   STM32) with live fault injection. Multi-week. Real-time and
   certification considerations dominate. Off-topic for the
   autonomous-loop methodology work; it's an avionics-engineering
   project.
4. **Less-guided cross-module exercise.** Same shape as
   `counter_producer` but with a design note that gives the proof
   obligation and *not* the loop invariant. Tests whether the
   methodology supports discovery of bridging invariants, not just
   execution of pre-named ones. Small. High signal per dollar on the
   methodology side, no direct progress on the BFT goal.
5. **Stop and ship.** Three follow-up post sources are drafted in
   `writeup/`. Run the blog-writing agent against them, publish, see
   what response the work gets, resume after. Fine fallback if other
   options stall or attention is needed elsewhere.

When picking back up: re-read this list, the multi-module post draft
in `writeup/multi-module-post.md`, and the latest commit messages on
`main`. The choice depends on what the goal in the moment is
(advance the BFT problem → 1 or 2; learn more about the methodology
→ 4; engage an audience → 5; build a physical demo → 3).
