use std::io::stdin;
use std::process::ExitCode;

use crate::cnf::Clause;
use crate::parse::parse;
use crate::proof::format_proof;
use crate::res::resolution;

#[deprecated] // Ultimately we need this code gone
pub fn main(verbose: bool) -> ExitCode {
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
