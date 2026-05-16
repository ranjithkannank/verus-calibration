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
    // Step-1 skeleton per design: bitmap, early returns, threshold compare.
    // Minimal invariants only — enough for indexing to typecheck. The
    // proof-carrying invariants (b)/(c)/(d) come in later attempts.
    let mut seen: Vec<bool> = vec![false; n as usize];
    let mut i: usize = 0;
    while i < qc.votes.len()
        invariant
            i <= qc.votes@.len(),
            seen@.len() == n as nat,
            n > 0,
        decreases qc.votes@.len() - i,
    {
        let v_id: NodeId = qc.votes[i].voter;
        if v_id >= n {
            // witness for !all_voters_in_range
            return false;
        }
        let v: usize = v_id as usize;
        if seen[v] {
            // witness for !voters_distinct
            return false;
        }
        seen.set(v, true);
        i = i + 1;
    }
    // |voters(qc)| == qc.votes.len() under distinct + in-range (helper lemma in
    // later attempt). For now just emit the comparison — verification will
    // fail on the postcondition's has_quorum part; that's expected for step 1.
    let votes_len: u64 = qc.votes.len() as u64;
    let threshold: u64 = 2u64 * (n as u64) / 3u64 + 1u64;
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
