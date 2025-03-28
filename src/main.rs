mod expr;
mod cnf;
mod res;
mod parse;

use std::io::stdin;

use cnf::Clause;
use parse::parse;
use res::resolution;

fn main() {
    // Parse stdin
    let parsed = parse(stdin());

    if let Err(msg) = parsed {
        println!("Syntax error: {msg}");
        return;
    }

    // println!("{parsed}");

    // Convert to CNF
    let cnf = parsed.unwrap().to_cnf();
    let clauses = Clause::from_cnf(&cnf);

    // println!("{cnf}");

    // for clause in &clauses {
    //     println!("{clause}");
    // }

    // Resolve
    let cont_sat = resolution(&clauses);

    // The parser states the input as a "proof by contradiction"
    // so if we find a contradiction then the proof is satisifed.
    if !cont_sat {
        println!("sat");
    } else {
        println!("unsat");
    }
}
