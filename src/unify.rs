use std::collections::{BTreeMap, BTreeSet};

use crate::expro::{Name, Term};
use crate::expro::Term::*;

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
        let mut tf = BTreeSet::new();
        let mut unifier = Unifier::new();

        let vars = self.eqs.iter().flat_map(|(l, r)| {
            let mut vars = l.vars();
            vars.append(&mut r.vars());
            vars
        }).collect::<BTreeSet<_>>();

        let mut transformed = false;

        for e in self.eqs.clone().into_iter() {
            match e {
                (Const(t), Const(u)) => {
                    if t != u {
                        return StepResult::Fail;
                    }

                    transformed = true;
                },
                
                (l, Var(x)) => {
                    tf.insert((Var(x), l));

                    transformed = true;
                },
                
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
                
                (Func(x, mut xs), Func(y, mut ys)) => {
                    if x != y {
                        return StepResult::Fail;
                    }

                    if xs.len() != ys.len() {
                        return StepResult::Fail;
                    }

                    while let (Some(xa), Some(ya)) = (xs.pop(), ys.pop()) {
                        tf.insert((xa, ya));
                    }

                    transformed = true;
                },

                e => {
                    tf.insert(e);
                }
            }
        }

        if tf.is_empty() {
            return StepResult::Done(unifier);
        }

        if unifier.is_empty() {
            self.eqs = tf;
        } else {
            self.eqs.clear();

            for (l, r) in tf {
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
