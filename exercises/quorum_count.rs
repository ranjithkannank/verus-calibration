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

// Helper lemma: subrange(0, i+1) == subrange(0, i).push(s[i])
proof fn lemma_prefix_extend(s: Seq<NodeId>, i: int)
    requires 0 <= i < s.len(),
    ensures s.subrange(0, i + 1) =~= s.subrange(0, i).push(s[i]),
{
    assert forall|j: int| 0 <= j < s.subrange(0, i + 1).len() implies
        s.subrange(0, i + 1)[j] == s.subrange(0, i).push(s[i])[j]
    by {
        if j < i {
            // s.subrange(0, i+1)[j] == s[j] == s.subrange(0, i)[j] == s.subrange(0,i).push(s[i])[j]
        } else {
            // j == i: s.subrange(0, i+1)[i] == s[i] == s.subrange(0,i).push(s[i])[i]
        }
    };
}

// Helper lemma: s.push(x).to_set() == s.to_set().insert(x)
proof fn lemma_push_to_set(s: Seq<NodeId>, x: NodeId)
    ensures s.push(x).to_set() =~= s.to_set().insert(x),
{
    assert forall|y: NodeId|
        s.push(x).to_set().contains(y) <==> s.to_set().insert(x).contains(y)
    by {
        // Both sides reduce to: s.contains(y) || y == x
        // s.push(x).to_set().contains(y) iff s.push(x).contains(y) (by axiom_seq_to_set_contains)
        // s.push(x).contains(y) iff (s.contains(y) || y == x) (by push definition + contains)
        // s.to_set().insert(x).contains(y) iff (s.to_set().contains(y) || y == x)
        //   iff (s.contains(y) || y == x) (by axiom_seq_to_set_contains)
        //
        // Forward: s.push(x).to_set().contains(y) ==> s.to_set().insert(x).contains(y)
        if s.push(x).to_set().contains(y) {
            assert(s.push(x).contains(y));
            if y == x {
                // s.to_set().insert(x).contains(x) is trivially true
            } else {
                // y != x, so y must be in s
                let j = choose|j: int| 0 <= j < s.push(x).len() && s.push(x)[j] == y;
                if j < s.len() {
                    assert(s[j] == y);
                    assert(s.contains(y));
                } else {
                    // j == s.len(), so s.push(x)[j] == x == y, contradicts y != x
                    assert(s.push(x)[s.len() as int] == x);
                    assert(false);
                }
            }
        }
        // Backward: s.to_set().insert(x).contains(y) ==> s.push(x).to_set().contains(y)
        if s.to_set().insert(x).contains(y) {
            if y == x {
                // s.push(x) has x as its last element
                assert(s.push(x)[s.len() as int] == x);
                assert(s.push(x).contains(x));
            } else {
                // y in s.to_set(), so y in s
                assert(s.to_set().contains(y));
                assert(s.contains(y));
                // y is in s, so in s.push(x) too
                let j = choose|j: int| 0 <= j < s.len() && s[j] == y;
                assert(s.push(x)[j] == s[j]);
                assert(s.push(x).contains(y));
            }
        }
    };
    assert(s.push(x).to_set() =~= s.to_set().insert(x));
}

// Helper lemma: s.to_set() is finite
proof fn lemma_to_set_finite(s: Seq<NodeId>)
    ensures s.to_set().finite(),
{
    // This follows from the vstd axiom: axiom_seq_to_set_finite
}

// Helper lemma: inserting x not in s increases len by 1
proof fn lemma_set_insert_new_len(s: Set<NodeId>, x: NodeId)
    requires s.finite(), !s.contains(x),
    ensures s.insert(x).len() == s.len() + 1,
{
    vstd::set::axiom_set_insert_len(s, x);
}

// Helper lemma: inserting x already in s keeps same set
proof fn lemma_set_insert_existing(s: Set<NodeId>, x: NodeId)
    requires s.finite(), s.contains(x),
    ensures s.insert(x) =~= s,
{
    assert forall|y: NodeId| s.insert(x).contains(y) <==> s.contains(y) by {
        // If y == x: both sides true (s.contains(x) is true)
        // If y != x: s.insert(x).contains(y) iff s.contains(y) (by insert axiom)
    };
}

// Helper lemma: the set of NodeIds strictly below n is finite with exactly n elements
proof fn lemma_range_nodeid_len(n: u32)
    ensures
        Set::<NodeId>::new(|k: NodeId| (k as int) < n as int).finite(),
        Set::<NodeId>::new(|k: NodeId| (k as int) < n as int).len() == n as nat,
    decreases n,
{
    let s = Set::<NodeId>::new(|k: NodeId| (k as int) < n as int);
    if n == 0 {
        // u32 values are always >= 0, so s is empty
        assert(s =~= Set::<NodeId>::empty());
    } else {
        let n1: u32 = (n - 1) as u32;
        let m: NodeId = n1;
        let s1 = Set::<NodeId>::new(|k: NodeId| (k as int) < n1 as int);
        lemma_range_nodeid_len(n1);
        // s = s1 ∪ {m}: k < n iff k < n-1 or k == n-1
        assert(s1.insert(m) =~= s) by {
            assert forall|k: NodeId| s1.insert(m).contains(k) <==> s.contains(k) by {};
        };
        assert(!s1.contains(m)); // m = n-1 is not < n-1
        vstd::set::axiom_set_insert_len(s1, m);
    }
}

pub fn is_byzantine_quorum(voters: &Vec<NodeId>, n: u32) -> (result: bool)
    requires
        n > 0,
        all_in_range(voters@, n),
    ensures
        result == (distinct_count(voters@) >= byzantine_threshold(n)),
{
    // Initialize bitmap: seen[k] == true iff k has been seen in voters[0..i)
    let mut seen: Vec<bool> = vec![false; n as usize];

    // The vec macro gives us these specs:
    assert(seen@.len() == n as nat);
    assert(forall|k: int| 0 <= k < n as int ==> seen@[k] == false);

    // Establish loop-entry invariant (d) at i=0: subrange(0,0) is empty, to_set is empty, len 0
    assert(voters@.subrange(0, 0int) =~= Seq::<NodeId>::empty());
    assert(voters@.subrange(0, 0int).to_set() =~= Set::<NodeId>::empty());

    let mut count: u64 = 0;
    let mut i: usize = 0;

    while i < voters.len()
        invariant
            // (a) cursor and bitmap bounds
            i <= voters@.len(),
            seen@.len() == n as nat,
            n > 0,
            // (b) re-carry the precondition
            all_in_range(voters@, n),
            // (c) bitmap abstraction: seen[k] iff k appears in prefix voters[0..i)
            forall|k: int| 0 <= k < n as int ==>
                (seen@[k] == voters@.subrange(0, i as int).contains(k as NodeId)),
            // (d) counter abstraction: count == |to_set of prefix|
            count as nat == voters@.subrange(0, i as int).to_set().len(),
            // (e) sanity bound
            count as nat <= n as nat,
        decreases voters@.len() - i,
    {
        let v_id: NodeId = voters[i];
        let v: usize = v_id as usize;

        // v < n from all_in_range
        assert((voters@[i as int] as int) < n as int);
        assert(v_id == voters@[i as int]);

        let ghost pref_old = voters@.subrange(0, i as int);
        let ghost v_ghost: NodeId = voters@[i as int];

        // v_ghost as int equals v as int (since v = v_id as usize = v_ghost as usize)
        assert(v_ghost as int == v as int) by {
            assert(v_id == v_ghost);
            assert(v as int == v_id as int);
        };

        if !seen[v] {
            // Case A: new voter – seen[v] == false, so v_ghost not in pref_old

            let ghost count_old: u64 = count;

            // From invariant (c) with k = v_ghost as int:
            assert(seen@[v_ghost as int] == pref_old.contains(v_ghost));
            assert(!pref_old.contains(v_ghost));

            seen.set(v, true);
            count = count + 1;
            i = i + 1;

            let ghost pref_new = voters@.subrange(0, i as int);

            // pref_new == pref_old.push(v_ghost)
            assert(pref_new =~= pref_old.push(v_ghost)) by {
                lemma_prefix_extend(voters@, i as int - 1);
            };

            // Re-establish (c): seen@[k] == pref_new.contains(k as NodeId)
            assert forall|k: int| 0 <= k < n as int implies
                (seen@[k] == pref_new.contains(k as NodeId))
            by {
                if k == v_ghost as int {
                    // seen@[v_ghost as int] is now true (we set it)
                    assert(seen@[v_ghost as int] == true);
                    // pref_new contains v_ghost at index pref_old.len()
                    assert(pref_new.contains(v_ghost)) by {
                        assert(0 <= pref_old.len() < pref_new.len());
                        assert(pref_new[pref_old.len() as int] == v_ghost);
                    };
                } else {
                    // seen@[k] is unchanged (frame property of Vec::set)
                    // pref_new.contains(k as NodeId) iff pref_old.contains(k as NodeId)
                    // because the only new element is v_ghost != k as NodeId
                    assert(seen@[k] == pref_old.contains(k as NodeId));
                    assert(pref_new.contains(k as NodeId) <==> pref_old.contains(k as NodeId)) by {
                        // Forward: if pref_new contains k as NodeId, find the witness
                        if pref_new.contains(k as NodeId) {
                            let j = choose|j: int| 0 <= j < pref_new.len() && pref_new[j] == (k as NodeId);
                            if j < pref_old.len() {
                                // pref_new[j] == pref_old[j] == k as NodeId
                                assert(pref_old[j] == k as NodeId);
                            } else {
                                // j == pref_old.len(), pref_new[j] == v_ghost
                                assert(pref_new[pref_old.len() as int] == v_ghost);
                                // But pref_new[j] == k as NodeId and v_ghost == k as NodeId
                                // contradicts k != v_ghost as int (since k < n <= u32::MAX)
                                assert(v_ghost == k as NodeId);
                                assert(v_ghost as int == k);
                                assert(false);
                            }
                        }
                        // Backward: if pref_old contains k as NodeId, same witness works in pref_new
                        if pref_old.contains(k as NodeId) {
                            let j = choose|j: int| 0 <= j < pref_old.len() && pref_old[j] == (k as NodeId);
                            assert(j < pref_new.len());
                            assert(pref_new[j] == pref_old[j]);
                        }
                    };
                }
            };

            // Re-establish (d): count == pref_new.to_set().len()
            assert(count as nat == pref_new.to_set().len()) by {
                lemma_push_to_set(pref_old, v_ghost);
                // pref_new.to_set() == pref_old.to_set().insert(v_ghost)
                assert(pref_new.to_set() =~= pref_old.to_set().insert(v_ghost));
                // v_ghost not in pref_old.to_set()
                assert(!pref_old.to_set().contains(v_ghost));
                // So inserting increases len by 1
                lemma_to_set_finite(pref_old);
                lemma_set_insert_new_len(pref_old.to_set(), v_ghost);
                assert(pref_old.to_set().insert(v_ghost).len() == pref_old.to_set().len() + 1);
                // count = count_old + 1 = pref_old.to_set().len() + 1 = pref_new.to_set().len()
                assert(count_old as nat == pref_old.to_set().len());
            };

            // (e): count = count_old + 1 <= n
            // all voters are < n, so at most n distinct NodeIds exist in pref_new
            assert(count as nat <= n as nat) by {
                let universe = Set::<NodeId>::new(|k: NodeId| (k as int) < n as int);
                lemma_range_nodeid_len(n);
                assert(pref_new.to_set().subset_of(universe)) by {
                    assert forall|k: NodeId| pref_new.to_set().contains(k) implies
                        universe.contains(k)
                    by {
                        if pref_new.to_set().contains(k) {
                            let j = choose|j: int| 0 <= j < pref_new.len() && pref_new[j] == k;
                            assert(pref_new[j] == voters@[j]);
                            assert((voters@[j] as int) < n as int);
                        }
                    };
                };
                vstd::set_lib::lemma_len_subset::<NodeId>(pref_new.to_set(), universe);
            };

        } else {
            // Case B: duplicate voter – seen[v] == true, so v_ghost already in pref_old

            let ghost count_old: u64 = count;

            // From invariant (c) with k = v_ghost as int:
            assert(seen@[v_ghost as int] == pref_old.contains(v_ghost));
            assert(pref_old.contains(v_ghost));

            i = i + 1;

            let ghost pref_new = voters@.subrange(0, i as int);

            // pref_new == pref_old.push(v_ghost)
            assert(pref_new =~= pref_old.push(v_ghost)) by {
                lemma_prefix_extend(voters@, i as int - 1);
            };

            // Re-establish (c)
            assert forall|k: int| 0 <= k < n as int implies
                (seen@[k] == pref_new.contains(k as NodeId))
            by {
                if k == v_ghost as int {
                    assert(seen@[v_ghost as int] == true);
                    assert(pref_new.contains(v_ghost)) by {
                        assert(0 <= pref_old.len() < pref_new.len());
                        assert(pref_new[pref_old.len() as int] == v_ghost);
                    };
                } else {
                    assert(seen@[k] == pref_old.contains(k as NodeId));
                    assert(pref_new.contains(k as NodeId) <==> pref_old.contains(k as NodeId)) by {
                        if pref_new.contains(k as NodeId) {
                            let j = choose|j: int| 0 <= j < pref_new.len() && pref_new[j] == (k as NodeId);
                            if j < pref_old.len() {
                                assert(pref_old[j] == k as NodeId);
                            } else {
                                assert(pref_new[pref_old.len() as int] == v_ghost);
                                assert(v_ghost == k as NodeId);
                                assert(v_ghost as int == k);
                                assert(false);
                            }
                        }
                        if pref_old.contains(k as NodeId) {
                            let j = choose|j: int| 0 <= j < pref_old.len() && pref_old[j] == (k as NodeId);
                            assert(j < pref_new.len());
                            assert(pref_new[j] == pref_old[j]);
                        }
                    };
                }
            };

            // Re-establish (d): count unchanged because v_ghost was already in the set
            assert(count as nat == pref_new.to_set().len()) by {
                lemma_push_to_set(pref_old, v_ghost);
                assert(pref_new.to_set() =~= pref_old.to_set().insert(v_ghost));
                assert(pref_old.to_set().contains(v_ghost));
                lemma_to_set_finite(pref_old);
                // Inserting existing element keeps same set (same len)
                lemma_set_insert_existing(pref_old.to_set(), v_ghost);
                assert(pref_old.to_set().insert(v_ghost) =~= pref_old.to_set());
                assert(pref_new.to_set().len() == pref_old.to_set().len());
                assert(count_old as nat == pref_old.to_set().len());
            };
        }
    }

    // After loop: prefix(voters.len()) == voters@
    assert(voters@.subrange(0, voters@.len() as int) =~= voters@);
    assert(count as nat == distinct_count(voters@));

    // Compute threshold: 2n/3 + 1
    let threshold: u64 = 2u64 * (n as u64) / 3u64 + 1u64;
    assert(threshold as nat == byzantine_threshold(n));

    count >= threshold
}

} // verus!
