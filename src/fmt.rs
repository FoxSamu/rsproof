use std::collections::BTreeMap;
use std::fmt::{Formatter, Write, Result, Arguments};
use std::ops::Deref;

use crate::expr::Name;

/// A formatter that supports the resolution of numeric names. It is used to print expressions that internally use
/// numeric names with their original names, given a table that maps numeric names to strings.
pub struct NamedFormatter<'l, 'f> {
    // 'l is the lifetime of references in this struct
    // 'f is the lifetime of references in the Formatter, which seems to be distinct from 'l

    formatter: &'l mut Formatter<'f>,
    name_table: Option<&'l BTreeMap<Name, String>>
}

impl<'l, 'f> Write for NamedFormatter<'l, 'f> {
    fn write_str(&mut self, s: &str) -> Result {
        self.formatter.write_str(s)
    }
    
    fn write_char(&mut self, c: char) -> Result {
        self.formatter.write_char(c)
    }

    fn write_fmt(&mut self, args: Arguments<'_>) -> Result {
        self.formatter.write_fmt(args)
    }
}

impl<'l, 'f> NamedFormatter<'l, 'f> {
    pub fn resolve_name(&mut self, n: Name) -> String {
        if let Some(table) = self.name_table {
            if let Some(name) = table.get(&n) {
                return name.clone()
            }
        }

        format!("{n}")
    }

    pub fn write_name(&mut self, n: Name) -> Result {
        if let Some(table) = self.name_table {
            if let Some(name) = table.get(&n) {
                return write!(self, "{name}")
            }
        }

        write!(self, "{n}")
    }

    pub fn write_named<D>(&mut self, nd: D) -> Result where D : NamedDisplay {
        nd.named_fmt(self)
    }
}

/// A value with the [NamedDisplay] trait has the ability to format itself to a [Formatter] given a name table.
/// The name table is responsible for mapping back numeric [Name]s to [String]s.
pub trait NamedDisplay {
    fn named_fmt(&self, f: &mut NamedFormatter) -> Result;


    fn fmt_raw(&self, f: &mut Formatter<'_>) -> Result {
        let mut nf: NamedFormatter = NamedFormatter {
            formatter: f,
            name_table: None
        };

        self.named_fmt(&mut nf)
    }

    fn fmt_named(&self, f: &mut Formatter<'_>, names: &BTreeMap<Name, String>) -> Result {
        let mut nf = NamedFormatter {
            formatter: f,
            name_table: Some(names)
        };

        self.named_fmt(&mut nf)
    }
}

// This here implements NamedDisplay on any sort of reference to a NamedDisplay, allowing us to pass
// things like a &Box<NamedDisplay> as argument where a NamedDisplay is expected, and saving us from weirdly
// looking `&**n` re-de-dereferences
impl<D, N> NamedDisplay for D where D : Deref<Target = N>, N : NamedDisplay {
    fn named_fmt(&self, f: &mut NamedFormatter) -> Result {
        self.deref().named_fmt(f)
    }
}
