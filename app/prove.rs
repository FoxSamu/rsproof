use std::process::ExitCode;

use rsplib::expr::{BExpr, Stmt};
use rsplib::fmt::DisplayNamed;
use rsplib::nf::NormalForm;
use rsplib::parser::{Output, ParseContext};
use rsplib::res::{Proof, Resolver};

use crate::options::Verbosity;

use super::options::InputSource;

fn try_parse(input: InputSource) -> Result<Output<Stmt>, String> {
    let input = input.read_to_string()?;
    ParseContext::new().stmt_output(input).map_err(|err| format!("{err}"))
}

fn to_conj(mut expr: Vec<BExpr>) -> BExpr {
    if let Some(mut e) = expr.pop() {
        while let Some(n) = expr.pop() {
            e = e & n
        }

        e
    } else {
        BExpr::True
    }
}

pub fn main(input: InputSource, tseitin: bool, max_steps: usize, verbosity: Verbosity) -> ExitCode {
    let Output { result, name_table } = match try_parse(input) {
        Ok(ok) => ok,
        Err(err) => {
            eprintln!("{err}");

            return ExitCode::FAILURE;
        },
    };

    let (premises, conclusions) = result.into();

    let premises = to_conj(premises);
    let conclusions = to_conj(conclusions);

    // Statement
    let stmt = premises & !conclusions;
    
    // CNF
    let cnf = if tseitin {
        NormalForm::tseitin_cnf(stmt)
    } else {
        NormalForm::equiv_cnf(stmt)
    };


    // Resolver
    let mut resolver = Resolver::new();
    resolver.assume_cnf(cnf.clone()); // clone CNF so we can print it later

    // Resolution
    let result = if max_steps == 0 {
        resolver.step_indefinitely()
    } else {
        if let Some(r) = resolver.step_n_times(max_steps) {
            r
        } else {
            println!("undec");

            if let Verbosity::Verbose = verbosity {
                println!("No proof found after {max_steps} deductions.");
            }

            return ExitCode::FAILURE;
        }
    };

    let n = result.deductions_made;

    match result.proof {
        Proof::Proven(deductions) => {
            println!("sat");

            if verbosity >= Verbosity::Normal {
                println!("CNF:");
                println!("  {}", cnf.with_table(&name_table));

                println!("Refutation proof using resolution:");

                let mut line = 0usize;
                for elem in deductions.into_iter() {
                    println!("  {}: {}", line, elem.with_table(&name_table));
                    line += 1;
                }
            }

            if verbosity >= Verbosity::Verbose {
                println!("{n} deductions made.");
            }
        },

        Proof::Disproven => {
            println!("unsat");

            if verbosity >= Verbosity::Verbose {
                println!("{n} deductions made.");
            }
        }
    }

    ExitCode::SUCCESS
}