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

    #[token("`")]
    Tick,
    #[token("~")]
    Tilde,
    #[token("!")]
    Bang,
    #[token("@")]
    At,
    #[token("#")]
    Hash,
    #[token("%")]
    Percent,
    #[token("^")]
    Caret,
    #[token("&")]
    And,
    #[token("*")]
    Star,
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[token("-")]
    Dash,
    #[token("=")]
    Eq,
    #[token("+")]
    Plus,
    #[token("[")]
    OpenBracket,
    #[token("]")]
    CloseBracket,
    #[token("{")]
    OpenCurly,
    #[token("}")]
    CloseCurly,
    #[token("\\")]
    Backslash,
    #[token("|")]
    Pipe,
    #[token(";")]
    SemiColon,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token("<")]
    Lt,
    #[token(".")]
    Dot,
    #[token(">")]
    Gt,
    #[token("/")]
    Slash,
    #[token("?")]
    Question,

    #[token("->")]
    Arrow,
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

    #[regex(r"\$.*")]
    #[regex(r"\$\$\$(?:[^$]|\$[^$]|\$\$[^$])*\$\$\$", priority = 3)]
    Comment(&'a str),

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

            Token::Tick => "`",
            Token::Tilde => "~",
            Token::Bang => "!",
            Token::At => "@",
            Token::Hash => "#",
            Token::Percent => "%",
            Token::Caret => "^",
            Token::And => "&",
            Token::Star => "*",
            Token::OpenParen => "(",
            Token::CloseParen => ")",
            Token::Dash => "-",
            Token::Eq => "=",
            Token::Plus => "+",
            Token::OpenBracket => "[",
            Token::CloseBracket => "]",
            Token::OpenCurly => "{",
            Token::CloseCurly => "}",
            Token::Backslash => "\\",
            Token::Pipe => "|",
            Token::SemiColon => ";",
            Token::Colon => ":",
            Token::Comma => ",",
            Token::Lt => "<",
            Token::Dot => ".",
            Token::Gt => ">",
            Token::Slash => "/",
            Token::Question => "?",

            Token::Arrow => "->",
            Token::EqEq => "==",
            Token::BangEq => "!=",
            Token::PlusEq => "+=",
            Token::DashEq => "-=",
            Token::StarEq => "*=",
            Token::SlashEq => "/=",

            Token::String => "string",
            Token::Comment(c) => *c,
            Token::Error => "<error>",
        };
        write!(f, "{}", to_write)
    }
}

pub fn tokenize(file: &str) -> Vec<(Token<'_>, Range<usize>)> {
    Token::lexer(file)
        .spanned()
        .filter(|(t, _)| !matches!(t, Token::Comment(_)))
        .collect()
}

/*pub fn tokenize_with_comments(file: &str) -> Vec<(Token<'_>, Range<usize>)> {
    Token::lexer(file).spanned().collect()
}*/
