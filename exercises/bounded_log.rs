// Exercise 2: verified bounded append-only message log.
//
// This is the meat. The spec includes a frame property — append must not
// overwrite existing entries — which is the kind of obligation SMT solvers
// handle unevenly. How the loop deals with it is the most interesting data
// point of the weekend.
//
// The spec below is FROZEN. Iteration cap: 20. See AGENTS.md.
//
// NOTE (operator intervention 2026-05-15): the original spec used bare
// `self` in `&mut self` postconditions. Verus 0.2026.05.13 hard-rejects
// this; see https://github.com/verus-lang/verus/blob/main/source/docs/migration-mut-ref.md
// On the first calibration run, the implementer correctly identified this
// as an irreconcilable conflict and wrote a blocker report (preserved in
// the git history before this commit). The operator re-froze the spec to
// use `final(self)` for post-state references, which is semantically
// identical to the original intent and is the syntax current Verus
// requires. The `spec-frozen-bounded_log` tag has been force-moved to
// this commit.

use vstd::prelude::*;

verus! {

pub type Message = u64;

pub struct Log {
    cap: usize,
    msgs: Vec<Message>,
}

impl Log {
    pub closed spec fn capacity(&self) -> nat {
        self.cap as nat
    }

    pub closed spec fn view(&self) -> Seq<Message> {
        self.msgs@
    }

    pub closed spec fn well_formed(&self) -> bool {
        self.msgs.len() <= self.cap
    }

    pub fn new(capacity: usize) -> (result: Self)
        ensures
            result.well_formed(),
            result.capacity() == capacity as nat,
            result.view().len() == 0,
    {
        Log { cap: capacity, msgs: Vec::new() }
    }

    pub fn len(&self) -> (result: usize)
        requires self.well_formed(),
        ensures result as nat == self.view().len(),
    {
        self.msgs.len()
    }

    pub fn get(&self, index: usize) -> (result: Option<Message>)
        requires self.well_formed(),
        ensures
            (index as int) < self.view().len() ==>
                result == Some::<Message>(self.view()[index as int]),
            (index as int) >= self.view().len() ==> result.is_none(),
    {
        if index < self.msgs.len() {
            Some(self.msgs[index])
        } else {
            None
        }
    }

    pub fn append(&mut self, msg: Message) -> (result: Result<(), ()>)
        requires old(self).well_formed(),
        ensures
            final(self).well_formed(),
            final(self).capacity() == old(self).capacity(),
            result.is_ok() ==> {
                &&& final(self).view().len() == old(self).view().len() + 1
                &&& final(self).view()[old(self).view().len() as int] == msg
                // Frame property: existing entries are unchanged.
                &&& forall|i: int| 0 <= i < old(self).view().len() ==>
                        final(self).view()[i] == old(self).view()[i]
            },
            result.is_err() ==> {
                &&& old(self).view().len() == old(self).capacity()
                &&& final(self).view() == old(self).view()
            },
    {
        if self.msgs.len() < self.cap {
            self.msgs.push(msg);
            // Help the solver with the frame property
            assert(self.msgs@ == old(self).msgs@.push(msg));
            assert(forall|i: int| 0 <= i < old(self).msgs@.len()
                   ==> self.msgs@[i] == old(self).msgs@[i]);
            Ok(())
        } else {
            Err(())
        }
    }
}

} // verus!
