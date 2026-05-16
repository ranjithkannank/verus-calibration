# Design: `binary_search`

## Spec recap

```rust
pub fn binary_search(v: &Vec<i64>, target: i64) -> (result: Option<usize>)
    requires is_sorted(v@),
    ensures
        result.is_some() ==> {
            let i = result.unwrap();
            &&& (i as int) < v@.len()
            &&& v@[i as int] == target
        },
        result.is_none() ==> forall|i: int| 0 <= i < v@.len() ==> v@[i] != target,
```

Sortedness is on `Seq<i64>` (view of `Vec<i64>`). No struct, no ghost state needed — just a function body.

## 1. Representation choice

Stay on the `Vec<i64>` directly. We index with `usize`, but reason about positions as `int` whenever bounds arithmetic could overflow. Verus' `Vec::len()` returns a `usize`; `v.len() as int` matches `v@.len()`.

No auxiliary ghost variables. The two `usize` cursors `lo` and `hi` are exec state; their relationship to the spec is captured purely by the loop invariant.

Use a *half-open* window `[lo, hi)`:
- Initial state: `lo = 0`, `hi = v.len()`.
- Termination: `lo == hi` → target absent.
- Hit: `v[mid] == target` → return `Some(mid)`.

Half-open avoids the `hi = mid - 1` underflow problem (`usize` cannot go negative), and matches Verus' usual sequence-slicing conventions.

## 2. Algorithmic sketch

```rust
let mut lo: usize = 0;
let mut hi: usize = v.len();
while lo < hi
    invariant <see §4>
{
    let mid: usize = lo + (hi - lo) / 2;     // overflow-safe
    let x = v[mid];                           // requires mid < v.len()
    if x == target {
        return Some(mid);
    } else if x < target {
        lo = mid + 1;
    } else {
        hi = mid;
    }
}
None
```

`mid` is computed with `lo + (hi - lo) / 2` rather than `(lo + hi) / 2` so we never need to reason about `lo + hi` overflowing `usize`. Since `hi <= v.len() <= usize::MAX`, `hi - lo` is fine and `lo + (hi - lo)/2 < hi <= usize::MAX`.

## 3. Key invariants

There is no struct, so all invariants are local to the loop. The function-level "invariants" are exactly the `requires`/`ensures` of the spec:

- Pre: `is_sorted(v@)`.
- Post (Some branch): index in range and element equals target.
- Post (None branch): target nowhere in the whole sequence.

## 4. Loop invariant sketch

The non-trivial conjunct is the "everything outside `[lo, hi)` is not the target" part. Spell it as two `forall`s split at the cursors:

```rust
while lo < hi
    invariant
        is_sorted(v@),                                  // (a) carry the precondition
        0 <= lo <= hi <= v@.len(),                      // (b) cursor bounds (note: lo, hi are usize so >= 0 is automatic, but stating as int is fine)
        hi <= v.len(),                                  // (c) usize-level bound for indexing (redundant with (b) once usize/int are linked)
        forall|k: int| 0 <= k < lo as int ==> v@[k] != target,
        forall|k: int| hi as int <= k < v@.len() ==> v@[k] != target,
{
    ...
}
```

Conjunct rationale:
- **(a)** Needed when we narrow `lo = mid + 1` and want to conclude every `v@[k]` with `k < mid+1` is `!= target`. Concretely: `v@[mid] < target` plus sortedness gives `v@[k] <= v@[mid] < target` for `k <= mid`.
- **(b),(c)** Cursor bounds. `lo` and `hi` are `usize`; the `as int` casts let us state things uniformly.
- **The two `forall`s** are the heart of the proof. They are vacuously true at entry (`lo = 0`, `hi = v.len()`).

On loop exit (`lo == hi`), the two `forall`s tile the whole index range, giving the None postcondition directly.

## 5. Updates per branch — what to prove

When `v[mid] < target` and we set `lo' = mid + 1`:
- Need: `forall|k: int| 0 <= k < (mid+1) as int ==> v@[k] != target`.
- Old left half (`k < mid`) preserved from prior invariant *only if* mid >= lo, which holds since `lo <= mid < hi`. Actually we need the new statement for all `k < mid+1`, i.e. `k <= mid`. For `k < lo`: prior invariant. For `lo <= k <= mid`: need sortedness. `v@[k] <= v@[mid] < target` ⇒ `!= target`. ✓

When `v[mid] > target` and we set `hi' = mid`:
- Need: `forall|k: int| mid as int <= k < v@.len() ==> v@[k] != target`.
- For `k > hi-1` (old right half): prior invariant.
- For `mid <= k < hi`: sortedness gives `v@[k] >= v@[mid] > target`. ✓

The implementer may have to write **explicit asserts** for both of these — see §7.

## 6. Helper lemmas predicted

For this exercise I don't expect a separate `proof fn` is necessary; the sortedness fact is a single `forall` that the SMT solver can usually instantiate when prompted with the right `assert`. But if the solver balks, the canonical helper is:

```rust
proof fn lemma_sorted_lt(s: Seq<i64>, i: int, j: int)
    requires
        is_sorted(s),
        0 <= i <= j < s.len(),
    ensures
        s[i] <= s[j],
{
    // body: just unfolds is_sorted; SMT auto.
}
```

Call it with `(v@, k as int, mid as int)` in the `lo` update and `(v@, mid as int, k as int)` in the `hi` update if direct sortedness instantiation fails. **Do not** write this lemma unless the loop body fails without it; it's noise otherwise.

## 7. SMT trouble spots

1. **Sortedness instantiation across the cursor update.** The forall in `is_sorted` is over `int, int`. After `lo = mid + 1`, the solver must pick `(k, mid)` to discharge the new "left half" forall. Likely needs:
   ```rust
   assert(forall|k: int| 0 <= k <= mid as int ==> v@[k] <= v@[mid as int]);
   ```
   inside the `x < target` branch, before the assignment.
2. **`usize` ↔ `int` plumbing.** `mid as int`, `lo as int`, etc. — the invariant should use one consistent form (recommend casting cursors to `int` when stating `forall|k: int|` bounds). Mixing `mid` (usize) with `k: int` in the same comparison sometimes confuses the solver; prefer `k < mid as int`.
3. **`mid < hi` derivation.** Needed to (a) index `v[mid]` and (b) show `mid + 1 <= hi` for usize subtraction. From `lo < hi` and `mid = lo + (hi - lo)/2`, we have `mid < hi`. The solver gets this in one step if `lo < hi` is in scope; if not, assert `mid < hi` after computing it.
4. **No overflow on `mid + 1`.** `mid < hi <= v.len() <= usize::MAX`, so `mid + 1 <= usize::MAX`. Verus usually handles this without help, but if `--triggers` whines, an `assert(mid + 1 <= v.len())` clears it.
5. **Final `None` step.** After the loop, `lo == hi`, so the two forall conjuncts cover `[0, v.len())`. Should be automatic; if not, one assert combining them suffices:
   ```rust
   assert(forall|k: int| 0 <= k < v@.len() ==> v@[k] != target);
   ```

## 8. Suggested order of operations

1. Write the loop skeleton with `lo`, `hi`, `mid`, and the three-branch `if`/`else if`/`else`. Return `Some(mid)` and `None` at the right places.
2. Add the cursor-bound conjuncts of the loop invariant (`0 <= lo <= hi <= v@.len()`). Run verus — confirm the indexing `v[mid]` typechecks. This is the cheapest postcondition path: the `Some` branch requires only `mid < v.len()` and `v@[mid] == target`, both immediate.
3. Add the two `forall` conjuncts to the invariant. Run verus — the `None` postcondition should now go through (loop-exit covers everything).
4. The two narrowing branches will fail. Inside each branch, add the sortedness `assert`s from §7 #1 to give the solver a trigger. If still failing, add `lemma_sorted_lt` from §6 and call it.
5. Last: clean up redundant asserts. Leave only the ones the verifier actually needs.

Expected total body: ~25-40 lines, including invariant block.

## Summary: Half-open binary search on `Vec<i64>` with a 4-conjunct loop invariant; the only likely SMT friction is forcing sortedness instantiation at each cursor narrowing via a single `assert` per branch.
