use crate::nf::Clause;
use crate::res::heuristic::disjunct_count::disjunct_count;
use crate::res::heuristic::symbol_count::symbol_count;

mod symbol_count;
mod disjunct_count;

#[derive(Clone, Copy, Debug)]
pub enum Heuristic {
    /// The most naive heuristic. It does not prioritise any clause more than the other.
    Naive,

    /// A pretty naive heuristic that prioritises the empty clause over a non-empty clause,
    /// but does not prioritise between non-empty clauses.
    PreferEmpty,

    /// A somewhat naive heuristic that prioritises clauses that are less resolution steps away
    /// from any premise over those that are more resolution steps away from any premise.
    /// However, the empty clause is always prioritised.
    Distance,

    /// A heuristic that counts all the symbols in the clause. That is, it counts all predicates,
    /// all functions and all variables.
    SymbolCount,

    /// A heuristic that counts all the disjunct in the clause. That is, it counts all predicates,
    /// but disregards all functions and variables.
    DisjunctCount,

    /// Sum of [Heuristic::SymbolCount] and [Heuristic::Distance].
    SymbolCountPlusDistance,

    /// Sum of [Heuristic::DisjunctCount] and [Heuristic::Distance].
    DisjunctCountPlusDistance,
}

impl Heuristic {
    pub fn heuristic(&self, clause: &Clause, distance: u64) -> u64 {
        match self {
            Heuristic::Naive => 0,
            Heuristic::PreferEmpty => if clause.is_empty() { 0 } else { 1 },
            Heuristic::Distance => if clause.is_empty() { 0 } else { distance },
            Heuristic::SymbolCount => symbol_count(clause),
            Heuristic::DisjunctCount => disjunct_count(clause),
            Heuristic::SymbolCountPlusDistance => symbol_count(clause) + if clause.is_empty() { 0 } else { distance },
            Heuristic::DisjunctCountPlusDistance => disjunct_count(clause) + if clause.is_empty() { 0 } else { distance },
        }
    }
}
