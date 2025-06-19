use std::collections::{BTreeMap, BTreeSet};

use crate::expr::AExpr;
use crate::nf::Atom;
use crate::uni::Unifier;


#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Substitutee {
    pub from: AExpr,
    pub to: AExpr,
    pub mgu: Unifier
}

pub fn find_subexpr_unifiers(search: AExpr, elem: AExpr) -> BTreeMap<AExpr, Unifier> {
    let subexprs: BTreeSet<_> = search.subexprs();

    let mut map = BTreeMap::new();
    
    for expr in subexprs {
        if let Some(mgu) = Unifier::mgu(&expr, &elem) {
            map.insert(expr, mgu);
        }
    }

    map
}

pub fn substitute_aexpr(expr: AExpr, from: &AExpr, to: &AExpr) -> AExpr {
    if expr == *from {
        return to.clone();
    }

    match expr {
        AExpr::Var(name) => AExpr::var(name),
        AExpr::Fun(name, args) => AExpr::fun(name, substitute_aexprs(args, from, to)),
    }
}

pub fn substitute_aexprs(exprs: Vec<AExpr>, from: &AExpr, to: &AExpr) -> Vec<AExpr> {
    exprs.into_iter().map(|it| substitute_aexpr(it, from, to)).collect()
}

pub fn substitute_atom(atom: Atom, from: &AExpr, to: &AExpr) -> Atom {
    match atom {
        Atom::Pred(name, args) => Atom::Pred(name, substitute_aexprs(args, from, to)),
    }
}
