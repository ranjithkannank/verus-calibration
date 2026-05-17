# Design — `marzullo`

One obligation: an exec function `marzullo(intervals, f) -> Interval` whose
output is an interval supporting (in its interior) a point covered by at
least `n - f` input intervals. Direct interval generalisation of
`ft_midpoint`; the proof skeleton is almost identical, but the existence
lemma is *easier* because the Helly-1D precondition
`correct_intervals_overlap` is given by hand rather than derived.

The trust boundary `correct_at(i)` is uninterpreted. The exec body must
not branch on it; the proof reasons about it through `correct_indices(n)`
plus `correct_intervals_overlap`.

---

## 1. Representation choice

Input is a frozen `&Vec<Interval>`. No auxiliary state, no ghost fields.
The whole proof is set-theoretic; counts are `Set<int>` cardinalities so
the pigeonhole step is just `lemma_len_subset`.

The output is a *degenerate single-point interval* `Interval { lo: p,
hi: p }`. This trivially satisfies `result.lo <= result.hi`, and lets the
existential postcondition `exists|p: Reading| result.lo <= p && p <=
result.hi && containing(_, p).len() >= n - f` use the obvious witness
`p`. Returning a wider interval would require either an additional
`min_hi`-of-supporting-intervals scan or arguing that the same `p` lies
inside it — wasted work the spec doesn't reward.

Counter for the count loop is `u32` (`intervals.len() <= u32::MAX`).

---

## 2. Algorithmic sketch

Approach: try each `intervals[i].lo` as candidate point `p`. For at least
one `i` (provably one of the correct sensors with maximum `lo` among
correct sensors), the count of input intervals containing `p` is `>= n -
f`. Return `Interval { lo: p, hi: p }`.

```text
for i in 0 .. n:
    p = intervals[i].lo
    c = count_containing(intervals, p)
    if c >= n - f:
        return Interval { lo: p, hi: p }
// unreachable per lemma_exists_supported_lo
```

**Why this works (proof of existence).** Let `s = correct_indices(n)`.
By Helly-1D (`correct_intervals_overlap`) plus `s.len() >= n - f >= f +
1 >= 1` (since `n >= 2f+1`), pick `jm = argmax over s of intervals[j].lo`.
Let `p = intervals[jm].lo`. For any correct `k`:
- `intervals[k].lo <= intervals[jm].lo = p` (argmax property)
- `intervals[k].hi >= intervals[jm].lo = p` (Helly-1D applied to `(jm, k)`)

So every correct index `k` is in `intervals_containing(intervals@, p)`,
hence `correct_indices(n) ⊆ intervals_containing(_, p)`, hence the count
at `p` is `>= n - f`.

**Why not interval-sweep.** The classical sweep-line Marzullo algorithm
requires sorting endpoints, maintaining a "current depth," and reasoning
about which endpoint segment is the witness — three separate proof
obligations the spec doesn't need. Linear scan over candidate `lo`s
costs O(n²) with one proof obligation.

**Why not `intervals[i].hi`.** Symmetric (argmin over `hi`s) would also
work but doesn't save anything. Lock in `lo` to avoid duplication.

---

## 3. Spec helpers (proof-only — not in the frozen spec)

```rust
spec fn containing_upto(intervals: Seq<Interval>, p: Reading, m: int) -> Set<int> {
    Set::new(|i: int| 0 <= i < m && i < intervals.len() && point_in_interval(p, intervals[i]))
}
```

`intervals_containing(intervals, p)` is already in the frozen spec; this
is the prefix variant for the count loop. `set_int_range(0, n)` and
`lemma_int_range` live in `vstd::set_lib`.

---

## 4. Key invariants

No struct. No `well_formed`. All obligations are function-level; the
function-level pre/post are the frozen `requires`/`ensures` and must
not be touched.

---

## 5. Loop invariant sketches

### 5a. `count_containing(intervals, p) -> (c: u32)`

Direct port of `count_le` from `ft_midpoint`.

```rust
fn count_containing(intervals: &Vec<Interval>, p: Reading) -> (c: u32)
    requires intervals.len() <= u32::MAX as nat,
    ensures
        c as nat == intervals_containing(intervals@, p).len(),
        intervals_containing(intervals@, p).finite(),
        c as nat <= intervals.len() as nat,
{
    let mut c: u32 = 0;
    let mut i: usize = 0;
    proof { assert(containing_upto(intervals@, p, 0) =~= Set::<int>::empty()); }
    while i < intervals.len()
        invariant
            0 <= i as int <= intervals@.len() as int,
            intervals.len() <= u32::MAX as nat,
            c as nat == containing_upto(intervals@, p, i as int).len(),
            containing_upto(intervals@, p, i as int).finite(),
            c as nat <= i as nat,
        decreases intervals.len() - i,
    {
        let iv = intervals[i];
        proof {
            lemma_containing_upto_extend(intervals@, p, i as int);
            lemma_containing_upto_in_range(intervals@, p, (i + 1) as int);
        }
        if iv.lo <= p && p <= iv.hi {
            proof { assert(!containing_upto(intervals@, p, i as int).contains(i as int)); }
            c = c + 1;
        }
        i = i + 1;
    }
    proof {
        assert(containing_upto(intervals@, p, intervals@.len() as int)
            =~= intervals_containing(intervals@, p));
        lemma_containing_in_range(intervals@, p);
    }
    c
}
```

### 5b. Main loop in `marzullo`

```rust
let n: usize = intervals.len();
let n_f: u32 = (n as u32) - f;       // threshold; overflow-safe since 2f+1 <= n
let mut i: usize = 0;
while i < n
    invariant
        0 <= i as int <= n as int,
        n == intervals.len(),
        intervals.len() <= u32::MAX as nat,
        intervals.len() as nat >= 2 * (f as nat) + 1,
        n_f as nat == intervals.len() as nat - f as nat,
        well_formed(intervals@),
        correct_indices(intervals.len() as nat).len() >= intervals.len() as nat - f as nat,
        correct_intervals_overlap(intervals@),
        // no earlier candidate worked
        forall|j2: int| 0 <= j2 < i as int ==>
            intervals_containing(intervals@, #[trigger] intervals@[j2].lo).len()
                < intervals.len() as nat - f as nat,
    decreases n - i,
{
    let iv = intervals[i];
    let p: Reading = iv.lo;
    let c: u32 = count_containing(intervals, p);
    if c >= n_f {
        // Witness for the existential is p itself.
        return Interval { lo: p, hi: p };
    }
    proof {
        assert(c < n_f);
        assert(p == intervals@[i as int].lo);
        assert(intervals_containing(intervals@, intervals@[i as int].lo).len()
            < intervals.len() as nat - f as nat);
    }
    i = i + 1;
}
// Post-loop: i == n. lemma_exists_supported_lo gives a witness < n
// that contradicts the loop invariant.
proof {
    lemma_exists_supported_lo(intervals@, f as nat);
    let jw = choose|jx: int|
        0 <= jx < intervals@.len()
        && intervals_containing(intervals@, intervals@[jx].lo).len()
            >= intervals@.len() - f as nat;
    assert(0 <= jw < i as int);
    assert(intervals_containing(intervals@, intervals@[jw].lo).len()
        < intervals.len() as nat - f as nat);
    assert(false);
}
Interval { lo: 0, hi: 0 }  // unreachable — discharged by assert(false)
```

The precondition components have to be carried as invariants because
Verus does not auto-import them into the loop scope.

**Trigger discipline.** Tag the inner `intervals_containing(...,
#[trigger] intervals@[j2].lo)` so that when `lemma_exists_supported_lo`
produces a witness `jx` and the proof asserts
`intervals_containing(intervals@, intervals@[jx].lo).len() >= ...`,
Verus instantiates the loop's `forall` at `j2 = jx`. (This is the same
trigger lesson the implementer hit in `ft_midpoint`; promote it from
day 1.)

---

## 6. Predicted helper lemmas

Most are direct ports from `ft_midpoint`. Names are non-binding.

### L1. Subset / finiteness bounds (three tiny lemmas)

```rust
proof fn lemma_containing_in_range(intervals: Seq<Interval>, p: Reading)
    ensures
        intervals_containing(intervals, p).subset_of(set_int_range(0, intervals.len() as int)),
        intervals_containing(intervals, p).finite(),
        intervals_containing(intervals, p).len() <= intervals.len() as nat,
```

```rust
proof fn lemma_containing_upto_in_range(intervals: Seq<Interval>, p: Reading, m: int)
    requires 0 <= m <= intervals.len(),
    ensures
        containing_upto(intervals, p, m).subset_of(set_int_range(0, m)),
        containing_upto(intervals, p, m).finite(),
        containing_upto(intervals, p, m).len() <= m as nat,
```

```rust
proof fn lemma_correct_indices_in_range(n: nat)
    ensures
        correct_indices(n).subset_of(set_int_range(0, n as int)),
        correct_indices(n).finite(),
        correct_indices(n).len() <= n,
```

All three: assert the subset relation by membership, then call
`vstd::set_lib::lemma_len_subset`. Copy the body from `ft_midpoint`
verbatim.

### L2. Prefix extension lemma

```rust
proof fn lemma_containing_upto_extend(intervals: Seq<Interval>, p: Reading, i: int)
    requires 0 <= i < intervals.len(),
    ensures
        point_in_interval(p, intervals[i]) ==>
            containing_upto(intervals, p, i + 1)
                =~= containing_upto(intervals, p, i).insert(i),
        !point_in_interval(p, intervals[i]) ==>
            containing_upto(intervals, p, i + 1)
                =~= containing_upto(intervals, p, i),
```

Body: empty (the two `=~=`s drop straight out of `Set::new`).

### L3. Argmax-`lo` over a finite index set

Direct adaptation of `lemma_max_reading_in_set` from `ft_midpoint`,
mapping over `intervals[j].lo` instead of `readings[j]`.

```rust
proof fn lemma_max_lo_in_set(s: Set<int>, intervals: Seq<Interval>) -> (jm: int)
    requires
        s.finite(),
        s.len() >= 1,
        forall|j: int| s.contains(j) ==> 0 <= j < intervals.len(),
    ensures
        s.contains(jm),
        forall|j: int| s.contains(j) ==> intervals[j].lo <= intervals[jm].lo,
    decreases s.len(),
```

Body: copy `lemma_max_reading_in_set` and substitute `intervals[_].lo`
for `readings[_]`. The two `assert forall|j: int| s.contains(j) implies
... by { if j != j0 { assert(s2.contains(j)); } }` nudges remain the
load-bearing pieces.

### L4. Existence — the only hard lemma

```rust
proof fn lemma_exists_supported_lo(intervals: Seq<Interval>, f: nat)
    requires
        intervals.len() >= 2 * f + 1,
        well_formed(intervals),
        correct_indices(intervals.len()).len() >= intervals.len() - f,
        correct_intervals_overlap(intervals),
    ensures
        exists|j: int|
            0 <= j < intervals.len()
            && intervals_containing(intervals, intervals[j].lo).len()
                >= intervals.len() - f,
```

Body sketch:

```text
Let s = correct_indices(intervals.len()).
lemma_correct_indices_in_range(intervals.len())  // finite + bounded
s.len() >= n - f >= f + 1 >= 1 (from n >= 2f + 1)
let jm = lemma_max_lo_in_set(s, intervals);     // jm in s, argmax-lo
let p = intervals[jm].lo;

// Claim: s ⊆ intervals_containing(intervals, p).
// For any k in s:
//   intervals[k].lo <= intervals[jm].lo = p   (argmax)
//   intervals[k].hi >= intervals[jm].lo = p   (Helly-1D at (jm, k))
//   ⇒ point_in_interval(p, intervals[k])
assert(s.subset_of(intervals_containing(intervals, p))) by {
    assert forall|k: int| s.contains(k)
           implies intervals_containing(intervals, p).contains(k) by {
        // s.contains(k) ⇒ correct_at(k) ∧ 0 <= k < n
        // s.contains(jm) ⇒ correct_at(jm)
        // correct_intervals_overlap gives intervals[jm].lo <= intervals[k].hi
        // argmax gives intervals[k].lo <= intervals[jm].lo
    };
};
lemma_containing_in_range(intervals, p);        // finite
lemma_len_subset(s, intervals_containing(intervals, p));
// Now |intervals_containing(_, p)| >= |s| >= n - f.
// jm is the witness for the existential.
```

The Helly-1D step is the *only* new ingredient relative to
`ft_midpoint`'s existence lemma — and it's a direct quantifier
instantiation, no pigeonhole arithmetic. The proof should be *shorter*
than `lemma_exists_midpoint` in `ft_midpoint`.

Note: unlike `ft_midpoint`'s `lemma_exists_midpoint`, no contradiction /
`Lo ∪ Hi` set-decomposition is needed. Marzullo's existence is a
direct constructive argument from Helly-1D + argmax.

---

## 7. SMT trouble spots

1. **`Set::new(|i| ...)` finiteness never fires automatically.** Every
   use of `.len()` must be preceded by a subset bound to
   `set_int_range(0, n)` plus `lemma_len_subset`. The L1 lemmas do this
   once; call them whenever a `Set::new`'s `.len()` is referenced.

2. **`=~=` for set equalities.** L2's two extensions, the
   `containing_upto(_, _, n) =~= intervals_containing(_, _)` post-loop
   equation, and `s.remove(j0).insert(j0) =~= s` inside L3 all need
   `=~=` plus an `assert forall|x| ... <==> ... by { ... }` block.

3. **`choose|jx: int| ...` for the post-loop contradiction.** The
   existential from `lemma_exists_supported_lo` needs `let jw =
   choose|jx: int| ...;` so that the loop invariant's `forall|j2|`
   can be instantiated at `j2 = jw`. Use the trigger discipline below.

4. **Trigger on `intervals@[j2].lo`.** Tag the loop's invariant body
   `intervals_containing(intervals@, #[trigger] intervals@[j2].lo)`.
   Without an explicit trigger, the witness produced by `choose` may
   not align with Verus' default trigger choice, and the contradiction
   won't fire. (This was the day-15 pain point in `ft_midpoint`;
   inheriting it eliminates a guaranteed escalation.)

5. **Argmax case selection (L3).** Each branch of the recursive case
   needs `assert forall|j: int| s.contains(j) implies intervals[j].lo
   <= intervals[jm].lo by { if j != j0 { assert(s2.contains(j)); } };`.
   Without the inner `assert(s2.contains(j))` the recursive forall is
   not in scope.

6. **`s.remove(j0).len() == s.len() − 1`** is `axiom_set_remove_len`;
   usually fires after an explicit `assert(s.contains(j0));` right
   after `let j0 = choose|x: int| s.contains(x);` (preceded by
   `axiom_is_empty_len0(s); axiom_is_empty(s);` to flip `s.len() >= 1`
   into `s.contains(_)` via `choose`).

7. **Helly-1D instantiation.** Inside L4, after `let p =
   intervals[jm].lo;`, the SMT may not automatically instantiate
   `correct_intervals_overlap` at `(jm, k)`. The reliable form:
   ```rust
   assert forall|k: int| s.contains(k)
          implies intervals_containing(intervals, p).contains(k) by {
       assert(correct_at(k));                       // from s.contains(k)
       assert(correct_at(jm));                      // from s.contains(jm)
       assert(intervals[jm].lo <= intervals[k].hi); // Helly-1D at (jm, k)
       assert(intervals[k].lo <= intervals[jm].lo); // argmax at k
   };
   ```
   Each `assert` is a deliberate trigger.

8. **`well_formed(intervals)` for `intervals[k].lo <= intervals[k].hi`.**
   Not strictly required by the existence argument (the argmax/Helly
   chain already gives `intervals[k].lo <= p <= intervals[k].hi`), but
   keep it as a precondition for `lemma_exists_supported_lo` so the
   implementer doesn't have to wonder later. The exec function already
   has it as a `requires`.

9. **Overflow on `n - f`.** `n: usize`, `f: u32`. `n >= 2f + 1` ⇒ `n -
   f >= f + 1 >= 1`, so the subtraction is safe. The cast `(n as u32) -
   f` requires `n <= u32::MAX`, which is a precondition. Defensive:
   ```rust
   assert(intervals.len() as nat >= f as nat + 1);
   let n_u32: u32 = intervals.len() as u32;
   let n_f: u32 = n_u32 - f;
   assert(n_f as nat == intervals.len() as nat - f as nat);
   ```

10. **Unreachable post-loop return.** After `proof { ... assert(false);
    }`, return a concrete `Interval { lo: 0, hi: 0 }`. **Do not use
    `unreachable!()`** — it is on the cheat-list and would fail the
    reviewer's audit. Pattern from `ft_midpoint`'s post-loop:
    `assert(false)` makes every postcondition vacuous, so any concrete
    return value type-checks.

11. **Frame property: none.** `intervals` is `&Vec`, no mutation. The
    `&mut self` defensive asserts do not apply.

12. **`Interval` field copies.** `Interval` derives no traits in the
    frozen spec; verify it's `Copy` (it is — two `i64`s, no `String`).
    If not, replace `let iv = intervals[i];` with `let iv = &intervals[i];`
    or read the fields directly via `intervals[i].lo`.

---

## 8. Suggested order of operations

Same shape as `ft_midpoint`, minus the second pigeonhole lemma family.

1. Spec helper `containing_upto`.
2. L1 subset/finiteness lemmas (three of them).
3. L2 prefix extension lemma.
4. `count_containing` exec function, full loop invariant + post-loop
   collapse assert.
5. L3 argmax-`lo` lemma (port of `lemma_max_reading_in_set`).
6. L4 existence lemma using L3 + Helly-1D.
7. Main exec loop with in-loop early return; leave the post-loop
   block as a TODO so Verus localises the existence obligation.
8. Wire L4 + `choose` witness + `assert(false)` + concrete fallback
   return into the post-loop block.
9. Clean up redundant asserts.

---

## Sub-tasks

1. Add the proof-only `containing_upto` spec helper and stub `marzullo`
   with a body that returns `Interval { lo: 0, hi: 0 }`. Add `use
   vstd::set_lib::*;`. Confirm the file parses.
2. Land `lemma_containing_in_range` (subset + finite + bounded).
3. Land `lemma_containing_upto_in_range` (mirror, with the `0 <= m <=
   intervals.len()` requires).
4. Land `lemma_correct_indices_in_range` (port of the ft_midpoint
   version).
5. Land `lemma_containing_upto_extend` with the two `=~=` ensures and
   empty body.
6. Write `count_containing` with its loop invariant, the empty-prefix
   `=~=` assert at entry, the per-iter `lemma_containing_upto_extend`
   call, and the post-loop `containing_upto(_, _, n) =~=
   intervals_containing(_, _)` assert. Verify.
7. Land `lemma_max_lo_in_set` (port of `lemma_max_reading_in_set`,
   substituting `intervals[_].lo` for `readings[_]`).
8. Land `lemma_exists_supported_lo` using L3 + Helly-1D. The body is
   constructive (no contradiction shape required); follow §6/L4 sketch.
9. Replace the `marzullo` stub with the main `while` loop, the in-loop
   `count_containing` + threshold compare + `return Interval { lo: p,
   hi: p }`, and the per-iter "this candidate failed" assert that
   maintains the strengthened invariant. Leave the post-loop block as
   `Interval { lo: 0, hi: 0 }` so the verus error localises the
   existence obligation. Verify everything except the post-loop path.
10. Wire `lemma_exists_supported_lo` + `choose` witness + `assert(false)`
    into the post-loop block (above the fallback `Interval { lo: 0,
    hi: 0 }` return). End-to-end verus.
11. Strip non-load-bearing asserts; final clean.

## Summary: Linear scan over `intervals[i].lo` candidates with a single `count_containing` helper, closed by an argmax-of-correct-`lo`-plus-Helly-1D existence lemma — interval generalisation of `ft_midpoint` whose existence proof is strictly easier (constructive, not contradiction-via-inclusion-exclusion).
