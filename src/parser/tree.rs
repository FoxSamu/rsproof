use std::fmt::Debug;

use crate::expr::{AExpr, BExpr, Stmt};
use crate::uni::Unifiable;

use super::namer::NameContext;
use super::result::ParseResult;
use super::token::Token;
use super::coord::{InputCoord, InputRange};

/// A syntax tree node
pub trait SynNode: PartialEq + Eq + Clone + Debug {
    /// Starting coordinate of the node in the text
    fn from(&self) -> InputCoord;

    /// Ending coordinate of the node in the text
    fn to(&self) -> InputCoord;

    /// Range of the node in the text
    fn range(&self) -> InputRange {
        InputRange { from: self.from(), to: self.to() }
    }
}

impl SynNode for Token {
    fn from(&self) -> InputCoord {
        self.from
    }

    fn to(&self) -> InputCoord {
        self.to
    }
}


/// An expression node
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ExpNode {
    /// Starting coordinate of the node in the text
    pub from: InputCoord,

    /// Ending coordinate of the node in the text
    pub to: InputCoord,

    /// The expression tree
    pub tree: ExpTree
}

impl SynNode for ExpNode {
    fn from(&self) -> InputCoord {
        self.from
    }

    fn to(&self) -> InputCoord {
        self.to
    }
}




impl ExpNode {
    /// Parses the given string as a number and reports at the given range if the number could not be parsed
    pub fn parse_nr(str: &String, range: InputRange) -> ParseResult<i64> {
        let parsed = if str.len() < 3 {
            str.parse()
        } else {
            match (&str[0..2], &str[2..]) {
                ("0x" | "0X", rest) => i64::from_str_radix(rest, 16),
                ("0b" | "0B", rest) => i64::from_str_radix(rest, 2),
                _ => str.parse(),
            }
        };

        parsed.or_else(|_| range.error(format!("Invalid number: {str}")))
    }

    /// Parses the given vector of expression nodes as arithmetic expressions
    pub fn as_aexprs(vec: Vec<ExpNode>, nc: &mut NameContext) -> ParseResult<Vec<AExpr>> {
        let mut out = Vec::new();
        for node in vec {
            out.push(node.as_aexpr(nc)?);
        }
        return Ok(out);
    }

    /// Parses the given expression node as an arithmetic expression
    pub fn as_aexpr(self, nc: &mut NameContext) -> ParseResult<AExpr> {
        let range = self.range();

        let res = match self.tree {
            // ExpTree::Num(val) => AExpr::num(Self::parse_nr(&val, range)?),

            ExpTree::Ident(name) => nc.resolve_bound(&name).map(|it| AExpr::bound(it)).unwrap_or_else(|| AExpr::unbound(nc.resolve_static(name))),
            ExpTree::Bound(name) => AExpr::bound(nc.resolve_static(name)),

            ExpTree::Fun(name, args) => AExpr::fun(nc.resolve_static(name), Self::as_aexprs(args, nc)?),

            // ExpTree::UnOp(UnOp::Neg, rhs) => AExpr::neg(rhs.as_aexpr(nc)?),
            ExpTree::UnOp(UnOp::Par, rhs) => rhs.as_aexpr(nc)?,

            // ExpTree::BinOp(BinOp::Add, lhs, rhs) => AExpr::add(lhs.as_aexpr(nc)?, rhs.as_aexpr(nc)?),
            // ExpTree::BinOp(BinOp::Sub, lhs, rhs) => AExpr::sub(lhs.as_aexpr(nc)?, rhs.as_aexpr(nc)?),
            // ExpTree::BinOp(BinOp::Mul, lhs, rhs) => AExpr::mul(lhs.as_aexpr(nc)?, rhs.as_aexpr(nc)?),
            // ExpTree::BinOp(BinOp::Div, lhs, rhs) => AExpr::div(lhs.as_aexpr(nc)?, rhs.as_aexpr(nc)?),
            // ExpTree::BinOp(BinOp::Rem, lhs, rhs) => AExpr::rem(lhs.as_aexpr(nc)?, rhs.as_aexpr(nc)?),

            _ => range.error("Not an arithmetic expression")?,
        };

        Ok(res)
    }

    /// Parses the given vector of expression nodes as boolean expressions
    pub fn as_bexprs(vec: Vec<ExpNode>, nc: &mut NameContext) -> ParseResult<Vec<BExpr>> {
        let mut out = Vec::new();
        for node in vec {
            out.push(node.as_bexpr(nc)?);
        }
        return Ok(out);
    }

    // /// Expands a quantifier into a recursive boolean expression. E.g. it turns `all a, b, c:` into `all a: all b: all c:`.
    // fn expand_quant(q: Quant, names: Vec<String>, rhs: Box<ExpNode>, nc: &mut NameContext) -> ParseResult<BExpr> {
    //     let name_count = names.len();

    //     let mut mapped_names = Vec::new();
    //     for name in names {
    //         mapped_names.push(nc.enter(name));
    //     }
        
    //     let mut exp = rhs.as_bexpr(nc)?;

    //     while let Some(name) = mapped_names.pop() {
    //         exp = match q {
    //             Quant::All => BExpr::all(name, exp),
    //             Quant::Exists => BExpr::exists(name, exp),
    //             Quant::No => BExpr::no(name, exp)
    //         }
    //     }


    //     for _ in 0..name_count {
    //         nc.leave();
    //     }

    //     Ok(exp)
    // }

    /// Parses the given expression node as a boolean expression
    pub fn as_bexpr(self, nc: &mut NameContext) -> ParseResult<BExpr> {
        let range = self.range();

        let res = match self.tree {
            ExpTree::False => BExpr::False,
            ExpTree::True => BExpr::True,

            ExpTree::Ident(name) => BExpr::sym(nc.resolve_static(name)),

            ExpTree::Fun(name, args) => BExpr::pred(nc.resolve_static(name), Self::as_aexprs(args, nc)?),

            ExpTree::UnOp(UnOp::Not, rhs) => BExpr::not(rhs.as_bexpr(nc)?),
            ExpTree::UnOp(UnOp::Par, rhs) => rhs.as_bexpr(nc)?,

            // ExpTree::BinOp(BinOp::Eq, lhs, rhs) => BExpr::eq(lhs.as_aexpr(nc)?, rhs.as_aexpr(nc)?),
            // ExpTree::BinOp(BinOp::NEq, lhs, rhs) => BExpr::neq(lhs.as_aexpr(nc)?, rhs.as_aexpr(nc)?),
            // ExpTree::BinOp(BinOp::Lt, lhs, rhs) => BExpr::lt(lhs.as_aexpr(nc)?, rhs.as_aexpr(nc)?),
            // ExpTree::BinOp(BinOp::Gt, lhs, rhs) => BExpr::gt(lhs.as_aexpr(nc)?, rhs.as_aexpr(nc)?),
            // ExpTree::BinOp(BinOp::LtEq, lhs, rhs) => BExpr::lteq(lhs.as_aexpr(nc)?, rhs.as_aexpr(nc)?),
            // ExpTree::BinOp(BinOp::GtEq, lhs, rhs) => BExpr::gteq(lhs.as_aexpr(nc)?, rhs.as_aexpr(nc)?),

            ExpTree::BinOp(BinOp::And, lhs, rhs) => BExpr::and(lhs.as_bexpr(nc)?, rhs.as_bexpr(nc)?),
            ExpTree::BinOp(BinOp::Or, lhs, rhs) => BExpr::or(lhs.as_bexpr(nc)?, rhs.as_bexpr(nc)?),
            ExpTree::BinOp(BinOp::Im, lhs, rhs) => BExpr::im(lhs.as_bexpr(nc)?, rhs.as_bexpr(nc)?),
            ExpTree::BinOp(BinOp::RevIm, lhs, rhs) => BExpr::revim(lhs.as_bexpr(nc)?, rhs.as_bexpr(nc)?),
            ExpTree::BinOp(BinOp::Equiv, lhs, rhs) => BExpr::equiv(lhs.as_bexpr(nc)?, rhs.as_bexpr(nc)?),

            // ExpTree::Quant(q, names, rhs) => Self::expand_quant(q, names, rhs, nc)?,

            _ => range.error("Not a boolean expression")?,
        };

        Ok(res)
    }
}


/// A type of binary operator
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum BinOp {
    Add, Sub, Mul, Div, Rem,
    Eq, NEq, LtEq, GtEq, Lt, Gt,
    And, Or, Im, RevIm, Equiv
}

/// A type of unary operator
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum UnOp {
    Par,
    Neg,
    Not
}

/// A type of quantifier
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Quant {
    All,
    Exists,
    No
}

/// A type of expression tree, as child of an [ExpNode].
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ExpTree {
    /// A numerical literal
    Num(String),

    /// A variable or symbol
    Ident(String),

    /// A variable or symbol
    Bound(String),

    /// The `True` boolean literal
    True,

    /// The `False` boolean literal
    False,

    /// A function or predicate
    Fun(String, Vec<ExpNode>),

    /// A unary operator on an operand
    UnOp(UnOp, Box<ExpNode>),

    /// A binary operator on two operands
    BinOp(BinOp, Box<ExpNode>, Box<ExpNode>),

    /// A quantifier over several names
    Quant(Quant, Vec<String>, Box<ExpNode>)
}

/// A statement syntax node
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct StmtNode {
    /// Starting coordinate of the node in the text
    pub from: InputCoord,

    /// Ending coordinate of the node in the text
    pub to: InputCoord,

    /// List of premise expressions before the `|-`
    pub premises: Vec<ExpNode>,

    /// List of concluding expressions after the `|-`
    pub conclusions: Vec<ExpNode>
}

impl SynNode for StmtNode {
    fn from(&self) -> InputCoord {
        self.from
    }

    fn to(&self) -> InputCoord {
        self.to
    }
}

impl StmtNode {
    /// Converts this node into a [Stmt]
    pub fn as_stmt(self, nc: &mut NameContext) -> ParseResult<Stmt> {
        Ok(Stmt::from_implication(
            ExpNode::as_bexprs(self.premises, nc)?,
            ExpNode::as_bexprs(self.conclusions, nc)?
        ))
    }
}

/// A unifiable syntax node
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct UnifiableNode {
    /// Starting coordinate of the node in the text
    pub from: InputCoord,

    /// Ending coordinate of the node in the text
    pub to: InputCoord,

    /// The left hand side expression
    pub left: Vec<ExpNode>,

    /// The right hand side expression
    pub right: Vec<ExpNode>
}

impl SynNode for UnifiableNode {
    fn from(&self) -> InputCoord {
        self.from
    }

    fn to(&self) -> InputCoord {
        self.to
    }
}

impl UnifiableNode {
    /// Converts this node into a `(Unifiable, Unifiable)`
    pub fn as_unifiable(self, nc: &mut NameContext) -> ParseResult<(Vec<AExpr>, Vec<AExpr>)> {
        let left = ExpNode::as_aexprs(self.left, nc)?;
        let right = ExpNode::as_aexprs(self.right, nc)?;
        Ok((left, right))
    }
}



/// A syntax node that is a vector of syntax nodes
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct VecNode<N> where N : SynNode {
    /// Starting coordinate of the node in the text
    pub from: InputCoord,

    /// Ending coordinate of the node in the text
    pub to: InputCoord,

    /// Elements
    pub elems: Vec<N>
}

impl<N> SynNode for VecNode<N> where N : SynNode {
    fn from(&self) -> InputCoord {
        self.from
    }

    fn to(&self) -> InputCoord {
        self.to
    }
}

impl VecNode<ExpNode> {
    pub fn as_aexprs(self, nc: &mut NameContext) -> ParseResult<Vec<AExpr>> {
        ExpNode::as_aexprs(self.elems, nc)
    }

    pub fn as_bexprs(self, nc: &mut NameContext) -> ParseResult<Vec<BExpr>> {
        ExpNode::as_bexprs(self.elems, nc)
    }
}


/// A syntax node that is a pair of syntax nodes
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct PairNode<N0, N1> where N0 : SynNode, N1 : SynNode {
    /// Starting coordinate of the node in the text
    pub from: InputCoord,

    /// Ending coordinate of the node in the text
    pub to: InputCoord,

    /// Pair
    pub pair: (N0, N1)
}

impl<N0, N1> SynNode for PairNode<N0, N1> where N0 : SynNode, N1 : SynNode {
    fn from(&self) -> InputCoord {
        self.from
    }

    fn to(&self) -> InputCoord {
        self.to
    }
}