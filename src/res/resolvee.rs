use crate::expr::Name;
use crate::nf::{Atom, Clause, PredicateIndex};
use crate::uni::Unifier;


#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Resolvee {
    pub a: Atom,
    pub b: Atom,
    pub a_neg: bool,
    pub b_neg: bool,
    pub mgu: Unifier
}


fn find_resolvees_by_name(a: &PredicateIndex, b: &PredicateIndex, a_neg: bool, name: Name) -> Option<Vec<Resolvee>> {
    let a_preds = a.get_preds(&name)?;
    let b_preds = b.get_preds(&name)?;
    
    let mut out = Vec::new();

    for a_pred in a_preds {
        for b_pred in b_preds {
            if let Some(mgu) = Unifier::mgu(a_pred, b_pred) {
                out.push(Resolvee {
                    a: Atom::Pred(name, a_pred.clone()),
                    b: Atom::Pred(name, b_pred.clone()),
                    a_neg, b_neg: !a_neg,
                    mgu
                });
            }
        }
    }

    Some(out)
}

fn find_resolvees_index(a: &PredicateIndex, b: &PredicateIndex, a_neg: bool) -> Vec<Resolvee> {
    let mut resolvees = Vec::new();

    for name in a.iter_pred_names() {
        if b.contains_preds(name) {
            if let Some(mut elem) = find_resolvees_by_name(&a, &b, a_neg, *name) {
                resolvees.append(&mut elem);
            }
        }
    }

    resolvees
}

pub fn find_resolvees(a: &Clause, b: &Clause) -> Vec<Resolvee> {
    let mut v = Vec::new();

    v.append(&mut find_resolvees_index(a.pos(), b.neg(), false));
    v.append(&mut find_resolvees_index(a.neg(), b.pos(), true));

    v
}
