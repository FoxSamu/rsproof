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
    exprs '|-' exprs <eof>
    '|-' exprs <eof>

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


## License

    Copyright (C) 2025  Olaf W. Nankman

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.