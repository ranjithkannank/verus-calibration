# Design: VE__utils__init_vec_u8

External-validity task from VeruSAGE-Bench (upstream prefix `VE` =
vest, the Verus-verified serializer). Operator-authored design note —
the agent's THINK phase is skipped because the task arrives spec-only
with a body that just needs verification annotations.

## Frozen obligation

```
pub exec fn init_vec_u8(n: usize) -> (res: Vec<u8>)
    ensures
        res@.len() == n,
```

The body — initialize `ret: Vec<u8>` of length `n` with all zeros via
a counted `while` loop — is already present in the upstream task
file. The shape of the algorithm is fixed; the obligation is to make
verus accept it.

## Why verus currently rejects the file as-is

Verus requires every `while` loop to declare an `invariant` block and
either a `decreases` clause or an explicit
`#[verifier::exec_allows_no_decreases_clause]` attribute. The
upstream task has neither. Without the invariant, verus cannot prove
`res@.len() == n` at the function's exit; without `decreases`, the
loop is rejected as non-terminating from the verifier's perspective.

## Suggested order of operations

1. Run `verus exercises/VE__utils__init_vec_u8.rs --crate-type=lib`
   once unmodified to see the exact rejection messages. There may be
   more than one (missing invariant + missing decreases + possibly a
   post-loop obligation about `ret@.len()`).
2. Add a minimal `invariant` block carrying the two facts the
   postcondition needs: a bound on the loop counter, and the
   relationship between the counter and `ret@.len()`. Two conjuncts
   should be enough.
3. Add a `decreases` clause that strictly decreases each iteration.
   `n - i` is the natural measure.
4. If verus still complains after the loop closes, a single
   `assert(...)` inside the loop body that names the relevant index
   may be needed to nudge `Vec::push`'s spec for the SMT solver. See
   the bounded_log discovered-pattern note in AGENTS.md for the
   precedent — verus sometimes wants the new element's index named
   explicitly after a `push`.

## Forbidden (audited at commit time)

- Adding `assume(`, `external_body`, `unreachable!()`, `panic!(`, or
  `assume_specification` anywhere in the file.
- Modifying the `ensures` clause.
- Restructuring the loop into a different shape (e.g. recursive
  helper, `Vec::from_elem`). The upstream task's algorithm is the
  algorithm under test.

## Sub-tasks

1. Run verus on the unmodified file. Capture the exact errors.
2. Add the `invariant` block — two conjuncts as described above.
3. Add the `decreases` clause.
4. If the post-loop or push frame still fails, add the indexing
   assert inside the loop body.

Expectation: 1-2 attempts. The loop-invariant pattern here is the
same family as `binary_search` (already in the playbook).
