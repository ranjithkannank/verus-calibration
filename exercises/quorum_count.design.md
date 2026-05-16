# Design — `quorum_count`

## Spec recap

```rust
pub fn is_byzantine_quorum(voters: &Vec<NodeId>, n: u32) -> (result: bool)
    requires
        n > 0,
        all_in_range(voters@, n),
    ensures
        result == (distinct_count(voters@) >= byzantine_threshold(n)),
```

Where:
- `NodeId == u32`,
- `distinct_count(voters) = voters.to_set().len()` (a `nat`),
- `byzantine_threshold(n) = ((2 * n) / 3 + 1) as nat`,
- `all_in_range(voters, n)` means every `voters[i] as int < n as int`.

The hard part is the **concrete↔abstract gap**: the executable code has to count distinct ids by walking the `Vec`, but the spec speaks of `Set::len()`. We bridge the two by an inductive invariant on a "seen" bitmap.

## 1. Representation choice

Two pieces of exec state:

- `seen: Vec<bool>` of length `n` — a bitmap where `seen[k] == true` iff some `voters[j]` with `j < i` (the loop cursor) is equal to `k`.
- `count: u64` — the running count of distinct voters observed so far.

Why a bitmap and not a `Vec<NodeId>` of distinct ids, or a ghost `Set<NodeId>`?

- **Vs. a `Vec<NodeId>` accumulator.** Membership check would be O(|distinct|) per step ⇒ harder to reason about, and would force a nested loop invariant. The bitmap gives O(1) membership.
- **Vs. a pure ghost `Set<NodeId>`.** A ghost-only set has nothing for the executable to compare against; we still need a concrete counter, and we still need to track *which* ids we've seen to know whether to increment. The bitmap is exactly that "which" in exec form. No ghost field is necessary; the bitmap *is* the state.

`count` is `u64`, not `u32`, only to make the threshold comparison roomy: `2 * n` overflows `u32` for `n > u32::MAX / 2`. Computing in `u64` is safe (`2 * u32::MAX < u64::MAX`). The count itself fits in `u32` since `count <= n`, but we keep it `u64` to avoid mixed-width casts at the comparison.

## 2. Algorithmic sketch

```rust
let mut seen: Vec<bool> = vec![false; n as usize];   // init bitmap
let mut count: u64 = 0;
let mut i: usize = 0;
while i < voters.len()
    invariant <see §4>
{
    let v: usize = voters[i] as usize;               // v < n by all_in_range
    if !seen[v] {
        seen.set(v, true);
        count = count + 1;
    }
    i = i + 1;
}
let threshold: u64 = 2u64 * (n as u64) / 3u64 + 1u64;
count >= threshold
```

Two complications the implementer should know going in:

- `vec![false; n as usize]` — vstd supports this macro and gives `result@.len() == n as nat` and `forall|k| 0 <= k < n ==> result@[k] == false`. If that macro form's spec is awkward to use, fall back to a small init loop (yet another loop invariant — prefer the macro).
- `Vec::set(i, x)` (i.e. `seen.set(v, true)`) is the vstd-friendly way to mutate; direct `seen[v] = true` may not have an exec spec depending on Verus version. Use `set` and rely on its frame postcondition.

## 3. Key invariants

There is no struct. All proof obligations live inside `is_byzantine_quorum`. The function-level pre/post are the frozen `requires`/`ensures`.

The two derived facts the proof must produce, from the post-loop state:

1. `count as nat == voters@.to_set().len()` — bitmap counter matches distinct count.
2. `(count >= threshold) <==> (distinct_count(voters@) >= byzantine_threshold(n))` — comparing `u64`s matches comparing `nat`s.

(2) is automatic once both sides agree as `nat`. (1) is the work of the loop invariant.

## 4. Loop invariant sketch

Let `prefix(j) == voters@.subrange(0, j as int)`. Conjuncts:

```rust
while i < voters.len()
    invariant
        // (a) cursor and bitmap bounds
        0 <= i <= voters@.len(),
        seen@.len() == n as nat,
        n > 0,

        // (b) re-carry the precondition (some Verus versions drop it
        // across the loop boundary; cheap to re-state)
        all_in_range(voters@, n),

        // (c) bitmap abstraction:  seen[k]  iff  k appears in prefix(i)
        forall|k: int| 0 <= k < n as int ==>
            (seen@[k] == prefix(i as int).contains(k as u32)),

        // (d) counter abstraction
        count as nat == prefix(i as int).to_set().len(),

        // (e) sanity bound (helps the solver know `count` won't overflow,
        // and the set is finite)
        count as nat <= n as nat,
{ ... }
```

Initial state (`i == 0`): `prefix(0)` is the empty seq, so its `to_set()` is empty (len 0) and contains nothing → (c), (d), (e) hold trivially.

Termination (`i == voters@.len()`): `prefix(voters@.len()) == voters@` (extensional equality; may need `assert(prefix(voters@.len() as int) =~= voters@);`), so (d) becomes `count as nat == voters@.to_set().len() == distinct_count(voters@)`. Done.

### What each branch must re-establish

Inside the loop body, let `v_id = voters@[i as int]` (a `NodeId`/`u32`).

**Case A — `seen[v_id as int] == false` (new voter):**

- After `seen.set(v_id as usize, true)` and `count += 1` and `i += 1`:
  - (c): for `k == v_id`, the new prefix contains `v_id` (it's the last element), and `seen@[v_id as int]` is now `true`. For `k != v_id`, `seen@[k]` unchanged and `prefix(i+1).contains(k) == prefix(i).contains(k)` (the new element is `v_id`, not `k`).
  - (d): `prefix(i+1).to_set() == prefix(i).to_set().insert(v_id)`. Since `v_id` was *not* in `prefix(i).to_set()` (by invariant (c) and the `false` branch), `insert` grows length by 1 ⇒ `count + 1`.

**Case B — `seen[v_id as int] == true` (duplicate):**

- After `i += 1`, `seen` and `count` unchanged:
  - (c): `prefix(i+1).contains(k) == prefix(i).contains(k) || (k == v_id)`. For `k == v_id`, both sides are `true` (invariant (c) and `seen[v_id] == true`). For `k != v_id`, same as Case A.
  - (d): `prefix(i+1).to_set() == prefix(i).to_set().insert(v_id)`, but `v_id` is already in the set ⇒ length unchanged.

## 5. Predicted helper lemmas

Three small ones. Names are suggestions; signatures are the contract.

```rust
proof fn lemma_prefix_extend(s: Seq<NodeId>, i: int)
    requires 0 <= i < s.len(),
    ensures s.subrange(0, i + 1) =~= s.subrange(0, i).push(s[i]),
{ /* trivial: extensional equality on Seq */ }
```

```rust
proof fn lemma_push_to_set(s: Seq<NodeId>, x: NodeId)
    ensures s.push(x).to_set() =~= s.to_set().insert(x),
{
    // Should follow from vstd axioms about Seq::to_set and Seq::push,
    // possibly via an `assert forall ... by { ... }` over membership.
}
```

```rust
proof fn lemma_set_insert_len(s: Set<NodeId>, x: NodeId)
    requires s.finite(),
    ensures
        s.insert(x).len() == s.len() + (if s.contains(x) { 0int } else { 1int }) as nat,
{
    // Direct from Set::axiom_set_insert_len (already in vstd; this is a
    // thin wrapper to put it in scope with the right cast).
}
```

A fourth that may be needed if the solver doesn't see `to_set` is finite:

```rust
proof fn lemma_to_set_finite(s: Seq<NodeId>)
    ensures s.to_set().finite() && s.to_set().len() <= s.len(),
{
    // vstd has `axiom_seq_to_set_finite` and `lemma_seq_to_set_len` or
    // similar; this proof is a one-liner that calls them.
}
```

**Do not write any of these speculatively.** Write the loop, hit the failure, *then* introduce exactly the lemma the failing assert needs.

## 6. SMT trouble spots

1. **`prefix(i+1) == prefix(i).push(voters@[i])`.** Subrange equalities almost never close on their own; use `assert(prefix(i + 1) =~= prefix(i).push(voters@[i as int]));` to force extensional reasoning. This is the single most important hint in the loop body.

2. **`push.to_set() == to_set.insert(x)`.** Even with the lemma above, the SMT solver may not chain `to_set` through `push`. If `lemma_push_to_set` does not exist in vstd under a known name, prove it by extensional set equality:

   ```rust
   assert forall|y: NodeId|
       s.push(x).to_set().contains(y) <==> s.to_set().insert(x).contains(y)
   by { /* membership unfolds to existence in seq */ };
   assert(s.push(x).to_set() =~= s.to_set().insert(x));
   ```

3. **`Vec::set` frame property.** After `seen.set(v, true)`:
   - `seen@.len()` unchanged (need this to keep invariant (a)).
   - `seen@[v as int] == true`.
   - `forall|k: int| 0 <= k < seen@.len() && k != v as int ==> seen@[k] == old(seen)@[k]`.

   vstd's `Vec::set` provides all three; if invariant (c) doesn't re-close, paste the third assert explicitly.

4. **`count` overflow.** `count <= n <= u32::MAX < u64::MAX`, so `count + 1` never overflows. Invariant (e) (`count as nat <= n as nat`) plus `n: u32` should be enough; if not, `assert(count < n as u64)` before the `+= 1` (we know strictly `<` because there's at least one not-yet-seen id, namely `v_id`).

5. **Threshold computation.** `2u64 * (n as u64) / 3 + 1` must equal `byzantine_threshold(n) as nat == ((2 * n as nat) / 3 + 1) as nat`. The catch is operator order and casts. State it explicitly:

   ```rust
   let threshold: u64 = 2u64 * (n as u64) / 3u64 + 1u64;
   assert(threshold as nat == byzantine_threshold(n));
   ```

   If the assert fails, decompose: `assert(2u64 * (n as u64) == (2 * n as nat) as u64);` and similarly for the division. The pitfall is `nat` division vs `u64` division agree only because both operands are nonnegative — Verus knows this, but the assert nudges it.

6. **Comparison with mixed widths.** Final return is `count >= threshold` (both `u64`). The spec says `distinct_count(voters@) >= byzantine_threshold(n)` (both `nat`). Once `count as nat == distinct_count(voters@)` and `threshold as nat == byzantine_threshold(n)`, the `>=` agrees because `as nat` is monotone on nonnegative integers. If the solver hesitates, one final assert before `return`:

   ```rust
   assert((count >= threshold) == (count as nat >= threshold as nat));
   ```

7. **`all_in_range` access.** Inside the loop, you need `voters@[i as int] as int < n as int` to safely cast `voters[i] as usize` for `seen` indexing. This is a direct instantiation of `all_in_range` at `k = i`. Usually automatic given (b) is in the loop invariant, but if Verus complains about indexing `seen` out of bounds, add `assert((voters@[i as int] as int) < n as int);` right before the `seen.set(...)` call.

8. **`prefix(voters@.len()) =~= voters@`.** Outside the loop, to convert (d) into the postcondition shape, assert this extensional equality. `Seq::subrange(0, len) == self` is true but often not auto-fired.

## 7. Suggested order of operations

1. **Write the bitmap init + loop skeleton + threshold compare, no invariants.** Verify it compiles (it will fail verification — that's expected). This pins down the algorithm.

2. **Add invariants (a), (b), (e) only** (cursor bounds, range re-carry, count bound). Confirm the loop body's `voters[i] as usize` indexing into `seen` typechecks. Don't worry about the postcondition yet.

3. **Add invariant (c)** (bitmap abstraction). In each branch, add an `assert forall|k: int| ... by { ... }` to re-establish it after the `set`/`+=` updates. Use the `seen.set` frame property explicitly.

4. **Add invariant (d)** (counter abstraction). This is where the helper lemmas earn their keep:
   - In Case A: assert `prefix(i+1) =~= prefix(i).push(v_id)`, call `lemma_push_to_set`, then `lemma_set_insert_len` with `s.contains(v_id) == false` (justified by inv (c) and the `false` branch).
   - In Case B: same chain, but `s.contains(v_id) == true` ⇒ length unchanged.

5. **After the loop**, write the extensional equality `assert(voters@.subrange(0, voters@.len() as int) =~= voters@);` to convert (d) into `count as nat == distinct_count(voters@)`.

6. **Threshold and return.** Compute `threshold`, assert `threshold as nat == byzantine_threshold(n)`, return `count >= threshold`. Add the mixed-width comparison assert from §6 #6 only if Verus complains.

7. **Cleanup.** Strip asserts that aren't load-bearing. Final body should be ~40-70 lines including invariants and asserts; helper lemmas another ~20-40 lines combined.

### Stretch (skip unless everything above verifies green)

The exercise file mentions a `quorum_intersection_lemma`. Treat it as a *separate* `proof fn` after `is_byzantine_quorum`. The statement (informally): for `n > 3 * f`, two sets `A, B ⊆ [0, n)` with `|A|, |B| >= byzantine_threshold(n)` satisfy `A ∩ B != ∅` whenever `n > 3f` and... — re-read the spec carefully before formalizing. **Do not** modify `is_byzantine_quorum`'s ensures clause to mention this lemma.

## Summary: Bitmap-backed distinct-count with a four-conjunct loop invariant (cursor bounds, bitmap-vs-prefix abstraction, counter-vs-set-len abstraction) bridged by three small helper lemmas about `Seq::push`, `Seq::to_set`, and `Set::insert` length.
