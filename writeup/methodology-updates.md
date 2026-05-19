# Source material — revisions to the May 10 post

This file is raw input for the blog-writing agent. Goal: refresh the
existing post at <https://ranjithkannan.com/2026/05/10/verus-calibration-formal-verifier-loop/>
with the methodology changes made since publication. The post is
still in draft, so direct edits are appropriate.

Editorial decision 2026-05-18: this is the single revision source
for May 10. Earlier this week the find-fix-audit arc (witness-read
leak in the original implementer whitelist → hardening +
empirical probe → clean re-test on `swap_multiset` → audit re-runs
of the two prior discovery exercises) was a candidate for its own
short methodology vignette in the style of "Splitting Audit from
Decision." Decision: keep it as a section inside the May 10
revision instead. May 10 is already the methodology meta-post and
its "What the loop got wrong" / "Methodology contributions"
sections are the right shape for this content; promoting it to its
own post fragments a story that lives naturally inside the
methodology baseline.

Tone target: same as the existing post (technical-practical, "we"
voice, minimal em-dashes, no folksy aphorisms, no time-of-day
framing, no cost numbers). Every claim below is backed by a commit
in <https://github.com/ranjithkannank/verus-calibration>.

## What is and isn't changing

**Staying.** The trust-ladder framing, the three-role architecture
overview, the two-sandbox-layer description, the calibration
exercises, the "what's different from existing work" section, the
reproducibility links.

**Updating.** The model used by the implementer. The
methodology-contributions section adds new items (per-iteration
scoping, architect sub-task lists, pre-spec witness verification,
deliberate discovery tests, witness-deny whitelist hardening
empirically verified by a probe). The limitations section has two
items that are no longer limitations (they were fixed) plus a new
one (the original implementer whitelist allowed witness reads —
fixed but worth flagging honestly). The "what we'd do next"
section is reworked: items already done move out; the BFT
direction is now an active artifact, not a hypothetical.

**Adding.** A short paragraph in the architecture section about
prompt-level scoping per iteration. New rows in the results table
covering all post-calibration exercises through `swap_multiset`,
including the `vec_swap` / `vec_swap_v2` INVALIDATED rows kept as
honest history.

**NOT covered by this file.** The composition exercises
(`sensor_poll`, `sensor_poll_signed`, `sensor_poll_honest`) and
the discovery-test audit results for `sensor_poll_honest` and
`counter_filler` belong in the May 17 sensor-fusion post revision,
not in May 10. See `composition-post.md` for that material. The
May 10 revision *mentions* those exercises in the results table
and links forward to the revised May 17 post, but does not
describe them in detail.

## Section-by-section delta

### Architecture (section 2 of the existing post)

The model table currently shows:

| Role        | Model              | ... |
| Architect   | claude-opus-4-7    | ... |
| Implementer | claude-sonnet-4-6  | ... |
| Reviewer    | claude-opus-4-7    | ... |

Change the implementer row's model to `claude-opus-4-7`. Add a
sentence after the table:

> The implementer ran on Sonnet 4.6 for the original three
> exercises and handled them cleanly. For BFT-path exercises starting
> with quorum_cert the implementer was switched to Opus 4.7. Those
> proof obligations involve genuine cardinality and pigeonhole
> reasoning where the model needs to plan over many tokens of
> internal thinking before producing structured output. Aligning the
> hardest role to the strongest model, even when that role is the
> most expensive.

The state-machine diagram is unchanged.

The "boring on purpose" `claude -p` invocation snippet is unchanged.
The `--` separator note is still useful.

After the per-attempt-commits paragraph, add:

> Each implementer attempt is also scoped narrowly. The prompt
> directs the implementer to either pick the next unfinished
> sub-task from the architect's design or scope its edits to the
> specific failing function from the latest verifier output. Small,
> surgical edits per iteration, not file-wide rewrites. The
> orchestrator iterates; the implementer does not need to fix
> everything in one call.

### What happened (section 4)

The results-at-a-glance table needs further rows. Final shape:

| Exercise              | Status | Attempts | Notable                                      |
|-----------------------|--------|----------|----------------------------------------------|
| binary_search         | DONE   | 1        | (unchanged)                                  |
| bounded_log           | DONE   | 1 (post re-freeze) | (unchanged)                        |
| quorum_count          | DONE   | 2        | (unchanged)                                  |
| quorum_cert           | DONE   | 6        | First BFT-shaped exercise. Six narrow iterations through the architect's sub-task list. Pigeonhole-via-contradiction proof of a quorum certificate's honest-voter guarantee. |
| ft_midpoint           | DONE   | 7        | First sensor-fusion exercise. Inclusion-exclusion + argmax-over-correct, both surfaced over multiple iterations. |
| marzullo              | DONE   | 1 (post re-freeze) | Interval variant of ft_midpoint. Operator re-froze with Helly-1D precondition that the agent surfaced via constructive counterexample. |
| cross_module_counter  | DONE   | 1        | First multi-module exercise. Nested `mod` blocks inside one `verus!{}`. |
| counter_multifile     | DONE   | 1        | First multi-file exercise. Same algorithm, sibling-file layout. Tooling test. |
| counter_producer      | DONE   | 1        | First cross-module composition. Producer's loop invariant carries facts the counter doesn't expose. |
| sensor_poll           | DONE   | 1        | First composition-of-primitives exercise. Projection lemma bridges marzullo's frame to the caller's frame. |
| sensor_poll_signed    | DONE   | 1        | Cryptographic trust boundary threaded purely at the spec layer. |
| sensor_poll_honest    | DONE   | 1 (audit-confirmed) | First deliberate discovery test. Re-run under hardened whitelist with prior playbook entry stripped: 1 attempt again. |
| counter_filler        | DONE   | 1 (audit-confirmed) | Second discovery test, different proof family. Re-run under hardened whitelist: 1 attempt again. |
| vec_swap              | INVALIDATED | 1   | First attempted invention test; agent read the witness under the permissive whitelist and ported the proof. Kept on disk as evidence. |
| vec_swap_v2           | INVALIDATED | 1   | Second attempt; operator's `cp vec_swap.rs vec_swap_v2.rs` copied the agent's already-filled body as the scaffold. Agent honestly flagged "no edits made". Kept on disk as evidence. |
| swap_multiset         | DONE   | 1        | First clean invention test (proof family the playbook did not document). Hand-typed scaffold, hardened whitelist. Proof structurally different from the operator-authored witness. |

Per-exercise narratives for the post-calibration entries live in the
separate writeup drafts: `quorum-cert-post.md`, `sensor-fusion-post.md`,
`multi-module-post.md`, and `composition-post.md`. For this update,
the existing post should either summarise each in two sentences and
link to the follow-up post, or add a §4.N mirroring the §4.1–§4.3
structure. Author's call.

### Methodology contributions (section 5)

Currently lists three: no-spec-weakening at two layers, per-attempt
commits, model-per-role wiring. Add two more, written in the same
shape as the existing three:

> **Per-iteration scoping in the implementer prompt.** Each
> implementer call is directed at the smallest unfinished sub-task
> from the architect's design, or at the specific failing function
> from the latest verifier output. Without this, an implementer
> attempt could rewrite the whole file and still pass the
> "one-attempt-per-call" rule; with it, attempts are narrow and the
> next iteration picks up where the previous left off. The
> orchestrator does the iterating; the implementer does one thing
> well per call.

> **Architect produces a sub-task list as part of the design.**
> Every design note ends with a numbered list of sub-tasks, ordered
> easiest to hardest, each small enough to land in one
> edit-verus-iterate cycle. The implementer reads this list and
> works through it in order. The list is what makes the per-call
> scoping concrete: "scope to the smallest unfinished sub-task"
> means something specific because the architect listed them.

> **Pre-spec verification via operator-authored witness files.**
> Before tagging `spec-frozen-<name>`, the operator writes a
> reference implementation in `exercises/<name>_witness.rs` carrying
> the same spec block as the exercise file, then runs
> `ralph/check-spec.sh <name>`. If the witness verifies under Verus
> with no cheat tokens, the spec admits a model and the freeze is
> safe. If verus rejects it, the spec is unprovable or the witness
> is wrong, and the operator fixes one before the agent loop ever
> starts. Two of the calibration exercises required the operator to
> re-freeze the spec mid-run. `bounded_log` because the original
> syntax stopped compiling under a newer Verus release;
> `marzullo` because the original spec omitted a precondition (the
> Helly-1D condition that all correct sensors' intervals share at
> least one common point) that the agent took several attempts to
> surface via constructive counterexample. Both held the
> methodology — the agent refused to weaken specs and wrote
> blocked reports cleanly — but both also burned agent cycles on
> bugs that were ultimately in operator-authored input. The
> witness check catches that class at operator time. The empirical
> negative test in
> [`scripts/test-witness-catches-bad-spec.sh`](https://github.com/ranjithkannank/verus-calibration/blob/main/scripts/test-witness-catches-bad-spec.sh)
> strips the Helly-1D precondition from a copy of the marzullo
> witness and confirms verus rejects it. One operator-time Verus
> run would have replaced those agent attempts.

> **A deliberate discovery test on the architect-execution caveat.**
> Every 1-attempt success since `marzullo` carried the same caveat:
> the design note pre-named the load-bearing proof construct (the
> loop invariant, the bridging lemma, the helper-set shape), so the
> agent executed a designed proof rather than discovering one. The
> caveat is worth taking seriously — methodology that only handles
> executing pre-designed proofs is a much narrower claim than
> methodology that supports discovery. Two exercises were set up
> specifically to test the discovery half: `sensor_poll_honest`
> (whose design note states the proof obligation and the informal
> "why" but does not name lemmas, helper-set constructions, or
> trigger annotations) and `counter_filler` (a counter_producer-shaped
> cross-module exercise whose design note states the obligation only
> and warns against copying the parent exercise's loop invariant).
> Both verified in one attempt. The first produced a new helper
> lemma `lemma_honest_supporter_exists` using inclusion-exclusion
> via `lemma_set_intersect_union_lens` against a universe-finite
> bridge via `lemma_int_range` and `lemma_len_subset`, then
> `axiom_is_empty_len0` / `axiom_is_empty` to extract the witness.
> The second produced a 4-conjunct target-bounded invariant in
> place of the 6-conjunct counter-bounded form documented in the
> playbook. In both cases, the agent recognised a proof family
> the playbook already documented (from `ft_midpoint` and
> `counter_producer` respectively) and adapted it to a new
> obligation. Two data points, two distinct proof families.
> Both subsequently audited (see the next bullet); the 1-attempt
> claim survived under the hardened whitelist and with each
> exercise's own prior playbook summary stripped from AGENTS.md.

> **Witness-access hardening, an empirical probe, and an audit.**
> The discovery-test framing above depends on the agent not
> reading the operator-authored witness file (`exercises/<name>_witness.rs`,
> the reference implementation used by `ralph/check-spec.sh` to
> confirm the spec admits a model). The original implementer
> tool whitelist granted generic `Read` and `Glob` with no path
> qualifier, so the witness file was readable. A first attempt at
> an invention test on a single-function exercise (`vec_swap`,
> with a multiset-preservation postcondition the playbook did
> not anchor) made this concrete: the agent's iter-1 commit
> message read "port witness proof", and its self-authored
> playbook entry recorded "the architect-owned witness file was
> visible to the implementer and gave the proof skeleton verbatim.
> For pure invention measurement, future exercises should hide
> the witness." That entry — written by the agent, against its
> own run — is the cleanest possible flagging of the gap. The
> `DISALLOWED_TOOLS` list in `ralph/run-exercise.sh` was then
> extended with `Read(**/*_witness*)`, `Read(**/*_witness/**)`,
> `Glob` / `Grep` equivalents, and `Bash(cat|grep|rg|head|tail|ls *_witness*)`
> patterns. An empirical probe (`scripts/probe-witness-deny.sh`)
> invokes `claude -p` with the same disallowed-tools array and
> asks the agent to read a witness file via six different paths;
> each is blocked with an explicit permission denial, agent's
> own verdict: "I was unable to read any of the witness file's
> contents through any of the attempted methods." With the
> hardened whitelist in place, two further runs followed: a
> fresh-named invention test (`swap_multiset`, same spec as
> `vec_swap`, hand-typed scaffold) verified in one attempt with
> a proof structurally different from the witness's; and an
> audit re-run of both prior discovery tests
> (`sensor_poll_honest` and `counter_filler`) reset to spec-frozen
> scaffold and with each exercise's own prior playbook entry
> stripped from `AGENTS.md`. Both re-audits verified in one
> attempt, with solutions structurally identical to the originals.
> The discovery and invention claims hold under audit.

> **An external-validity probe: the harness travels on a
> third-party benchmark; the methodology claim does not yet.**
> *Honest scope, leading:* this section reports n=2, and both tasks
> shipped with operator-authored design notes that named the
> relevant Verus tooling family before the implementer ran. The
> result therefore says the **harness** accepts externally-supplied
> specs cleanly; it does not yet say the **methodology** travels
> without operator hand-holding. That is a separate claim and is
> the next deliberate experiment, not a settled finding.
>
> Two tasks were drawn from Microsoft's
> [VeruSAGE-Bench](https://github.com/microsoft/verus-proof-synthesis)
> (849-task benchmark across distributed systems, OS kernels,
> storage, and verified utility libraries). Selected from the
> smallest end of the size distribution to test that the harness
> accepts the upstream task shape at all before investing in
> larger runs. `MA__bin_sizes__mul_assoc` is a pure `proof fn`
> arithmetic identity over `nat`: `(x * y) * z == y * (x * z)`.
> `VE__utils__init_vec_u8` is an exec function from the vest
> verified-serializer's utilities — a counted `while` loop that
> fills a `Vec<u8>` with zeros, shipped without the loop invariant
> or `decreases` clause Verus needs to accept it. Both verified in
> one implementer attempt and the reviewer APPROVED on first read.
> The first closed with a single
> `assert((x * y) * z == y * (x * z)) by (nonlinear_arith);`. The
> second added a two-conjunct invariant (`i <= n`, `ret@.len() == i`)
> and `decreases n - i`; the implementer's solution is strictly
> smaller than the upstream ground-truth witness, which carries an
> additional `assert(ret@[i as int] == 0);` that turns out to be
> unnecessary when the postcondition only constrains length.
>
> The harness adaptation surface was almost nothing. Two iteration-cap
> entries in `ralph/run-exercise.sh`'s per-exercise `case` block and
> a new section in AGENTS.md naming the external-validity track as
> separate from the numbered internal-exercise sequence. The
> pre-commit hook's path whitelist, spec-line preservation, and
> cheat-token detection all worked unmodified on the upstream task
> shape (which carries `fn main() {}` and one `verus!{}` wrapper,
> different from our internal exercises). The witness-deny ACL
> matches `*_witness*` regardless of name shape, so
> `exercises/MA__bin_sizes__mul_assoc_witness.rs` was denied by the
> same patterns that protect `marzullo_witness.rs`. The upstream
> `microsoft/verus-proof-synthesis` repository is cloned outside
> the verus-calibration working tree, so the agent cannot reach the
> ground-truth solutions that ship alongside the unverified task
> files; the path scoping is structural rather than ACL-based.
>
> Direct comparison to AutoVerus and VeruSAGE on these specific
> tasks is not yet possible: the upstream README states the
> per-task leaderboard is "TO COME". The AutoVerus and VeruSAGE
> papers report aggregate pass rates across the full benchmarks
> (Verus-Bench: 150 algorithmic tasks; VeruSAGE-Bench: 849
> repository-level tasks); n=2 is too small to set against either
> aggregate. The honest aggregate-vs-aggregate sentence is "this
> harness verified the first two tasks it was given from
> VeruSAGE-Bench; both single-attempt; more tasks are needed before
> 'comparable to AutoVerus / VeruSAGE' is a defensible phrasing."
>
> Implication for the methodology claim. The "the methodology
> works on tasks we did not design" framing remains a deliberate
> next experiment. The way to earn it is a run of five to ten more
> tasks with deliberately neutral design notes — no tooling-family
> hints, no proof-structure suggestions, just the obligation and
> the upstream context — and report whatever happens. Negative
> results from that run are more informative than positive ones,
> because they tell us where the methodology actually stops
> traveling.

> **External-validity probe, batch 2: methodology probe with
> neutral design notes (n=8, 6 verified, 2 distinct blocking
> findings).** *Honest scope, leading:* this batch tested the
> methodology claim that the prior n=2 batch could not, because
> the prior design notes named the Verus tooling family before
> the implementer ran. Batch 2's design notes state the obligation
> and a standard sub-task ordering only — no tooling-family names,
> no lemma names, no proof-structure suggestions, no
> cross-references to internal exercises by name. Of the eight
> tasks attempted, six verified, one was blocked by a harness gap
> that we then fixed, and one was blocked by an upstream Verus
> version mismatch that is out of scope to fix tonight. The
> methodology did travel on the six that ran.
>
> Tasks were drawn from `tasks-sampled-100/` across six different
> source projects: `AL` (anvil-library), `IR` (ironkv), `MA`
> (memory-allocator, a second task in a different proof family
> from batch 1's `mul_assoc`), `NR` (nrkernel, two tasks), `OS`
> (atmosphere, an OS kernel), and `NO` (node-replication). Sizes
> ranged from 171 B to 3768 B. The mix is intentional — covers
> pure ghost proof fns over data structures, arithmetic chains,
> exec functions with loops, and one stress test on size.
>
> Attempts-to-verify across the six that completed: five
> single-attempt successes (NR `set_of_first_n_nat_is_finite`,
> AL `submap_of_a_finite_map_is_finite`, MA `shift_is_div`, IR
> `lemma_if_everything_in_seq_satisfies_filter_then_filter_is_identity`,
> NR `lemma_maxphyaddr_facts`, NO `get_fresh_nat_not_in`) and one
> two-attempt success on IR `singleton_seq_to_set_is_singleton_set`.
> The IR singleton's attempt 1 tried the obvious `=~=` extensional
> equality, watched verus reject it ("`=~=` alone doesn't trigger
> the `to_set` axiom needed to expand `seq![x].to_set().contains(y)`"
> — agent's own notes), then attempt 2 closed via vstd's
> `lemma_push_to_set_commute` plus two extensional-equality
> bridges. That two-attempt arc — agent reads its failure, names
> the missing axiom, finds the vstd lemma family that supplies it,
> applies it — is the load-bearing data point on the methodology
> travelling without operator hand-holding.
>
> Two tasks were blocked, with distinct findings.
>
> The first, `NR__definitions_u__lemma_maxphyaddr_facts`, was
> initially blocked by a gap in our own pre-commit hook. The
> upstream task ships with `#[verifier(external_body)]` on an
> opaque-constant declaration, which is the standard Verus pattern
> for declaring a trust-boundary value constrained by a separate
> `pub axiom fn`. Our hook's cheat-token detector treated every
> line of the brand-new scaffold file as "added" (because there
> was no prior baseline to diff against) and rejected the
> `external_body` marker as agent-introduced cheating. The
> `check-spec.sh` witness check had the symmetric gap, refusing
> witnesses that mirrored the exercise file's baseline
> `external_body`. Both layers were fixed in a single commit: the
> pre-commit hook now diffs against the spec-frozen tag when it
> exists and treats the scaffold commit as a baseline event; the
> witness check now counts cheat-token occurrences in the witness
> minus the count in the paired exercise file, flagging only
> excess. Both fixes preserve the original boundary intent —
> agents still cannot introduce new cheat tokens — and add a
> capability the harness needed to accept external tasks that
> legitimately carry baseline `external_body` in their scaffold.
> The task ran cleanly under the fix and verified in one attempt.
>
> The second, `OS__array__impl4__init2none`, was blocked by an
> upstream/downstream Verus version mismatch and stays blocked.
> The task's `ensures` clauses use the pre-`final(self)` syntax
> for `&mut self` postconditions (`self.seq@ ...`,
> `self@[index].is_None()`), which the current Verus build in
> this repo rejects. This is the same class of finding the
> witness mechanism is designed to catch (AGENTS.md, "Pre-spec
> verification (operator)", bug class 2: spec syntax that no
> longer compiles); the witness for this task fails to verify
> under our Verus, and our pinned Verus is newer than the one
> VeruSAGE-Bench was authored against. Two ways forward (both out
> of scope tonight): pin Verus to VeruSAGE-Bench's authored
> version, or port the spec to current Verus and lose direct
> upstream byte-equivalence. The pinned-Verus path is the right
> scientific move and is queued for the next iteration.
>
> Two findings of qualitatively different shape from one batch
> are useful — the first surfaced a real harness adaptation need
> that the prior batch did not stress; the second confirms that
> the witness-based pre-spec check generalises to externally
> authored specs as well as our own. Both are recorded as
> `logs/<task>/blocked.md` (with the first task's original
> blocked marker retained as `blocked.first-attempt.md` after the
> harness fix unblocked it).
>
> Aggregate-level honesty. Batch 2 raises the external-validity
> data point from "first probe with operator hints" (batch 1) to
> "n=6 verified on tasks we did not design, with neutral design
> notes that did not name the Verus tooling family before the
> implementer ran". That is meaningfully stronger evidence that
> the methodology travels on small-to-medium-sized proof-fn and
> exec tasks. It is not yet evidence that the methodology
> travels at the scale VeruSAGE-Bench measures (849 tasks
> including 7k+ B and 24k+ B examples that we deliberately
> deferred). The natural next batch is a larger size tier: pick
> 5–10 tasks in the 2–10 KB range, mostly from `AC` (anvil
> controller, temporal logic) and the larger `ST` (storage) /
> `OS` files, and report whatever happens.

> **Harness boundary refinement: cheat-token policy now
> distinguishes baseline scaffold from agent edits.** The
> external-validity probe surfaced a real gap in the original
> hook design: it treated every line of a brand-new exercise file
> as "added", which silently equated baseline scaffold content
> with agent introduction. For internally-authored exercises this
> was harmless — none of our scaffolds carried cheat tokens to
> begin with — but for VeruSAGE-Bench tasks that legitimately
> declare opaque constants via `#[verifier(external_body)]`, the
> hook rejected the scaffold commit and made the task
> unrun-able. Two surgical fixes:
>
> 1. `scripts/git-hooks/pre-commit` now diffs against
>    `spec-frozen-<exercise>` when the tag exists, and skips the
>    cheat-token detection entirely on the scaffold commit (no
>    prior baseline to compare against). The next commit and all
>    subsequent ones diff against the tag, so any genuinely new
>    cheat token an agent introduces is still caught.
> 2. `ralph/check-spec.sh` now counts cheat-token occurrences in
>    the witness file and subtracts the count in the paired
>    exercise file. A witness may carry the same baseline tokens
>    as the exercise; only excess occurrences flag as
>    operator-introduced bypass.
>
> Tests for the pair-aware behaviour: the
> `NR__definitions_u__lemma_maxphyaddr_facts` scaffold (which
> failed under the old policy, then ran cleanly under the new
> one) is itself the empirical demonstration; both commits are
> in the public history. The original boundary intent — agents
> cannot introduce cheat tokens — is preserved exactly.

### What the loop got wrong (section 6 / limitations)

Two items in the existing post should be marked as fixed:

> **The hook's spec-preservation check has a known gap.** It looks
> for lines whose first token is `requires` or `ensures`. The body
> content of those clauses, the continuation lines, is not on the
> frozen-line list. [...]

This is now fixed. The hook walks the frozen file by indentation and
extracts the complete clause body, not just keyword lines. Rewrite
this bullet to past-tense and link to the commit:

> **The hook's spec-preservation check had a known gap.** It used
> to look for lines whose first token was `requires` or `ensures`,
> missing the body content underneath. The bounded_log REJECT was
> caught only by the reviewer because the diff touched body lines.
> The hook now walks the frozen file by indentation and extracts the
> complete clause body, not just keyword lines. The fix sits in
> [`scripts/git-hooks/pre-commit`](https://github.com/ranjithkannank/verus-calibration/blob/main/scripts/git-hooks/pre-commit)
> with tests under `scripts/test-hook-spec-preservation.sh`.

Similarly for the signal-aware bullet:

> **The orchestrator treats every non-zero claude exit code as
> verus-failed.** It isn't. [...]

This is also fixed. Rewrite:

> **The orchestrator used to treat every non-zero claude exit code
> as verus-failed.** A rate-limited response, a budget cap firing, a
> network blip, an invocation error — all produced the same exit
> code and the loop would try the next iteration against the same
> transient problem. The first quorum_count run burned 27
> rate-limited iterations before its outer ceiling fired. The
> orchestrator now classifies failures by grepping the iteration log
> against known signatures and exits cleanly on infrastructure
> failures rather than churning. Tests in
> [`ralph/test-classify-failure.sh`](https://github.com/ranjithkannank/verus-calibration/blob/main/ralph/test-classify-failure.sh).

The "three exercises is not a benchmark" bullet remains true and is
now slightly stronger ("four exercises is still not a benchmark").

### What we'd do next (section 9)

Items 1 and 2 ("reproduce and fix the silent wrapper hang" — actually
no, that one we didn't reproduce, leave it) and item 3 ("fix the
hook's spec-preservation gap") need handling.

- Item 1 (wrapper hang) — leave as-is. Not investigated to root
  cause.
- Item 2 (signal-aware orchestrator) — move out, since it's now
  done. Mention in the limitations rewrite above.
- Item 3 (hook gap) — move out, done.
- Item 4 (stream-mode logging) — leave as-is. Not done.
- Item 5 (per-attempt time and token measurement in attempts.md) —
  partly done; the agent self-reports its approach now but not
  costs. Leave.
- Item 6 (fourth exercise needing escalation) — replace with: a
  fourth exercise has now run (quorum_cert), but the architect's
  initial design was correct and no escalation fired. The path that
  needs exercising remains untested.
- Item 7 (no spaces in working directory) — leave as-is. Not done.

Add a new closing item about the BFT direction:

> The next rung — applying the methodology to multi-module Verus
> code with cross-module invariants — has its first artifact now.
> `quorum_cert` is a single-module quorum-certificate library with a
> safety lemma, verified end-to-end through the same loop that
> produced the calibration exercises. The full progression continues
> with a verified Byzantine agreement primitive, then verified
> sensor fusion for safety-critical use, then a hardware-deployed
> demonstration.

And a second new closing item about external validity:

> The methodology was probed against tasks we did not design. The
> first batch (n=2) showed the harness accepts external tasks
> cleanly. The second batch (n=8 attempted, 6 verified with
> deliberately neutral design notes, 1 unblocked by a hook fix, 1
> blocked by upstream Verus version mismatch) shows the methodology
> travels at small-to-medium proof-fn and exec sizes when the
> design note does not name the tooling family. The next batch is
> the size tier above this one: 5–10 tasks in the 2–10 KB range,
> mostly AC (anvil controller, temporal logic) and the larger ST /
> OS files. Run on a pinned Verus matching VeruSAGE-Bench's
> authored version so the `final(self)` migration that blocked OS
> init2none does not block the next pick. The right question is
> "where does the methodology stop travelling at scale", which the
> size tier above this one is the cheapest way to probe.

### Where this fits (section 8)

No structural changes. The Schubert / Huntley / Karpathy / Microsoft
contrast is still accurate. The closing sentence about the next rung
can now be more concrete:

> The next rung is multi-module Verus code with cross-module
> invariants. The first single-module BFT primitive (a verified
> quorum certificate with its safety lemma) is in the repo as
> `quorum_cert`; the multi-module step is the next concrete artifact
> on the path.

### Reproducing (section 10)

Models list needs to show the implementer is now Opus 4.7. Add a
parenthetical:

> Models: `claude-opus-4-7` for all three roles. The implementer was
> originally `claude-sonnet-4-6` and handled the three calibration
> exercises competently; switched for the BFT-path exercises. The
> bash script in `ralph/run-exercise.sh` holds the model choices.

## Commits supporting these claims

For the writing agent's reference. Each refinement maps to a specific
public commit:

- Hook spec-preservation extension: `e30a65d`
- Signal-aware orchestrator: `3575527`
- Architect playbook (initial six patterns): `1ae18a9`
- `quorum_cert` scaffolded: `4c2b880`
- `MODEL_IMPLEMENTER` switched to Opus, prompt scoping, architect Sub-tasks requirement: `bda7dbd`
- `quorum_cert` DONE: `5a2a87d`
- Pre-spec verification tooling (`ralph/check-spec.sh`, marzullo
  witness, empirical negative test): `17a4fdd`
- `sensor_poll` DONE (composition demonstration): `2a2036b`
- `sensor_poll_signed` scaffold + witness (signature trust
  boundary at the spec layer): `dacd129`
- `sensor_poll_signed` DONE: `75e54f0`
- `sensor_poll_honest` scaffold + witness (deliberate discovery
  test, design note omits lemma names): `f85bca5`
- `sensor_poll_honest` agent attempt-1 (introduces
  `lemma_honest_supporter_exists` via inclusion-exclusion
  recognised from the ft_midpoint playbook entry): `bbb8e69`
- `sensor_poll_honest` DONE: `ad91c63`
- `counter_filler` scaffold + witness (second discovery test, target-bounded loop): `d026237`
- `counter_filler` DONE: `f0c9a2b`
- `vec_swap` (invalidated invention test, witness was readable): `aad05c8` (scaffold) and `b7cd862` (DONE). Agent's iter-1 commit `6d7e6e2` is titled "port witness proof".
- Whitelist hardening + `vec_swap_v2` (also invalidated, copy-paste error): `7586365`
- `swap_multiset` (third invention attempt, clean): scaffold `f7f9a3d`, agent's iter-1 `678f267`, DONE `b00039a`
- Empirical witness-deny probe + script: included in `54c42dc`. Run log under `logs/_probe/`.
- `sensor_poll_honest` audit reset: `bc3054e`; audit DONE: `0371dca`
- `counter_filler` audit reset: `4d35f5a`; audit DONE: `7447760`
- Audit results commit (AGENTS.md playbook entries restored with `audit-confirmed 2026-05-18` tag): `cf31fe7`
- VeruSAGE-Bench section added to BACKLOG.md (with the "what would need to be true to take it on" criteria): `ec241e0`
- `MA__bin_sizes__mul_assoc` (first external-validity task, pure proof fn): scaffold `afc44ff`, agent's iter-1 `dcb66b5`, reviewer APPROVE `6b5e5f5`, DONE `2031559`
- `VE__utils__init_vec_u8` (second external-validity task, exec + loop invariant): scaffold `70dd66c`, agent's iter-1 `adab22c`, reviewer APPROVE `8f6f3fa`, DONE `bfe19a6`
- Batch 2 harness setup (8 iteration caps + AGENTS.md neutral-design-notes section): `ded995a`
- `IR__seq_is_unique__singleton_seq_to_set_is_singleton_set` (batch 2, two-attempt success — agent failed shortcut then bridged via vstd's `lemma_push_to_set_commute`): scaffold `e66ebd7`, witness fix `ec6e7e5`, iter-1 attempt `4aef0f0`, iter-2 attempt `5ed2c61`, DONE `e7b8801`
- `NR__extra__lemma_set_of_first_n_nat_is_finite` (batch 2, structural induction): scaffold `67cd690`, iter-1 attempt `73cd4ea`, DONE `0c865d8`
- `AL__a_submap_of_a_finite_map_is_finite` (batch 2, `subset_of` + `lemma_len_subset`; structurally different from upstream ground truth): iter-1 attempt `905c2eb`, DONE `6160a66`
- `MA__bin_sizes__shift_is_div` (batch 2, recursion + bit_vector + pow2 helpers): scaffold `acab088`, iter-1 attempt `5f0f2a8`, DONE `b6f2974`
- `IR__verus_extra__lemma_if_everything_in_seq_satisfies_filter_then_filter_is_identity` (batch 2, induction with `reveal(Seq::filter)`): scaffold `2f3ad26`, iter-1 attempt `da2362f`, DONE `f787d5c`
- `NR__definitions_u__lemma_maxphyaddr_facts` — original blocked-by-hook commit `e2a2cb6`, harness fix `041df77`, re-scaffold `b3b6014`, rename of blocked.md to `blocked.first-attempt.md` `5a972ce`, iter-1 attempt `1b21cfc`, DONE `248a6cc`. The blocked-then-fixed-then-verified arc is itself the empirical demonstration of the harness boundary refinement.
- `OS__array__impl4__init2none` (batch 2 task 7, BLOCKED by Verus version mismatch — upstream task uses pre-`final(self)` postcondition syntax): scaffold + blocked.md committed in `602e6ae`. No agent attempts; the witness mechanism caught the spec-syntax issue before the loop ran, exactly as designed.
- `NO__spec__unbounded_log__get_fresh_nat_not_in` (batch 2 task 8, largest in batch at 3.7 KB, multiple operator-axiom helpers): iter-1 attempt `4cb3405`, DONE `0633908`

All on `main` at <https://github.com/ranjithkannank/verus-calibration>.
