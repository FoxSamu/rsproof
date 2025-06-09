// https://mdaines.github.io/grammophone/?s=c3RtdCAtPiBleHBzICJ8LSIgZXhwcyAuCmV4cHMgLT4gIHwgYmV4cCAiLCIgZXhwcyB8IGJleHAgLgoKYmV4cCAtPiAiYWxsIiBpZGVudCAiOiIgYmltcCB8ICJleGlzdHMiIGlkZW50ICI6IiBiaW1wIHwgIm5vIiBpZGVudCAiOiIgYmltcCB8IGJpbXAgLgpiaW1wIC0+IGJkaXMgIi0+IiBiZXhwIHwgYmRpcyAiPC0iIGJleHAgfCBiZGlzICI8LT4iIGJleHAgfCBiZGlzIC4KYmRpcyAtPiBiY29uICJ8IiBiZGlzIHwgYmNvbiAuCmJjb24gLT4gYmZhYyAiJiIgYmNvbiB8IGJmYWMgLgpiZmFjIC0+ICIoIiBiZXhwICIpIiB8IHByZWQgfCBpZGVudCB8ICJ0cnVlIiB8ICJmYWxzZSIgfCAiISIgYmZhYyAuCnByZWQgLT4gaWRlbnQgIigiIGFyZ3MgIikiIC4KCmFleHAgLT4gYW11bCAiKyIgYWV4cCB8IGFtdWwgIi0iIGFleHAgfCBhbXVsIC4KYW11bCAtPiBhZmFjICIqIiBhbXVsIHwgYWZhYyAiLyIgYW11bCB8IGFmYWMgIiUiIGFtdWwgfCBhZmFjIC4KYWZhYyAtPiAiKCIgYWV4cCAiKSIgfCBudW1iZXIgfCBmdW5jIHwgaWRlbnQgfCAiLSIgYWZhYy4KZnVuYyAtPiBpZGVudCAiKCIgYXJncyAiKSIgLgphcmdzIC0+ICB8IGFleHAgIiwiIGFyZ3MgfCBhZXhwIC4KCg==

#![allow(dead_code, unused)]

use crate::expr::*;
use crate::fmt::NameTable;
use crate::parser::result::ParseResult;

pub use input::*;
pub use output::*;
pub use error::*;
pub use coord::*;
pub use namer::*;

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



pub struct ParseContext {
    nc: NameContext
}

impl ParseContext {
    pub fn new() -> Self {
        Self { nc: NameContext::new() }
    }

    pub fn name_table(&self) -> &NameTable {
        self.nc.rev_table()
    }

    fn parse<T, S, F>(&mut self, input: S, name: &str, func: F) -> Result<T, Error>
    where
    S : Input,
    F : FnOnce(&mut Parser<S::Iter>, &mut NameContext) -> ParseResult<T> {
        Parser::new(input.char_stream()).parse(
            |p, nc| func(p, nc),
            name,
            &mut self.nc
        )
    }

    fn with_output<T>(&self, result: T) -> Output<T> {
        Output { result, name_table: self.name_table().clone() }
    }


    pub fn name_valid<S>(&mut self, input: S) -> Result<(), Error> where S : Input {
        self.parse(input, "ident", |p, _| p.ident())?;
        Ok(())
    }

    pub fn name<S>(&mut self, input: S) -> Result<Name, Error> where S : Input {
        self.parse(input, "ident", |p, nc| {
            let id = p.ident()?;
            Ok(nc.resolve_static(id))
        })
    }

    pub fn name_output<S>(&mut self, input: S) -> Result<Output<Name>, Error> where S : Input {
        self.name(input).map(|it| self.with_output(it))
    }


    pub fn stmt_valid<S>(&mut self, input: S) -> Result<(), Error> where S : Input {
        self.parse(input, "stmt", |p, _| p.stmt())?;
        Ok(())
    }

    pub fn stmt<S>(&mut self, input: S) -> Result<Stmt, Error> where S : Input {
        self.parse(input, "stmt", |p, nc| p.stmt()?.as_stmt(nc))
    }

    pub fn stmt_output<S>(&mut self, input: S) -> Result<Output<Stmt>, Error> where S : Input {
        self.stmt(input).map(|it| self.with_output(it))
    }


    pub fn unifiable_valid<S>(&mut self, input: S) -> Result<(), Error> where S : Input {
        self.parse(input, "unifiable", |p, _| p.unifiable())?;
        Ok(())
    }

    pub fn unifiable<S>(&mut self, input: S) -> Result<(Vec<AExpr>, Vec<AExpr>), Error> where S : Input {
        self.parse(input, "unifiable", |p, nc| p.unifiable()?.as_unifiable(nc))
    }

    pub fn unifiable_output<S>(&mut self, input: S) -> Result<Output<(Vec<AExpr>, Vec<AExpr>)>, Error> where S : Input {
        self.unifiable(input).map(|it| self.with_output(it))
    }


    pub fn aexpr_valid<S>(&mut self, input: S) -> Result<(), Error> where S : Input {
        self.parse(input, "exp", |p, _| p.exp())?;
        Ok(())
    }

    pub fn aexpr<S>(&mut self, input: S) -> Result<AExpr, Error> where S : Input {
        self.parse(input, "exp", |p, nc| p.exp()?.as_aexpr(nc))
    }

    pub fn aexpr_output<S>(&mut self, input: S) -> Result<Output<AExpr>, Error> where S : Input {
        self.aexpr(input).map(|it| self.with_output(it))
    }


    pub fn bexpr_valid<S>(&mut self, input: S) -> Result<(), Error> where S : Input {
        self.parse(input, "exp", |p, _| p.exp())?;
        Ok(())
    }

    pub fn bexpr<S>(&mut self, input: S) -> Result<BExpr, Error> where S : Input {
        self.parse(input, "exp", |p, nc| p.exp()?.as_bexpr(nc))
    }

    pub fn bexpr_output<S>(&mut self, input: S) -> Result<Output<BExpr>, Error> where S : Input {
        self.bexpr(input).map(|it| self.with_output(it))
    }
}


/// Parse a statement
#[deprecated(note = "Use ParseContext")]
pub fn parse_stmt<S>(input: S, nc: &mut NameContext) -> Result<Output<Stmt>, Error> where S : Input {
    Parser::new(input.char_stream()).parse(|p, nc| p.stmt()?.as_stmt(nc).map(|result| Output { result, name_table: nc.rev_table().clone() }), "stmt", nc)
}

/// Parse a unifiable
#[deprecated(note = "Use ParseContext")]
pub fn parse_unifiable<S>(input: S, nc: &mut NameContext) -> Result<Output<(Vec<AExpr>, Vec<AExpr>)>, Error> where S : Input {
    Parser::new(input.char_stream()).parse(|p, nc| p.unifiable()?.as_unifiable(nc).map(|result| Output { result, name_table: nc.rev_table().clone() }), "unifiable", nc)
}

/// Parse an arithmetic expression
#[deprecated(note = "Use ParseContext")]
pub fn parse_aexpr<S>(input: S, nc: &mut NameContext) -> Result<Output<AExpr>, Error> where S : Input {
    Parser::new(input.char_stream()).parse(|p, nc| p.exp()?.as_aexpr(nc).map(|result| Output { result, name_table: nc.rev_table().clone() }), "exp", nc)
}

/// Parse a boolean expression
#[deprecated(note = "Use ParseContext")]
pub fn parse_bexpr<I>(input: I, nc: &mut NameContext) -> Result<Output<BExpr>, Error> where I : Input {
    Parser::new(input.char_stream()).parse(|p, nc| p.exp()?.as_bexpr(nc).map(|result| Output { result, name_table: nc.rev_table().clone() }), "exp", nc)
}
