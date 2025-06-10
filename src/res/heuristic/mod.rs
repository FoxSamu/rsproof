use crate::nf::Clause;
use crate::res::heuristic::symbol_count::symbol_count;

mod symbol_count;

#[derive(Clone, Copy, Debug)]
pub enum Heuristic {
    /// The most naive heuristic. It does not prioritise any clause more than the other.
    Naive,

    /// A pretty naive heuristic that prioritises the empty clause over a non-empty clause,
    /// but does not prioritise between non-empty clauses.
    PreferEmpty,

    /// A heuristic that counts all the symbols in the clause. That is, it counts all predicates,
    /// all functions and all variables.
    SymbolCount
}

impl Heuristic {
    pub fn heuristic(&self, clause: &Clause) -> u64 {
        match self {
            Heuristic::Naive => 0,
            Heuristic::PreferEmpty => if clause.is_empty() { 0 } else { 1 },
            Heuristic::SymbolCount => symbol_count(clause),
        }
    }
}