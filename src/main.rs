mod expro;
mod cnf;
mod res;
mod parse;
mod proof;
mod fmto;
mod unify;

use std::process::ExitCode;

/// App.
mod app;

/// Expression trees.
mod expr;

/// Normal forms (CNF and DNF).
mod nf;

/// Unification and unifiers.
mod uni;

/// Formatting module that formats numeric names with human-readable names.
mod fmt;

/// Parsing module.
mod parser;

/// Miscellaneous utilities.
mod util;

#[cfg(test)]
mod test;


fn main() -> ExitCode {
    app::main()
}
