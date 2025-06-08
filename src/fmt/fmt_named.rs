use std::collections::{BTreeSet, HashSet};
use std::fmt::{Debug, Display, Formatter, Result};
use std::ops::Deref;
use std::rc::Rc;

use crate::expr::Names;

use super::NameTable;

pub trait DisplayNamed {
    fn fmt_named(&self, f: &mut Formatter<'_>, names: &NameTable) -> Result;

    fn with_table<'elem, 'table>(&'elem self, table: &'table NameTable) -> WithTable<'elem, 'table, Self> where Self : Sized {
        WithTable { elem: self, table }
    }
}

pub fn write_comma_separated<'a, N>(f: &mut Formatter<'_>, names: &NameTable, elems: impl Iterator<Item = N>) -> Result where N : DisplayNamed + 'a {
    let mut comma = false;

    for elem in elems {
        if comma { write!(f, ", ")?; } else { comma = true; }

        elem.fmt_named(f, names)?;
    };

    Ok(())
}


pub struct WithTable<'elem, 'table, N> where N : DisplayNamed + Sized {
    elem: &'elem N,
    table: &'table NameTable
}

impl<'elem, 'table, N> Display for WithTable<'elem, 'table, N> where N : DisplayNamed + Sized {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.elem.fmt_named(f, self.table)
    }
}

impl<'elem, 'table, N> Debug for WithTable<'elem, 'table, N> where N : DisplayNamed + Sized {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.elem.fmt_named(f, self.table)
    }
}



impl<T> DisplayNamed for &T where T : DisplayNamed {
    fn fmt_named(&self, f: &mut Formatter<'_>, names: &NameTable) -> Result {
        (*self).fmt_named(f, names)
    }
}

impl<T> DisplayNamed for Box<T> where T : DisplayNamed {
    fn fmt_named(&self, f: &mut Formatter<'_>, names: &NameTable) -> Result {
        self.deref().fmt_named(f, names)
    }
}

impl<T> DisplayNamed for Rc<T> where T : DisplayNamed {
    fn fmt_named(&self, f: &mut Formatter<'_>, names: &NameTable) -> Result {
        self.deref().fmt_named(f, names)
    }
}

impl<T> DisplayNamed for Option<T> where T : DisplayNamed {
    fn fmt_named(&self, f: &mut Formatter<'_>, names: &NameTable) -> Result {
        match self {
            Some(elem) => write!(f, "Some({})", elem.with_table(names)),
            None => write!(f, "None"),
        }
    }
}

impl<T> DisplayNamed for Vec<T> where T : DisplayNamed {
    fn fmt_named(&self, f: &mut Formatter<'_>, names: &NameTable) -> Result {
        let mut comma = false;

        write!(f, "[")?;
        write_comma_separated(f, names, self.iter())?;
        write!(f, "]")?;

        Ok(())
    }
}

impl<T> DisplayNamed for BTreeSet<T> where T : DisplayNamed {
    fn fmt_named(&self, f: &mut Formatter<'_>, names: &NameTable) -> Result {
        let mut comma = false;

        write!(f, "{{")?;
        write_comma_separated(f, names, self.iter())?;
        write!(f, "}}")?;

        Ok(())
    }
}

impl<T> DisplayNamed for HashSet<T> where T : DisplayNamed {
    fn fmt_named(&self, f: &mut Formatter<'_>, names: &NameTable) -> Result {
        let mut comma = false;

        write!(f, "{{")?;
        write_comma_separated(f, names, self.iter())?;
        write!(f, "}}")?;

        Ok(())
    }
}

impl<T0, T1> DisplayNamed for (T0, T1) where T0 : DisplayNamed, T1 : DisplayNamed {
    fn fmt_named(&self, f: &mut Formatter<'_>, names: &NameTable) -> Result {
        write!(f, "({}, {})", self.0.with_table(names), self.1.with_table(names))?;

        Ok(())
    }
}