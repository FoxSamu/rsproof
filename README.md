# RSProof

A small theorem prover in Rust, proving lemmas in propositional logic.


## Usage

Run using `cargo run`. The program reads the input from the standard input, until it reaches the end of stream, and it is ideally used by piping a file into the program.

The input is of the form `A, B, C ... |- D, E, F ...`, which tells the prover: given `A, B, C ...`, prove `D, E, F ...`. The prover will then output
`sat` if it could prove the statement or `unsat` if not. The `|-` is a turnstile, it can be read as "entails".

## Input syntax

The input parser ignores any whitespaces between tokens, as well as comments between a `#` and the end of the line.
Those ignored tokens not included, the exact syntax is as follows (starting from `input`):

```
input:
    exprs '|-' exprs

exprs:
    expr ',' exprs
    expr

expr:
    and '|' expr
    and

and:
    xor '&' and
    xor

xor:
    impl '^' xor
    impl

impl:
    atom '->' impl
    atom '<-' impl
    atom '<->' impl
    atom

atom:
    /[a-zA-Z][a-zA-Z0-9_]*/
    '!' atom
    '(' expr ')'
```

## Examples

```
# Prove DeMorgan's law
!(A & B) |- (!A | !B)
```

```
# A simple contradiction
A |- !A
```

Various examples can be found in the `examples` folder.