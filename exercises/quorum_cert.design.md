# Design — `quorum_cert`

Two obligations on a frozen spec:

1. `verify_qc_structure` (exec): return `true` iff `voters_distinct(*qc) && all_voters_in_range(*qc, n) && has_quorum(*qc, n)`.
2. `lemma_qc_has_honest_voter` (proof): pigeonhole over `voters(qc)` vs `byzantine`.

`signature_valid` and `pk_of` are uninterpreted; we never touch them. `all_signatures_valid` is part of `is_valid_qc` (hypothesis to the lemma) but is not in the structural check's ensures, so the exec function never has to prove anything about signatures.

---

## 1. Representation choice

`QuorumCert` is fixed by the spec. The exec function operates on `&QuorumCert`; the proof function on `QuorumCert` by value (ghost).

Inside `verify_qc_structure` the only auxiliary exec state is a bitmap:

- `seen: Vec<bool>` of length `n`, where `seen[k]` means "some `qc.votes[j]` with `j < i` has voter `k`".

This is the same trick that worked in `quorum_count.rs`. Membership becomes O(1), no nested loop. **Do not** use a `Vec<NodeId>` accumulator — distinctness check would be quadratic and force a nested invariant.

No ghost fields. No struct invariant — `QuorumCert` has no `well_formed` and we are not allowed to add one.

For the proof of obligation 1 we need to bridge the spec quantity `voters(qc).len()` to the exec quantity `qc.votes.len()` *under the assumption that voters are distinct*. We do this via a helper lemma on a `Seq<NodeId>` projection of the votes (no new spec functions exposed — the projection lives inside the lemma).

---

## 2. Algorithmic sketch

### Obligation 1 — `verify_qc_structure`

```rust
let mut seen: Vec<bool> = vec![false; n as usize];     // bitmap
let mut i: usize = 0;
while i < qc.votes.len()
    invariant <see §4>
{
    let v_id: NodeId = qc.votes[i].voter;
    if v_id >= n {
        // witness for !all_voters_in_range -> conjunction false
        return false;
    }
    let v: usize = v_id as usize;
    if seen[v] {
        // witness for !voters_distinct -> conjunction false
        return false;
    }
    seen.set(v, true);
    i = i + 1;
}
// Postcondition of the loop: all in range AND pairwise distinct.
// So |voters(qc)| == qc.votes.len()  (helper lemma).
// Then has_quorum  <==>  qc.votes.len() >= byzantine_threshold(n).
let votes_len: u64 = qc.votes.len() as u64;
let threshold:  u64 = 2u64 * (n as u64) / 3u64 + 1u64;
votes_len >= threshold
```

### Obligation 2 — `lemma_qc_has_honest_voter`

```text
universe U = { k : NodeId | (k as int) < (n as int) }
|U| = n  (recursive lemma, identical to quorum_count's lemma_range_nodeid_len)

voters(qc) ⊆ U   (by all_voters_in_range)
voters(qc).finite()  (subset of finite U)
voters(qc).len() >= 2n/3 + 1   (has_quorum)

Suppose negation: ∀ h ∈ voters(qc) . h ∈ byzantine.
Then voters(qc) ⊆ byzantine.
By vstd::set_lib::lemma_len_subset:
     voters(qc).len() <= byzantine.len().
But 3 * byzantine.len() < n  and  voters(qc).len() >= 2n/3 + 1
  ⇒ 3 * voters(qc).len() >= 2n + 1 > n > 3 * byzantine.len()
  ⇒ voters(qc).len() > byzantine.len().
Contradiction.
```

---

## 3. Key invariants

There is no struct invariant — `QuorumCert` is plain data. All invariants are local to the loop in `verify_qc_structure`. The function-level "invariants" are the frozen `requires`/`ensures`.

---

## 4. Loop invariant sketch (verify_qc_structure)

Let `pref(j) == qc.votes@.subrange(0, j as int)`. Conjuncts (English then Verus):

```rust
while i < qc.votes.len()
    invariant
        // (a) cursor / bitmap bounds
        0 <= i <= qc.votes@.len(),
        seen@.len() == n as nat,
        n > 0,

        // (b) all voters in prefix are in [0, n)
        forall|j: int| 0 <= j < i as int ==>
            (qc.votes@[j].voter as int) < n as int,

        // (c) all voters in prefix are pairwise distinct
        forall|j: int, k: int| 0 <= j < k < i as int ==>
            qc.votes@[j].voter != qc.votes@[k].voter,

        // (d) bitmap abstraction:  seen[m]  iff  m appears as a voter in prefix
        forall|m: int| 0 <= m < n as int ==>
            (seen@[m] == (exists|j: int|
                0 <= j < i as int && qc.votes@[j].voter as int == m)),
    decreases qc.votes@.len() - i,
```

Initial state (`i == 0`): (b)/(c) vacuous; (d) — the inner `exists` is false for every `m`, and every `seen@[m]` is `false` (from `vec![false; n as usize]`). ✓

After the loop (`i == qc.votes@.len()`): (b) is exactly `all_voters_in_range(*qc, n)`; (c) is exactly `voters_distinct(*qc)`. So we exit the loop with both halves of the spec predicates established.

### What each branch must re-establish

`if v_id >= n` — early return, nothing to re-establish; the invariant is already broken-by-design (this very index would violate (b) if we continued). The return-`false` path uses the fact that index `i` is a counter-example to `all_voters_in_range`.

`if seen[v]` — early return; (d) at `m = v_id` gives `exists j < i. qc.votes@[j].voter == v_id`, and we *also* have `qc.votes@[i].voter == v_id`, so `j` and `i` are a counter-example to `voters_distinct`. Pick the witness with `let ghost j = choose|j| ...;`.

Fall-through (`v_id < n`, `seen[v] == false`, then `seen.set(v, true); i += 1;`):
- (a) and the length part of (d) are trivially preserved (Vec::set doesn't change length).
- (b) — old prefix + new index that we just checked.
- (c) — old prefix was distinct; new element `v_id` is different from all earlier voters because none of them set the bit at position `v_id` (this is (d) read backwards).
- (d) at `m = v_id as int`: now `true` on both sides (we set the bit; the prefix now contains `v_id` at index `i`).
- (d) at `m != v_id as int`: `seen@[m]` unchanged (Vec::set frame); the new prefix contains a voter `== m` iff the old prefix did (the only new voter is `v_id != m`).

---

## 5. Predicted helper lemmas

### L1. Universe size — copy from `quorum_count`

```rust
proof fn lemma_range_nodeid_len(n: u32)
    ensures
        Set::<NodeId>::new(|k: NodeId| (k as int) < n as int).finite(),
        Set::<NodeId>::new(|k: NodeId| (k as int) < n as int).len() == n as nat,
    decreases n,
```

Body: identical to the version in `exercises/quorum_count.rs` lines 125–147. Lift it verbatim.

### L2. Distinct seq has set-len equal to seq-len

```rust
proof fn lemma_distinct_seq_to_set_len(s: Seq<NodeId>)
    requires forall|i: int, j: int| 0 <= i < j < s.len() ==> s[i] != s[j],
    ensures
        s.to_set().finite(),
        s.to_set().len() == s.len(),
    decreases s.len(),
```

Induction on `s.len()`. Inductive step: `s == prefix.push(last)`. Use `lemma_push_to_set` (predicted below or lifted from `quorum_count`) plus `axiom_set_insert_len`, noting `last` is not in `prefix.to_set()` by the distinctness hypothesis.

### L3. `push` lifts to `to_set` insert — copy from `quorum_count`

```rust
proof fn lemma_push_to_set(s: Seq<NodeId>, x: NodeId)
    ensures s.push(x).to_set() =~= s.to_set().insert(x),
```

Body: identical to `exercises/quorum_count.rs` lines 48–96. Lift verbatim.

### L4. Voter set equals the to-set of the voter projection

```rust
proof fn lemma_voters_as_to_set(qc: QuorumCert)
    ensures
        voters(qc) =~= voter_seq(qc).to_set(),
```

where `voter_seq` is an *internal* spec fn (not in the frozen file — only used inside this lemma's body and immediate callers):

```rust
spec fn voter_seq(qc: QuorumCert) -> Seq<NodeId> {
    Seq::new(qc.votes@.len(), |i: int| qc.votes@[i].voter)
}
```

Proof: `assert forall|n: NodeId| voters(qc).contains(n) <==> voter_seq(qc).to_set().contains(n) by { ... };` — both unfold to `exists|i: int| 0 <= i < qc.votes@.len() && qc.votes@[i].voter == n` (the second via the `Seq::to_set` membership axiom). This is the `=~=` extensional-equality pattern.

### L5. Bridge: distinct voters ⇒ `|voters(qc)| == qc.votes.len()`

```rust
proof fn lemma_distinct_voters_len(qc: QuorumCert)
    requires voters_distinct(qc),
    ensures
        voters(qc).finite(),
        voters(qc).len() == qc.votes@.len(),
```

Body: chain L4 + L2.
```rust
lemma_voters_as_to_set(qc);
// voters(qc) =~= voter_seq(qc).to_set()
assert(forall|i: int, j: int| 0 <= i < j < voter_seq(qc).len()
    ==> voter_seq(qc)[i] != voter_seq(qc)[j]);   // from voters_distinct
lemma_distinct_seq_to_set_len(voter_seq(qc));
// voter_seq(qc).len() == qc.votes@.len() by definition
```

### Optional sub-lemma for L2 induction

If `lemma_distinct_seq_to_set_len` proves awkward, factor a one-step inductive lemma:

```rust
proof fn lemma_distinct_seq_to_set_len_step(s: Seq<NodeId>, x: NodeId)
    requires
        forall|i: int| 0 <= i < s.len() ==> s[i] != x,
        s.to_set().finite(),
        s.to_set().len() == s.len(),
    ensures
        s.push(x).to_set().len() == s.len() + 1,
```

But the recursive form is usually fine.

**Implementation discipline:** do not pre-write all five lemmas. Write the loop and the lemma body for obligation 2 first; introduce L1–L5 only when an assert fails because the solver can't see one of these facts.

---

## 6. SMT trouble spots

1. **`pref(i+1) =~= pref(i).push(qc.votes@[i])`.** Subrange equalities never close on their own. State extensional equality explicitly inside the fall-through branch *before* the index increment, then re-state it after, so both forms are in scope. Pattern from `quorum_count`:
   ```rust
   let ghost pref_old = qc.votes@.subrange(0, i as int);
   // ... mutation ...
   let ghost pref_new = qc.votes@.subrange(0, i as int);
   assert(pref_new =~= pref_old.push(qc.votes@[(i as int) - 1]));
   ```

2. **Bitmap frame after `seen.set(v, true)`.** vstd's `Vec::set` gives the length-preserving + pointwise-update spec; the typical nudge:
   ```rust
   assert(seen@.len() == old_seen_len);
   assert(seen@[v as int] == true);
   assert(forall|k: int| 0 <= k < seen@.len() && k != v as int
       ==> seen@[k] == old_seen@[k]);
   ```
   The third line is the load-bearing one for re-establishing (d) at `m != v_id`.

3. **Witness extraction in the duplicate-found branch.** From (d) with `m = v_id as int`:
   ```rust
   let ghost j = choose|j: int|
       0 <= j < i as int && qc.votes@[j].voter as int == v_id as int;
   ```
   Then `qc.votes@[j].voter == v_id` (cast direction may need an extra `assert((qc.votes@[j].voter as int) == (v_id as int) ==> qc.votes@[j].voter == v_id);`).

4. **`forall ... by { assert(invariant); }` nudges for re-establishing (d).** When proving the new (d) at index `m != v_id`, wrap the membership-preservation step inside `assert forall|m: int| ... by { ... }` and inside that block, use `choose` to extract the index witness from the old `exists`. This is the canonical pattern from `quorum_count` (Case B re-establishment).

5. **Threshold arithmetic.** Same pattern as `quorum_count`:
   ```rust
   let threshold: u64 = 2u64 * (n as u64) / 3u64 + 1u64;
   assert(threshold as nat == byzantine_threshold(n));
   ```
   If it fails, decompose into `assert((2u64 * (n as u64)) as nat == 2 * (n as nat));` and the division step. `n: u32` ⇒ `2 * n` fits in `u64`.

6. **`qc.votes.len() as u64` vs `qc.votes@.len() as nat`.** Cast chain `usize -> u64 -> nat`. Verus knows `usize` fits in `u64` and the chain commutes through nat. If the final comparison fails, add:
   ```rust
   assert(votes_len as nat == qc.votes@.len());
   ```

7. **`!all_voters_in_range` / `!voters_distinct` at early returns.** The witness is the current index `i`. State both the local fact and the negated predicate:
   ```rust
   assert((qc.votes@[i as int].voter as int) >= n as int);   // not in range
   // ⇒ !all_voters_in_range(*qc, n), so the conjunction is false
   ```
   The solver should chain these automatically once you instantiate the universal at `i`.

8. **Obligation 2: contradiction via subset.** The reliable pattern (mirrors the playbook's pigeonhole entry):
   ```rust
   if !(exists|honest: NodeId|
            voters(qc).contains(honest) && !byzantine.contains(honest)) {
       assert(voters(qc).subset_of(byzantine)) by {
           assert forall|h: NodeId| voters(qc).contains(h)
                  implies byzantine.contains(h) by { };
       };
       // need voters(qc).finite() for some axioms; come from subset of universe
       let universe = Set::<NodeId>::new(|k: NodeId| (k as int) < n as int);
       lemma_range_nodeid_len(n);
       assert(voters(qc).subset_of(universe)) by {
           assert forall|h: NodeId| voters(qc).contains(h)
                  implies universe.contains(h) by { };
       };
       vstd::set_lib::lemma_len_subset::<NodeId>(voters(qc), universe);   // finiteness
       vstd::set_lib::lemma_len_subset::<NodeId>(voters(qc), byzantine);
       // Now voters(qc).len() <= byzantine.len(), but >= 2n/3 + 1, contradiction.
       assert(false);
   }
   ```

9. **Nat division arithmetic for the contradiction.** The solver may not see `3 * (2n/3 + 1) > n` instantly. The robust form:
   ```rust
   // From has_quorum:
   //     voters(qc).len() >= 2 * (n as nat) / 3 + 1
   // Multiply by 3:
   assert(3 * voters(qc).len() >= 3 * ((2 * (n as nat)) / 3) + 3);
   // 3 * (x/3) >= x - 2 for nat x
   assert(3 * ((2 * (n as nat)) / 3) >= 2 * (n as nat) - 2)
       by (nonlinear_arith) requires n as nat >= 1;     // or just by default
   // ⇒ 3 * voters(qc).len() >= 2*n + 1 > n
   assert(3 * voters(qc).len() > n as nat);
   // Combined with byzantine.len() * 3 < n and voters(qc).len() <= byzantine.len():
   assert(3 * voters(qc).len() <= 3 * byzantine.len());
   assert(3 * voters(qc).len() < n as nat);
   // contradiction
   ```
   Try without `by (nonlinear_arith)` first; Verus' default `int` arithmetic often handles it. If it doesn't, the `nonlinear_arith` hint or splitting into two `assert`s usually closes it.

10. **`voters(qc).finite()` for `lemma_len_subset`.** The version of `lemma_len_subset` we're calling requires the *superset* to be finite. `byzantine.finite()` is given. For the `voters(qc) ⊆ universe` direction, `universe` finite via `lemma_range_nodeid_len`.

---

## 7. Suggested order of operations

1. **Obligation 1, skeleton.** Write the loop with bitmap, early returns, and threshold compare, **no invariants yet**. Confirm it compiles (verification will fail; that's expected).

2. **Obligation 1, invariant (a)+(b).** Add cursor bounds, length of `seen`, and the in-range conjunct. Verus should now accept indexing `seen[v]` (since `v < n` after the early return on `v_id >= n`).

3. **Obligation 1, invariant (d).** Add the bitmap abstraction. Re-establish in the fall-through branch with the `seen.set` frame asserts (trouble spot #2) plus `lemma_prefix_extend`-style extensional equality (trouble spot #1).

4. **Obligation 1, invariant (c).** Add pairwise-distinctness. The fall-through branch needs the fact "v_id is not yet in the prefix" — read this off (d) at `m = v_id as int`.

5. **Obligation 1, early-return correctness.** For each `return false`, add asserts that show the conjunction is false: index `i` is a witness for either `!all_voters_in_range` or `!voters_distinct`.

6. **Obligation 1, threshold step.** Lift `lemma_range_nodeid_len`, `lemma_push_to_set` from `quorum_count` (verbatim). Write `lemma_distinct_seq_to_set_len`, `lemma_voters_as_to_set`, `lemma_distinct_voters_len`. Call `lemma_distinct_voters_len(*qc)` after the loop. Assert `threshold as nat == byzantine_threshold(n)`. Return `votes_len >= threshold`.

7. **Obligation 2, skeleton.** Open with the universe construction + `lemma_range_nodeid_len(n)`. Show `voters(qc).subset_of(universe)` and conclude `voters(qc).finite()` via `lemma_len_subset`.

8. **Obligation 2, contradiction block.** Add the `if !exists ... { ... assert(false); }` from trouble spot #8. Inside the block, drive the arithmetic contradiction from trouble spot #9.

9. **Cleanup.** Strip asserts that aren't load-bearing. Final body expected: ~80–120 lines for `verify_qc_structure` including invariants, ~30–50 lines for `lemma_qc_has_honest_voter`, ~60–100 lines for helper lemmas total (most of which is lift-from-`quorum_count`).

### What NOT to do

- **Do not** add a `well_formed` predicate on `QuorumCert`. It isn't there in the frozen spec, and adding one would change the meaning of `verify_qc_structure`'s ensures (which references the raw `*qc`).
- **Do not** try to prove anything about `signature_valid` or `pk_of`. The structural check's ensures does not mention `all_signatures_valid`; the safety lemma takes `is_valid_qc` as a *hypothesis* and never reasons inside the predicate.
- **Do not** rewrite `byzantine_threshold` arithmetically (e.g., replace `2n/3 + 1` with `(2n + 3) / 3`). The spec uses one form; keep the assert chain matching that form.
- **Do not** introduce a `Vec<NodeId>` accumulator for distinctness — the bitmap is strictly better and `quorum_count` already proved out the pattern.

---

## Summary: Bitmap-backed single-pass `verify_qc_structure` with a four-conjunct loop invariant (cursor bounds, in-range prefix, distinct prefix, bitmap-vs-prefix abstraction), plus a contradiction-via-subset proof for `lemma_qc_has_honest_voter`, riding on five helper lemmas of which three are lifted verbatim from `quorum_count`.
