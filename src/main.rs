mod expr;
mod cnf;
mod res;
mod parse;

#[cfg(test)]
mod test;

use std::collections::BTreeSet;
use std::env;
use std::io::stdin;
use std::process::ExitCode;

use cnf::Clause;
use parse::parse;
use res::resolution;

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
    let cnf = parsed.unwrap().to_cnf();
    let clauses = Clause::from_cnf(&cnf);

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
    }

    return ExitCode::SUCCESS;
}
