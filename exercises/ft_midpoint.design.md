# Design — `ft_midpoint`

One obligation: an exec function `ft_midpoint(readings, f) -> Reading` whose
result is bracketed by some correct reading on each side. No struct, no
mutation of inputs, no ghost fields. All work lives inside the function
plus a small bag of `spec fn` / `proof fn` helpers.

The trust boundary `correct_at(i)` is uninterpreted. The exec body must
not branch on it; the proof reasons about it through `correct_indices(n)`.

---

## 1. Representation choice

Input is a frozen `&Vec<Reading>` (`Reading = i64`). No auxiliary state,
no ghost fields. The whole proof is set-theoretic; counts are expressed
as `Set<int>` cardinalities so that the pigeonhole step between
`correct_indices` and the "at most v" set is a single inclusion-exclusion
identity, not a bridge between a `Seq` recursion and a `Set::len()`.

Counter for the exec count loops is `u32` (the spec pins
`readings.len() <= u32::MAX`, so a count over indices fits).

---

## 2. Algorithmic sketch

Approach (b) from the spec comment — brute-force O(n²). For each
candidate value `v = readings[j]`, count how many readings are `<= v`
and how many are `>= v`; return the first `v` where both counts are
`>= f + 1`. The pigeonhole `(n − f) + (f + 1) > n` then forces a correct
reading on each side.

```text
for j in 0 .. n:
    v   = readings[j]
    lec = #{ i in 0..n : readings[i] <= v }
    gec = #{ i in 0..n : readings[i] >= v }
    if lec >= f + 1 && gec >= f + 1:
        return v
// unreachable per lemma_exists_midpoint
```

**Why not sort.** A median-via-sort approach would force permutation
reasoning (multiset preservation, mapping sorted positions back to
correctness witnesses). Brute force trades O(n²) runtime for proof
locality: each loop iteration's check is self-contained, and the only
non-trivial proof is one existence lemma that drives the
no-fall-through case.

**Why not a recursive `Seq` count.** A `spec fn` recurrence over the
`Seq` is loop-friendly but forces a separate bridge lemma to convert it
to `Set::len()` for the pigeonhole step. Defining the count *as* a
`Set::len()` of a prefix collapses that bridge.

---

## 3. Spec helpers (proof-only — not in the frozen spec)

```rust
spec fn int_range_to(n: int) -> Set<int> {
    Set::new(|i: int| 0 <= i < n)
}

spec fn le_set(readings: Seq<Reading>, v: Reading) -> Set<int> {
    Set::new(|i: int| 0 <= i < readings.len() && readings[i] <= v)
}

spec fn ge_set(readings: Seq<Reading>, v: Reading) -> Set<int> {
    Set::new(|i: int| 0 <= i < readings.len() && readings[i] >= v)
}

// Prefix variants used by the count loops:
spec fn le_set_upto(readings: Seq<Reading>, v: Reading, m: int) -> Set<int> {
    Set::new(|i: int| 0 <= i < m && i < readings.len() && readings[i] <= v)
}

spec fn ge_set_upto(readings: Seq<Reading>, v: Reading, m: int) -> Set<int> {
    Set::new(|i: int| 0 <= i < m && i < readings.len() && readings[i] >= v)
}
```

`int_range_to` may already exist in `vstd::set_lib` (look for
`set_int_range(0, n)` and its companion `lemma_int_range_finite`). Use
whichever name vstd exposes; otherwise define locally as above. The
implementer should not export any of these — they are local spec
helpers.

---

## 4. Key invariants

No struct. No `well_formed`. All obligations are function-level.
Function-level pre/post are the frozen `requires`/`ensures` and must
not be touched.

---

## 5. Loop invariant sketches

### 5a. `count_le(readings, v) -> (c: u32)` (and the symmetric `count_ge`)

```rust
fn count_le(readings: &Vec<Reading>, v: Reading) -> (c: u32)
    requires readings.len() <= u32::MAX as nat,
    ensures
        c as nat == le_set(readings@, v).len(),
        le_set(readings@, v).finite(),
        c as nat <= readings.len() as nat,
{
    let mut c: u32 = 0;
    let mut i: usize = 0;
    while i < readings.len()
        invariant
            0 <= i <= readings.len(),
            readings.len() <= u32::MAX as nat,
            c as nat == le_set_upto(readings@, v, i as int).len(),
            le_set_upto(readings@, v, i as int).finite(),
            c as nat <= i as nat,
        decreases readings.len() - i,
    { /* body uses lemma_le_set_upto_extend */ }
    // After loop: le_set_upto(_, _, n) =~= le_set(_, _).
    c
}
```

Loop body in English: read `readings[i]`; if `<= v`, increment `c`. The
helper lemma `lemma_le_set_upto_extend(readings@, v, i as int)` rewrites
`le_set_upto(_, v, (i+1))` to either `le_set_upto(_, v, i).insert(i)`
(when `readings[i] <= v`) or `le_set_upto(_, v, i)` (otherwise). Both
finiteness and length follow.

After the loop, assert
`le_set_upto(readings@, v, readings@.len() as int) =~= le_set(readings@, v)`
extensionally. This collapses the prefix-set to the full set, matching
the postcondition.

`count_ge` is the mirror image (swap `<=` for `>=`).

### 5b. Main loop in `ft_midpoint`

```rust
let n: u32 = readings.len() as u32;
let threshold: u32 = f + 1;       // overflow safe: f + 1 <= n <= u32::MAX
let mut j: usize = 0;
while j < n as usize
    invariant
        0 <= j as int <= n as int,
        n as nat == readings.len() as nat,
        threshold as nat == f as nat + 1,
        readings.len() as nat >= 2 * (f as nat) + 1,
        correct_indices(readings.len() as nat).len()
            >= readings.len() as nat - f as nat,
        // no earlier index worked:
        forall|j2: int| 0 <= j2 < j as int ==>
            le_set(readings@, readings@[j2]).len() < f as nat + 1
            || ge_set(readings@, readings@[j2]).len() < f as nat + 1,
    decreases (n as int) - (j as int),
{
    let v: Reading = readings[j];
    let lec: u32 = count_le(readings, v);
    let gec: u32 = count_ge(readings, v);
    if lec >= threshold && gec >= threshold {
        proof {
            // pigeonhole on (correct_indices, le_set)
            lemma_pigeonhole_le(readings@, v, f as nat);
            // pigeonhole on (correct_indices, ge_set)
            lemma_pigeonhole_ge(readings@, v, f as nat);
        }
        return v;
    }
    j = j + 1;
}
// fall-through: invariant says no j < n works
proof {
    lemma_exists_midpoint(readings@, f as nat);
    // The existential conflicts with the loop-exit invariant.
    assert(false);
}
unreachable!()                          // discharged by assert(false)
```

The pre-requisites of the precondition need to be re-carried as
invariants because some Verus versions don't carry function-level
`requires` into the loop (cheap to restate).

---

## 6. Predicted helper lemmas

Eight lemmas. Suggested signatures only — bodies are the implementer's.
Names are non-binding.

### L1. `int_range_to` finiteness and length

```rust
proof fn lemma_int_range_to(n: int)
    requires 0 <= n,
    ensures int_range_to(n).finite(), int_range_to(n).len() == n as nat,
    decreases n,
```

Body by induction on `n`. May already exist in `vstd::set_lib` under
the name `lemma_int_range` or similar — search before writing.

### L2. Subset bound for `le_set` / `ge_set` / `correct_indices`

```rust
proof fn lemma_le_set_in_range(readings: Seq<Reading>, v: Reading)
    ensures
        le_set(readings, v).subset_of(int_range_to(readings.len() as int)),
        le_set(readings, v).finite(),
        le_set(readings, v).len() <= readings.len() as nat,
```

```rust
proof fn lemma_ge_set_in_range(readings: Seq<Reading>, v: Reading)
    /* symmetric */
```

```rust
proof fn lemma_correct_indices_in_range(n: nat)
    ensures
        correct_indices(n).subset_of(int_range_to(n as int)),
        correct_indices(n).finite(),
        correct_indices(n).len() <= n,
```

All three are one-line proofs: assert the subset by membership, then
`vstd::set_lib::lemma_len_subset` against `int_range_to(n)`.

### L3. Count loop extensions

```rust
proof fn lemma_le_set_upto_extend(readings: Seq<Reading>, v: Reading, i: int)
    requires 0 <= i < readings.len(),
    ensures
        readings[i] <= v ==>
            le_set_upto(readings, v, i + 1)
                =~= le_set_upto(readings, v, i).insert(i),
        readings[i] >  v ==>
            le_set_upto(readings, v, i + 1)
                =~= le_set_upto(readings, v, i),
        le_set_upto(readings, v, i).finite(),
        le_set_upto(readings, v, i + 1).finite(),
```

Mirror for `ge_set_upto`. Bodies: `=~=` plus subset-of-`int_range_to`
for finiteness.

### L4. Pigeonhole — safety direction

```rust
proof fn lemma_pigeonhole_le(readings: Seq<Reading>, v: Reading, f: nat)
    requires
        correct_indices(readings.len()).len() >= readings.len() - f,
        le_set(readings, v).len() >= f + 1,
    ensures some_correct_le(readings, v),
```

```rust
proof fn lemma_pigeonhole_ge(readings: Seq<Reading>, v: Reading, f: nat)
    /* symmetric */
```

Body sketch (for `le`):

```text
A := correct_indices(readings.len())
B := le_set(readings, v)
A, B ⊆ int_range_to(n) where n = readings.len() as int.
Hence |A ∪ B| <= n.
|A ∩ B| = |A| + |B| − |A ∪ B| >= (n − f) + (f + 1) − n = 1.
So A ∩ B is non-empty; choose i in it. That i is the existential witness
for some_correct_le.
```

Inclusion-exclusion identity may not be in vstd by that name. The
implementer can either find it (look in `vstd::set_lib` for
`lemma_set_union_inclusion_exclusion` / `lemma_int_intersect_lens`) or
prove it inline by decomposing `A ∪ B = (A − B) ⊎ B`, `A = (A − B) ⊎ (A ∩ B)`,
which only needs `lemma_set_disjoint_lens` (definitely in vstd).

### L5. Argmax / argmin over a finite set of indices

```rust
proof fn lemma_max_reading_in_set(s: Set<int>, readings: Seq<Reading>)
    requires
        s.finite(),
        s.len() >= 1,
        forall|j: int| s.contains(j) ==> 0 <= j < readings.len(),
    ensures exists|jm: int|
        s.contains(jm)
        && forall|j: int| s.contains(j) ==> readings[j] <= readings[jm],
    decreases s.len(),
```

Recursive proof: `let j0 = s.choose();` `let s' = s.remove(j0);`.
If `s'.len() == 0`, then `jm = j0`. Otherwise recurse, get `jm'`,
and pick `jm = j0` if `readings[j0] >= readings[jm']` else `jm = jm'`.
Needs `s.contains(j0)` (from `s.len() >= 1` ⇒ `!s.is_empty()`) and
`s' .insert(j0) == s` (extensional, by `=~=`).

```rust
proof fn lemma_min_reading_in_set(s: Set<int>, readings: Seq<Reading>)
    /* symmetric — picks the minimum-reading index in s */
```

### L6. Existence — the heavy lemma

```rust
proof fn lemma_exists_midpoint(readings: Seq<Reading>, f: nat)
    requires readings.len() >= 2 * f + 1,
    ensures exists|j: int|
        0 <= j < readings.len()
        && le_set(readings, readings[j]).len() >= f + 1
        && ge_set(readings, readings[j]).len() >= f + 1,
```

Body by contradiction:

```text
Let n = readings.len() (so n >= 2f + 1).

Suppose the ensures' exists is false. Then for every j in [0, n),
  le_set(readings, readings[j]).len() <= f
  OR
  ge_set(readings, readings[j]).len() <= f.

Define
  Lo = { j : 0 <= j < n && le_set(readings, readings[j]).len() <= f }
  Hi = { j : 0 <= j < n && ge_set(readings, readings[j]).len() <= f }

Then Lo ∪ Hi = int_range_to(n). Both Lo and Hi are subsets of
int_range_to(n) (so finite, by L1+lemma_len_subset).

|Lo| + |Hi| >= |Lo ∪ Hi| = n >= 2f + 1
⇒ at least one of |Lo|, |Hi| >= f + 1.

Case |Lo| >= f + 1:
    By L5 (max), pick jm in Lo with readings[jm] maximal among Lo.
    Then every j in Lo has readings[j] <= readings[jm],
    so j in le_set(readings, readings[jm]).
    Hence Lo ⊆ le_set(readings, readings[jm]).
    By lemma_len_subset, |le_set(readings, readings[jm])| >= |Lo| >= f + 1.
    But jm in Lo gives |le_set(readings, readings[jm])| <= f. Contradiction.

Case |Hi| >= f + 1:
    Symmetric with L5 (min).
```

The case split is "either `|Lo| >= f + 1` or `|Hi| >= f + 1`," which
the implementer should state as a `case_split` (or simply check both
disjuncts after establishing `|Lo| + |Hi| >= 2f + 1`).

---

## 7. SMT trouble spots

1. **`Set::new(|i| ...)` finiteness never fires automatically.** Every
   use of `.len()` on a freshly-built `Set::new` must be preceded by a
   subset bound (L1+L2 pattern). Defensive `assert(<set>.finite());`
   after each construction is worth its weight.

2. **`=~=` for set equalities.** L3's `le_set_upto` extension, the
   `le_set_upto(_, _, n) =~= le_set(_, _)` post-loop equation, and the
   `s.remove(j0).insert(j0) =~= s` step in L5 all need `=~=` with an
   `assert forall|x| ... <==> ... by { ... }` membership-equivalence
   block. Plain `==` won't fire.

3. **`choose|j: int| ...` to extract the argmax / intersection witness.**
   In L4, the intersection-nonempty conclusion has the shape
   `exists|i| i in A ∩ B`. Bind it with `let i = choose|i: int|
   (correct_indices(...).contains(i) && le_set(...).contains(i));` to
   reach the existential body needed for `some_correct_le`.

4. **Inclusion-exclusion in L4.** vstd may not have the asymmetric
   identity directly. The reliable form:
   ```rust
   // A and B subsets of universe U of size n.
   // A − B and B are disjoint, union to A ∪ B:
   lemma_set_disjoint_lens(A.difference(B), B);
   // (A − B) and (A ∩ B) disjoint, union to A:
   lemma_set_disjoint_lens(A.difference(B), A.intersect(B));
   // chain to |A ∩ B| = |A| + |B| − |A ∪ B|.
   ```
   Names may differ; the disjoint-len axiom is the load-bearing piece.

5. **Cardinality arithmetic crossing `nat` and `int`.** `|Lo ∪ Hi| <= n`
   in L6 needs `Lo ∪ Hi ⊆ int_range_to(n as int)`. The subset bound
   gives `|Lo ∪ Hi| <= n as nat`. Keep all the lengths in `nat` and the
   set domains in `int`; do not mix.

6. **L5 recursion termination.** `decreases s.len()`. Verus must see
   `s.remove(j0).len() == s.len() − 1` whenever `s.contains(j0)`. This
   is `axiom_set_remove_len` (or similar) — usually fires given an
   explicit `assert(s.contains(j0));` right after `let j0 = s.choose();`.

7. **Argmax case selection.** After the IH gives `jm'`, the case
   `readings[j0] >= readings[jm']` ⇒ pick `j0`; else pick `jm'`. The
   forall in the conclusion needs the recursive forall plus the chosen
   element's relation to `j0`. Wrap each branch in
   `assert forall|j: int| s.contains(j) implies readings[j] <= readings[jm]
   by { ... };` to nudge the solver.

8. **Cast and overflow at `let threshold: u32 = f + 1`.** `f` is `u32`,
   `f + 1` might overflow if `f == u32::MAX`. The spec rules that out:
   `2 * f + 1 <= readings.len() <= u32::MAX` ⇒ `f <= (u32::MAX − 1) / 2`,
   so `f + 1 <= (u32::MAX + 1) / 2 < u32::MAX`. State this as
   `assert(f as nat + 1 <= u32::MAX as nat);` before the addition.

9. **`n as usize` versus `n as u32`.** Stick to one: do
   `while j < readings.len() { ... }` with `j: usize` and only cast
   `j as int` inside ghost contexts. Avoid mixing `n: u32` and
   `len: usize` cursors — pick one cursor type and live with it.

10. **`forall j2 < j ==> ...` exit-of-loop ⇒ contradiction.** The
    fall-through asserts `assert(false)` after invoking
    `lemma_exists_midpoint`. The existential witness from the lemma
    contradicts the loop's "no earlier worked" invariant. If Verus
    doesn't see this, extract the witness:
    ```rust
    let jw = choose|j2: int| 0 <= j2 < readings@.len()
        && le_set(readings@, readings@[j2]).len() >= f as nat + 1
        && ge_set(readings@, readings@[j2]).len() >= f as nat + 1;
    // jw < n == j (loop exit), so the invariant applies at jw:
    // contradiction.
    ```

11. **Frame property: there is none.** `readings` is `&Vec`, no
    mutation. The usual `&mut self` defensive asserts do not apply.

---

## 8. Suggested order of operations

Easiest postcondition first; existence lemma last. The two pigeonhole
lemmas let the per-iteration return discharge without needing
`lemma_exists_midpoint`; the existence lemma only matters for the
unreachable post-loop case. Defer it.

1. Spec helpers (§3) and find/restate L1.
2. L2 (subset bounds + finiteness — three tiny lemmas).
3. L3 + `count_le` exec function.
4. Mirror: `count_ge` with its L3 mirror.
5. L4 pigeonhole lemmas (the algebra-heavy ones).
6. Main loop body with the in-loop early return only. Use the
   pigeonhole lemmas; leave the post-loop case as `unreachable!()`
   with a TODO. Verus will flag the missing existence proof.
7. L5 argmax + argmin lemmas.
8. L6 existence lemma using L5.
9. Slot L6 + `assert(false)` into the post-loop block.
10. Clean up redundant asserts.

---

## Sub-tasks

1. Add the proof-only spec helpers (`int_range_to`, `le_set`, `ge_set`,
   `le_set_upto`, `ge_set_upto`) and a stub body for `ft_midpoint` that
   simply returns `0` so the file parses cleanly. Do not run verus yet.
2. Land `lemma_int_range_to(n)` (or find it in vstd) and confirm verus
   accepts it standalone.
3. Land `lemma_le_set_in_range`, `lemma_ge_set_in_range`,
   `lemma_correct_indices_in_range` (three tiny subset/finiteness
   lemmas).
4. Land `lemma_le_set_upto_extend` and verify it standalone.
5. Land `lemma_ge_set_upto_extend` (mirror).
6. Write `count_le` exec with its loop invariant and the
   prefix-set-collapse assert at the bottom. Verify.
7. Write `count_ge` exec (mirror).
8. Land `lemma_pigeonhole_le`. Decompose into `lemma_set_disjoint_lens`
   calls if the inclusion-exclusion identity isn't a single vstd name.
9. Land `lemma_pigeonhole_ge` (mirror).
10. Replace the `ft_midpoint` stub with the main `while` loop, the
    in-loop count + threshold compare, the proof block that calls the
    two pigeonhole lemmas, and a `proof { assume(false); } unreachable!()`
    placeholder at the bottom **only as a debug aid** — flip it to a
    bare `unreachable!()` (the verus error will localise the existence
    obligation). *Strictly do not commit code with `assume(false)`*;
    use it only to isolate failures locally. Verify that everything
    except the existence obligation discharges.
11. Land `lemma_max_reading_in_set` (recursive over `s.len()`).
12. Land `lemma_min_reading_in_set` (mirror).
13. Land `lemma_exists_midpoint` using L5: build `Lo` and `Hi`, derive
    `|Lo| + |Hi| >= n`, case split on which side is `>= f + 1`, apply
    the argmax / argmin, conclude contradiction.
14. Wire `lemma_exists_midpoint` + `choose` witness + `assert(false)`
    into the post-loop block. End-to-end verus.
15. Strip non-load-bearing asserts; final clean.

## Summary: brute-force O(n²) midpoint via `Set`-indexed counts, with two pigeonhole lemmas (correct-set ∩ le/ge-set inclusion-exclusion) closing the per-iteration safety case and one argmax-on-Lo / argmin-on-Hi contradiction lemma closing the existence-driven post-loop case.
