use std::collections::{BTreeSet, HashSet, VecDeque};
use std::fmt::Display;
use std::mem::replace;
use std::rc::Rc;

use crate::fmt::DisplayNamed;
use crate::util::fold::{Max, Min};

/// A name is a value that can be used in place of a name. Names have a full order.
/// To obtain a name, use [Name::any]. This will give any name. To obtain a name that is
/// distinct from another name, call [Name::succ].
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Clone, Copy)]
pub struct Name(u64);

impl Name {
    /// Creates any name.
    pub const fn any() -> Self {
        Self(0)
    }

    /// Get this name's successor.
    pub fn succ(&self) -> Name {
        return Name(self.0 + 1);
    }

    /// Increments this name and returns what it was before the increment.
    pub fn incr(&mut self) -> Name {
        replace(self, self.succ())
    }
}

impl Default for Name {
    fn default() -> Self {
        Self::any()
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DisplayNamed for Name {
    fn fmt_named(&self, f: &mut std::fmt::Formatter<'_>, names: &crate::fmt::NameTable) -> std::fmt::Result {
        write!(f, "{}", names.entry(self).write(self))
    }
}






/// A value that has [Name]s. Typically this is some sort of expression.
pub trait Names {
    /// Collects all the names used in this named object. It may repeat the same name multiple times,
    /// collect into some sort of set to avoid this.
    fn names<A>(&self) -> A where A : FromIterator<Name>;

    /// Test whether a specific name is used in this named object.
    fn has_name<A>(&self, name: &Name) -> bool {
        let names: BTreeSet<Name> = self.names();
        names.contains(name)
    }

    /// Returns the highest ordered name in this named object.
    fn max(&self) -> Option<Name> {
        let max: Max<Name> = self.names();
        max.result
    }

    /// Returns the lowest ordered name in this named object.
    fn min(&self) -> Option<Name> {
        let min: Min<Name> = self.names();
        min.result
    }

    /// Returns a name not used in this named object.
    fn free(&self) -> Name {
        match self.max() {
            Some(name) => name.succ(),
            None => Name::any(),
        }
    }
}

impl Names for Name {
    fn names<A>(&self) -> A where A : FromIterator<Name> {
        Some(*self).into_iter().collect()
    }

    fn has_name<A>(&self, name: &Name) -> bool {
        return *self == *name;
    }

    fn max(&self) -> Option<Name> {
        Some(*self)
    }

    fn min(&self) -> Option<Name> {
        Some(*self)
    }

    fn free(&self) -> Name {
        self.succ()
    }
}

impl<N> Names for &N where N : Names {
    fn names<A>(&self) -> A where A : FromIterator<Name> {
        (*self).names()
    }
}

impl<N> Names for Box<N> where N : Names {
    fn names<A>(&self) -> A where A : FromIterator<Name> {
        self.as_ref().names()
    }
}

impl<N> Names for Rc<N> where N : Names {
    fn names<A>(&self) -> A where A : FromIterator<Name> {
        self.as_ref().names()
    }
}

impl<N> Names for Option<N> where N : Names {
    fn names<A>(&self) -> A where A : FromIterator<Name> {
        self.into_iter().flat_map(|it| it.names::<Vec<_>>()).collect()
    }
}

impl<N> Names for Vec<N> where N : Names {
    fn names<A>(&self) -> A where A : FromIterator<Name> {
        self.into_iter().flat_map(|it| it.names::<Vec<_>>()).collect()
    }
}

impl<N> Names for VecDeque<N> where N : Names {
    fn names<A>(&self) -> A where A : FromIterator<Name> {
        self.into_iter().flat_map(|it| it.names::<Vec<_>>()).collect()
    }
}

impl<N> Names for BTreeSet<N> where N : Names {
    fn names<A>(&self) -> A where A : FromIterator<Name> {
        self.into_iter().flat_map(|it| it.names::<Vec<_>>()).collect()
    }
}

impl<N> Names for HashSet<N> where N : Names {
    fn names<A>(&self) -> A where A : FromIterator<Name> {
        self.into_iter().flat_map(|it| it.names::<Vec<_>>()).collect()
    }
}

impl<N, const S: usize> Names for [N; S] where N : Names {
    fn names<A>(&self) -> A where A : FromIterator<Name> {
        self.into_iter().flat_map(|it| it.names::<Vec<_>>()).collect()
    }
}

impl<N> Names for [N] where N : Names {
    fn names<A>(&self) -> A where A : FromIterator<Name> {
        self.into_iter().flat_map(|it| it.names::<Vec<_>>()).collect()
    }
}

impl<N1, N2> Names for (N1, N2) where N1 : Names, N2 : Names {
    fn names<A>(&self) -> A where A : FromIterator<Name> {
        let l: Vec<Name> = self.0.names();
        let r: Vec<Name> = self.1.names();

        l.into_iter().chain(r.into_iter()).collect()
    }
}

impl<N1, N2, N3> Names for (N1, N2, N3) where N1 : Names, N2 : Names, N3 : Names {
    fn names<A>(&self) -> A where A : FromIterator<Name> {
        let l: Vec<Name> = self.0.names();
        let m: Vec<Name> = self.1.names();
        let r: Vec<Name> = self.2.names();

        l.into_iter().chain(m.into_iter()).chain(r.into_iter()).collect()
    }
}