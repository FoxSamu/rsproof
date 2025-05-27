use std::env;
use std::process::ExitCode;

use options::Options;

mod options;
mod legacy;
mod unify;

pub fn main() -> ExitCode {
    let opts = Options::parse();
    
    match opts.run_mode {
        options::RunMode::Legacy(verbose) => legacy::main(verbose),
        options::RunMode::Prove(input) => todo!(),
        options::RunMode::Unify(input) => unify::main(input),

        options::RunMode::Help => print_help(opts.base_command),
        options::RunMode::Error(err) => print_error(opts.base_command, err),
    }
}

fn print_error(base: String, err: String) -> ExitCode {
    println!("{err}");
    println!("Run `{base} help` for detailed usage instructions.");

    ExitCode::FAILURE
}

fn print_help(base: String) -> ExitCode {
    println!("Usage: `{base} [<command> <arguments>]`");
    println!("
    {base} help
        Prints this menu.");

    println!("
    {base} legacy [-v]
        Runs the legacy prover. Input is read from stdin.
        -v               Enables verbose mode.");

    println!("
    {base} prove (-i | -f <filename> | [-r] <raw_input>)
        Prove a specific statement. The statement is an input of the form
        `P, Q, ... |- R, S, ...`, which proves the statements `R, S, ...`
        from the given premises `P, Q, ...`. The `|-` in the input can be
        read as \"entails\", so `P |- Q` reads \"P entails Q\".
        -i               Read input from stdin.
        -f <filename>    Read input from given file.
        -r <raw_input>   Use the given argument as raw input.");

    println!("
    {base} unify (-i | -f <filename> | [-r] <raw_input>)
        Unify a set of equalities.
        -i               Read input from stdin.
        -f <filename>    Read input from given file.
        -r <raw_input>   Use the given argument as raw input.");

    ExitCode::SUCCESS
}