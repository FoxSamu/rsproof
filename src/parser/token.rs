use super::coord::InputCoord;

/// A kind of token
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum TKind {
    // Basic literals

    /// Numbers: `314`, `0x88`, `0b101`
    Num,

    /// Identifiers: `p`, `fox`, `agent_007`
    Ident,


    // Keywords

    /// Keyword `true`
    True,

    /// Keyword `false`
    False,

    /// Keyword `all`
    All,

    /// Keyword `exists`
    Exists,

    /// Keyword `no`
    No,

    /// Keyword `let`
    Let,

    /// Keyword `prove`
    Prove,


    // Arithmetic

    /// `+`
    Plus,
    
    /// `-`
    Minus,

    /// `*`
    Star,

    /// `/`
    Slash,

    /// `%`
    Perc,


    // Equality

    /// `==`
    Eq,

    /// `!=`
    NEq,

    /// `<=`
    LtEq,

    /// `>=`
    GtEq,

    /// `<`
    Lt,

    /// `>`
    Gt,


    // Logic

    /// `!`
    Excl,

    /// `&`
    Amp,

    /// `|`
    Bar,

    /// `->`
    RArrow,

    /// `<-`
    LArrow,

    /// `<->`
    LRArrow,

    
    // Punctuation

    /// `:`
    Colon,

    /// `,`
    Comma,

    /// `(`
    LPar,

    /// `)`
    RPar,


    // Statement

    /// `|-`
    Ent,


    // Other

    /// Any illegal symbol
    Illegal
}


/// A token
#[derive(Clone, Debug)]
pub struct Token {
    /// The kind of token
    pub kind: TKind,

    /// The token text
    pub text: String,

    /// The coordinate at the start of the token
    pub from: InputCoord,

    /// The coordinate at the end of the token
    pub to: InputCoord
}