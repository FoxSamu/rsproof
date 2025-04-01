# RSProof

A small theorem prover in Rust, proving lemmas in propositional logic.

## Requirements

You need the `cargo` build tool for Rust. This can be installed using `rustup`, see https://www.rust-lang.org/tools/install.

## Usage

Run using `cargo run`. The program reads the input from the standard input, until it reaches the end of stream, and it is ideally used by piping a file into the program.

The input is of the form `A, B, C ... |- D, E, F ...`, which tells the prover: given `A, B, C ...`, prove `D, E, F ...`. The prover will then output
`sat` if it could prove the statement or `unsat` if not. The `|-` is a turnstile, it can be read as "entails". At least one claim is expected after the turnstile, but no premises need to be specified before the statement. For example, it can prove `|- a==a` just fine.

## Input syntax

The input parser ignores any whitespaces between tokens, as well as comments between a `#` and the end of the line.
Those ignored tokens not included, the exact syntax is as follows (starting from `input`):

```
input:
    exprs '|-' exprs
    '|-' exprs

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
    pred
    '!' atom
    '(' expr ')'
    '*'
    '~'

pred:
    sym '(' args ')'
    sym '==' sym
    sym '!=' sym
    sym

args:
    sym ',' args
    sym

sym:
    /[a-zA-Z][a-zA-Z0-9_]*/
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

```
# Equality
P(a, b), a==b |- P(b, a)
```

```
# Transitivity
a==b, b==c |- c==a
```

```
# Proves 'true' (the '*' symbol stands for 'true')
|- *
```

Various examples can be found in the `examples` folder.