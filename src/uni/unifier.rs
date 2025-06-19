use std::collections::btree_map::IntoIter;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::hash::Hash;
use std::rc::Rc;

use crate::expr::{AExpr, Name, Names, Vars};
use crate::fmt::{write_comma_separated, DisplayNamed};

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
    /// - There is a substitution `x := a` where `to` is a variable of `a`.
    /// - There is a substitution `from := a` for some expression `a`.
    /// - `from` is a variable of `to`.
    /// 
    /// In other words, it panics when the integrity of the unifier would be
    /// violated.
    pub fn add(&mut self, from: Name, to: AExpr) {
        self.try_add(from, to).unwrap()
    }

    /// Attempts to add a new substitution `from := to` to the unifier. Returns an error when:
    /// - There is a substitution `x := a` where `to` is a variable of `a`.
    /// - There is a substitution `from := a` for some expression `a`.
    /// - `from` is a variable of `to`.
    /// 
    /// In other words, it returns an error when the integrity of the unifier would be
    /// violated.
    pub fn try_add(&mut self, from: Name, to: AExpr) -> Result<(), &'static str> {
        if self.table.contains_key(&from) {
            return Err("Left hand side of substitution is already the left hand side of another substitution.");
        }

        if self.table.values().any(|e| e.has_var(&from)) {
            return Err("Left hand side of substitution appears on the right hand side of another substitution.");
        }

        if to.has_var(&from) {
            return Err("Added substitution is recursive.");
        }

        self.table.insert(from, to);

        Ok(())
    }

    pub fn remove(&mut self, name: &Name) -> bool {
        self.table.remove(name).is_some()
    }

    pub fn without(mut self, name: &Name) -> Self {
        self.remove(name);
        self
    }

    pub fn clone_without(&self, name: &Name) -> Self {
        self.clone().without(name)
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


    /// Returns an iterator over the borrowed substitution pairs in this [Unifier].
    /// To move the elements, use [Self::into_iter].
    pub fn iter(&self) -> impl Iterator<Item = (&Name, &AExpr)> {
        self.table.iter()
    }


    /// Attempts to find the Most General Unifier between two [Unifiable] values.
    /// 
    /// Generally, an MGU can only exist if all of these conditions hold:
    /// 1. [Unifiable::can_resolve_mgu] returns `true` for the two inputs.
    /// 2. Both inputs returned [Some] from [Unifiable::mgu_arguments].
    /// 3. The MGU argument vectors from the previous step have the same length.
    /// 
    /// If an MGU exists, the MGU is returned. Otherwise, [None] is returned.
    pub fn mgu<U>(left: &U, right: &U) -> Option<Self> where U : Unifiable {
        if !U::can_resolve_mgu(left, right) {
            return None;
        }

        let l = left.mgu_arguments()?;
        let r = right.mgu_arguments()?;

        super::mgu::mgu(l, r)
    }
}


impl TryFrom<BTreeMap<Name, AExpr>> for Unifier {
    type Error = &'static str;

    fn try_from(value: BTreeMap<Name, AExpr>) -> Result<Self, Self::Error> {
        let mut new = Self::new();

        for (l, r) in value {
            new.try_add(l, r)?;
        }

        Ok(new)
    }
}

impl TryFrom<BTreeSet<(Name, AExpr)>> for Unifier {
    type Error = &'static str;

    fn try_from(value: BTreeSet<(Name, AExpr)>) -> Result<Self, Self::Error> {
        let mut new = Self::new();

        for (l, r) in value {
            new.try_add(l, r)?;
        }

        Ok(new)
    }
}

impl TryFrom<Vec<(Name, AExpr)>> for Unifier {
    type Error = &'static str;

    fn try_from(value: Vec<(Name, AExpr)>) -> Result<Self, Self::Error> {
        let mut new = Self::new();

        for (l, r) in value {
            new.try_add(l, r)?;
        }

        Ok(new)
    }
}

impl Into<BTreeMap<Name, AExpr>> for Unifier {
    fn into(self) -> BTreeMap<Name, AExpr> {
        self.table
    }
}

impl Into<BTreeSet<(Name, AExpr)>> for Unifier {
    fn into(self) -> BTreeSet<(Name, AExpr)> {
        self.table.into_iter().collect()
    }
}

impl Into<Vec<(Name, AExpr)>> for Unifier {
    fn into(self) -> Vec<(Name, AExpr)> {
        self.table.into_iter().collect()
    }
}

impl IntoIterator for Unifier {
    type Item = (Name, AExpr);

    type IntoIter = IntoIter<Name, AExpr>;

    fn into_iter(self) -> Self::IntoIter {
        self.table.into_iter()
    }
}

impl Names for Unifier {
    fn names<A>(&self) -> A where A : FromIterator<Name> {
        self.iter().flat_map(|pair| pair.names::<Vec<_>>()).collect()
    }
}

impl Vars for Unifier {
    fn vars<A>(&self) -> A where A : FromIterator<Name> {
        self.iter().flat_map(|(l, r)| {
            let mut v = r.vars::<Vec<_>>();
            v.push(*l);
            v
        }).collect()
    }
}

impl DisplayNamed for Unifier {
    fn fmt_named(&self, f: &mut std::fmt::Formatter<'_>, names: &crate::fmt::NameTable) -> std::fmt::Result {
        struct Sub<'a>(&'a Name, &'a AExpr);

        impl<'a> Names for Sub<'a> {
            fn names<A>(&self) -> A where A : FromIterator<Name> {
                (self.0, self.1).names()
            }
        }

        impl<'a> DisplayNamed for Sub<'a> {
            fn fmt_named(&self, f: &mut std::fmt::Formatter<'_>, names: &crate::fmt::NameTable) -> std::fmt::Result {
                write!(f, "{} = {}", self.0.with_table(names), self.1.with_table(names))
            }
        }

        write!(f, "{{")?;
        write_comma_separated(f, names, self.iter().map(|(l, r)| Sub(l, r)))?;
        write!(f, "}}")?;

        Ok(())
    }
}






/// A value that can be transformed with a [Unifier]. Between such values, MGUs often exist, so
/// this trait also facilitates the methods that help finding MGUs. See [Unifier::mgu].
pub trait Unifiable {
    /// Unify this value.
    fn unify(self, unifier: &Unifier) -> Self;

    /// Test if two values of this type can be resolved into an MGU.
    /// - When this returns `false`, the [MGU][Unifier::mgu] between the two values
    ///   does for sure not exist.
    /// - When this returns `true`, the [MGU][Unifier::mgu] between the two values
    ///   **may** exist, but it does not necessarily have to exist.
    /// 
    /// This function is mostly just checks if it makes any sense to find a MGU
    /// between the two given values. Some values can never have a MGU with any other value,
    /// some values can only have an MGU with another value of similar form, and some types don't
    /// even have MGUs at all.
    fn can_resolve_mgu(a: &Self, b: &Self) -> bool;

    /// Extract a vector of terms to find an [MGU][Unifier::mgu] over.
    /// 
    /// If no MGU can ever exist between this value and any other, then this function returns [None].
    /// That does not mean that when this function returns [Some] vector, that it makes any sense to
    /// find an MGU with any other value.
    ///
    /// Before this is called, one must *always* check [`can_resolve_mgu`][Unifiable::can_resolve_mgu]
    /// with this value.
    /// This function may panic if [`can_resolve_mgu`][Unifiable::can_resolve_mgu] returned
    /// `false` when this value was one of the arguments. This is because for some values
    /// it is simply not possible to find an MGU. Only when [`can_resolve_mgu`][Unifiable::can_resolve_mgu]
    /// returns `true`, is it valid to call [`mgu_arguments`][Unifiable::mgu_arguments].
    fn mgu_arguments(&self) -> Option<Vec<AExpr>>;
}

impl<U> Unifiable for Box<U> where U : Unifiable {
    fn unify(self, unifier: &Unifier) -> Self {
        Box::new((*self).unify(unifier))
    }

    fn can_resolve_mgu(a: &Self, b: &Self) -> bool {
        U::can_resolve_mgu(a.as_ref(), b.as_ref())
    }
    
    fn mgu_arguments(&self) -> Option<Vec<AExpr>> {
        self.as_ref().mgu_arguments()
    }
}

impl<U> Unifiable for Option<U> where U : Unifiable {
    fn unify(self, unifier: &Unifier) -> Self {
        self.map(|it| it.unify(unifier))
    }

    fn can_resolve_mgu(a: &Self, b: &Self) -> bool {
        match (a, b) {
            (None, None) => true,
            (None, Some(_)) => false,
            (Some(_), None) => false,
            (Some(a), Some(b)) => U::can_resolve_mgu(a, b),
        }
    }

    fn mgu_arguments(&self) -> Option<Vec<AExpr>> {
        match self {
            Some(val) => val.mgu_arguments(),
            None => Some(vec![]),
        }
    }
}

impl<U> Unifiable for Vec<U> where U : Unifiable {
    fn unify(self, unifier: &Unifier) -> Self {
        self.into_iter().map(|it| it.unify(unifier)).collect()
    }

    fn can_resolve_mgu(a: &Self, b: &Self) -> bool {
        if a.len() != b.len() {
            return false;
        }

        a.iter().zip(b.iter())
            .all(|(l, r)| U::can_resolve_mgu(l, r))
    }

    fn mgu_arguments(&self) -> Option<Vec<AExpr>> {
        let mut result = Vec::new();
        for elem in self {
            result.append(&mut elem.mgu_arguments()?);
        }

        Some(result)
    }
}

impl<U> Unifiable for HashSet<U> where U : Unifiable + Hash + Eq {
    fn unify(self, unifier: &Unifier) -> Self {
        self.into_iter().map(|it| it.unify(unifier)).collect()
    }

    fn can_resolve_mgu(a: &Self, b: &Self) -> bool {
        if a.len() != b.len() {
            return false;
        }

        a.iter().zip(b.iter())
            .all(|(l, r)| U::can_resolve_mgu(l, r))
    }

    fn mgu_arguments(&self) -> Option<Vec<AExpr>> {
        let mut result = Vec::new();
        for elem in self {
            result.append(&mut elem.mgu_arguments()?);
        }

        Some(result)
    }
}

impl<U> Unifiable for BTreeSet<U> where U : Unifiable + Ord {
    fn unify(self, unifier: &Unifier) -> Self {
        self.into_iter().map(|it| it.unify(unifier)).collect()
    }

    fn can_resolve_mgu(a: &Self, b: &Self) -> bool {
        if a.len() != b.len() {
            return false;
        }

        a.iter().zip(b.iter())
            .all(|(l, r)| U::can_resolve_mgu(l, r))
    }

    fn mgu_arguments(&self) -> Option<Vec<AExpr>> {
        let mut result = Vec::new();
        for elem in self {
            result.append(&mut elem.mgu_arguments()?);
        }

        Some(result)
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

    fn can_resolve_mgu(a: &Self, b: &Self) -> bool {
        if a.len() != b.len() {
            return false;
        }

        a.iter().zip(b.iter())
            .all(|(l, r)| U::can_resolve_mgu(l, r))
    }

    fn mgu_arguments(&self) -> Option<Vec<AExpr>> {
        let mut result = Vec::new();
        for elem in self {
            result.append(&mut elem.mgu_arguments()?);
        }

        Some(result)
    }
}

impl Unifiable for () {
    fn unify(self, _: &Unifier) -> Self {
        ()
    }

    fn can_resolve_mgu(_: &Self, _: &Self) -> bool {
        true
    }

    fn mgu_arguments(&self) -> Option<Vec<AExpr>> {
        Some(vec![])
    }
}

impl<U0, U1> Unifiable for (U0, U1) where U0 : Unifiable, U1 : Unifiable {
    fn unify(self, unifier: &Unifier) -> Self {
        (self.0.unify(unifier), self.1.unify(unifier))
    }

    fn can_resolve_mgu(a: &Self, b: &Self) -> bool {
           U0::can_resolve_mgu(&a.0, &b.0)
        && U1::can_resolve_mgu(&a.1, &b.1)
    }

    fn mgu_arguments(&self) -> Option<Vec<AExpr>> {
        let mut vec = Vec::new();
        vec.append(&mut self.0.mgu_arguments()?);
        vec.append(&mut self.1.mgu_arguments()?);
        Some(vec)
    }
}

impl<U0, U1, U2> Unifiable for (U0, U1, U2) where U0 : Unifiable, U1 : Unifiable, U2 : Unifiable {
    fn unify(self, unifier: &Unifier) -> Self {
        (self.0.unify(unifier), self.1.unify(unifier), self.2.unify(unifier))
    }

    fn can_resolve_mgu(a: &Self, b: &Self) -> bool {
           U0::can_resolve_mgu(&a.0, &b.0)
        && U1::can_resolve_mgu(&a.1, &b.1)
        && U2::can_resolve_mgu(&a.2, &b.2)
    }

    fn mgu_arguments(&self) -> Option<Vec<AExpr>> {
        let mut vec = Vec::new();
        vec.append(&mut self.0.mgu_arguments()?);
        vec.append(&mut self.1.mgu_arguments()?);
        vec.append(&mut self.2.mgu_arguments()?);
        Some(vec)
    }
}