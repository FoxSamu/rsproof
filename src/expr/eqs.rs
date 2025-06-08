use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::uni::Unifier;

use super::AExpr;

/// A collection of equalities.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash)]
pub struct Eqs {
    eqs: BTreeSet<(AExpr, AExpr)>
}

impl Eqs {
    pub fn new() -> Self {
        Self { eqs: BTreeSet::new() }
    }

    pub fn eqs(&self) -> &BTreeSet<(AExpr, AExpr)> {
        &self.eqs
    }

    pub fn into_eqs(self) -> BTreeSet<(AExpr, AExpr)> {
        self.eqs
    }

    pub fn unzip(self) -> (Vec<AExpr>, Vec<AExpr>) {
        self.eqs.into_iter().unzip()
    }
}

impl Default for Eqs {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<(AExpr, AExpr)>> for Eqs {
    fn from(value: Vec<(AExpr, AExpr)>) -> Self {
        Self { eqs: value.into_iter().collect() }
    }
}

impl From<BTreeSet<(AExpr, AExpr)>> for Eqs {
    fn from(value: BTreeSet<(AExpr, AExpr)>) -> Self {
        Self { eqs: value }
    }
}

impl From<BTreeMap<AExpr, AExpr>> for Eqs {
    fn from(value: BTreeMap<AExpr, AExpr>) -> Self {
        Self { eqs: value.into_iter().collect() }
    }
}

impl From<HashSet<(AExpr, AExpr)>> for Eqs {
    fn from(value: HashSet<(AExpr, AExpr)>) -> Self {
        Self { eqs: value.into_iter().collect() }
    }
}

impl From<HashMap<AExpr, AExpr>> for Eqs {
    fn from(value: HashMap<AExpr, AExpr>) -> Self {
        Self { eqs: value.into_iter().collect() }
    }
}

impl From<Unifier> for Eqs {
    fn from(value: Unifier) -> Self {
        Self { eqs: value.into_iter().map(|(l, r)| (AExpr::Var(l), r)).collect() }
    }
}

impl TryInto<Unifier> for Eqs {
    type Error = &'static str;

    fn try_into(self) -> Result<Unifier, Self::Error> {
        let mut uni = Unifier::new();

        for (l, r) in self.eqs {
            if let AExpr::Var(v) = l {
                uni.try_add(v, r)?;
            } else {
                return Err("Some left hand side in equation set is not a variable.");
            }
        }

        Ok(uni)
    }
}