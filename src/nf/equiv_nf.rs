use crate::expr::*;
use super::*;

/// Concatenate two clause sets, i.e. take the union of the two.
/// In CNF this is the AND of two CNFs, in DNF this is the OR of two DNFs.
fn concat(l: Clauses, r: Clauses) -> Clauses {
    l.iter().chain(r.iter()).map(|it| it.clone()).collect()
}

/// Distribute two clause sets, i.e. concatenate the cartesian product. 
/// In CNF this is the OR of two CNFs, in DNF this is the AND of two DNFs.
fn distribute(l: Clauses, r: Clauses) -> Clauses {
    l.iter()
     .flat_map(|le| r.iter().map(|re| le.clone().concat(re.clone()))) // Cartesian product concat
     .filter(|e| e.is_disjoint()) // Remove tautologies
     .collect()
}

/// Inverts the literals in a clause set, i.e. flips all the logical NOT signs.
/// This converts a CNF into the DNF of its inverse, and a DNF into the CNF of its inverse.
fn invert(c: Clauses) -> Clauses {
    c.into_iter().map(|it| it.reverse()).collect() 
}

// fn demorgan_pos(e: BExpr) -> BExpr {
//     match e {
//         BExpr::And(lhs, rhs) => demorgan_pos(*lhs) & demorgan_pos(*rhs),
//         BExpr::Or(lhs, rhs) => demorgan_pos(*lhs) | demorgan_pos(*rhs),
//         BExpr::Not(rhs) => demorgan_neg(*rhs),

//         e => e
//     }
// }

// fn demorgan_neg(e: BExpr) -> BExpr {
//     match e {
//         BExpr::And(lhs, rhs) => demorgan_neg(*lhs) | demorgan_neg(*rhs),
//         BExpr::Or(lhs, rhs) => demorgan_neg(*lhs) & demorgan_neg(*rhs),
//         BExpr::Not(rhs) => demorgan_pos(*rhs),

//         e => !e
//     }
// }

// fn expr_const(e: &BExpr) -> Option<bool> {
//     match e {
//         BExpr::True => Some(true),
//         BExpr::False => Some(false),
//         BExpr::Pred(_, _) => None,
//         BExpr::And(lhs, rhs) => match (expr_const(lhs.as_ref()), expr_const(rhs.as_ref())) {
//             (_, Some(false)) => Some(false),
//             (Some(false), _) => Some(false),
//             (Some(true), Some(true)) => Some(true),
//             _ => None
//         },
//         BExpr::Or(lhs, rhs) => match (expr_const(lhs.as_ref()), expr_const(rhs.as_ref())) {
//             (_, Some(true)) => Some(true),
//             (Some(true), _) => Some(true),
//             (Some(false), Some(false)) => Some(false),
//             _ => None
//         },
//         BExpr::Not(rhs) => match expr_const(rhs.as_ref()) {
//             Some(true) => Some(false),
//             Some(false) => Some(true),
//             None => None,
//         }
//     }
// }

// fn resolve_consts(e: BExpr) -> BExpr {
//     match e {
//         BExpr::And(lhs, rhs) => match (expr_const(lhs.as_ref()), expr_const(rhs.as_ref())) {
//             (None, None) => *lhs & *rhs,

//             (None, Some(true)) => *lhs,
//             (Some(true), None) => *rhs,

//             (_, Some(false)) => BExpr::False,
//             (Some(false), _) => BExpr::False,

//             (Some(true), Some(true)) => BExpr::True,
//         },

//         BExpr::Or(lhs, rhs) => match (expr_const(lhs.as_ref()), expr_const(rhs.as_ref())) {
//             (None, None) => *lhs | *rhs,
            
//             (None, Some(false)) => *lhs,
//             (Some(false), None) => *rhs,

//             (_, Some(true)) => BExpr::True,
//             (Some(true), _) => BExpr::True,

//             (Some(false), Some(false)) => BExpr::False,
//         },

//         BExpr::Not(rhs) => match expr_const(rhs.as_ref()) {
//             Some(true) => BExpr::False,
//             Some(false) => BExpr::True,
//             None => !*rhs,
//         },

//         e => e
//     }
// }

/// Converts an expression into CNF.
pub fn cnf(e: BExpr) -> Clauses {
    match e {
        BExpr::True => Clauses::new(),
        BExpr::False => Clauses::from([Clause::new()]),
        BExpr::Pred(name, args) => Clauses::from([Atom::Pred(name, args).into()]),
        BExpr::And(lhs, rhs) => concat(cnf(*lhs), cnf(*rhs)),
        BExpr::Or(lhs, rhs) => distribute(cnf(*lhs), cnf(*rhs)),
        BExpr::Not(rhs) => invert(dnf(*rhs)),
    }
}

/// Converts an expression into DNF.
pub fn dnf(e: BExpr) -> Clauses {
    match e {
        BExpr::False => Clauses::new(),
        BExpr::True => Clauses::from([Clause::new()]),
        BExpr::Pred(name, args) => Clauses::from([Atom::Pred(name, args).into()]),
        BExpr::And(lhs, rhs) => distribute(dnf(*lhs), dnf(*rhs)),
        BExpr::Or(lhs, rhs) => concat(dnf(*lhs), dnf(*rhs)),
        BExpr::Not(rhs) => invert(cnf(*rhs)),
    }
}
