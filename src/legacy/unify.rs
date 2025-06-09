use std::collections::{BTreeMap, BTreeSet};

use super::expro::{Name, Term};
use super::expro::Term::*;

pub type Unifier = BTreeMap<Name, Term>;

enum StepResult {
    Fail,
    Changed,
    Done(Unifier)
}

struct Unification {
    eqs: BTreeSet<(Term, Term)>
}

impl Unification {
    fn new(eqs: BTreeSet<(Term, Term)>) -> Self {
        Self {
            eqs
        }
    }

    fn unify_term(term: Term, unifier: &Unifier) -> Term {
        match term {
            Const(cst) => Const(cst),
            Var(var) => unifier.get(&var).cloned().unwrap_or(Var(var)),
            Func(fun, args) => Func(fun, args.into_iter().map(|e| Self::unify_term(e, unifier)).collect()),
        }
    }

    fn step(&mut self) -> StepResult {
        let mut g = BTreeSet::new();
        let mut unifier = Unifier::new();

        let vars = self.eqs.iter().flat_map(|(l, r)| {
            let mut vars = l.vars();
            vars.append(&mut r.vars());
            vars
        }).collect::<BTreeSet<_>>();

        let mut _transformed = false;

        for (l, r) in self.eqs.clone().into_iter() {
            let e = (Self::unify_term(l, &unifier), Self::unify_term(r, &unifier));
            
            match e {
                // Conflict
                (Const(t), Const(u)) => {
                    if t != u {
                        return StepResult::Fail;
                    }

                    _transformed = true;
                },

                // Eliminate
                (Var(x), r) => {
                    if let f @ Func(_, _) = &r {
                        if f.contains_var(&x) {
                            return StepResult::Fail;
                        }
                    }

                    if vars.contains(&x) && !r.contains_var(&x) {
                        unifier.insert(x, r);
                    }
                },

                // Swap
                (l, Var(x)) => { // <-- PROBLEM!
                    g.insert((Var(x), l));

                    _transformed = true;
                },

                // Decompose
                (Func(x, mut xs), Func(y, mut ys)) => {
                    if x != y {
                        return StepResult::Fail;
                    }

                    if xs.len() != ys.len() {
                        return StepResult::Fail;
                    }

                    while let (Some(xa), Some(ya)) = (xs.pop(), ys.pop()) {
                        g.insert((xa, ya));
                    }

                    _transformed = true;
                },

                // No Op
                e => {
                    g.insert(e);
                }
            }

        }

        unifier = dbg!(unifier);
        g = dbg!(g);

        if g.is_empty() {
            let mut result = Unifier::new();

            for (l, r) in &unifier {
                result.insert(*l, Self::unify_term(r.clone(), &unifier));
            }

            return StepResult::Done(result);
        }

        if unifier.is_empty() {
            self.eqs = g;
        } else {
            self.eqs.clear();

            for (l, r) in g {
                self.eqs.insert((Self::unify_term(l, &unifier), Self::unify_term(r, &unifier)));
            }

            for (l, r) in &unifier {
                self.eqs.insert((Var(*l), Self::unify_term(r.clone(), &unifier)));
            }
        }

        return StepResult::Changed;
    }

    fn unify(&mut self) -> Option<Unifier> {
        loop {
            match self.step() {
                StepResult::Fail => return None,
                StepResult::Changed => {},
                StepResult::Done(unifier) => return Some(unifier)
            }
        }
    }
}

pub fn unify(left: &Vec<Term>, right: &Vec<Term>) -> Option<Unifier> {
    if left.len() != right.len() {
        return None;
    }

    let set = left.clone().into_iter().zip(right.clone().into_iter()).collect();
    let mut unify = Unification::new(set);

    unify.unify()
}

pub trait Unifiable {
    fn apply(self, unifier: &Unifier) -> Self;
}

impl<U> Unifiable for Vec<U> where U : Unifiable {
    fn apply(self, unifier: &Unifier) -> Self {
        self.into_iter().map(|e| e.apply(unifier)).collect()
    }
}

impl<U> Unifiable for BTreeSet<U> where U : Unifiable, U : Ord {
    fn apply(self, unifier: &Unifier) -> Self {
        self.into_iter().map(|e| e.apply(unifier)).collect()
    }
}
