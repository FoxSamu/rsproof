pub mod expro;
pub mod cnf;
pub mod reso;
pub mod parse;
pub mod proof;
pub mod fmto;
pub mod unify;

/// App.
pub mod app;

/// Expression trees.
pub mod expr;

/// Normal forms (CNF and DNF).
pub mod nf;

/// Unification and unifiers.
pub mod uni;

/// Formatting module that formats numeric names with human-readable names.
pub mod fmt;

/// Parsing module.
pub mod parser;

/// Resolution engine.
pub mod res;

/// Miscellaneous utilities.
pub mod util;

#[cfg(test)]
pub mod test;
