mod expro;
mod cnf;
mod res;
mod parse;
mod proof;
mod fmto;
mod unify;

use std::collections::BTreeSet;
use std::env;
use std::io::stdin;
use std::process::ExitCode;

use cnf::Clause;
use parse::parse;
use proof::format_proof;
use res::resolution;

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
    let args: BTreeSet<String> = env::args().collect();
    let verbose = args.contains(&String::from("-v"));

    // Parse stdin
    let parsed = parse(stdin());

    if let Err(msg) = parsed {
        println!("Syntax error: {msg}");
        return ExitCode::FAILURE;
    }

    // Convert to CNF
    let (expr, name_table) = parsed.unwrap();
    let cnf = dbg!(expr.to_cnf());
    let clauses = dbg!(Clause::from_cnf(&cnf));

    // Resolve
    let resolution = resolution(&clauses);

    // The parser states the input as a "proof by contradiction"
    // so if we find a contradiction then what we prove is satisifed.
    if !resolution.satisfied {
        println!("sat");
    } else {
        println!("unsat");
    }

    if verbose {
        println!("Clauses learned:    {}", resolution.clauses_learned);
        if let Some(proof) = resolution.proof {
            println!("Proof:");
            let fmt = format_proof(&proof, &name_table);
            for line in fmt {
                println!("  {line}");
            }
        }
    }

    return ExitCode::SUCCESS;
}
