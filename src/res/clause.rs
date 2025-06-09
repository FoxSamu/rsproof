use crate::expr::Name;
use crate::fmt::DisplayNamed;
use crate::nf::{Atom, Clause, PredicateIndex};
use crate::uni::Unifier;

#[derive(Debug, PartialEq, Eq)]
pub struct Resolvee {
    pub a: Atom,
    pub b: Atom,
    pub mgu: Unifier
}

fn find_resolvee_by_name(a: &PredicateIndex, b: &PredicateIndex, name: Name) -> Option<Resolvee> {
    let a_preds = a.get_preds(&name)?;
    let b_preds = b.get_preds(&name)?;

    let mut resolvee = None;
    
    for a_pred in a_preds {
        for b_pred in b_preds {
            if let Some(mgu) = Unifier::mgu(a_pred, b_pred) {
                match resolvee {
                    None => resolvee = Some(Resolvee {
                        a: Atom::Pred(name, a_pred.clone()),
                        b: Atom::Pred(name, b_pred.clone()),
                        mgu
                    }),

                    Some(_) => return None
                }
            }
        }
    }

    resolvee
}

fn find_resolvee_index(a: &PredicateIndex, b: &PredicateIndex) -> Option<Resolvee> {
    let mut resolvee = None;

    for name in a.iter_pred_names() {
        if b.contains_preds(name) {
            match resolvee {
                None => resolvee = find_resolvee_by_name(&a, &b, *name),

                Some(_) => return None // Second candidate
            }
        }
    }

    resolvee
}

pub fn find_resolvee(a: &Clause, b: &Clause) -> Option<Resolvee> {
    None
        .or_else(|| find_resolvee_index(a.pos(), b.neg()))
        .or_else(|| find_resolvee_index(a.neg(), b.pos()))
}




impl DisplayNamed for Resolvee {
    fn fmt_named(&self, f: &mut std::fmt::Formatter<'_>, names: &crate::fmt::NameTable) -> std::fmt::Result {
        write!(f, "Resolvee {{\n")?;
        write!(f, "    a:   {},\n", self.a.with_table(names))?;
        write!(f, "    b:   {},\n", self.b.with_table(names))?;
        write!(f, "    mgu: {}\n", self.mgu.with_table(names))?;
        write!(f, "}}")?;

        Ok(())
    }
}

