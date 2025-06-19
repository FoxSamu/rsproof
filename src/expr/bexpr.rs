use std::fmt::Display;
use std::mem::replace;
use std::ops::BitAnd;
use std::ops::BitAndAssign;
use std::ops::BitOr;
use std::ops::BitOrAssign;
use std::ops::Not;

use crate::fmt::write_comma_separated;
use crate::fmt::DisplayNamed;
use crate::fmt::NameTable;
use crate::uni::{Unifiable, Unifier};

use super::Name;
use super::AExpr;
use super::Names;
use super::Vars;

/// A Boolean expression, i.e. any expression that evaluates to true or false.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone)]
pub enum BExpr {
    /// The true constant. i.e. the undoubted tautology whose truth nobody could possibly ever question.
    True,

    /// The false constant. i.e. the undoubted contradiction whose falsehood nobody could possibly ever question.
    False,

    /// An atomic unit of truth, one with no specific implication behind it. It is an uninterpreted
    /// function from some non-boolean values to a boolean value.
    /// A predicate is represented by a name and a vector of arguments. The name and the size
    /// of the arguments vector uniquely identify the predicate. A predicate with zero arguments
    /// is called a symbol. The arguments of a predicate are [AExpr]s.
    Pred(Name, Vec<AExpr>),

    /// A conjunction of two Boolean subexpressions. A conjunction is true if and only if both
    /// of its subexpressions are true. The two subexpressions are [BExpr]s, boxed to satisfy
    /// Rust's memory requirments.
    And(Box<BExpr>, Box<BExpr>),

    /// A disjunction of two Boolean subexpressions. A disjunction is true if and only if at least one
    /// of its subexpressions is true. The two subexpressions are [BExpr]s, boxed to satisfy
    /// Rust's memory requirments.
    Or(Box<BExpr>, Box<BExpr>),

    /// The inverse of a Boolean subexpression. The inverse if true if and only if its subexpression is
    /// false. The subexpression is a [BExpr], boxed to satisfy Rust's memory requirements.
    Not(Box<BExpr>),

    /// A statement quantified by universal quantification
    All(Name, Box<BExpr>),

    /// A statement quantified by existential quantification
    Some(Name, Box<BExpr>),
}


impl BExpr {
    pub fn bool(val: bool) -> BExpr {
        match val {
            true => BExpr::True,
            false => BExpr::False
        }
    }

    pub fn sym(name: Name) -> BExpr {
        BExpr::Pred(name, vec![])
    }

    pub fn pred(name: Name, args: Vec<AExpr>) -> BExpr {
        BExpr::Pred(name, args)
    }

    pub fn and(lhs: BExpr, rhs: BExpr) -> BExpr {
        lhs & rhs
    }

    pub fn or(lhs: BExpr, rhs: BExpr) -> BExpr {
        lhs | rhs
    }

    pub fn not(rhs: BExpr) -> BExpr {
        !rhs
    }

    pub fn im(lhs: BExpr, rhs: BExpr) -> BExpr {
        !lhs | rhs
    }

    pub fn revim(lhs: BExpr, rhs: BExpr) -> BExpr {
        lhs | !rhs
    }

    pub fn equiv(lhs: BExpr, rhs: BExpr) -> BExpr {
        let lhs2 = lhs.clone();
        let rhs2 = rhs.clone();
        (lhs | !rhs) & (!lhs2 | rhs2)
    }

    pub fn all(name: Name, rhs: BExpr) -> BExpr {
        BExpr::All(name, Box::new(rhs))
    }

    pub fn some(name: Name, rhs: BExpr) -> BExpr {
        BExpr::Some(name, Box::new(rhs))
    }

    pub fn no(name: Name, rhs: BExpr) -> BExpr {
        !BExpr::some(name, rhs)
    }


    /// Tests whether this [BExpr] is quantifier-free, that is, whether this [BExpr]
    /// contains no quantifiers.
    pub fn is_quantifier_free(&self) -> bool {
        match self {
            BExpr::True => true,
            BExpr::False => true,
            BExpr::Pred(_, _) => true,

            BExpr::And(lhs, rhs) => lhs.is_quantifier_free() && rhs.is_quantifier_free(),
            BExpr::Or(lhs, rhs) => lhs.is_quantifier_free() && rhs.is_quantifier_free(),
            BExpr::Not(rhs) => rhs.is_quantifier_free(),

            // These are the culprits
            BExpr::All(_, _) => false,
            BExpr::Some(_, _) => false,
        }
    }

    /// Tests whether this [BExpr] is a sentence, that is, whether it contains no
    /// unbound variables.
    pub fn is_sentence(&self) -> bool {
        let vars: Vec<_> = self.vars();
        vars.is_empty()
    }
}

impl Default for BExpr {
    fn default() -> Self {
        BExpr::False
    }
}

impl BitAnd for BExpr {
    type Output = BExpr;

    fn bitand(self, rhs: Self) -> Self::Output {
        BExpr::And(Box::new(self), Box::new(rhs))
    }
}

impl BitAndAssign for BExpr {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = replace(self, BExpr::True) & rhs;
    }
}

impl BitOr for BExpr {
    type Output = BExpr;

    fn bitor(self, rhs: Self) -> Self::Output {
        BExpr::Or(Box::new(self), Box::new(rhs))
    }
}

impl BitOrAssign for BExpr {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = replace(self, BExpr::True) | rhs;
    }
}

impl Not for BExpr {
    type Output = BExpr;
    
    fn not(self) -> Self::Output {
        BExpr::Not(Box::new(self))
    }
}


impl Names for BExpr {
    fn names<A>(&self) -> A where A : FromIterator<Name> {
        match self {
            BExpr::True | BExpr::False => None.into_iter().collect(),
            BExpr::Pred(name, args) => (name, args).names(),
            BExpr::And(lhs, rhs) => (lhs, rhs).names(),
            BExpr::Or(lhs, rhs) => (lhs, rhs).names(),
            BExpr::Not(rhs) => rhs.names(),

            BExpr::All(name, rhs) => (name, rhs).names(),
            BExpr::Some(name, rhs) => (name, rhs).names(),
        }
    }
}

impl Vars for BExpr {
    fn vars<A>(&self) -> A where A : FromIterator<Name> {
        match self {
            BExpr::True | BExpr::False => None.into_iter().collect(),
            BExpr::Pred(_, args) => args.vars(),
            BExpr::And(lhs, rhs) => (lhs, rhs).vars(),
            BExpr::Or(lhs, rhs) => (lhs, rhs).vars(),
            BExpr::Not(rhs) => rhs.vars(),

            // Bound variables don't count as unbound variables
            BExpr::All(name, rhs) | BExpr::Some(name, rhs) => {
                let mut vars = rhs.vars::<Vec<_>>();

                vars.retain(|e| e != name);
                vars.into_iter().collect()
            }
        }
    }

    fn has_var(&self, v: &Name) -> bool {
        match self {
            BExpr::True | BExpr::False => false,
            BExpr::Pred(_, args) => args.has_var(v),
            BExpr::And(lhs, rhs) => lhs.has_var(v) || rhs.has_var(v),
            BExpr::Or(lhs, rhs) => lhs.has_var(v) || rhs.has_var(v),
            BExpr::Not(rhs) => rhs.has_var(v),

            // Bound variables don't count as unbound variables
            BExpr::All(name, rhs) | BExpr::Some(name, rhs) => {
                if v == name {
                    false
                } else {
                    rhs.has_var(v)
                }
            }
        }
    }
}

impl Unifiable for BExpr {
    fn unify(self, unifier: &Unifier) -> Self {
        match self {
            BExpr::True => BExpr::True,
            BExpr::False => BExpr::False,
            BExpr::Pred(name, args) => BExpr::Pred(name, args.unify(unifier)),
            BExpr::And(lhs, rhs) => BExpr::And(lhs.unify(unifier), rhs.unify(unifier)),
            BExpr::Or(lhs, rhs) => BExpr::Or(lhs.unify(unifier), rhs.unify(unifier)),
            BExpr::Not(rhs) => BExpr::Not(rhs.unify(unifier)),

            // The name a quantifier binds to redefines what that variable means - it is essentially
            // not a variable anymore. To counteract that the unifier unifies the newly bound variable,
            // we create a copy unifier that does not have the substitution for that variable.
            //
            // This is not very efficient since we're copying the whole unifier, but the best we can do.
            // If all goes well, we should not be unifying BExprs anyway.
            BExpr::All(name, rhs) => BExpr::All(name, rhs.unify(&unifier.clone_without(&name))),
            BExpr::Some(name, rhs) => BExpr::Some(name, rhs.unify(&unifier.clone_without(&name))),
        }
    }

    fn can_resolve_mgu(a: &Self, b: &Self) -> bool {
        match (a, b) {
            (BExpr::Pred(p, ps), BExpr::Pred(q, qs)) => p == q && Vec::can_resolve_mgu(ps, qs),

            (BExpr::True, BExpr::True) => true,
            (BExpr::False, BExpr::False) => true,

            // We cannot find MGUs between boolean operators or quantifiers
            _ => false
        }
    }
    
    fn mgu_arguments(&self) -> Option<Vec<AExpr>> {
        match self {
            BExpr::Pred(_, args) => Some(args.clone()),

            BExpr::True => Some(vec![]),
            BExpr::False => Some(vec![]),

            // We cannot find MGUs between boolean operators
            _ => None
        }
    }
}

impl DisplayNamed for BExpr {
    fn fmt_named(&self, f: &mut std::fmt::Formatter<'_>, names: &crate::fmt::NameTable) -> std::fmt::Result {
        match self {
            BExpr::True => write!(f, "true")?,
            BExpr::False => write!(f, "false")?,
            BExpr::Pred(name, args) => {
                write!(f, "{}(", name.with_table(names))?;
                write_comma_separated(f, names, args.iter())?;
                write!(f, ")")?;
            },
            BExpr::And(lhs, rhs) => {
                write!(f, "({} & {})", lhs.with_table(names), rhs.with_table(names))?;
            },
            BExpr::Or(lhs, rhs) => {
                write!(f, "({} | {})", lhs.with_table(names), rhs.with_table(names))?;
            },
            BExpr::Not(rhs) => {
                write!(f, "!({})", rhs.with_table(names))?;
            },
            BExpr::All(name, rhs) => {
                write!(f, "all {}: ({})", name.with_table(names), rhs.with_table(names))?;
            },
            BExpr::Some(name, rhs) => {
                write!(f, "some {}: ({})", name.with_table(names), rhs.with_table(names))?;
            },
        }

        Ok(())
    }
}

impl Display for BExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.with_table(&NameTable::new()).fmt(f)
    }
}


