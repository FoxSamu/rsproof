use std::process::ExitCode;

use options::Options;

mod options;
mod legacy;
mod mgu;
mod prove;

fn main() -> ExitCode {
    let opts = Options::parse();

    #[allow(deprecated)]
    match opts.run_mode {
        options::RunMode::Legacy(verbose) => legacy::main(verbose),
        options::RunMode::Prove(input, (t, s, v, h)) => prove::main(input, t, s, v, false, h),
        options::RunMode::Disprove(input, (t, s, v, h)) => prove::main(input, t, s, v, true, h),
        options::RunMode::Mgu(input) => mgu::main(input),

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
    println!("RSPROOF | A resolution-based theorem prover.

Usage: `{base} [<command> <arguments>]`
    {base} help
        Prints this menu.

    {base} legacy [-v]
        Runs the legacy prover. Input is read from stdin.
          -v                                Enables verbose mode.

    {base} (prove | disprove) ((-i | --stdin) | (-f | --file) <filename>
            | [-r | --raw] <raw_input>) ((-v | --verbose) | (-q |
            --quiet) | (-t | --tseitin) | (-s | --steps) <number> | (-H |
            --heuristic) (naive|prefer_empty|symbol_count))*
        Prove (or disprove) a specific statement. The statement is an
        input of the form `P, Q, ... |- R, S, ...`, which proves the
        statements `R, S, ...` from the given premises `P, Q, ...`. The
        `|-` in the input can be read as \"entails\", so `P |- Q` reads \"P
        entails Q\".
        Both the `prove` and `disprove` command do the same, but `prove`
        will strive to refute the opposite of the statement whereas
        `disprove` will strive to refute the statement itself.
          -i   --stdin                      Read input from stdin.
          -f   --file           <path>      Read input from given file.
          -r   --raw            <input>     Use the given argument as raw
                                            input. You may omit the `-r`.
          -v   --verbose                    Print extra information with
                                            the proof.
          -q   --quiet                      Print only `sat`, `unsat` or
                                            `undec` (see below).
          -t   --tseitin                    Convert the proof to Tseitin
                                            CNF rather than equivalent
                                            CNF.
          -s   --steps          <number>    Restrict the prover to a 
                                            specific amount of resolution
                                            steps.
          -H   --heuristic      <heuristic> Use a specific heuristic to
                                            determine clause priority.
                                            `naive` assigns all clauses
                                            the same heuristic.
                                            `prefer_empty` is the same as
                                            `naive` but it assigns the
                                            empty clause a higher
                                            priority.
                                            `symbol_count` prioritises
                                            clauses based on the amount
                                            of symbols.
        The output starts with one of 3 keywords, with the following 
        meanings:
          proven                            A proof was found.
          disproven                         A counterproof was found.
          exhausted                         All possibilities were
                                            explored, no proof or
                                            counterproof was found.
          undecided                         The prover was undecided 
                                            after a limited amount of
                                            steps.

    {base} mgu ((-i | --stdin) | (-f | --file) <filename> | [-r | --raw]
            <raw_input>)
        Find a most general unifier of an equivalence. The equivalence is
        an input of the form `P === Q`, in which `P` and `Q` are boolean 
        expressions. `P` and `Q` can use boolean connectives, though an
        MGU never exists for such expressions. Use the `:x` syntax to
        denote a variable named `x`, as the syntax `x` will denote a
        constant.
          -i   --stdin                      Read input from stdin.
          -f   --file           <path>      Read input from given file.
          -r   --raw            <input>     Use the given argument as raw
                                            input. You may omit the `-r`.
");

    ExitCode::SUCCESS
}