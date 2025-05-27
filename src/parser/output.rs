use crate::expr::Name;
use crate::fmt::NameTable;


/// A parser output, consisting of a resulting value and a name table that maps numeric names back to the
/// identifiers that the parser encountered.
pub struct Output<O> {
    pub result: O,
    pub name_table: NameTable
}

