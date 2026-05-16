// Exercise 4: verified Byzantine quorum certificate.
//
// A quorum certificate is the building block every BFT consensus
// protocol uses: a bundle of signed votes from a quorum of nodes,
// witnessing agreement on some proposal. Real protocols (PBFT,
// HotStuff, Tendermint, BFT-SMaRt) all carry these around. The library
// is genuinely missing in publicly-verified form.
//
// This exercise verifies TWO obligations:
//
// 1. `verify_qc_structure` (exec): given a QuorumCert and the total
//    node count, return true iff the certificate has distinct voters
//    in range and meets the Byzantine threshold (2n/3 + 1). The
//    *cryptographic* validity of signatures is treated as an
//    uninterpreted predicate — verifying the crypto layer is out of
//    scope for this exercise; the goal is the structural reasoning a
//    consensus protocol layers on top of it.
//
// 2. `lemma_qc_has_honest_voter` (proof): the core safety property of
//    quorum certificates. Given a valid QC over n nodes, and a
//    Byzantine set of size at most f where n > 3f, the QC's voter set
//    contains at least one node that is NOT Byzantine. This is the
//    pigeonhole step every BFT protocol relies on.
//
// The crypto layer is deliberately abstract. `signature_valid` is an
// `uninterp spec fn` — no body, no exec wrapper, no trust boundary in
// this file. A real user of this library would connect it to a vetted
// crypto implementation (ring, libsignal, NaCl) via a thin exec
// wrapper outside this repo whose postcondition matches the spec
// predicate. Our job here is the BFT-layer reasoning, which is what
// consensus protocols actually want from a QC library.
//
// The spec below is FROZEN. Iteration cap: 20. See AGENTS.md.

use vstd::prelude::*;

verus! {

// --- Types ------------------------------------------------------------------

pub type NodeId = u32;
pub type Hash = u64;
pub type PubKey = [u8; 32];
pub type Signature = [u8; 64];

pub struct SignedVote {
    pub voter: NodeId,
    pub sig: Signature,
}

pub struct QuorumCert {
    pub proposal: Hash,
    pub votes: Vec<SignedVote>,
}

// --- Cryptographic abstractions (uninterpreted) -----------------------------
//
// These have no body. Do NOT add one — they are deliberately opaque. A
// real deployment connects `signature_valid` to a vetted crypto library
// outside this repo.

pub uninterp spec fn pk_of(node: NodeId) -> PubKey;

pub uninterp spec fn signature_valid(pk: PubKey, msg: Hash, sig: Signature) -> bool;

// --- Helper spec predicates -------------------------------------------------

pub open spec fn all_signatures_valid(qc: QuorumCert) -> bool {
    forall|i: int|
        0 <= i < qc.votes@.len() ==>
            signature_valid(pk_of(qc.votes@[i].voter), qc.proposal, qc.votes@[i].sig)
}

pub open spec fn voters(qc: QuorumCert) -> Set<NodeId> {
    Set::new(|n: NodeId|
        exists|i: int|
            0 <= i < qc.votes@.len() && qc.votes@[i].voter == n)
}

pub open spec fn voters_distinct(qc: QuorumCert) -> bool {
    forall|i: int, j: int|
        0 <= i < j < qc.votes@.len() ==>
            qc.votes@[i].voter != qc.votes@[j].voter
}

pub open spec fn byzantine_threshold(n: u32) -> nat {
    ((2 * (n as nat)) / (3 as nat) + (1 as nat)) as nat
}

pub open spec fn all_voters_in_range(qc: QuorumCert, n: u32) -> bool {
    forall|i: int|
        0 <= i < qc.votes@.len() ==>
            (qc.votes@[i].voter as int) < (n as int)
}

pub open spec fn has_quorum(qc: QuorumCert, n: u32) -> bool {
    voters(qc).len() >= byzantine_threshold(n)
}

pub open spec fn is_valid_qc(qc: QuorumCert, n: u32) -> bool {
    &&& all_signatures_valid(qc)
    &&& voters_distinct(qc)
    &&& all_voters_in_range(qc, n)
    &&& has_quorum(qc, n)
}

// --- Internal helpers (not part of the frozen spec) -------------------------

// The Seq projection of a QC's votes onto their voter NodeIds.
spec fn voter_seq(qc: QuorumCert) -> Seq<NodeId> {
    Seq::new(qc.votes@.len(), |i: int| qc.votes@[i].voter)
}

// --- Helper lemmas (lifted from quorum_count.rs) ---------------------------

// Helper lemma: s.push(x).to_set() == s.to_set().insert(x)
proof fn lemma_push_to_set(s: Seq<NodeId>, x: NodeId)
    ensures s.push(x).to_set() =~= s.to_set().insert(x),
{
    assert forall|y: NodeId|
        s.push(x).to_set().contains(y) <==> s.to_set().insert(x).contains(y)
    by {
        if s.push(x).to_set().contains(y) {
            assert(s.push(x).contains(y));
            if y == x {
            } else {
                let j = choose|j: int| 0 <= j < s.push(x).len() && s.push(x)[j] == y;
                if j < s.len() {
                    assert(s[j] == y);
                    assert(s.contains(y));
                } else {
                    assert(s.push(x)[s.len() as int] == x);
                    assert(false);
                }
            }
        }
        if s.to_set().insert(x).contains(y) {
            if y == x {
                assert(s.push(x)[s.len() as int] == x);
                assert(s.push(x).contains(x));
            } else {
                assert(s.to_set().contains(y));
                assert(s.contains(y));
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
}

// Helper lemma: inserting x not in s increases len by 1
proof fn lemma_set_insert_new_len(s: Set<NodeId>, x: NodeId)
    requires s.finite(), !s.contains(x),
    ensures s.insert(x).len() == s.len() + 1,
{
    vstd::set::axiom_set_insert_len(s, x);
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
        assert(s =~= Set::<NodeId>::empty());
    } else {
        let n1: u32 = (n - 1) as u32;
        let m: NodeId = n1;
        let s1 = Set::<NodeId>::new(|k: NodeId| (k as int) < n1 as int);
        lemma_range_nodeid_len(n1);
        assert(s1.insert(m) =~= s) by {
            assert forall|k: NodeId| s1.insert(m).contains(k) <==> s.contains(k) by {};
        };
        assert(!s1.contains(m));
        vstd::set::axiom_set_insert_len(s1, m);
    }
}

// Helper lemma: distinct seq's to_set has the same length as the seq.
proof fn lemma_distinct_seq_to_set_len(s: Seq<NodeId>)
    requires
        forall|i: int, j: int| 0 <= i < j < s.len() ==> s[i] != s[j],
    ensures
        s.to_set().finite(),
        s.to_set().len() == s.len(),
    decreases s.len(),
{
    if s.len() == 0 {
        assert(s.to_set() =~= Set::<NodeId>::empty()) by {
            assert forall|y: NodeId| !s.to_set().contains(y) by {
                if s.to_set().contains(y) {
                    let j = choose|j: int| 0 <= j < s.len() && s[j] == y;
                    assert(false);
                }
            };
        };
    } else {
        let last_idx = (s.len() - 1) as int;
        let prefix = s.subrange(0, last_idx);
        let last = s[last_idx];
        // s == prefix.push(last)
        assert(s =~= prefix.push(last)) by {
            assert forall|k: int| 0 <= k < s.len() implies s[k] == prefix.push(last)[k] by {
                if k < last_idx {
                    assert(prefix[k] == s[k]);
                } else {
                    assert(k == last_idx);
                }
            };
        };
        // prefix is also distinct
        assert(forall|i: int, j: int| 0 <= i < j < prefix.len() ==> prefix[i] != prefix[j]) by {
            assert forall|i: int, j: int| 0 <= i < j < prefix.len() implies prefix[i] != prefix[j] by {
                assert(prefix[i] == s[i]);
                assert(prefix[j] == s[j]);
            };
        };
        lemma_distinct_seq_to_set_len(prefix);
        // last is not in prefix
        assert(!prefix.to_set().contains(last)) by {
            if prefix.to_set().contains(last) {
                let j = choose|j: int| 0 <= j < prefix.len() && prefix[j] == last;
                assert(s[j] == last);
                assert(s[last_idx] == last);
                assert(j < last_idx);
                assert(s[j] != s[last_idx]);
                assert(false);
            }
        };
        lemma_push_to_set(prefix, last);
        lemma_set_insert_new_len(prefix.to_set(), last);
        assert(s.to_set() =~= prefix.to_set().insert(last));
    }
}

// Helper lemma: voters(qc) is exactly the to_set of voter_seq(qc).
proof fn lemma_voters_as_to_set(qc: QuorumCert)
    ensures
        voters(qc) =~= voter_seq(qc).to_set(),
{
    let vs = voter_seq(qc);
    assert forall|x: NodeId| voters(qc).contains(x) <==> vs.to_set().contains(x) by {
        // voters(qc).contains(x) <==> exists|i| 0 <= i < qc.votes@.len() && qc.votes@[i].voter == x
        // vs.to_set().contains(x) <==> exists|i| 0 <= i < vs.len() && vs[i] == x
        // and vs.len() == qc.votes@.len(), vs[i] == qc.votes@[i].voter
        if voters(qc).contains(x) {
            let i = choose|i: int| 0 <= i < qc.votes@.len() && qc.votes@[i].voter == x;
            assert(vs[i] == qc.votes@[i].voter);
            assert(vs[i] == x);
            assert(vs.contains(x));
        }
        if vs.to_set().contains(x) {
            let i = choose|i: int| 0 <= i < vs.len() && vs[i] == x;
            assert(vs[i] == qc.votes@[i].voter);
        }
    };
}

// Bridge: distinct voters ⇒ |voters(qc)| == qc.votes.len()
proof fn lemma_distinct_voters_len(qc: QuorumCert)
    requires voters_distinct(qc),
    ensures
        voters(qc).finite(),
        voters(qc).len() == qc.votes@.len(),
{
    let vs = voter_seq(qc);
    // vs is distinct because voters_distinct(qc)
    assert(forall|i: int, j: int| 0 <= i < j < vs.len() ==> vs[i] != vs[j]) by {
        assert forall|i: int, j: int| 0 <= i < j < vs.len() implies vs[i] != vs[j] by {
            assert(vs[i] == qc.votes@[i].voter);
            assert(vs[j] == qc.votes@[j].voter);
        };
    };
    lemma_distinct_seq_to_set_len(vs);
    lemma_voters_as_to_set(qc);
    assert(vs.len() == qc.votes@.len());
}

// --- Obligation 1: structural runtime check ---------------------------------
//
// Verify the parts of `is_valid_qc` that do NOT depend on the crypto
// abstraction: distinct voters, voters all in range [0, n), threshold
// met. Signature validity is checked by the caller via a separate
// crypto wrapper (out of scope here).

pub fn verify_qc_structure(qc: &QuorumCert, n: u32) -> (result: bool)
    requires
        n > 0,
    ensures
        result == (voters_distinct(*qc) && all_voters_in_range(*qc, n) && has_quorum(*qc, n)),
{
    // Step-3 per design: add invariant (d) bitmap abstraction and
    // re-establish in the fall-through branch using seen.set frame +
    // existential reasoning. The duplicate early-return and the final
    // threshold step still fail; those are steps 5 and 6.
    let mut seen: Vec<bool> = vec![false; n as usize];

    // Initial bitmap state from vec! macro
    assert(seen@.len() == n as nat);
    assert(forall|k: int| 0 <= k < n as int ==> seen@[k] == false);

    let mut i: usize = 0;
    while i < qc.votes.len()
        invariant
            i <= qc.votes@.len(),
            seen@.len() == n as nat,
            n > 0,
            // (b) in-range prefix
            forall|j: int| 0 <= j < i as int ==>
                (#[trigger] qc.votes@[j].voter as int) < n as int,
            // (c) pairwise distinct voters in the prefix
            forall|j: int, k: int| 0 <= j < k < i as int ==>
                qc.votes@[j].voter != qc.votes@[k].voter,
            // (d) bitmap abstraction: seen[k] iff some prefix voter equals k
            forall|k: int| 0 <= k < n as int ==>
                (#[trigger] seen@[k]) == (exists|j: int|
                    0 <= j < i as int && (#[trigger] qc.votes@[j].voter as int) == k),
        decreases qc.votes@.len() - i,
    {
        let v_id: NodeId = qc.votes[i].voter;
        if v_id >= n {
            // witness for !all_voters_in_range at index i
            assert(qc.votes@[i as int].voter == v_id);
            assert((qc.votes@[i as int].voter as int) >= n as int);
            assert(!all_voters_in_range(*qc, n));
            return false;
        }
        let v: usize = v_id as usize;
        if seen[v] {
            // witness for !voters_distinct -- next attempt
            return false;
        }
        // Fall-through path: v_id < n, seen[v] == false.
        let ghost v_ghost: NodeId = qc.votes@[i as int].voter;
        let ghost old_i: int = i as int;
        assert(v_ghost == v_id);
        assert(v_ghost as int == v as int);

        // From invariant (d) at k = v_ghost as int: since seen[v] == false,
        // no prior index has voter == v_ghost. We capture that now while
        // both invariants (c) and (d) still refer to the old `i`.
        assert(seen@[v_ghost as int] == false);
        assert(forall|j: int|
            0 <= j < old_i ==> qc.votes@[j].voter as int != v_ghost as int)
        by {
            // Contrapositive of (d) at k = v_ghost as int.
            assert(!(exists|j: int|
                0 <= j < old_i && qc.votes@[j].voter as int == v_ghost as int));
        };

        seen.set(v, true);
        i = i + 1;

        // Re-establish (c) at the new i = old_i + 1.
        assert forall|j: int, k: int| 0 <= j < k < i as int implies
            qc.votes@[j].voter != qc.votes@[k].voter
        by {
            if k < old_i {
                // Both indices in old prefix: covered by old (c).
            } else {
                // k == old_i, so qc.votes@[k].voter == v_ghost.
                assert(qc.votes@[old_i].voter == v_ghost);
                assert(qc.votes@[j].voter as int != v_ghost as int);
            }
        };

        // Re-establish (d) at the new i = old_i + 1.
        assert forall|k: int| 0 <= k < n as int implies
            (#[trigger] seen@[k]) == (exists|j: int|
                0 <= j < i as int && (#[trigger] qc.votes@[j].voter as int) == k)
        by {
            if k == v_ghost as int {
                // seen@[k] is now true (we just set it at index v == v_ghost as int).
                assert(seen@[v_ghost as int] == true);
                // Existential is witnessed by j == old_i.
                assert(0 <= old_i < i as int);
                assert(qc.votes@[old_i].voter as int == k);
            } else {
                // seen@[k] unchanged by Vec::set frame (since k != v as int).
                // The new existential range only differs by j == old_i, whose voter is v_ghost.
                assert(qc.votes@[old_i].voter as int == v_ghost as int);
                assert(v_ghost as int != k);
                // Backward: an exists in the new range yields one in the old range.
                if exists|j: int| 0 <= j < i as int && qc.votes@[j].voter as int == k {
                    let j0 = choose|j: int| 0 <= j < i as int && qc.votes@[j].voter as int == k;
                    if j0 == old_i {
                        assert(qc.votes@[j0].voter as int == v_ghost as int);
                        assert(false);
                    }
                    assert(0 <= j0 < old_i);
                }
                // Forward direction is automatic: any j < old_i is also j < i.
            }
        };
    }
    // At loop exit: invariant (b) is all_voters_in_range, (c) is voters_distinct.
    // Bridge: voters(qc).len() == qc.votes.len() under voters_distinct.
    proof {
        assert(voters_distinct(*qc));
        assert(all_voters_in_range(*qc, n));
        lemma_distinct_voters_len(*qc);
        assert(voters(*qc).len() == qc.votes@.len());
    }

    let votes_len: u64 = qc.votes.len() as u64;
    let threshold: u64 = 2u64 * (n as u64) / 3u64 + 1u64;
    assert(threshold as nat == byzantine_threshold(n));
    assert(votes_len as nat == qc.votes@.len());
    assert((votes_len >= threshold) == (voters(*qc).len() >= byzantine_threshold(n)));
    votes_len >= threshold
}

// --- Obligation 2: safety lemma ---------------------------------------------
//
// If a QC is valid over n nodes, and the Byzantine set has size strictly
// less than n/3 (i.e., n > 3f for f = byzantine.len()), then the QC's
// voter set contains at least one honest voter — a node that signed but
// is NOT in the Byzantine set.
//
// This is the pigeonhole step that makes quorum certificates useful for
// BFT: any two valid QCs intersect in at least one honest voter, which
// is what prevents a Byzantine quorum of liars from producing
// conflicting certificates.

pub proof fn lemma_qc_has_honest_voter(qc: QuorumCert, n: u32, byzantine: Set<NodeId>)
    requires
        n > 0,
        is_valid_qc(qc, n),
        byzantine.finite(),
        byzantine.len() * 3 < n as nat,
        forall|b: NodeId| byzantine.contains(b) ==> (b as int) < (n as int),
    ensures
        exists|honest: NodeId| voters(qc).contains(honest) && !byzantine.contains(honest),
{
    // TODO(loop): proof. The argument is pigeonhole on cardinality:
    //   |voters(qc)| >= 2n/3 + 1   (by has_quorum)
    //   |byzantine|  <= n/3 - 1    (by the strict inequality)
    //   voters(qc) ⊆ {k : k < n}   (by all_voters_in_range)
    // So voters(qc) and the non-Byzantine subset of {k : k < n} must
    // intersect; pick any element of that intersection as `honest`.
    //
    // Useful vstd helpers:
    //   vstd::set::axiom_set_insert_len
    //   vstd::set_lib::lemma_len_subset
    //   The pattern from quorum_count's lemma_range_nodeid_len is a
    //   reasonable starting point for the size-of-{k : k < n} step.
    //
    // Do not modify any spec above.
}

} // verus!
