// Exercise 3: verified Byzantine quorum check.
//
// Given a Vec<NodeId> of voters (may contain duplicates) and a total node
// count n, return true iff the distinct voters constitute a Byzantine
// quorum: at least (2n/3 + 1) distinct ids, all within range 0..n.
//
// The interesting obligation is relating an algorithmic distinct-count
// (which must walk the Vec) to the mathematical Set::len(). This is the
// concrete-matches-abstract gap that scales the worst on bigger projects.
//
// The spec below is FROZEN. Iteration cap: 20. See AGENTS.md.

use vstd::prelude::*;

verus! {

pub type NodeId = u32;

pub open spec fn all_in_range(voters: Seq<NodeId>, n: u32) -> bool {
    forall|i: int| 0 <= i < voters.len() ==> (voters[i] as int) < (n as int)
}

pub open spec fn distinct_count(voters: Seq<NodeId>) -> nat {
    voters.to_set().len()
}

pub open spec fn byzantine_threshold(n: u32) -> nat {
    ((2 * (n as nat)) / (3 as nat) + (1 as nat)) as nat
}

pub fn is_byzantine_quorum(voters: &Vec<NodeId>, n: u32) -> (result: bool)
    requires
        n > 0,
        all_in_range(voters@, n),
    ensures
        result == (distinct_count(voters@) >= byzantine_threshold(n)),
{
    // TODO(loop): fill in. Do not modify any spec above.
    //
    // Stretch (only if the above verifies cleanly):
    //   prove `quorum_intersection_lemma`: any two byzantine quorums share
    //   at least one node id when n > 3f and there are at most f byzantine
    //   nodes. Add it as a separate `proof fn` below — do NOT change
    //   `is_byzantine_quorum`'s spec to include it.
    unimplemented!()
}

} // verus!
