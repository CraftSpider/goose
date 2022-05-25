use core::fmt;
use core::ops::Range;
use logos::Logos;

#[derive(Logos, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Token<'a> {
    #[token("def")]
    Def,
    #[token("unique")]
    #[token("u")]
    Unique,
    #[token("sync")]
    Sync,
    #[token("fn")]
    Fn,
    #[token("once")]
    Once,
    #[token("carryover")]
    CarryOver,
    #[token("type")]
    Type,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident(&'a str),
    #[regex(r"\d+")]
    Int(&'a str),
    #[regex(r"\d+\.\d*")]
    Float(&'a str),
    #[regex(r"(0|1)b")]
    Bit(&'a str),
    #[regex(r"'.'")]
    Char(&'a str),
    #[regex(r#""([^"]|\\")*""#)]
    Str(&'a str),

    #[token("[")]
    OpenBracket,
    #[token("]")]
    CloseBracket,
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[token("{")]
    OpenCurly,
    #[token("}")]
    CloseCurly,
    #[token(",")]
    Comma,
    #[token("|")]
    Pipe,
    #[token(":")]
    Colon,
    #[token(";")]
    SemiColon,
    #[token("->")]
    Arrow,
    #[token("=")]
    Eq,
    #[token("==")]
    EqEq,
    #[token("!=")]
    BangEq,
    #[token("+=")]
    PlusEq,
    #[token("-=")]
    DashEq,
    #[token("*=")]
    StarEq,
    #[token("/=")]
    SlashEq,

    #[token("str")]
    #[token("string")]
    #[token("Str")]
    #[token("String")]
    String,

    #[regex(r"\$.*", logos::skip)]
    #[regex(r"\$\$\$(?:[^$]|\$[^$]|\$\$[^$])*\$\$\$", priority = 3, callback = logos::skip)]
    #[regex(r"[ \t\r\n]+", logos::skip)]
    #[error]
    Error,
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let to_write = match self {
            Token::Def => "def",
            Token::Unique => "unique",
            Token::Sync => "sync",
            Token::Fn => "fn",
            Token::Once => "once",
            Token::CarryOver => "carryover",
            Token::Type => "type",

            Token::Ident("...") => "<ident>",
            Token::Ident(i) => *i,
            Token::Int("...") => "<int>",
            Token::Int(i) => *i,
            Token::Float("...") => "<float>",
            Token::Float(f) => *f,
            Token::Bit("...") => "<bit>",
            Token::Bit(b) => *b,
            Token::Char("...") => "<char>",
            Token::Char(c) => *c,
            Token::Str("...") => "<chararray>",
            Token::Str(s) => *s,

            Token::OpenBracket => "[",
            Token::CloseBracket => "]",
            Token::OpenParen => "(",
            Token::CloseParen => ")",
            Token::OpenCurly => "{",
            Token::CloseCurly => "}",
            Token::Comma => ",",
            Token::Pipe => "|",
            Token::Colon => ":",
            Token::SemiColon => ";",
            Token::Arrow => "->",
            Token::Eq => "=",
            Token::EqEq => "==",
            Token::BangEq => "!=",
            Token::PlusEq => "+=",
            Token::DashEq => "-=",
            Token::StarEq => "*=",
            Token::SlashEq => "/=",

            Token::String => "string",
            Token::Error => "<error>",
        };
        write!(f, "{}", to_write)
    }
}

pub fn tokenize(file: &str) -> Vec<(Token<'_>, Range<usize>)> {
    Token::lexer(file).spanned().collect()
}
