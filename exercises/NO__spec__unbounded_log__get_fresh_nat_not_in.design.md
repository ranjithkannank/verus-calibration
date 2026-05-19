# Design: NO__spec__unbounded_log__get_fresh_nat_not_in

External-validity task from VeruSAGE-Bench (upstream prefix `NO` =
node-replication). Operator-authored design note — neutral by
construction; this batch is a methodology probe and the design note
deliberately withholds Verus tooling hints, lemma names, and
proof-structure suggestions. Largest task in this batch (3768 B);
included as a stress test on size.

## Frozen obligation

```
pub proof fn get_fresh_nat_not_in(reqs: Set<ReqId>, combiner: Map<NodeId, CombinerState>)
    requires
        reqs.finite(),
        combiner.dom().finite(),
    ensures
        !reqs.contains(get_fresh_nat(reqs, combiner)),
        combiner_request_id_fresh(combiner, get_fresh_nat(reqs, combiner)),
```

A pure ghost-level existence lemma: there is a `nat` not in `reqs`
and "fresh" with respect to the in-flight combiner state. The file
declares:

- A `ghost enum CombinerState` with five variants and a `queued_ops`
  method.
- A `spec fn combiner_request_ids` that recursively collects all
  in-flight request ids across the combiner map, with a `via_fn`
  decreases witness.
- A `spec fn combiner_request_id_fresh` predicate.
- Three operator-axiomatized helper proof fns
  (`combiner_request_ids_not_contains`,
  `combiner_request_ids_finite`, `element_outside_set`) marked with
  the scaffold baseline's verification-bypass markers. These are
  trust-boundary helpers; do not introduce new ones.
- A `closed spec fn get_fresh_nat` that chooses such a fresh nat.
- The target lemma above.

The body is empty.

## Forbidden (audited at commit time)

- Adding any verification-bypass tokens beyond what the scaffold
  baseline already carries.
- Modifying the `requires` or `ensures` clauses on
  `get_fresh_nat_not_in`, or the bodies of the spec fns / axiomatized
  helpers.
- Removing `fn main() {}`, the `verus!{}` wrapper, or the
  `pub type ReqId=nat;` / `NodeId` / `LogIdx` aliases.

## Sub-tasks

1. Run `verus exercises/NO__spec__unbounded_log__get_fresh_nat_not_in.rs --crate-type=lib`
   on the unmodified scaffold. Capture the exact rejection messages.
2. Attempt the smallest possible body that could close the
   obligation. Run verus.
3. If step 2 fails, read the rejection and iterate. Standard
   per-attempt protocol from AGENTS.md applies (one attempt per
   iteration, log each attempt, escalate after 3 consecutive
   failures on the same obligation).
