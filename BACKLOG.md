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
