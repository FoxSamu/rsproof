use std::collections::{BTreeMap, BTreeSet};
use std::mem::{replace, take};

use crate::expr::{AExpr, Name, Vars};
use crate::expr::AExpr::*;

use super::Unifier;

/// An data structure for finding [MGUs][mgu]. It has a set of equalities `G` and
/// a unifier `U`.
/// 
/// Initially, the MGU finder places a bunch of equalities in `G` and leaves `U` empty.
/// It then transforms `G` and extracts substitutions into `U` repeatedly. Eventually,
/// `G` will become the empty set, which means all substitutions have been extracted into
/// `U` and `U` has become the MGU we are looking for.
/// 
/// This algorithm is an improved implementation of the algorithm presented by
/// [A. Martelli and U. Montanari (1982)](https://dl.acm.org/doi/pdf/10.1145/357162.357169).
struct MguFinder {
    /// The current `G` set, which is the set of expressions that have yet to be unified.

    // TODO: Can we replace this with a vector? Makes insertion O(1) instead of O(log(n)) but may
    // unnecessarily repeat unifications if they occur multiple times.
    g: BTreeSet<(AExpr, AExpr)>,

    /// The current `U` map, which is the currently resolved unifier.
    u: BTreeMap<Name, AExpr>
}

impl MguFinder {
    /// Initialises a new MGU finder from an input `G` set.
    fn new(g: BTreeSet<(AExpr, AExpr)>) -> Self {
        Self {
            g,
            u: BTreeMap::new()
        }
    }

    /// Transforms a term by a given unifier.
    fn apply_unifier(term: AExpr, unifier: &BTreeMap<Name, AExpr>) -> AExpr {
        match term {
            Var(var) => unifier.get(&var).cloned().unwrap_or(Var(var)),
            Fun(fun, args) => Fun(fun, args.into_iter().map(|e| Self::apply_unifier(e, unifier)).collect()),
        }
    }

    /// A step in the process of unification. This transforms both `G` and `U`.
    fn step(&mut self) -> bool {
        // Move out old `G` and `U` by moving in empty sets/maps.
        let mut g = take(&mut self.g);
        let mut u = take(&mut self.u);


        // Find all the variables that are mentioned in `G` or `U`.

        // TODO is this still used?
        let mut _vars = BTreeSet::new();

        for (l, r) in &self.g {
            _vars.append(&mut l.vars());
            _vars.append(&mut r.vars());
        }

        for (l, r) in &self.u {
            _vars.insert(*l);
            _vars.append(&mut r.vars());
        }


        // Transform each equality in `G`.
        for e in take(&mut g).into_iter() {
            match e {
                // ELIMINATE
                (Var(x), r) => {
                    if r == Var(x) {
                        // This expression is of the form `x = x`, it is pointless. We continue without it.
                        continue;
                    }

                    if r.has_var(&x) {
                        // If `x` is a subexpression of `r`, then the unifier is infinitely recursive.
                        // This cannot happen so we fail.
                        return false;
                    }

                    // If neither of the above conditions is true, it means we have a valid substitution to
                    // move to the unifier.
                    u.insert(x, r);
                },

                // SWAP
                // Has to be checked AFTER the Eliminate step! If not done so,
                // `x = y` will be infinitely swapped to `y = x` and back.
                (l, Var(x)) => {
                    // `f(...) = x` becomes `x = f(...)`
                    g.insert((Var(x), l));
                },

                // DECOMPOSE
                (Fun(x, xs), Fun(y, ys)) => {
                    // `f(x1, x2, ...) = f(y1, y2, ...)` becomes `x1 = y1, x2 = y2, ...` if possible.
                    
                    if x != y {
                        // This means we have `f(...) = g(...)` where `f !== g`. We cannot unify this,
                        // so we fail.
                        return false;
                    }

                    if xs.len() != ys.len() {
                        // This means we have two functions `f` equal to eachother, but their amount of
                        // arguments differs. We treat these to be different functions as well, so we
                        // fail.
                        return false;
                    }

                    // Zip the function arguments, we will now unify these pairs.
                    let xi = xs.into_iter();
                    let yi = ys.into_iter();

                    for pair in xi.zip(yi) {
                        g.insert(pair);
                    }
                }
            }
        }

        // Update current `G` and `U`.

        if u.is_empty() {
            // If there is no unifier, there is no need to clone stuff around.
            self.g = g;
            self.u = u;
        } else {
            // Unify `G`
            for (l, r) in g {
                self.g.insert((Self::apply_unifier(l, &u), Self::apply_unifier(r, &u)));
            }

            // Unify `U`
            for (l, r) in &u {
                self.u.insert(*l, Self::apply_unifier(r.clone(), &u));
            }
        }

        return true;
    }

    /// Runs the MGU finding algorithm.
    fn run(mut self) -> Option<BTreeMap<Name, AExpr>> {
        // Repeatedly step until `G` is empty or a failure is detected.
        while self.step() {
            if self.g.is_empty() {
                // If `G` is empty, we're done, so we stop. `U` is now our MGU.
                return Some(self.u);
            }
        }

        // If we reach here, `step()` returned `false` so we failed.
        return None;
    }
}

/// Find a Most General Unifier (MGU).
/// 
/// The input is two vectors with [AExpr]s. These vector are the arguments of two respective predicates that
/// must be unified. Say, `P(x, y)` needs to be unified with `P(f(a), b)`, then the input vectors are
/// `[x, y]` and `[f(a), b]`. The result is that that `{x = f(a), y = b}`, the two inputs zipped into a set,
/// will be transformed to a unifier.
/// 
/// The unification will return a unifier if unification succeeds, that is, there is a unifier `U` such that
/// `U.unify(left) == U.unify(right)`. The unifier will be the most general, i.e. it has the least amount of
/// substitutions needed to achieve the aforementioned expectation.
/// 
/// If such unifier does not exist, [None] is returned. Such unifier will for sure not exist when the input
/// vectors have different lengths.
pub fn mgu(left: Vec<AExpr>, right: Vec<AExpr>) -> Option<Unifier> {

    // There is no unifier if input vectors differ in length.
    if left.len() != right.len() {
        return None;
    }

    // Zip input vectors into a set of equalities.
    let l = left.into_iter();
    let r = right.into_iter();
    let set = l.zip(r).collect();

    // Find the unifier
    MguFinder::new(set)              // Create new MguFinder
        .run()                       // Run the algorithm to find MGU
        .map(Unifier::try_from)      // If found, convert to `Unifier` object
        .transpose()                 // Convert `Option<Result<..., ...>>` into `Result<Option<...>, ...>`
        .unwrap()                    // Panic if there is an error
}

