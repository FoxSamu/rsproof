use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::hash::Hash;

use crate::expr::{AExpr, Name, Vars};

/// A unifier is a set of substitutions `x := a` where `x` is some bound variable name and `a` an [AExpr],
/// with two additional restrictions:
/// 1.  A variable may only appear on the left hand side of a substitution if and only if it does not appear
///     anywhere on the right hand side of a substitution. That is, `{x := f(x)}` is an invalid unifier and
///     so is `{x := y, y := z}`. In other words, a unifier may not recursively define itself.
/// 2.  A variable may only appear on the left hand side of a substitution once. That makes that
///     `{x := y, x := z}` is an invalid unifier. In other words, a unifier is a mapping.
/// 
/// A substitution is a transformation on an expression where a [bound variable][AExpr::Var] is substituted
/// by some other [AExpr]. A unifier is therefore a composite transformation of several substitutions, the
/// transformation being referred to as "unification". Thanks to the first restriction mentioned above, the
/// order of substitution does not matter.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct Unifier {
    table: BTreeMap<Name, AExpr>
}

impl Unifier {
    /// Creates a new unifier with no substitutions.
    pub fn new() -> Self {
        Self {
            table: BTreeMap::new()
        }
    }

    /// Adds a new substitution `from := to` to the unifier. Panics when:
    /// - There is a substitution `x := a` where `to` is a variable of `Q`.
    /// - There is a substitution `from := a` for some expression `a`.
    pub fn add(&mut self, from: Name, to: AExpr) {
        if self.table.contains_key(&from) {
            panic!("Left hand side of substitution is already the left hand side of another substitution.");
        }

        if self.table.values().any(|e| e.has_var(&from)) {
            panic!("Left hand side of substitution appears on the right hand side of another substitution.");
        }

        if to.has_var(&from) {
            panic!("Added substitution is recursive.")
        }

        self.table.insert(from, to);
    }

    /// Unifies the given [Name] to an [AExpr] as defined by this unifier. The given name is said to be the name
    /// of a bound variable, which could be unified. If the unifier has a substitution for this name, said substitution
    /// is cloned and returned. Otherwise, it generates a [variable expression][AExpr::Var] with the given name.
    pub fn unify(&self, name: &Name) -> AExpr {
        if let Some(unified) = self.table.get(name) {
            // If there is a substitution, we return that
            unified.clone()
        } else {
            // Otherwise we don't unify the variable any further
            AExpr::Var(*name)
        }
    }
}




/// A value that can be unified with a [Unifier].
pub trait Unifiable {
    /// Unify this value.
    fn unify(self, unifier: &Unifier) -> Self;
}

impl<U> Unifiable for Box<U> where U : Unifiable {
    fn unify(self, unifier: &Unifier) -> Self {
        Box::new((*self).unify(unifier))
    }
}

impl<U> Unifiable for Option<U> where U : Unifiable {
    fn unify(self, unifier: &Unifier) -> Self {
        self.map(|it| it.unify(unifier))
    }
}

impl<U> Unifiable for Vec<U> where U : Unifiable {
    fn unify(self, unifier: &Unifier) -> Self {
        self.into_iter().map(|it| it.unify(unifier)).collect()
    }
}

impl<U> Unifiable for HashSet<U> where U : Unifiable + Hash + Eq {
    fn unify(self, unifier: &Unifier) -> Self {
        self.into_iter().map(|it| it.unify(unifier)).collect()
    }
}

impl<U> Unifiable for BTreeSet<U> where U : Unifiable + Ord {
    fn unify(self, unifier: &Unifier) -> Self {
        self.into_iter().map(|it| it.unify(unifier)).collect()
    }
}

impl<U, const N: usize> Unifiable for [U; N] where U : Unifiable {
    fn unify(self, unifier: &Unifier) -> Self {
        // Only way is to convert to vector
        let vec: Vec<U> = self.into_iter().map(|it| it.unify(unifier)).collect();

        // Convert back to slice
        if let Ok(res) = vec.try_into() {
            res
        } else {
            panic!();
        }
    }
}

impl Unifiable for () {
    fn unify(self, _: &Unifier) -> Self {
        ()
    }
}

impl<U1, U2> Unifiable for (U1, U2) where U1 : Unifiable, U2 : Unifiable {
    fn unify(self, unifier: &Unifier) -> Self {
        (self.0.unify(unifier), self.1.unify(unifier))
    }
}

impl<U1, U2, U3> Unifiable for (U1, U2, U3) where U1 : Unifiable, U2 : Unifiable, U3 : Unifiable {
    fn unify(self, unifier: &Unifier) -> Self {
        (self.0.unify(unifier), self.1.unify(unifier), self.2.unify(unifier))
    }
}