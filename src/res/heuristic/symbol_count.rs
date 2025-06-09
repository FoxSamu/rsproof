use crate::expr::AExpr;
use crate::nf::Clause;

pub fn aexpr_size(e: &AExpr) -> u64 {
    match e {
        AExpr::Var(_) => 1,
        AExpr::Fun(_, args) => 1 + aexprs_size(args)
    }
}

pub fn aexprs_size(e: &Vec<AExpr>) -> u64 {
    e.iter().map(|it| aexpr_size(it)).sum()
}

pub fn symbol_count(c: &Clause) -> u64 {
    let mut heuristic = 0u64;

    for pred in c.pos().iter_preds() {
        heuristic += 1;
        heuristic += aexprs_size(pred.1);
    }

    for pred in c.neg().iter_preds() {
        heuristic += 1;
        heuristic += aexprs_size(pred.1);
    }
    
    heuristic
}