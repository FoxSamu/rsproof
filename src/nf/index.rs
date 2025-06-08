use std::collections::{BTreeMap, BTreeSet};
use std::mem::{replace, take};

use crate::expr::{AExpr, Name, Names, Vars};
use crate::nf::Atom;
use crate::uni::Unifiable;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash)]
pub struct PredicateIndex {
    preds: BTreeMap<Name, BTreeSet<Vec<AExpr>>>
}

impl PredicateIndex {
    pub fn new() -> Self {
        Self {
            preds: BTreeMap::new()
        }
    }

    pub fn clear(&mut self) {
        self.preds.clear();
    }

    pub fn insert(&mut self, atom: Atom) -> bool {
        match atom {
            Atom::Pred(name, args) => self.insert_pred(name, args)
        }
    }

    pub fn remove(&mut self, atom: &Atom) -> bool {
        match atom {
            Atom::Pred(name, args) => self.remove_pred(name, args)
        }
    }

    pub fn contains(&self, atom: &Atom) -> bool {
        match atom {
            Atom::Pred(name, args) => self.contains_pred(name, args)
        }
    }

    pub fn insert_pred(&mut self, pred: Name, args: Vec<AExpr>) -> bool {
        let set = self.preds.entry(pred).or_insert_with(|| BTreeSet::new());
        set.insert(args)
    }

    pub fn remove_pred(&mut self, pred: &Name, args: &Vec<AExpr>) -> bool {
        let mut empty = false;
        let mut rmv = false;

        if let Some(r) = self.preds.get_mut(pred) {
            rmv = r.remove(args);
            empty = r.is_empty();
        }

        if empty {
            self.preds.remove(pred);
        }

        rmv
    }

    pub fn contains_pred(&self, pred: &Name, args: &Vec<AExpr>) -> bool {
        if let Some(r) = self.preds.get(pred) {
            r.contains(args)
        } else {
            false
        }
    }

    pub fn remove_preds(&mut self, pred: &Name) -> bool {
        if let Some(r) = self.preds.remove(pred) {
            !r.is_empty()
        } else {
            false
        }
    }

    pub fn get_preds(&self, pred: &Name) -> Option<&BTreeSet<Vec<AExpr>>> {
        self.preds.get(pred).filter(|set| !set.is_empty())
    }

    pub fn contains_preds(&self, pred: &Name) -> bool {
        self.preds.get(pred).filter(|set| !set.is_empty()).is_some()
    }
    

    pub fn iter_pred_names(&self) -> impl Iterator<Item = &Name> {
        self.preds.keys()
    }

    pub fn iter_preds(&self) -> impl Iterator<Item = (Name, &Vec<AExpr>)> {
        self.preds.iter().flat_map(|(name, set)| {
            set.iter().map(|args| (*name, args))
        })
    }



    pub fn union(mut self, other: Self) -> Self {
        for (name, set) in other.preds.into_iter() {
            for args in set {
                self.insert_pred(name, args);
            }
        }

        self
    }

    pub fn is_empty(&self) -> bool {
        self.preds.is_empty() || self.preds.values().all(|it| it.is_empty())
    }

    pub fn is_disjoint(&self, other: &Self) -> bool {
        for (name, set) in other.preds.iter() {
            if let Some(own_set) = self.get_preds(name) {
                if !own_set.is_disjoint(set) {
                    return false;
                }
            }
        }

        true
    }
}

impl From<Atom> for PredicateIndex {
    fn from(value: Atom) -> Self {
        let mut new = Self::new();

        match value {
            Atom::Pred(name, args) => {
                new.insert_pred(name, args);
            }
        }

        new
    }
}

impl From<Vec<Atom>> for PredicateIndex {
    fn from(value: Vec<Atom>) -> Self {
        let mut new = Self::new();
        
        for Atom::Pred(name, args) in value {
            new.insert_pred(name, args);
        }

        new
    }
}

impl From<BTreeSet<Atom>> for PredicateIndex {
    fn from(value: BTreeSet<Atom>) -> Self {
        let mut new = Self::new();
        
        for Atom::Pred(name, args) in value {
            new.insert_pred(name, args);
        }

        new
    }
}

impl<const N: usize> From<[Atom; N]> for PredicateIndex {
    fn from(value: [Atom; N]) -> Self {
        let mut new = Self::new();
        
        for Atom::Pred(name, args) in value {
            new.insert_pred(name, args);
        }

        new
    }
}


impl Names for PredicateIndex {
    fn names<A>(&self) -> A where A : FromIterator<Name> {
        return self.preds.iter().flat_map(|it| it.names::<Vec<_>>()).collect();
    }
}

impl Vars for PredicateIndex {
    fn vars<A>(&self) -> A where A : FromIterator<Name> {
        return self.preds.values().flat_map(|it| it.names::<Vec<_>>()).collect();
    }
}

impl Unifiable for PredicateIndex {
    fn unify(mut self, unifier: &crate::uni::Unifier) -> Self {
        let mut empty = BTreeSet::new();

        for set in self.preds.values_mut() {
            let new_set = replace(set, empty).unify(unifier);
            empty = replace(set, new_set)
        }

        self
    }

    fn can_resolve_mgu(a: &Self, b: &Self) -> bool {
        false
    }

    fn mgu_arguments(&self) -> Option<Vec<AExpr>> {
        None
    }
}