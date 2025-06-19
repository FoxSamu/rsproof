use std::collections::{BTreeSet, HashSet};
use std::hash::Hash;

use crate::expr::AExpr;

pub trait Substitutable {
    /// Substitutes all exact occurences of an [AExpr] for another [AExpr] in this object.
    fn subst(self, from: &AExpr, to: &AExpr) -> Self;
}


impl<U> Substitutable for Box<U> where U : Substitutable {
    fn subst(self, from: &AExpr, to: &AExpr) -> Self {
        Box::new((*self).subst(from, to))
    }
}

impl<U> Substitutable for Option<U> where U : Substitutable {
    fn subst(self, from: &AExpr, to: &AExpr) -> Self {
        self.map(|it| it.subst(from, to))
    }
}

impl<U> Substitutable for Vec<U> where U : Substitutable {
    fn subst(self, from: &AExpr, to: &AExpr) -> Self {
        self.into_iter().map(|it| it.subst(from, to)).collect()
    }
}

impl<U> Substitutable for HashSet<U> where U : Substitutable + Hash + Eq {
    fn subst(self, from: &AExpr, to: &AExpr) -> Self {
        self.into_iter().map(|it| it.subst(from, to)).collect()
    }
}

impl<U> Substitutable for BTreeSet<U> where U : Substitutable + Ord {
    fn subst(self, from: &AExpr, to: &AExpr) -> Self {
        self.into_iter().map(|it| it.subst(from, to)).collect()
    }
}

impl<U, const N: usize> Substitutable for [U; N] where U : Substitutable {
    fn subst(self, from: &AExpr, to: &AExpr) -> Self {
        // Only way is to convert to vector
        let vec: Vec<U> = self.into_iter().map(|it| it.subst(from, to)).collect();

        // Convert back to slice
        if let Ok(res) = vec.try_into() {
            res
        } else {
            panic!();
        }
    }
}

impl Substitutable for () {
    fn subst(self, _from: &AExpr, _to: &AExpr) -> Self {
        ()
    }
}

impl<U0, U1> Substitutable for (U0, U1) where U0 : Substitutable, U1 : Substitutable {
    fn subst(self, from: &AExpr, to: &AExpr) -> Self {
        (self.0.subst(from, to), self.1.subst(from, to))
    }
}

impl<U0, U1, U2> Substitutable for (U0, U1, U2) where U0 : Substitutable, U1 : Substitutable, U2 : Substitutable {
    fn subst(self, from: &AExpr, to: &AExpr) -> Self {
        (self.0.subst(from, to), self.1.subst(from, to), self.2.subst(from, to))
    }
}