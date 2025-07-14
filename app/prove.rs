use std::process::ExitCode;

use rsplib::expr::Stmt;
use rsplib::fmt::DisplayNamed;
use rsplib::nf::NormalForm;
use rsplib::parser::{Output, ParseContext};
use rsplib::res::{Heuristic, Proof, Resolver};

use crate::options::Verbosity;

use super::options::InputSource;

fn try_parse(input: InputSource) -> Result<Output<Stmt>, String> {
    let input = input.read_to_string()?;
    ParseContext::new().stmt_output(input).map_err(|err| format!("{err}"))
}

pub fn main(input: InputSource, tseitin: bool, max_steps: usize, verbosity: Verbosity, prefer_counterproof: bool, heuristic: Heuristic) -> ExitCode {
    let Output { result, name_table } = match try_parse(input) {
        Ok(ok) => ok,
        Err(err) => {
            eprintln!("{err}");

            return ExitCode::FAILURE;
        },
    };

    // Statement
    let stmt = if prefer_counterproof {
        result.provable_expr()
    } else {
        result.refutable_expr()
    };
    
    // CNF
    let cnf = if tseitin {
        NormalForm::tseitin_cnf(stmt)
    } else {
        NormalForm::equiv_cnf(stmt)
    };

    // Resolver
    let mut resolver = Resolver::new();
    resolver.set_heuristic(heuristic);
    resolver.assume_cnf(cnf.clone()); // clone CNF so we can print it later

    // Resolution
    let result = if max_steps == 0 {
        resolver.step_indefinitely()
    } else {
        if let Some(r) = resolver.step_n_times(max_steps) {
            r
        } else {
            println!("undecided");

            if let Verbosity::Verbose = verbosity {
                println!("Clauses in learning order:");
                for clause in resolver.stats().learning_order {
                    println!("  - {}", clause.with_table(&name_table));
                }
                println!("No proof found after {max_steps} deductions.");
            }

            return ExitCode::FAILURE;
        }
    };

    let n = result.deductions_made;

    match result.proof {
        Proof::Proven(deductions) => {
            if prefer_counterproof {
                println!("disproven");
            } else {
                println!("proven");
            }

            if verbosity >= Verbosity::Normal {
                println!("Refutation proof using resolution:");

                let mut line = 0usize;
                for elem in deductions.into_iter() {
                    println!("  {}: {}", line, elem.with_table(&name_table));
                    line += 1;
                }
            }

            if verbosity >= Verbosity::Verbose {
                println!("Clauses in learning order:");
                for clause in result.learning_order {
                    println!("  - {}", clause.with_table(&name_table));
                }
                println!("{n} deductions made.");
            }
        },

        Proof::Disproven => {
            println!("exhausted");

            if verbosity >= Verbosity::Verbose {
                println!("Clauses in learning order:");
                for clause in result.learning_order {
                    println!("  - {}", clause.with_table(&name_table));
                }
                println!("{n} deductions made.");
            }
        }
    }

    ExitCode::SUCCESS
}