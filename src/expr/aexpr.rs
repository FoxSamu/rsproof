use std::collections::{BTreeSet, HashSet, VecDeque};
use std::fmt::Display;
use std::rc::Rc;

use crate::fmt::{write_comma_separated, DisplayNamed, NameTable};
use crate::uni::{Unifiable, Unifier};

use super::{Name, Names};

/// A non-boolean expression. Unlike [BExpr](super::BExpr), this does not evaluate to true or false.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone)]
pub enum AExpr {
    /// A bound variable. These are generated by the skolemisation of quantifiers.
    /// For example, the expression `all x: P(x)` generates the skolemisation `P(#x)` (note
    /// how we will mark bound variables with a `:` to distinguish them in documentation and
    /// debug output).
    /// However, the expression `P(x)` (given `x` is unbound) generates `P(x())`, turning `x` into a
    /// nullary function (see [AExpr::Fun]).
    Var(Name),

    /// A function call. Functions represent any unbound name, parametric or not. E.g. an unbound
    /// variable `a` will represent a nullary function `a()`.
    Fun(Name, Vec<AExpr>)
}


impl AExpr {
    pub fn con(name: Name) -> AExpr {
        AExpr::Fun(name, vec![])
    }
    
    pub fn var(name: Name) -> AExpr {
        AExpr::Var(name)
    }
    
    pub fn fun(name: Name, args: Vec<AExpr>) -> AExpr {
        AExpr::Fun(name, args)
    }
}

impl Default for AExpr {
    fn default() -> Self {
        AExpr::Fun(Default::default(), Default::default())
    }
}

impl Names for AExpr {
    fn names<A>(&self) -> A where A : FromIterator<Name> {
        match self {
            AExpr::Var(name) => name.names(),
            AExpr::Fun(name, args) => (name, args).names(),
        }
    }
}

impl Vars for AExpr {
    fn vars<A>(&self) -> A where A : FromIterator<Name> {
        match self {
            AExpr::Var(name) => name.names(),
            AExpr::Fun(_, args) => args.vars(),
        }
    }
}

impl Unifiable for AExpr {
    fn unify(self, unifier: &Unifier) -> Self {
        match self {
            AExpr::Var(name) => unifier.unify(&name),
            AExpr::Fun(name, args) => AExpr::Fun(name, args.unify(unifier)),
        }
    }

    fn can_resolve_mgu(_: &Self, _: &Self) -> bool {
        true // Of course not all AExpr pairs have an MGU but it is up to the MGU finder to figure this out
    }

    fn mgu_arguments(&self) -> Option<Vec<AExpr>> {
        Some(vec![self.clone()])
    }
}

impl DisplayNamed for AExpr {
    fn fmt_named(&self, f: &mut std::fmt::Formatter<'_>, names: &crate::fmt::NameTable) -> std::fmt::Result {
        match self {
            AExpr::Var(name) => {
                write!(f, ":{}", name.with_table(names))?;
            },
            AExpr::Fun(name, args) => {
                write!(f, "{}(", name.with_table(names))?;
                write_comma_separated(f, names, args.iter())?;
                write!(f, ")")?;
            },
        }

        Ok(())
    }
}

impl Display for AExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.with_table(&NameTable::new()).fmt(f)
    }
}



/// A value that has variables.
pub trait Vars {
    /// Collects all the names used in this named object. It may repeat the same name multiple times,
    /// collect into some sort of set to avoid this.
    fn vars<A>(&self) -> A where A : FromIterator<Name>;

    /// Test whether a specific name is used in this named object.
    fn has_var(&self, name: &Name) -> bool {
        let names: BTreeSet<Name> = self.vars();
        names.contains(name)
    }
}

impl<N> Vars for &N where N : Vars {
    fn vars<A>(&self) -> A where A : FromIterator<Name> {
        (*self).vars()
    }
}

impl<N> Vars for Box<N> where N : Vars {
    fn vars<A>(&self) -> A where A : FromIterator<Name> {
        self.as_ref().vars()
    }
}

impl<N> Vars for Rc<N> where N : Vars {
    fn vars<A>(&self) -> A where A : FromIterator<Name> {
        self.as_ref().vars()
    }
}

impl<N> Vars for Option<N> where N : Vars {
    fn vars<A>(&self) -> A where A : FromIterator<Name> {
        self.into_iter().flat_map(|it| it.vars::<Vec<_>>()).collect()
    }
}

impl<N> Vars for Vec<N> where N : Vars {
    fn vars<A>(&self) -> A where A : FromIterator<Name> {
        self.into_iter().flat_map(|it| it.vars::<Vec<_>>()).collect()
    }
}

impl<N> Vars for VecDeque<N> where N : Vars {
    fn vars<A>(&self) -> A where A : FromIterator<Name> {
        self.into_iter().flat_map(|it| it.vars::<Vec<_>>()).collect()
    }
}

impl<N> Vars for BTreeSet<N> where N : Vars {
    fn vars<A>(&self) -> A where A : FromIterator<Name> {
        self.into_iter().flat_map(|it| it.vars::<Vec<_>>()).collect()
    }
}

impl<N> Vars for HashSet<N> where N : Vars {
    fn vars<A>(&self) -> A where A : FromIterator<Name> {
        self.into_iter().flat_map(|it| it.vars::<Vec<_>>()).collect()
    }
}

impl<N, const S: usize> Vars for [N; S] where N : Vars {
    fn vars<A>(&self) -> A where A : FromIterator<Name> {
        self.into_iter().flat_map(|it| it.vars::<Vec<_>>()).collect()
    }
}

impl<N> Vars for [N] where N : Vars {
    fn vars<A>(&self) -> A where A : FromIterator<Name> {
        self.into_iter().flat_map(|it| it.vars::<Vec<_>>()).collect()
    }
}

impl<N1, N2> Vars for (N1, N2) where N1 : Vars, N2 : Vars {
    fn vars<A>(&self) -> A where A : FromIterator<Name> {
        let l: Vec<Name> = self.0.vars();
        let r: Vec<Name> = self.1.vars();

        l.into_iter().chain(r.into_iter()).collect()
    }
}

impl<N1, N2, N3> Vars for (N1, N2, N3) where N1 : Vars, N2 : Vars, N3 : Vars {
    fn vars<A>(&self) -> A where A : FromIterator<Name> {
        let l: Vec<Name> = self.0.vars();
        let m: Vec<Name> = self.1.vars();
        let r: Vec<Name> = self.2.vars();

        l.into_iter().chain(m.into_iter()).chain(r.into_iter()).collect()
    }
}

