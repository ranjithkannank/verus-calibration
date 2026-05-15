# Blog outline — two-post split

Continuation of the autonomous-loop trust infrastructure series on ranjithkannan.com.

The arc so far:
- Multi-agent TDD loop — make the loop do TDD
- Mutation testing — *check that the tests actually catch bugs*
- Audit / decision split — *the writer shouldn't grade itself*
- Integration test contracts — *green doesn't mean correct*
- **(next two posts)** — *what if the green check is a proof?*

Each post tightened the feedback signal. The verifier is the limit of that progression: an obligation the loop can't pass except by either (a) weakening the spec or (b) actually being correct. The whole experiment is testing whether (a) can be ruled out by rules alone.

---

## Post A — methodology

**Working title:** *"When the Loop's Green Check Is a Proof"*
**Alt titles:** *"Verus in the Loop: Rules for a Cheating-Free Calibration"*, *"Proof Obligations as a Feedback Signal"*
**Target length:** 1500-2000 words.
**Tone:** explanatory, sets up the experiment without claiming results.

### 1. Opening — the limit of the trust ladder (2-3 paragraphs)
- Recap, in two sentences each, what the previous four posts established. The reader who's been following gets the spine; the new reader gets enough to keep up.
- Pose the question: every feedback signal so far is heuristic. Tests can be wrong. Mutation testing can be gamed by mutation-aware code. The audit/decision split assumes the auditor itself is honest. What if the signal can't be gamed?
- Set up the answer: a formal verifier. Specifically, Verus, because it operates on real Rust.

### 2. Why Verus, not TLA+ or Dafny (1-2 paragraphs)
- TLA+ verifies a model; nothing connects the model to the code. Dafny verifies code in its own language; nothing connects that code to your production stack.
- Verus closes both gaps. The spec lives in the same file as the implementation, both written in Rust syntax, and the verifier checks that the executable code satisfies the spec it sits next to.
- One sentence on the vericoding benchmark — 44% first-try on Verus today — to motivate that this isn't aspirational.

### 3. The cheating problem (2 paragraphs — the load-bearing section)
- A verifier as a feedback signal is only cheat-proof if the loop can't change the question. List the four ways a loop can cheat:
  - Weaken the spec until the empty function passes.
  - Add `#[verifier::external_body]` to skip verification.
  - Add `assume(...)` to grant itself the obligation.
  - Use `unreachable!()` or partial functions to dodge cases.
- Each one is silent: the verifier still returns green. The methodology is exactly the set of rules forbidding these, plus a logging discipline that makes violations visible.
- This is the part to dwell on. It's also the part future readers will steal for their own experiments.

### 4. The three exercises and why each (2-3 paragraphs)
- Binary search — baseline. Standard tutorial difficulty. Calibrates whether the loop can handle Verus at all.
- Bounded log — frame reasoning. The "append doesn't overwrite existing entries" obligation is exactly the kind of property that scales badly. If the loop closes this cleanly, larger projects are plausible.
- Quorum count — concrete-to-abstract. Walking a `Vec` to compute a `Set::len()` is the gap that breaks naive proofs. Also directly relevant to Byzantine agreement, which is the eventual target.

### 5. AGENTS.md and the iteration cap (1 paragraph + code block)
- Paste the relevant parts of `AGENTS.md`. The cap (10/20/20) is calibrated to be tight enough that "the loop converges" is meaningful, loose enough that "the loop converges sometimes" shows up as data.
- Note the per-attempt log format. The logs are the evidence.

### 6. What I'm measuring (1 paragraph)
- The four numbers: first-try success rate, attempts to convergence, tokens per verified function, recurring failure categories.
- Why these four and not others (e.g. proof LOC ratio): these are the numbers that affect the decision about the next project.

### 7. Closing — what next post will say (1 paragraph)
- Next post: the numbers, the table, the taxonomy. State that the numbers are what they are; no preview.
- One sentence on what's at stake — if the results are encouraging, the trust-ladder posts have a credible top rung; if they aren't, the limit of autonomous-loop verification is somewhere short of "real proofs," and that's also worth knowing.

### Links to weave in
- The four previous posts (explicit, not footnoted)
- Verus repo & vericoding benchmark paper
- This repo, public from day one

---

## Post B — results

**Working title:** *"Three Verus Exercises in an Autonomous Loop: What Held and What Didn't"*
**Alt titles:** *"Calibrating Vericoding on Byzantine-Adjacent Primitives"*, *"The Numbers from a Weekend with Verus"*
**Target length:** 2000-2500 words.
**Tone:** empirical. Numbers carry the post; commentary is restrained.

### 1. Opening (1 paragraph)
- Reference the methodology post in one sentence — link, don't recap.
- State the experimental question and the headline result in two sentences. No preamble.

### 2. The results table (table + 1 paragraph)
- Drop the filled-in table from `results_template.md` directly into the post.
- One paragraph reading the table aloud: which exercise verified cleanest, which struggled, where tokens concentrated.

### 3. Failure taxonomy (largest section, 3-5 subsections)
- Each subsection: name the category, give one concrete example with the actual code and the actual verifier output, explain what unblocked it (or didn't).
- The post earns its keep here. Most public writing about LLM-assisted verification stays at the success-rate level; the taxonomy is the part that's actually useful to other people trying the same thing.
- Resist the urge to editorialize. Show the verifier output. Show the prompt. Show the patch.

### 4. The no-weakening rule in practice (1-2 paragraphs)
- Did the rule hold? How many times did the loop attempt a weakening that the logs caught? What does that say about whether the rule is sufficient or whether it needs automated enforcement (e.g. a spec-fingerprint check)?
- This is the methodology contribution. If the rule held with only honor-system enforcement, that's one finding; if it required hand-review of every diff, that's a different finding.

### 5. What this means for the next project (2-3 paragraphs)
- Three-question decision framework from the readme.
- State the decision: pursue the larger Byzantine-agreement project, narrow scope, or pivot.
- Avoid promising the larger project's results — only the decision about whether to attempt it.

### 6. Where this fits in the series (1 paragraph)
- Tie back to the trust-ladder framing. Each prior post closed a hole. The verifier closes a different kind of hole.
- One sentence on what would come next *in principle* (verified concurrency? verified hardware-software boundary?), without committing to writing about it.

### 7. Limitations (1 paragraph, tight)
- Three exercises is not a benchmark.
- Single model, single operator.
- Confound: my prompt-tweaking instinct is mixed in with the loop's capability.

### 8. Reproducibility (1 paragraph)
- Repo link, exact commands, all raw logs in-repo.

### Links to weave in
- Post A (explicitly)
- Verus repo, vericoding paper, any cited Verus Zulip threads
- The Phil Koopman / Marc Brooker community as the audience that'll care most

---

## Voice notes (apply to both posts)

- Titles understated. Never "AI" in the title. Never "revolutionizing."
- Avoid the words "leverage," "unlock," "explore." Direct verbs.
- Code blocks earn their place — show, don't summarize, when the thing is small.
- First sentence of each section should be a claim, not throat-clearing.
- End each post on the actual point, not a "stay tuned" or social-promo paragraph.
- Match the cadence of the existing posts: shortish paragraphs, occasional one-line aphorism, no headers under 50 words of content.

## Distribution

- Post A → Verus Zulip ("running a calibration experiment, methodology here, results next week")
- Post B → same Zulip thread + a single HN Show post + LinkedIn (Koopman, Brooker, AdaCore, Ferrous Systems, Galois engineers)
- Don't cross-post to /r/rust unless results are strongly positive — that audience punishes hedging.

## Drafting plan

- Friday evening: setup done, AGENTS.md frozen
- Saturday: exercises 1-2, log everything
- Sunday morning: exercise 3
- Sunday afternoon: Post B draft directly from the logs; Post A draft separately, since methodology should not be back-fit to results
- Sunday evening: publish Post A only; hold Post B until Tuesday to give the methodology post air

That last point matters. If Post B drops within hours of Post A, the methodology gets ignored and the post becomes "look at my numbers." A 36-48 hour gap gives the methodology a chance to be read and reacted to on its own.
