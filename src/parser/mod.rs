// https://mdaines.github.io/grammophone/?s=c3RtdCAtPiBleHBzICJ8LSIgZXhwcyAuCmV4cHMgLT4gIHwgYmV4cCAiLCIgZXhwcyB8IGJleHAgLgoKYmV4cCAtPiAiYWxsIiBpZGVudCAiOiIgYmltcCB8ICJleGlzdHMiIGlkZW50ICI6IiBiaW1wIHwgIm5vIiBpZGVudCAiOiIgYmltcCB8IGJpbXAgLgpiaW1wIC0+IGJkaXMgIi0+IiBiZXhwIHwgYmRpcyAiPC0iIGJleHAgfCBiZGlzICI8LT4iIGJleHAgfCBiZGlzIC4KYmRpcyAtPiBiY29uICJ8IiBiZGlzIHwgYmNvbiAuCmJjb24gLT4gYmZhYyAiJiIgYmNvbiB8IGJmYWMgLgpiZmFjIC0+ICIoIiBiZXhwICIpIiB8IHByZWQgfCBpZGVudCB8ICJ0cnVlIiB8ICJmYWxzZSIgfCAiISIgYmZhYyAuCnByZWQgLT4gaWRlbnQgIigiIGFyZ3MgIikiIC4KCmFleHAgLT4gYW11bCAiKyIgYWV4cCB8IGFtdWwgIi0iIGFleHAgfCBhbXVsIC4KYW11bCAtPiBhZmFjICIqIiBhbXVsIHwgYWZhYyAiLyIgYW11bCB8IGFmYWMgIiUiIGFtdWwgfCBhZmFjIC4KYWZhYyAtPiAiKCIgYWV4cCAiKSIgfCBudW1iZXIgfCBmdW5jIHwgaWRlbnQgfCAiLSIgYWZhYy4KZnVuYyAtPiBpZGVudCAiKCIgYXJncyAiKSIgLgphcmdzIC0+ICB8IGFleHAgIiwiIGFyZ3MgfCBhZXhwIC4KCg==

#![allow(dead_code, unused)]

use crate::expr::*;
use crate::uni::Unifiable;

pub use input::*;
pub use output::*;
pub use error::*;
pub use coord::*;

use parser::Parser;

mod input;
mod output;
mod error;
mod coord;

mod result;
mod tree;
mod token;
mod namer;

mod lexer;
mod parser;

#[cfg(test)]
mod test;


/// Parse a statement
pub fn parse_stmt<S>(input: S) -> Result<Output<Stmt>, Error> where S : Input {
    Parser::new(input.char_stream()).parse(|p, nc| p.stmt()?.as_stmt(nc), "stmt")
}

/// Parse a unifiable
pub fn parse_unifiable<S>(input: S) -> Result<Output<(BExpr, BExpr)>, Error> where S : Input {
    Parser::new(input.char_stream()).parse(|p, nc| p.unifiable()?.as_unifiable(nc), "unifiable")
}

/// Parse an arithmetic expression
pub fn parse_aexpr<S>(input: S) -> Result<Output<AExpr>, Error> where S : Input {
    Parser::new(input.char_stream()).parse(|p, nc| p.exp()?.as_aexpr(nc), "exp")
}

/// Parse a boolean expression
pub fn parse_bexpr<I>(input: I) -> Result<Output<BExpr>, Error> where I : Input {
    Parser::new(input.char_stream()).parse(|p, nc| p.exp()?.as_bexpr(nc), "exp")
}
