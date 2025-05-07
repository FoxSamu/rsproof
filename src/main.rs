mod expr;
mod cnf;
mod res;
mod parse;
mod proof;
mod fmt;
mod unify;

#[cfg(test)]
mod test;

use std::collections::BTreeSet;
use std::env;
use std::io::stdin;
use std::process::ExitCode;

use cnf::Clause;
use parse::parse;
use proof::format_proof;
use res::resolution;
use expr::Term::*;
use unify::unify;

fn main() -> ExitCode {
    let a = Func(0, vec![Func(1, vec![Var(2)]), Var(2)]);
    let b = Func(0, vec![Var(3), Const(4)]);

    let unify = unify(vec![a], vec![b]);
    dbg!(unify);

    //////
    
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
    let cnf = expr.to_cnf();
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
        if let Some(proof) = resolution.proof {
            let fmt = format_proof(&proof, &name_table);
            for line in fmt {
                println!("{line}");
            }
        }
    }

    return ExitCode::SUCCESS;
}
