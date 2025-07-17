use std::collections::BTreeSet;
use std::mem::take;

use crate::expr::{AExpr, Vars};
use crate::expr::AExpr::*;
use crate::uni::Unifiable;

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
    u: Unifier
}

impl MguFinder {
    /// Initialises a new MGU finder from an input `G` set.
    fn new(g: BTreeSet<(AExpr, AExpr)>) -> Self {
        Self {
            g,
            u: Unifier::new()
        }
    }

    /// A step in the process of unification. This transforms both `G` and `U`.
    fn step(&mut self) -> bool {
        // Move out old `G` and `U` by moving in empty sets/maps.
        let mut g = take(&mut self.g);
        let mut u = take(&mut self.u);


        // Transform each equality in `G`.
        for (l, r) in take(&mut g).into_iter() {
            // Apply current unifier to term immediately. This fixes a bug where
            // `f(:x, :y) = f(:y, :x)` would create the unifier `{x = :y, y = :x}`,
            // which is invalid and causes a panic upon construction of the `Unifier`
            // object.
            //
            // Note that it is still crucial that `G` gets unified after this loop. Some
            // updates in `U` may not be reflected in `G` yet.
            let e = (l.unify(&u), r.unify(&u));

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
                    u = u.chain(Unifier::singleton(x, r));
                },

                // SWAP
                // Has to be checked AFTER the Eliminate step! If not done so,
                // `x = y` will be infinitely swapped to `y = x` and back.
                (Fun(y, ys), Var(x)) => {
                    // `f(...) = x` becomes `x = f(...)`
                    g.insert((Var(x), Fun(y, ys)));
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

        // Unify and update current `G` and `U`.

        if u.is_empty() {
            // If there is no unifier, there is no need to clone stuff around.
            self.g = g;
            self.u = u;
        } else {
            // Unify `G`
            for (l, r) in g {
                self.g.insert((l.unify(&u), r.unify(&u)));
            }

            self.u = u;
        }

        return true;
    }

    /// Runs the MGU finding algorithm.
    fn run(mut self) -> Option<Unifier> {
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
    let set: BTreeSet<(AExpr, AExpr)> = l.zip(r).collect();

    // Find the unifier
    MguFinder::new(set)              // Create new MguFinder
        .run()                       // Run the algorithm to find MGU
}

