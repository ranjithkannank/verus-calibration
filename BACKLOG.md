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
during a check-in after the multi-module work landed. Option 1
*partially* done in this session — see `sensor_poll` exercise on
`main`; honest scope below. Options 2-5 still open.

1. **Compose the existing BFT primitives into a small system.** —
   *partially done 2026-05-17.* The `sensor_poll` exercise has a
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
