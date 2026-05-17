// Witness for counter_multifile/counter.rs.
//
// Operator-authored reference implementation. Spec block (the three
// closed spec fns + the three function signatures with their
// requires/ensures clauses) is byte-identical to
// exercises/counter_multifile/counter.rs. The bodies here are real;
// the file verifies under Verus.

use vstd::prelude::*;

verus! {

pub struct Counter {
    value: u32,
    bound: u32,
}

impl Counter {
    pub closed spec fn value(&self) -> u32 {
        self.value
    }

    pub closed spec fn bound(&self) -> u32 {
        self.bound
    }

    pub closed spec fn invariant(&self) -> bool {
        self.value <= self.bound
    }

    pub fn new(bound: u32) -> (c: Counter)
        ensures
            c.invariant(),
            c.value() == 0,
            c.bound() == bound,
    {
        Counter { value: 0, bound: bound }
    }

    pub fn incr(&mut self)
        requires
            old(self).invariant(),
            old(self).value() < old(self).bound(),
        ensures
            final(self).invariant(),
            final(self).value() == old(self).value() + 1,
            final(self).bound() == old(self).bound(),
    {
        self.value = self.value + 1;
    }

    pub fn get(&self) -> (v: u32)
        requires
            self.invariant(),
        ensures
            v == self.value(),
    {
        self.value
    }
}

} // verus!
