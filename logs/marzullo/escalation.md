# marzullo escalation — superseded

This exercise is **blocked**, not escalated for re-design. See
`logs/marzullo/blocked.md` for the full structural-gap analysis,
the constructive counterexample falsifying the postcondition, the
list of verified machinery, and the rejected alternatives.

The architect has reviewed three times (design.md revisions
`20260517T044650Z`, `20260517T044911Z`, `20260517T045216Z`) and
each revision has directed `blocked.md` to be written rather than
re-designing. There is no actionable escalation here; the spec is
frozen and the postcondition is provably unsatisfiable under a
satisfying assignment of the preconditions.

This file is preserved as a pointer to `blocked.md` because the
implementer prompt's `status` vocabulary (`verus_passed`,
`verus_failed`, `escalated`) does not include a `blocked` token,
and because earlier emptyings of this file appear to have caused
orchestrator re-trigger loops (see design.md
`20260517T044911Z` §"Tooling reality"). Keeping a one-paragraph
pointer here is more stable than an empty file.

Final verifier state: `8 verified, 2 errors` (the Helly bound at
line 292 and the downstream `marzullo` postcondition at line 334).
No further code or proof attempts on this exercise.
