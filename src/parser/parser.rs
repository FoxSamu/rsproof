use super::lexer::*;
use super::namer::*;
use super::result::*;
use super::token::*;
use super::tree::*;
use super::error::*;
use super::coord::*;
use super::output::*;

/// A parser
pub struct Parser<I> where I : Iterator<Item = char> {
    /// The lexical analyzer that provides tokens
    lexer: Lexer<I>,

    /// The [TKind] of the lookahead token
    la: Option<TKind>,

    /// The full lookahead token
    token: Option<Token>
}


#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
pub enum Precedence {
    Base,
    Mul,
    Add,
    Eq,
    Im,
    And,
    Or
}

impl Precedence {
    fn upper(&self) -> Precedence {
        match self {
            Precedence::Base => panic!("Base precedence is highest"),
            Precedence::Mul => Precedence::Base,
            Precedence::Add => Precedence::Mul,
            Precedence::Eq => Precedence::Add,
            Precedence::Im => Precedence::Eq,
            Precedence::And => Precedence::Im,
            Precedence::Or => Precedence::And,
        }
    }
}


impl<I> Parser<I> where I : Iterator<Item = char> {
    /// Creates a new [Parser]
    pub fn new(iter: I) -> Self {
        let lexer = Lexer::new(iter);

        let mut parser = Self {
            lexer,
            la: None,
            token: None
        };

        // Shift first token into lookahead
        parser.shift();

        parser
    }

    /// The current position, which is the start coordinate of the next token.
    /// If no next token is present, it is the coordinate after the very last character in the input.
    fn pos(&self) -> InputCoord {
        match &self.token {
            Some(tok) => tok.from,
            None => self.lexer.pos()
        }
    }

    /// Creates an absent value error at the next token.
    /// If no next token is present, it selects the 0-length range after the very last character.
    fn absent<T>(&self) -> ParseResult<T> {
        let (from, to) = match &self.token {
            Some(tok) => (tok.from, tok.to),
            None => (self.lexer.pos(), self.lexer.pos())
        };

        Err(ParseError::Absent { from, to })
    }


    /// Shifts to the next token
    fn shift(&mut self) {
        let tok = self.lexer.token();
        self.la = tok.as_ref().map(|e| e.kind);
        self.token = tok;
    }


    /// Expect a specific production rule. Maps the error value from absent to error with `Expected {rule}` being the
    /// error message.
    fn expect<T>(res: ParseResult<T>, rule: &str) -> ParseResult<T> {
        res.map_err(|e| e.to_error(format!("Expected {rule}")))
    }

    /// Expects a specific production rule at the final production rule. Instead of returning an internal [ParseError],
    /// it returns an [Error].
    fn expect_final<T>(res: ParseResult<T>, rule: &str) -> Result<T, Error> {
        res.map_err(|e| match e {
            ParseError::Absent { from, to } => Error { msg: format!("Expected {rule}"), from, to },
            ParseError::Error { from, to, msg } => Error { msg, from, to },
        })
    }

    /// Reads the end of the stream
    pub fn eof(&mut self) -> ParseResult<()> {
        match self.la {
            None => Ok(()),
            _ => self.absent()
        }
    }

    /// Reads a token of the given [TKind]
    pub fn lit(&mut self, kind: TKind) -> ParseResult<Token> {
        let tok = self.token.clone();

        match self.la {
            Some(la_kind) => {
                if la_kind == kind {
                    self.shift();
                    Ok(tok.unwrap())
                } else {
                    self.absent()
                }
            },
            None => self.absent()
        }
    }

    /// Reads an identifier
    pub fn ident(&mut self) -> ParseResult<String> {
        Ok(self.lit(TKind::Ident)?.text)
    }

    /// Reads a unary operator:
    /// ```
    /// unary_op = '!' | '-' | '+'
    /// ```
    pub fn unary_op(&mut self) -> ParseResult<UnOp> {
        if let Ok(_) = self.lit(TKind::Excl) {
            return Ok(UnOp::Not);
        }
        if let Ok(_) = self.lit(TKind::Minus) {
            return Ok(UnOp::Neg);
        }
        if let Ok(_) = self.lit(TKind::Plus) {
            return Ok(UnOp::Par);
        }
        self.absent()
    }

    /// Reads an unary expression: 
    /// ```
    /// unary_exp
    ///   = q_exp
    ///   | base_exp
    /// ```
    pub fn unary_exp(&mut self) -> ParseResult<ExpNode> {
        if let Ok(qexp) = self.q_exp() {
            return Ok(qexp);
        }
        
        self.base_exp()
    }

    /// Reads a base expression: 
    /// ```
    /// base_exp
    ///   = unary_op unary_exp
    ///   | '(' exp ')'
    ///   | 'True'
    ///   | 'False'
    ///   | Num
    ///   | call
    ///   | ':' Ident
    /// ```
    pub fn base_exp(&mut self) -> ParseResult<ExpNode> {
        let from = self.pos();

        if let Ok(op) = self.unary_op() {
            let exp = Self::expect(self.unary_exp(), "unary_exp")?;

            return Ok(ExpNode { 
                from, to: self.pos(),
                tree: ExpTree::UnOp(op, Box::new(exp))
            });
        }

        if let Ok(_) = self.lit(TKind::LPar) {
            let exp = Self::expect(self.exp(), "exp")?;
            Self::expect(self.lit(TKind::RPar), "RPar")?;

            return Ok(ExpNode { 
                from, to: self.pos(),
                tree: ExpTree::UnOp(UnOp::Par, Box::new(exp))
            });
        }

        if let Ok(_) = self.lit(TKind::True) {
            return Ok(ExpNode { 
                from, to: self.pos(),
                tree: ExpTree::True
            });
        }

        if let Ok(_) = self.lit(TKind::False) {
            return Ok(ExpNode { 
                from, to: self.pos(),
                tree: ExpTree::False
            });
        }

        if let Ok(t) = self.lit(TKind::Num) {
            return Ok(ExpNode { 
                from, to: self.pos(),
                tree: ExpTree::Num(t.text.clone())
            });
        }

        if let Ok(call) = self.call() {
            return Ok(call)
        }

        if let Ok(_) = self.lit(TKind::Colon) {
            let ident = Self::expect(self.ident(), "ident")?;

            return Ok(ExpNode { 
                from, to: self.pos(),
                tree: ExpTree::Bound(ident)
            });
        }

        self.absent()
    }

    /// Reads a comma separarted list of expressions
    /// ```
    /// args
    ///   = exp ',' args
    ///   | exp
    ///   | ''
    /// ```
    pub fn args(&mut self) -> ParseResult<Vec<ExpNode>> {
        if let Ok(exp) = self.exp() {
            if let Ok(_) = self.lit(TKind::Comma) {
                let mut rest = Self::expect(self.args(), "args")?;
                rest.insert(0, exp);
                return Ok(rest);
            }

            return Ok(vec![exp]);
        }

        return Ok(vec![]);
    }

    /// Reads a function call or identifier
    /// ```
    /// call
    ///   = Ident '(' args ')'
    ///   | Ident
    /// ```
    pub fn call(&mut self) -> ParseResult<ExpNode> {
        let from = self.pos();

        let name = self.ident()?;

        if let Ok(_) = self.lit(TKind::LPar) {
            let args = Self::expect(self.args(), "args")?;
            Self::expect(self.lit(TKind::RPar), "RPar")?;

            return Ok(ExpNode { 
                from, to: self.pos(),
                tree: ExpTree::Fun(name, args)
            });
        }

        return Ok(ExpNode { 
            from, to: self.pos(),
            tree: ExpTree::Ident(name)
        });
    }

    /// Reads a precedence-related binary operator.
    /// ```
    /// p_op(Mul) = mul_op
    /// p_op(Add) = add_op
    /// p_op(Eq)  = eq_op
    /// p_op(Im)  = im_op
    /// p_op(And) = and_op
    /// p_op(Or)  = or_op
    /// ```
    pub fn p_op(&mut self, p: Precedence) -> ParseResult<BinOp> {
        match p {
            Precedence::Mul => self.mul_op(),
            Precedence::Add => self.add_op(),
            Precedence::Eq => self.eq_op(),
            Precedence::Im => self.im_op(),
            Precedence::And => self.and_op(),
            Precedence::Or => self.or_op(),

            Precedence::Base => panic!(),
        }
    }

    /// Reads a precedence binary expression.
    /// ```
    /// p_exp(p) = p_exp(p.upper) (p_op(p) p_exp(p.upper))*
    /// ```
    pub fn p_exp(&mut self, p: Precedence) -> ParseResult<ExpNode> {
        if p == Precedence::Base {
            return self.base_exp();
        }

        let from = self.pos();

        let mut lhs = self.p_exp(p.upper())?;
        
        while let Ok(op) = self.p_op(p) {
            let rhs = Self::expect(self.p_exp(p.upper()), "exp")?;
            
            lhs = ExpNode { 
                from, to: self.pos(),
                tree: ExpTree::BinOp(op, Box::new(lhs), Box::new(rhs))
            };
        }

        Ok(lhs)
    }

    /// Reads a multiplication operator
    /// ```
    /// mul_op = '*' | '/' | '%'
    /// ```
    pub fn mul_op(&mut self) -> ParseResult<BinOp> {
        if let Ok(_) = self.lit(TKind::Star) {
            return Ok(BinOp::Mul);
        }
        if let Ok(_) = self.lit(TKind::Slash) {
            return Ok(BinOp::Div);
        }
        if let Ok(_) = self.lit(TKind::Perc) {
            return Ok(BinOp::Rem);
        }
        self.absent()
    }

    /// Reads an addition operator
    /// ```
    /// add_op = '+' | '-'
    /// ```
    pub fn add_op(&mut self) -> ParseResult<BinOp> {
        if let Ok(_) = self.lit(TKind::Plus) {
            return Ok(BinOp::Add);
        }
        if let Ok(_) = self.lit(TKind::Minus) {
            return Ok(BinOp::Sub);
        }
        self.absent()
    }

    /// Reads an equality operator
    /// ```
    /// eq_op = '<' | '>' | '<=' | '>=' | '==' | '!='
    /// ```
    pub fn eq_op(&mut self) -> ParseResult<BinOp> {
        if let Ok(_) = self.lit(TKind::Lt) {
            return Ok(BinOp::Lt);
        }
        if let Ok(_) = self.lit(TKind::Gt) {
            return Ok(BinOp::Gt);
        }
        if let Ok(_) = self.lit(TKind::LtEq) {
            return Ok(BinOp::LtEq);
        }
        if let Ok(_) = self.lit(TKind::GtEq) {
            return Ok(BinOp::GtEq);
        }
        if let Ok(_) = self.lit(TKind::Eq) {
            return Ok(BinOp::Eq);
        }
        if let Ok(_) = self.lit(TKind::NEq) {
            return Ok(BinOp::NEq);
        }

        self.absent()
    }

    /// Reads an implication operator
    /// ```
    /// im_op = '<-' | '->' | '<->'
    /// ```
    pub fn im_op(&mut self) -> ParseResult<BinOp> {
        if let Ok(_) = self.lit(TKind::RArrow) {
            return Ok(BinOp::Im);
        }
        if let Ok(_) = self.lit(TKind::LArrow) {
            return Ok(BinOp::RevIm);
        }
        if let Ok(_) = self.lit(TKind::LRArrow) {
            return Ok(BinOp::Equiv);
        }

        self.absent()
    }

    /// Reads a conjunction operator
    /// ```
    /// and_op = '&'
    /// ```
    pub fn and_op(&mut self) -> ParseResult<BinOp> {
        if let Ok(_) = self.lit(TKind::Amp) {
            return Ok(BinOp::And);
        }

        self.absent()
    }

    /// Reads a disjunction operator
    /// ```
    /// or_op = '|'
    /// ```
    pub fn or_op(&mut self) -> ParseResult<BinOp> {
        if let Ok(_) = self.lit(TKind::Bar) {
            return Ok(BinOp::Or);
        }

        self.absent()
    }

    /// Reads a quantifier keyword
    /// ```
    /// quant = 'all' | 'exists' | 'no'
    /// ```
    pub fn quant(&mut self) -> ParseResult<Quant> {
        if let Ok(_) = self.lit(TKind::All) {
            return Ok(Quant::All);
        }
        if let Ok(_) = self.lit(TKind::Exists) {
            return Ok(Quant::Exists);
        }
        if let Ok(_) = self.lit(TKind::No) {
            return Ok(Quant::No);
        }

        self.absent()
    }

    /// Reads a comma separated list of identifiers, at least one
    /// ```
    /// names = Ident (',' Ident)*
    /// ```
    pub fn names(&mut self) -> ParseResult<Vec<String>> {
        if let Ok(name) = self.ident() {
            if let Ok(_) = self.lit(TKind::Comma) {
                let mut rest = Self::expect(self.names(), "names")?;
                rest.insert(0, name);
                return Ok(rest);
            }

            return Ok(vec![name]);
        }

        return self.absent()
    }


    /// Reads a quantifier expression
    /// ```
    /// q_exp = quant names ':' exp
    /// ```
    pub fn q_exp(&mut self) -> ParseResult<ExpNode> {
        let from = self.pos();

        let quant = self.quant()?;
        let names = Self::expect(self.names(), "names")?;
        Self::expect(self.lit(TKind::Colon), "Colon")?;
        let exp = Self::expect(self.exp(), "exp")?;

        return Ok(ExpNode {
            from, to: self.pos(),
            tree: ExpTree::Quant(quant, names, Box::new(exp))
        });
    }

    /// Reads an expression
    /// ```
    /// exp
    ///   = q_exp
    ///   | p_exp(Or)
    /// ```
    pub fn exp(&mut self) -> ParseResult<ExpNode> {
        if let Ok(quant) = self.q_exp() {
            return Ok(quant);
        }

        self.p_exp(Precedence::Or)
    }

    /// Reads a statement
    /// ```
    /// stmt = args '|-' args
    /// ```
    pub fn stmt(&mut self) -> ParseResult<StmtNode> {
        let from = self.pos();

        let premises = self.args()?;
        Self::expect(self.lit(TKind::Ent), "Ent")?;
        let conclusions = Self::expect(self.args(), "args")?;

        Ok(StmtNode {
            from, to: self.pos(),
            premises, conclusions
        })
    }

    /// Reads a unifiable expression
    /// ```
    /// unifiable = exp '===' exp
    /// ```
    pub fn unifiable(&mut self) -> ParseResult<UnifiableNode> {
        let from = self.pos();

        let left = self.exp()?;
        Self::expect(self.lit(TKind::Equiv), "Equiv")?;
        let right = Self::expect(self.exp(), "exp")?;

        Ok(UnifiableNode {
            from, to: self.pos(),
            left, right
        })
    }

    /// Calls the given parse `fun`, if it fails reports that it expected `rule`. Then it expects EOF. Then it reports an [Error] upon failure.
    pub fn parse<F, R>(&mut self, fun: F, rule: &str) -> Result<Output<R>, Error> where F : Fn(&mut Parser<I>, &mut NameContext) -> ParseResult<R> {
        let mut nc = NameContext::new();

        let res = Self::expect_final(fun(self, &mut nc), rule)?;
        Self::expect_final(self.eof(), "EOF")?;

        Ok(Output { 
            result: res,
            name_table: nc.into_rev_table()
        })
    }
}
