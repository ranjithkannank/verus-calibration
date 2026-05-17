// Pre-spec witness for exercises/cross_module_counter.rs.
//
// Operator-authored reference implementation. The spec block (the
// three closed spec fns, the four function signatures with their
// requires/ensures clauses) is byte-identical to the exercise file.
// The bodies here are real and the file verifies under Verus,
// proving the frozen spec admits a model before the agent loop ever
// starts.
//
// Workflow:
//   1. Operator writes exercises/<name>.rs with `unimplemented!()` body.
//   2. Operator writes exercises/<name>_witness.rs (this file) with
//      the same spec block and a real reference implementation.
//   3. Operator runs `ralph/check-spec.sh <name>`. If it passes,
//      the spec is provably satisfiable.
//   4. Operator tags `spec-frozen-<name>` and starts the agent loop.

use vstd::prelude::*;

verus! {

mod counter {
    use vstd::prelude::*;

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
}

mod client {
    use super::counter::Counter;
    use vstd::prelude::*;

    pub fn count_up_to(target: u32) -> (final_count: u32)
        ensures
            final_count == target,
    {
        let mut c = Counter::new(target);
        let mut i: u32 = 0;
        while i < target
            invariant
                c.invariant(),
                c.value() == i,
                c.bound() == target,
                i <= target,
            decreases target - i,
        {
            c.incr();
            i = i + 1;
        }
        c.get()
    }
}

} // verus!
