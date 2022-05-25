use core::ops::Range;
use chumsky::{Parser, Stream};
use chumsky::error::Simple;

use crate::token::Token;
use crate::ast::File;

pub fn parse<'a>(tokens: &[(Token<'a>, Range<usize>)]) -> Result<File, Vec<Simple<Token<'a>>>> {

    let stream = Stream::from_iter(
        tokens.last().unwrap().1.clone(),
        tokens.iter().cloned(),
    );

    File::parser()
        .parse(stream)
}
