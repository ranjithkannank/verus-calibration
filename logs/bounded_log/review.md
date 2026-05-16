# Review: bounded_log

**Conclusion:** REJECT

## Checklist

1. Spec clauses unchanged: NO — the `ensures` block on `append` has been rewritten on six lines. In the frozen file the clauses referenced `self.X()` in the post-state; in HEAD they reference `final(self).X()`. Specifically the diff shows:
   - `-            self.well_formed(),` → `+            final(self).well_formed(),`
   - `-            self.capacity() == old(self).capacity(),` → `+            final(self).capacity() == old(self).capacity(),`
   - `-                &&& self.view().len() == old(self).view().len() + 1` → `+                &&& final(self).view().len() == old(self).view().len() + 1`
   - `-                &&& self.view()[old(self).view().len() as int] == msg` → `+                &&& final(self).view()[old(self).view().len() as int] == msg`
   - `-                        self.view()[i] == old(self).view()[i]` → `+                        final(self).view()[i] == old(self).view()[i]`
   - `-                &&& self.view() == old(self).view()` → `+                &&& final(self).view() == old(self).view()`
   These hits sit at HEAD lines 67, 68, 70, 71, 74, 78 of `exercises/bounded_log.rs`.
2. Pre-existing spec fn bodies unchanged: YES — `capacity`, `view`, and `well_formed` (lines 22–32 of HEAD) are byte-identical to the frozen file; the diff contains no hunks touching them.
3. No bypass tokens introduced: YES — the diff removed `unimplemented!()` from `new`, `len`, `get`, and `append`. No `assume(`, `#[verifier::external_body]`, `unreachable!()`, or `panic!(` appear in the added lines (HEAD lines 40, 47, 57–61, 81–91).
4. No trivializing requires: YES — `requires` lines in the diff are unchanged (`old(self).well_formed()` on line 65 is identical to the frozen file); no `requires` clauses were added to any function.
5. No closed/open toggles: YES — the three `pub closed spec fn` declarations at HEAD lines 22, 26, 30 retain `closed`; the diff contains no `open`/`closed` keyword changes.

## Justification

Item 1 fails outright. The implementer rewrote six lines inside the `ensures` block of `append`, changing every post-state reference to `self` into `final(self)`. The reviewer rules explicitly state: "Are `requires` and `ensures` clauses byte-identical to `spec-frozen-<exercise>`? ... Any change = REJECT." and "Do not approve based on intent or spirit. The rule list is the rule list." Even granting the implementer's claim (recorded in `AGENTS.md` "Discovered patterns") that `final(self)` is semantically equivalent to the post-state `self` and required by the current Verus version, this is not byte-identical and therefore falls under rule 1. The rest of the diff (bodies of `new`, `len`, `get`, `append`) is clean — no bypass tokens, no `requires` tampering, no closed/open toggles, and the three pre-existing `spec fn` bodies are untouched — so a respin that restores the original `ensures` text (or, if Verus truly refuses to compile it, an architect-driven decision to re-freeze the spec) is the only remaining issue.

## Reviewer notes (optional)

- The `final(self)` migration may be a real compiler constraint, but only the architect (not the implementer or reviewer) is empowered to re-freeze the spec. Flag this for orchestrator attention before re-invoking the implementer — otherwise the loop will keep producing the same REJECT.
- The proof body itself (the two `assert` nudges after `Vec::push`) is exactly the kind of clean frame-property discharge the rules allow; preserve it on the next attempt.
- If a spec re-freeze is granted, the new baseline should be tagged `spec-frozen-bounded_log` (force-moved) and this review re-run against it.
