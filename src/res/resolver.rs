use std::rc::Rc;

use crate::nf::Clause;
use crate::res::Heuristic;
use crate::res::KnowledgeBase;
use crate::res::Resolvee;
use crate::util::pqueue::PQueue;


struct Candidate {
    a: Rc<Clause>,
    b: Rc<Clause>,
    res: Resolvee,
    result: Clause,
    heuristic: u64
}



pub struct Resolver {
    /// The knowledge base, that is, all statements that the resolver currently believes to be true.
    kb: KnowledgeBase,

    /// The heuristic that determines how much a clause is preferred to be added to the knowledge base.
    heuristic: Heuristic,

    /// The queue of candidates
    queue: PQueue<Candidate, u64>
}

impl Resolver {
    pub fn new(heuristic: Heuristic) -> Self {
        Self {
            kb: KnowledgeBase::new(),
            heuristic,
            queue: PQueue::new()
        }
    }
}