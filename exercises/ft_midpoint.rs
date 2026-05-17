// Exercise 5: verified fault-tolerant midpoint (Schmid-Schossmaier 2001).
//
// Given n sensor readings where at most f are Byzantine and n >= 2f+1,
// produce a single output value that lies inside the range agreed on by
// the correct sensors. This is the simplest fault-tolerant aggregation
// primitive used in safety-critical sensor systems — three accelerometers
// report a number, the flight computer picks one. If at most one of the
// three is faulty, the picked value is guaranteed bracketed by readings
// from sensors that aren't.
//
// Reference: Ulrich Schmid and Klaus Schossmaier, "Interval-based
// clock synchronization," Real-Time Systems, 2001. The algorithm and
// safety property predate the paper (Marzullo 1984, Cristian 1989),
// but Schmid-Schossmaier give the cleanest treatment of the
// single-value variant we verify here.
//
// What the spec says:
//
//   - `readings: Vec<Reading>` is the input.
//   - `f: u32` is the maximum number of Byzantine sensors. The
//     precondition asserts `readings.len() >= 2*f + 1`.
//   - An uninterpreted per-index predicate `correct_at(i)` designates
//     which indices are correct. The implementer does NOT see this in
//     exec — only in the proof. A real deployment supplies a
//     correctness model from outside this module.
//   - The precondition also asserts at least `len - f` indices in
//     range are correct (i.e. at most `f` are Byzantine).
//   - The postcondition is the safety property: at least one correct
//     reading is <= result AND at least one correct reading is >=
//     result. Together these say the result is bracketed by correct
//     sensor values.
//
// Why this safety property:
//
//   The two-bracketing form is logically equivalent (given the
//   correct-count precondition guarantees at least one correct
//   reading exists) to "the result lies in the interval [min(correct),
//   max(correct)]." We use the existence form because it expresses in
//   one quantifier each side and avoids defining `min_correct` and
//   `max_correct` as separate spec functions.
//
// The spec below is FROZEN. Iteration cap: 20. See AGENTS.md.

use vstd::prelude::*;
use vstd::set_lib::*;

verus! {

pub type Reading = i64;

// --- Trust boundary (uninterpreted) -----------------------------------------
//
// `correct_at(i)` is true iff sensor at index i is a correct (non-Byzantine)
// sensor. The implementer must not provide a body. A real deployment
// supplies the correctness model as a ghost predicate from outside this
// module.

pub uninterp spec fn correct_at(i: int) -> bool;

// --- Spec helpers -----------------------------------------------------------

pub open spec fn correct_indices(n: nat) -> Set<int> {
    Set::new(|i: int| 0 <= i < n as int && correct_at(i))
}

pub open spec fn some_correct_le(readings: Seq<Reading>, v: Reading) -> bool {
    exists|i: int|
        0 <= i < readings.len() && correct_at(i) && readings[i] <= v
}

pub open spec fn some_correct_ge(readings: Seq<Reading>, v: Reading) -> bool {
    exists|i: int|
        0 <= i < readings.len() && correct_at(i) && readings[i] >= v
}

// --- Proof-only spec helpers ------------------------------------------------

spec fn le_set(readings: Seq<Reading>, v: Reading) -> Set<int> {
    Set::new(|i: int| 0 <= i < readings.len() && readings[i] <= v)
}

spec fn ge_set(readings: Seq<Reading>, v: Reading) -> Set<int> {
    Set::new(|i: int| 0 <= i < readings.len() && readings[i] >= v)
}

spec fn le_set_upto(readings: Seq<Reading>, v: Reading, m: int) -> Set<int> {
    Set::new(|i: int| 0 <= i < m && i < readings.len() && readings[i] <= v)
}

spec fn ge_set_upto(readings: Seq<Reading>, v: Reading, m: int) -> Set<int> {
    Set::new(|i: int| 0 <= i < m && i < readings.len() && readings[i] >= v)
}

// --- Subset / finiteness lemmas ---------------------------------------------

proof fn lemma_le_set_in_range(readings: Seq<Reading>, v: Reading)
    ensures
        le_set(readings, v).subset_of(set_int_range(0, readings.len() as int)),
        le_set(readings, v).finite(),
        le_set(readings, v).len() <= readings.len() as nat,
{
    lemma_int_range(0, readings.len() as int);
    assert(le_set(readings, v).subset_of(set_int_range(0, readings.len() as int)));
    lemma_len_subset(le_set(readings, v), set_int_range(0, readings.len() as int));
}

proof fn lemma_ge_set_in_range(readings: Seq<Reading>, v: Reading)
    ensures
        ge_set(readings, v).subset_of(set_int_range(0, readings.len() as int)),
        ge_set(readings, v).finite(),
        ge_set(readings, v).len() <= readings.len() as nat,
{
    lemma_int_range(0, readings.len() as int);
    assert(ge_set(readings, v).subset_of(set_int_range(0, readings.len() as int)));
    lemma_len_subset(ge_set(readings, v), set_int_range(0, readings.len() as int));
}

proof fn lemma_correct_indices_in_range(n: nat)
    ensures
        correct_indices(n).subset_of(set_int_range(0, n as int)),
        correct_indices(n).finite(),
        correct_indices(n).len() <= n,
{
    lemma_int_range(0, n as int);
    assert(correct_indices(n).subset_of(set_int_range(0, n as int)));
    lemma_len_subset(correct_indices(n), set_int_range(0, n as int));
}

proof fn lemma_le_set_upto_in_range(readings: Seq<Reading>, v: Reading, m: int)
    requires 0 <= m <= readings.len(),
    ensures
        le_set_upto(readings, v, m).subset_of(set_int_range(0, m)),
        le_set_upto(readings, v, m).finite(),
        le_set_upto(readings, v, m).len() <= m as nat,
{
    lemma_int_range(0, m);
    assert(le_set_upto(readings, v, m).subset_of(set_int_range(0, m)));
    lemma_len_subset(le_set_upto(readings, v, m), set_int_range(0, m));
}

proof fn lemma_ge_set_upto_in_range(readings: Seq<Reading>, v: Reading, m: int)
    requires 0 <= m <= readings.len(),
    ensures
        ge_set_upto(readings, v, m).subset_of(set_int_range(0, m)),
        ge_set_upto(readings, v, m).finite(),
        ge_set_upto(readings, v, m).len() <= m as nat,
{
    lemma_int_range(0, m);
    assert(ge_set_upto(readings, v, m).subset_of(set_int_range(0, m)));
    lemma_len_subset(ge_set_upto(readings, v, m), set_int_range(0, m));
}

// --- L3: prefix-set extension lemmas ---------------------------------------

proof fn lemma_le_set_upto_extend(readings: Seq<Reading>, v: Reading, i: int)
    requires 0 <= i < readings.len(),
    ensures
        readings[i] <= v ==>
            le_set_upto(readings, v, i + 1)
                =~= le_set_upto(readings, v, i).insert(i),
        readings[i] > v ==>
            le_set_upto(readings, v, i + 1)
                =~= le_set_upto(readings, v, i),
{
}

proof fn lemma_ge_set_upto_extend(readings: Seq<Reading>, v: Reading, i: int)
    requires 0 <= i < readings.len(),
    ensures
        readings[i] >= v ==>
            ge_set_upto(readings, v, i + 1)
                =~= ge_set_upto(readings, v, i).insert(i),
        readings[i] < v ==>
            ge_set_upto(readings, v, i + 1)
                =~= ge_set_upto(readings, v, i),
{
}

// --- Counting helpers -------------------------------------------------------

fn count_le(readings: &Vec<Reading>, v: Reading) -> (c: u32)
    requires
        readings.len() <= u32::MAX as nat,
    ensures
        c as nat == le_set(readings@, v).len(),
        le_set(readings@, v).finite(),
        c as nat <= readings.len() as nat,
{
    let mut c: u32 = 0;
    let mut i: usize = 0;
    proof {
        assert(le_set_upto(readings@, v, 0) =~= Set::<int>::empty());
    }
    while i < readings.len()
        invariant
            0 <= i as int <= readings@.len() as int,
            readings.len() <= u32::MAX as nat,
            c as nat == le_set_upto(readings@, v, i as int).len(),
            le_set_upto(readings@, v, i as int).finite(),
            c as nat <= i as nat,
        decreases readings.len() - i,
    {
        let r = readings[i];
        proof {
            lemma_le_set_upto_extend(readings@, v, i as int);
            lemma_le_set_upto_in_range(readings@, v, (i + 1) as int);
        }
        if r <= v {
            // le_set_upto(_, v, i+1) = le_set_upto(_, v, i).insert(i)
            // i was not previously in the set (since i not < i), so len += 1
            proof {
                assert(!le_set_upto(readings@, v, i as int).contains(i as int));
            }
            c = c + 1;
        }
        i = i + 1;
    }
    proof {
        assert(le_set_upto(readings@, v, readings@.len() as int) =~= le_set(readings@, v));
        lemma_le_set_in_range(readings@, v);
    }
    c
}

fn count_ge(readings: &Vec<Reading>, v: Reading) -> (c: u32)
    requires
        readings.len() <= u32::MAX as nat,
    ensures
        c as nat == ge_set(readings@, v).len(),
        ge_set(readings@, v).finite(),
        c as nat <= readings.len() as nat,
{
    let mut c: u32 = 0;
    let mut i: usize = 0;
    proof {
        assert(ge_set_upto(readings@, v, 0) =~= Set::<int>::empty());
    }
    while i < readings.len()
        invariant
            0 <= i as int <= readings@.len() as int,
            readings.len() <= u32::MAX as nat,
            c as nat == ge_set_upto(readings@, v, i as int).len(),
            ge_set_upto(readings@, v, i as int).finite(),
            c as nat <= i as nat,
        decreases readings.len() - i,
    {
        let r = readings[i];
        proof {
            lemma_ge_set_upto_extend(readings@, v, i as int);
            lemma_ge_set_upto_in_range(readings@, v, (i + 1) as int);
        }
        if r >= v {
            proof {
                assert(!ge_set_upto(readings@, v, i as int).contains(i as int));
            }
            c = c + 1;
        }
        i = i + 1;
    }
    proof {
        assert(ge_set_upto(readings@, v, readings@.len() as int) =~= ge_set(readings@, v));
        lemma_ge_set_in_range(readings@, v);
    }
    c
}

// --- L4: pigeonhole lemmas --------------------------------------------------
//
// If at least `n - f` correct indices exist and at least `f + 1` indices
// satisfy the reading-comparison, their intersection is non-empty (since
// both sets live in [0, n) and (n - f) + (f + 1) > n). The non-empty
// intersection witness directly proves `some_correct_le` / `some_correct_ge`.

proof fn lemma_pigeonhole_le(readings: Seq<Reading>, v: Reading, f: nat)
    requires
        correct_indices(readings.len()).len() >= readings.len() - f,
        le_set(readings, v).len() >= f + 1,
    ensures
        some_correct_le(readings, v),
{
    let n: int = readings.len() as int;
    let a: Set<int> = correct_indices(readings.len());
    let b: Set<int> = le_set(readings, v);

    lemma_correct_indices_in_range(readings.len());
    lemma_le_set_in_range(readings, v);
    lemma_int_range(0, n);

    // a + b is a subset of [0, n), hence (a + b).len() <= n.
    assert((a + b).subset_of(set_int_range(0, n))) by {
        assert forall|x: int| (a + b).contains(x) implies set_int_range(0, n).contains(x) by {
            assert(a.contains(x) || b.contains(x));
        }
    }
    lemma_len_subset(a + b, set_int_range(0, n));

    // Inclusion-exclusion: |a+b| + |a∩b| = |a| + |b|.
    lemma_set_intersect_union_lens(a, b);

    // Hence |a∩b| >= (n - f) + (f + 1) - n = 1, so a∩b is non-empty.
    assert(a.intersect(b).len() >= 1);
    assert(a.intersect(b).finite()) by {
        assert(a.intersect(b).subset_of(a));
        lemma_len_subset(a.intersect(b), a);
    }
    assert(!a.intersect(b).is_empty()) by {
        axiom_is_empty_len0(a.intersect(b));
    }
    // Extract a witness i in a∩b.
    axiom_is_empty(a.intersect(b));
    let i = choose|x: int| a.intersect(b).contains(x);
    assert(a.intersect(b).contains(i));
    assert(a.contains(i));
    assert(b.contains(i));
    // a.contains(i) ⇒ 0 <= i < n && correct_at(i)
    // b.contains(i) ⇒ readings[i] <= v
    assert(0 <= i < readings.len() && correct_at(i) && readings[i] <= v);
}

proof fn lemma_pigeonhole_ge(readings: Seq<Reading>, v: Reading, f: nat)
    requires
        correct_indices(readings.len()).len() >= readings.len() - f,
        ge_set(readings, v).len() >= f + 1,
    ensures
        some_correct_ge(readings, v),
{
    let n: int = readings.len() as int;
    let a: Set<int> = correct_indices(readings.len());
    let b: Set<int> = ge_set(readings, v);

    lemma_correct_indices_in_range(readings.len());
    lemma_ge_set_in_range(readings, v);
    lemma_int_range(0, n);

    assert((a + b).subset_of(set_int_range(0, n))) by {
        assert forall|x: int| (a + b).contains(x) implies set_int_range(0, n).contains(x) by {
            assert(a.contains(x) || b.contains(x));
        }
    }
    lemma_len_subset(a + b, set_int_range(0, n));

    lemma_set_intersect_union_lens(a, b);

    assert(a.intersect(b).len() >= 1);
    assert(a.intersect(b).finite()) by {
        assert(a.intersect(b).subset_of(a));
        lemma_len_subset(a.intersect(b), a);
    }
    assert(!a.intersect(b).is_empty()) by {
        axiom_is_empty_len0(a.intersect(b));
    }
    axiom_is_empty(a.intersect(b));
    let i = choose|x: int| a.intersect(b).contains(x);
    assert(a.intersect(b).contains(i));
    assert(a.contains(i));
    assert(b.contains(i));
    assert(0 <= i < readings.len() && correct_at(i) && readings[i] >= v);
}

// --- The exec entry point ---------------------------------------------------
//
// Returns a value bracketed by some correct reading on each side.
//
// Algorithmic latitude: the implementer chooses the algorithm. Two
// reasonable approaches are
//
//   (a) sort the readings and return the median (position f, since
//       n = 2f+1 implies position f is the middle);
//
//   (b) brute-force: for each reading v, count how many readings are
//       <= v and how many are >= v; return the first v with both
//       counts >= f+1.
//
// (b) is O(n^2) but easier to verify since it avoids reasoning about a
// sort permutation. (a) requires either an existing verified sort in
// vstd or implementing and proving one. The architect's design note
// should make the choice and predict the relevant helper lemmas.

pub fn ft_midpoint(readings: &Vec<Reading>, f: u32) -> (result: Reading)
    requires
        readings.len() <= u32::MAX as nat,
        readings.len() as nat >= 2 * (f as nat) + 1,
        correct_indices(readings.len() as nat).len() >= readings.len() as nat - f as nat,
    ensures
        some_correct_le(readings@, result),
        some_correct_ge(readings@, result),
{
    // Overflow-safety: 2*f + 1 <= readings.len() <= u32::MAX
    // ⇒ f <= (u32::MAX - 1)/2 < u32::MAX, so f + 1 fits in u32.
    assert(f as nat + 1 <= u32::MAX as nat) by {
        assert(2 * (f as nat) + 1 <= u32::MAX as nat);
    }
    let threshold: u32 = f + 1;
    let n: usize = readings.len();
    let mut j: usize = 0;
    while j < n
        invariant
            0 <= j as int <= n as int,
            n == readings.len(),
            threshold as nat == f as nat + 1,
            readings.len() <= u32::MAX as nat,
            readings.len() as nat >= 2 * (f as nat) + 1,
            correct_indices(readings.len() as nat).len()
                >= readings.len() as nat - f as nat,
        decreases n - j,
    {
        let v: Reading = readings[j];
        let lec: u32 = count_le(readings, v);
        let gec: u32 = count_ge(readings, v);
        if lec >= threshold && gec >= threshold {
            proof {
                assert(le_set(readings@, v).len() >= f as nat + 1);
                assert(ge_set(readings@, v).len() >= f as nat + 1);
                lemma_pigeonhole_le(readings@, v, f as nat);
                lemma_pigeonhole_ge(readings@, v, f as nat);
            }
            return v;
        }
        j = j + 1;
    }
    // Post-loop: requires lemma_exists_midpoint (L6), to be landed in a
    // subsequent attempt. Placeholder return — verifier will flag the
    // postcondition failure at this position, isolating the remaining
    // obligation.
    readings[0]
}

} // verus!
