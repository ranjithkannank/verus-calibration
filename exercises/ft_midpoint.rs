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
    // TODO(loop): fill in. Do not modify any spec above.
    unimplemented!()
}

} // verus!
