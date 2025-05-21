use std::collections::{BTreeMap, BTreeSet};

use crate::expr::*;
use super::*;

fn remove_true_false(expr: BExpr) -> BExpr {
    match expr {
        BExpr::And(lhs, rhs) => {
            let l = remove_true_false(*lhs);
            if l == BExpr::False {
                return l;
            }

            let r = remove_true_false(*rhs);
            if r == BExpr::False {
                return l;
            }

            if l == BExpr::True {
                return r;
            }

            if r == BExpr::True {
                return l;
            }

            return l & r;
        },

        BExpr::Or(lhs, rhs) => {
            let l = remove_true_false(*lhs);
            if l == BExpr::True {
                return l;
            }

            let r = remove_true_false(*rhs);
            if r == BExpr::True {
                return l;
            }

            if l == BExpr::False {
                return r;
            }

            if r == BExpr::False {
                return l;
            }

            return l | r;
        },

        BExpr::Not(rhs) => {
            let r = remove_true_false(*rhs);

            return match r {
                BExpr::True => BExpr::False,
                BExpr::False => BExpr::True,
                e => !e
            };
        },

        e => e
    }
}

#[derive(Clone)]
enum TseitinOperator {
    Conj(Atom, Atom),
    Disj(Atom, Atom),
    Inv(Atom),
    Ident,
}

#[derive(Clone)]
struct TseitinAssignment {
    atom: Atom,
    operator: TseitinOperator
}

struct Tseitin {
    namings: BTreeMap<BExpr, TseitinAssignment>,
    next_name: Name
}

impl Tseitin {
    fn next_name(&mut self) -> Name {
        self.next_name.incr()
    }

    fn assign(&mut self, expr: BExpr) -> Atom {
        if self.namings.contains_key(&expr) {
            return self.namings.get(&expr).unwrap().atom.clone();
        }

        let atom = match &expr {
            BExpr::True | BExpr::False => todo!("True and False must have been eliminated Tseitin transformation"),

            BExpr::Pred(name, args) => {
                Atom::Pred(*name, args.clone())
            },

            expr => {
                let vars: BTreeSet<Name> = expr.vars();
                let args: Vec<AExpr> = vars.into_iter().map(|it| AExpr::Var(it)).collect();

                Atom::Pred(self.next_name(), args)
            }
        };

        let operator = match &expr {
            BExpr::True | BExpr::False => panic!("True and False must have been eliminated Tseitin transformation"),

            BExpr::Pred(_, _) => TseitinOperator::Ident,

            BExpr::And(lhs, rhs) => TseitinOperator::Conj(
                self.assign(*lhs.clone()),
                self.assign(*rhs.clone())
            ),

            BExpr::Or(lhs, rhs) => TseitinOperator::Disj(
                self.assign(*lhs.clone()),
                self.assign(*rhs.clone())
            ),

            BExpr::Not(rhs) => TseitinOperator::Inv(
                self.assign(*rhs.clone())
            )
        };

        let assignment = TseitinAssignment {
            atom,
            operator
        };

        let entry = self.namings.entry(expr).or_insert(assignment);

        return entry.atom.clone();
    }

    fn to_cnf(self, base: Atom) -> Clauses {
        let mut cnf = Clauses::new();

        cnf.insert(Clause::from_pos(base));
        
        for assignment in self.namings.into_values() {
            assignment.to_cnf(&mut cnf);
        }

        cnf
    }
}

impl TseitinAssignment {
    fn to_cnf(self, cnf: &mut Clauses) {
        let x = self.atom;

        match self.operator {
            TseitinOperator::Conj(p, q) => {
                //      (X <-> (P & Q))
                // ===  (X -> (P & Q)) & (X <- (P & Q))
                // ===  (!X | (P & Q)) & (X | !(P & Q))
                // ===  (!X | (P & Q)) & (X | !P | !Q)
                // ===  (!X | P) & (!X | Q) & (X | !P | !Q)

                cnf.insert(Clause::from_slices([p.clone()], [x.clone()]));
                cnf.insert(Clause::from_slices([q.clone()], [x.clone()]));
                cnf.insert(Clause::from_slices([x], [p, q]));
            },

            TseitinOperator::Disj(p, q) => {
                //      (X <-> (P | Q))
                // ===  (X -> (P | Q)) & (X <- (P | Q))
                // ===  (!X | (P | Q)) & (X | !(P | Q))
                // ===  (!X | P | Q) & (X | (!P & !Q))
                // ===  (!X | P | Q) & (X | !P) & (X | !Q)

                cnf.insert(Clause::from_slices([x.clone()], [p.clone()]));
                cnf.insert(Clause::from_slices([x.clone()], [q.clone()]));
                cnf.insert(Clause::from_slices([p, q], [x]));
            },

            TseitinOperator::Inv(p) => {
                //      (X <-> !P)
                // ===  (X -> !P) & (X <- !P)
                // ===  (!X | !P) & (X | P)

                cnf.insert(Clause::from_slices([x.clone()], [p.clone()]));
                cnf.insert(Clause::from_slices([p], [x]));
            },

            TseitinOperator::Ident => {
                // Do not add anything
            }
        }
    }
}

fn base_cnf(expr: BExpr) -> Clauses {
    let mut tseitin = Tseitin {
        namings: BTreeMap::new(),
        next_name: expr.free()
    };

    let base = tseitin.assign(expr);
    tseitin.to_cnf(base)
}

pub fn cnf(mut expr: BExpr) -> Clauses {
    expr = remove_true_false(expr);

    if let BExpr::True | BExpr::False = expr {
        return equiv_nf::cnf(expr);
    }

    base_cnf(expr)
}

pub fn dnf(mut expr: BExpr) -> Clauses {
    expr = remove_true_false(expr);

    if let BExpr::True | BExpr::False = expr {
        return equiv_nf::cnf(expr);
    }

    // By DeMorgan: !DNF[expr] = CNF[!expr]
    let inv_cnf = base_cnf(!expr);
    inv_cnf.into_iter().map(|it| it.reverse()).collect()
}
