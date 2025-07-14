use crate::expr::AExpr;
use crate::nf::Clause;

pub fn disjunct_count(c: &Clause) -> u64 {
    let mut heuristic = 0u64;

    for pred in c.pos().iter_preds() {
        heuristic += 1;
    }

    for pred in c.neg().iter_preds() {
        heuristic += 1;
    }
    
    heuristic
}