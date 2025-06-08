use super::token::{TKind, Token};
use super::coord::InputCoord;

/// The lexical analyzer
pub struct Lexer<I> where I : Iterator<Item = char> {
    /// The current coordinate
    pos: InputCoord,

    /// A text buffer
    buf: String,

    /// The lookahead character, or [None] at end
    la: Option<char>,

    /// The lookbehind character, or [None] at start
    lb: Option<char>,

    /// The character stream
    itr: I
}

impl<I> Lexer<I> where I : Iterator<Item = char> {
    /// Creates a new [Lexer]
    pub fn new(mut itr: I) -> Self {
        Self {
            pos: InputCoord::new(),
            buf: String::new(),
            la: itr.next(),
            lb: None,
            itr
        }
    }

    pub fn pos(&self) -> InputCoord {
        self.pos
    }

    /// Shift one character. Does nothing at the end of the stream.
    fn shift(&mut self) {
        if self.la == None {
            // We reached EOF, do not advance any further
            return;
        }

        self.lb = self.la;
        self.la = self.itr.next();

        match (self.lb, self.la) {
            // We are in the middle of a CRLF, don't count CR as newline as we will count the LF
            (Some('\r'), Some('\n')) => self.pos.advance(),

            // We are past a CR and no LF follows, count a newline
            (Some('\r'), _) => self.pos.newline(),

            // We are past a LF, count a newline
            (Some('\n'), _) => self.pos.newline(),

            // We did not pass a line break
            _ => self.pos.advance(),
        }
    }

    /// Pushes the current lookahead onto the text buffer, and then shifts as by [Self::shift].
    fn push_shift(&mut self) {
        match self.la {
            Some(c) => self.buf.push(c),
            None => panic!("Can't push EOF")
        }

        self.shift();
    }

    /// Skips skippable tokens, like whitespaces and comments.
    fn skip(&mut self) {
        while let Some(c @ (' ' | '\n' | '\r' | '\t' | '#')) = self.la {
            if c == '#' {
                loop {
                    self.shift();
                    if let Some('\n' | '\r') | None = self.la {
                        break;
                    }
                }
            }

            self.shift();
        }
    }

    /// Classify the [TKind] of an identifier.
    fn classify_ident(ident: String) -> (String, TKind) {
        let kind = match ident.as_str() {
            // Keywords
            "all" => TKind::All,
            "exists" => TKind::Exists,
            "no" => TKind::No,
            "let" => TKind::Let,
            "prove" => TKind::Prove,
            "true" => TKind::True,
            "false" => TKind::False,

            _ => match ident.chars().next() {
                // Number (we'll parse it later)
                Some('0'..='9') => TKind::Num,

                // Identifier
                _ => TKind::Ident
            }
        };

        return (ident, kind)
    }

    /// Read an identifier and classify it.
    fn ident(&mut self) -> (String, TKind) {
        while let Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '$') = self.la {
            self.push_shift();
        }

        return Self::classify_ident(self.buf.clone());
    }

    /// Classify a token that starts with `<`.
    fn lt(&mut self) -> (String, TKind) {
        // We had an <, what now?
        self.push_shift();

        let kind = match self.la {
            // <=
            Some('=') => {
                self.push_shift();
                TKind::LtEq
            }

            // <- or <->
            Some('-') => {
                self.push_shift();
                
                if let Some('>') = self.la {
                    // <->
                    self.push_shift();
                    TKind::LRArrow
                } else {
                    // <-
                    TKind::LArrow
                }
            },

            // <
            _ => TKind::Lt
        };

        return (self.buf.clone(), kind);
    }

    /// Classify a token that starts with `-`.
    fn minus(&mut self) -> (String, TKind) {
        // We had an -, what now?
        self.push_shift();

        let kind = match self.la {
            // ->
            Some('>') => {
                self.push_shift();
                TKind::RArrow
            }

            _ => TKind::Minus
        };

        return (self.buf.clone(), kind);
    }

    /// Classify a token that starts with `!`.
    fn excl(&mut self) -> (String, TKind) {
        // We had an !, what now?
        self.push_shift();

        let kind = match self.la {
            // !=
            Some('=') => {
                self.push_shift();
                TKind::NEq
            }

            // !
            _ => TKind::Excl
        };

        return (self.buf.clone(), kind);
    }

    /// Classify a token that starts with `>`.
    fn gt(&mut self) -> (String, TKind) {
        // We had an >, what now?
        self.push_shift();

        let kind = match self.la {
            // >=
            Some('=') => {
                self.push_shift();
                TKind::GtEq
            }

            // >
            _ => TKind::Gt
        };

        return (self.buf.clone(), kind);
    }

    /// Classify a token that starts with `|`.
    fn bar(&mut self) -> (String, TKind) {
        // We had a |, what now?
        self.push_shift();

        let kind = match self.la {
            // |-
            Some('-') => {
                self.push_shift();
                TKind::Ent
            }

            // |
            _ => TKind::Bar
        };

        return (self.buf.clone(), kind);
    }

    /// Classify a token that starts with `=`.
    fn eq(&mut self) -> (String, TKind) {
        // We had an =, what now?
        self.push_shift();

        let kind = match self.la {
            // ==
            Some('=') => {
                self.push_shift();

                match self.la {
                    // ===
                    Some('=') => {
                        self.push_shift();
                        TKind::Equiv
                    }

                    // ==
                    _ => TKind::Eq
                }
            }

            // =
            _ => TKind::Is
        };

        return (self.buf.clone(), kind);
    }

    /// Classify a token by the given kind
    fn sym(&mut self, kind: TKind) -> (String, TKind) {
        self.push_shift();
        return (self.buf.clone(), kind);
    }

    /// Read a token. Returns [None] at the end of stream. Bad tokens are given as a token of [TKind::Illegal].
    pub fn token(&mut self) -> Option<Token> {
        self.skip();
        self.buf.clear();

        let from = self.pos;

        let (text, kind) = match self.la {
            Some('0'..='9' | 'a'..='z' | 'A'..='Z' | '_' | '$') => self.ident(),

            Some('=') => self.eq(),
            Some('<') => self.lt(),
            Some('>') => self.gt(),


            Some('+') => self.sym(TKind::Plus),
            Some('-') => self.minus(),
            Some('*') => self.sym(TKind::Star),
            Some('/') => self.sym(TKind::Slash),
            Some('%') => self.sym(TKind::Perc),

            Some('!') => self.excl(),
            Some('|') => self.bar(),
            Some('&') => self.sym(TKind::Amp),

            Some(':') => self.sym(TKind::Colon),
            Some(',') => self.sym(TKind::Comma),
            Some('(') => self.sym(TKind::LPar),
            Some(')') => self.sym(TKind::RPar),

            Some(_) => self.sym(TKind::Illegal),

            None => return None,
        };


        let to = self.pos;

        return Some(Token {
            kind,
            text,
            from,
            to
        });
    }
}
