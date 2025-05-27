use std::collections::BTreeSet;
use std::env;
use std::io::stdin;
use std::process::ExitCode;

use crate::cnf::Clause;
use crate::expro::Term;
use crate::parse::parse;
use crate::proof::format_proof;
use crate::res::resolution;
use crate::unify::unify;

pub fn main(verbose: bool) -> ExitCode {
    // f(x, y) = f(y, x)

    // x |-> y


    // f = 0, g = 1, X = 8, Y = 9, Z = 10, a = 3
    let a = Term::Const(3);
    let x = Term::Var(8);
    let y = Term::Var(9);
    let z = Term::Var(10);

    let gy = Term::Func(1, vec![y.clone()]);
    let gz = Term::Func(1, vec![z.clone()]);

    // x -> g(g(a))
    // y -> g(a)
    // z -> a
    let left = Term::Func(0, vec![x.clone(), y.clone()]);
    let right = Term::Func(0, vec![y.clone(), x.clone()]);

    let uni = unify(&vec![left], &vec![right]);
    dbg!(uni);

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
