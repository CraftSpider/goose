use chumsky::error::Simple;
use chumsky::{Parser, Stream};
use core::ops::Range;

use crate::ast::File;
use crate::token::Token;

pub fn parse<'a>(tokens: &[(Token<'a>, Range<usize>)]) -> Result<File, Vec<Simple<Token<'a>>>> {
    let stream = Stream::from_iter(
        tokens
            .last()
            .map(|(_, range)| range.clone())
            .unwrap_or(0..0),
        tokens.iter().cloned(),
    );

    File::parser().parse(stream)
}
