use std::collections::HashMap;
use std::io::Read;

use Token::*;
use ParseResult::*;

use crate::expr::*;

/// A token.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Token {
    /// `(`
    ParL,

    /// `)`
    ParR,

    /// `P`, `Q`, etc.
    Sym(u64),

    /// `!`
    Not,

    /// `|`
    Or,

    /// `&`
    And,

    /// `^`
    Xor,

    /// `<-`
    ImplL,

    /// `->`
    ImplR,

    /// `<->`
    Equiv,

    /// End of file
    Eof,

    /// `,`
    Comma,

    /// `|-`
    Turnstile,

    /// Invalid tokens
    Unknown
}



/// The result of a parsing operation. 
#[derive(Clone, PartialEq, Eq)]
enum ParseResult<T> {
    /// The parsing succeeded and the argument holds the parsed element.
    Ok(T),

    /// The parsed element is not present in the input.
    Absent(String, ParseCoord),

    /// The parsed element was present but in an invalid syntax.
    Error(String, ParseCoord)
}

impl<T> ParseResult<T> {
    /// If this result is [Absent], converts it to an [Error] with given message.
    fn to_error(self, msg: &str) -> ParseResult<T> {
        match self {
            Ok(t) => Ok(t),
            Absent(_, p) => Absent(String::from(msg), p),
            Error(s, p) => Error(s, p)
        }
    }

    /// If this result is [Ok], maps the value using the given function.
    fn apply<F, U>(self, func: F) -> ParseResult<U> where F : Fn(T) -> U {
        match self {
            Ok(t) => Ok(func(t)),
            Absent(s, p) => Absent(s, p),
            Error(s, p) => Error(s, p)
        }
    }
}



/// A coordinate in the parser input. It consists of an index, line number and column number.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ParseCoord {
    /// The index in the text, i.e. the amount of characters from the start of the text, starting at 0.
    pub index: usize,

    /// The line number, starting at 1.
    pub line: usize,

    /// The column number, starting at 1.
    pub col: usize
}

impl ParseCoord {
    /// Creates and initialises a [ParseCoord].
    fn new() -> Self {
        Self {
            index: 0,
            line: 1,
            col: 1
        }
    }

    /// Advances this coordinate to a new line, increasing the line number and resetting the column number.
    fn newline(&mut self) {
        self.index += 1;
        self.line += 1;
        self.col = 1;
    }

    /// Advances this coordinate to the next position, increasing the column number and leaving the line number the same.
    fn advance(&mut self) {
        self.index += 1;
        self.col += 1;
    }
}



/// The input parser.
struct Parser<I> where I : Iterator<Item = char> {
    /// The iterator of characters that iterates the input.
    iter: I,

    /// The lookbehind character.
    lb_chr: Option<char>,

    /// The lookahead character.
    la_chr: Option<char>,

    /// The lookahead [Token].
    la_tok: Token,

    /// The current coordinate, as a [ParseCoord].
    pos: ParseCoord,

    /// The symbol table, which maps names to integers
    symbols: HashMap<String, u64>,
    next_sym: u64
}

impl<I> Parser<I> where I : Iterator<Item = char> {
    /// Creates a new parser for the given character iterator.
    fn new(r: I) -> Self {
        let mut o = Self {
            iter: r,
            lb_chr: None,
            la_chr: None,
            la_tok: Unknown,
            pos: ParseCoord::new(),
            symbols: HashMap::new(),
            next_sym: 0
        };
        o.shift_chr();
        o.shift_tok();
        return o;
    }

    /// Shifts to the next character.
    fn shift_chr(&mut self) {
        self.lb_chr = self.la_chr;
        self.la_chr = self.iter.next();

        match (self.lb_chr, self.la_chr) {
            (Some('\r'), Some('\n')) => self.pos.advance(),
            (Some('\r'), _) => self.pos.newline(),
            (Some('\n'), _) => self.pos.newline(),
            (_, _) => self.pos.advance(),
        }
    }

    /// Shifts over as many characters that match the given `predicate` as possible, shifting until it finds
    /// a character that does not match or until it hits the end of the input.
    fn shift_chr_while<P>(&mut self, predicate: P) -> String where P: Fn(char) -> bool {
        let mut out = String::new();

        while let Some(c) = self.la_chr {
            if !predicate(c) {
                break;
            }

            self.shift_chr();
            out.push(c);
        }

        return out;
    }


    /// Skips over whitespaces and comments. Comments sit between a `#` and the end of a line.
    fn skip_ws(&mut self) {
        while let Some(c) = self.la_chr {
            if c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '#' {
                // Skip spaces
                self.shift_chr_while(|c| c == ' ' || c == '\t' || c == '\n' || c == '\r');

                // Skip comment
                if let Some('#') = self.la_chr {
                    self.shift_chr_while(|c| c != '\n' && c != '\r');
                }
            } else {
                break;
            }
        }
    }
    
    /// Shifts to the next token.
    fn shift_tok(&mut self) {
        self.skip_ws();

        let la = self.la_chr;
        self.la_tok = match la {
            // End of stream
            None => Eof,

            // Alphabetic character, this is a symbol
            Some('A'..='Z' | 'a'..='z') => {
                let sym = self.shift_chr_while(|c| matches!(c, 'A'..='Z' | 'a'..='z' | '0'..='9' | '_'));
                Sym(self.lookup_sym(sym))
            },

            // Either `|` or `|-`
            Some('|') => {
                self.shift_chr();

                if let Some('-') = self.la_chr {
                    self.shift_chr();
                    Turnstile
                } else {
                    Or
                }
            },

            // These are quite self-explanatory
            Some('&') => {
                self.shift_chr();
                And
            },

            Some('^') => {
                self.shift_chr();
                Xor
            },

            Some('!') => {
                self.shift_chr();
                Not
            },

            Some(',') => {
                self.shift_chr();
                Comma
            },

            Some('(') => {
                self.shift_chr();
                ParL
            },

            Some(')') => {
                self.shift_chr();
                ParR
            },


            // `->`
            Some('-') => {
                self.shift_chr();

                if let Some('>') = self.la_chr {
                    self.shift_chr();
                    
                    ImplR
                } else {
                    Unknown
                }
            },

            // Either `<-` or `<->`
            Some('<') => {
                self.shift_chr();

                if let Some('-') = self.la_chr {
                    self.shift_chr();

                    if let Some('>') = self.la_chr {
                        self.shift_chr();
                        
                        Equiv
                    } else {
                        ImplL
                    }
                } else {
                    Unknown
                }
            },

            // Anythinig else is invalid
            _ => Unknown
        }
    }

    /// Associates a symbol name with an [u64] that uniquely represents that symbol name.
    fn lookup_sym(&mut self, name: String) -> u64 {
        match self.symbols.get(&name) {
            Some(sym) => *sym,
            None => {
                let sym = self.next_sym;
                self.next_sym += 1;

                self.symbols.insert(name, sym);

                sym
            }
        }
    }

    fn expr(&mut self) -> ParseResult<Expr> {
        // expr  :  or
        self.or()
    }

    fn atom(&mut self) -> ParseResult<Expr> {
        // atom  :  Sym
        //       |  '!' atom
        //       |  '(' expr ')'

        match self.la_tok {
            // atom  :  Sym
            Sym(c) => {
                self.shift_tok();
                Ok(sym(c))
            },

            // atom  :  '!' atom
            Not => {
                self.shift_tok();

                self.atom().apply(|e| not(e)).to_error("Expected atom expression")
            },

            // atom  :  '(' expr ')'
            ParL => {
                self.shift_tok();

                let res = self.expr().to_error("Expected expression");
                let la = self.la_tok;

                match (res, la) {
                    // Expression parsed and we've got a closing )
                    (Ok(expr), ParR) => {
                        self.shift_tok();
                        Ok(expr)
                    },

                    // Expression parsed but there is no closing )
                    (Ok(_), _) => self.error("Expected ')'"),

                    // Expression failed to parse
                    (res, _) => res,
                }
            }

            _ => self.absent("Expected atomic expression")
        }
    }

    fn binary_op<T, L, A>(&mut self, token: &T, lower: &L, apply: &A, err: &str) -> ParseResult<Expr> where
        T : Fn(Token) -> bool,
        L : Fn(&mut Self) -> ParseResult<Expr>,
        A : Fn(Expr, Expr, Token) -> Expr
    {
        let res = lower(self);
        let la = self.la_tok;

        let lhs = match (res, la) {
            // There was no lower precedence expression
            (Error(m, p), _) => return Error(m, p),
            (Absent(m, p), _) => return Absent(m, p),

            // There was a lower precedence expression
            (Ok(lhs), tok) => {
                if !token(tok) {
                    // Token did not match, so the lower precedence expression is the whole binary expression
                    return Ok(lhs)
                } else {
                    // Token matched, so the lower precedence expression is our left hand side and we parse more
                    lhs
                }
            }
        };

        // Shift over the token
        self.shift_tok();

        // Parse right hand side
        let rhs = match self.binary_op(token, lower, apply, err) {
            // Right hand side parsed
            Ok(rhs) => rhs,

            // No right hand side parsed, return error
            e => return e.to_error(err),
        };

        Ok(apply(lhs, rhs, la))
    }

    fn implication(&mut self) -> ParseResult<Expr> {
        // implication  :  atom '->' implication
        //              |  atom '<-' implication
        //              |  atom '<->' implication
        //              |  atom

        self.binary_op(
            &|t| t == ImplL || t == ImplR || t == Equiv,
            &Self::atom,
            &|lhs, rhs, tok| match tok {
                ImplL => imp(rhs, lhs),
                ImplR => imp(lhs, rhs),
                Equiv => equiv(lhs, rhs),
                _ => panic!("Invalid token found")
            },
            "Expected implication"
        )
    }

    fn xor(&mut self) -> ParseResult<Expr> {
        // xor  :  implication '^' xor
        //      |  implication

        self.binary_op(
            &|t| t == Xor,
            &Self::implication,
            &|lhs, rhs, _| xor(lhs, rhs),
            "Expected xor expression"
        )
    }

    fn and(&mut self) -> ParseResult<Expr> {
        // and  :  xor '^' and
        //      |  xor

        self.binary_op(
            &|t| t == And,
            &Self::xor,
            &|lhs, rhs, _| and(lhs, rhs),
            "Expected and expression"
        )
    }

    fn or(&mut self) -> ParseResult<Expr> {
        // or  :  and '^' or
        //     |  and

        self.binary_op(
            &|t| t == Or,
            &Self::and,
            &|lhs, rhs, _| or(lhs, rhs),
            "Expected or expression"
        )
    }

    fn exprs(&mut self) -> ParseResult<Expr> {
        // exprs  :  expr ',' exprs
        //        |  expr

        self.binary_op(
            &|t| t == Comma,
            &Self::expr,
            &|lhs, rhs, _| and(lhs, rhs),
            "Expected expressions"
        )
    }

    fn turnstile(&mut self) -> ParseResult<Expr> {
        // turnstile  :  exprs '|-' exprs

        let lhs = match self.exprs() {
            Ok(lhs) => lhs,
            e => return e
        };
        
        if let Turnstile = self.la_tok {
            self.shift_tok();
        } else {
            return self.error("Expected '|-'");
        }

        let rhs = match self.exprs().to_error("Expected expressions") {
            Ok(rhs) => rhs,
            e => return e
        };
        
        if let Eof = self.la_tok {
        } else {
            return self.error("Expected EOF");
        }

        return Ok(and(lhs, not(rhs)));
    }

    fn absent<T>(&self, message: &str) -> ParseResult<T> {
        Absent(String::from(message), self.pos)
    }

    fn error<T>(&self, message: &str) -> ParseResult<T> {
        Error(String::from(message), self.pos)
    }
}

fn make_msg(m: String, c: ParseCoord) -> String {
    let mut out = String::new();
    
    out.push_str(format!("{}:{}: ", c.line, c.col).as_str());
    out.push_str(m.as_str());

    return out;
}

pub fn parse<R>(mut r: R) -> Result<Expr, String> where R : Read {
    let mut str = String::new();

    r.read_to_string(&mut str).map_err(|_| String::from("IO Error"))?;

    let mut parser = Parser::new(str.chars());

    match parser.turnstile() {
        Ok(e) => Result::Ok(e),
        Absent(m, c) | Error(m, c) => Err(make_msg(m, c))
    }
}