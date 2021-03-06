use chumsky::prelude::*;
use std::str::FromStr;

use super::*;
use crate::token::Token;

macro_rules! Parser {
    ($lt:lifetime, $ty:ty) => {
        impl chumsky::Parser<Token<$lt>, $ty, Error = chumsky::error::Simple<Token<$lt>>> + Clone + $lt
    }
}

impl Assign {
    pub fn parser<'a>(expr: Parser!['a, Expr]) -> Parser!['a, Self] {
        just(Token::Unique)
            .to(AssignTy::Unique)
            .or(just(Token::CarryOver).to(AssignTy::CarryOver))
            .or_not()
            .map(|ty| ty.unwrap_or(AssignTy::Default))
            .then(Ident::parser())
            .then(AssignOp::parser())
            .then(expr)
            .then_ignore(just(Token::SemiColon))
            .map(|(((ty, ident), assign_op), val)| Assign {
                ty,
                ident,
                assign_op,
                val,
            })
    }
}

impl AssignOp {
    pub fn parser<'a>() -> Parser!['a, Self] {
        just(Token::Eq)
            .to(AssignOp::Eq)
            .or(just(Token::PlusEq).to(AssignOp::PlusEq))
            .or(just(Token::DashEq).to(AssignOp::SubEq))
            .or(just(Token::StarEq).to(AssignOp::MulEq))
            .or(just(Token::SlashEq).to(AssignOp::DivEq))
    }
}

impl BinOp {
    pub fn cmp_parser<'a>() -> Parser!['a, Self] {
        just(Token::EqEq)
            .to(BinOp::Eq)
            .or(just(Token::BangEq).to(BinOp::Neq))
    }

    pub fn add_parser<'a>() -> Parser!['a, Self] {
        just(Token::Plus)
            .to(BinOp::Add)
            .or(just(Token::Dash).to(BinOp::Sub))
    }

    pub fn mul_parser<'a>() -> Parser!['a, Self] {
        just(Token::Star)
            .to(BinOp::Mul)
            .or(just(Token::Slash).to(BinOp::Div))
    }
}

impl Expr {
    pub fn parser<'a>() -> Parser!['a, Self] {
        recursive(|expr| {
            let atom = Literal::parser(expr.clone())
                .map(Expr::Literal)
                .or(just(Token::Ident("write"))
                    .ignore_then(just(Token::OpenParen))
                    .ignore_then(WriteTy::parser(expr.clone()))
                    .then_ignore(just(Token::Comma))
                    .then(expr.clone().separated_by(just(Token::Comma)))
                    .then_ignore(just(Token::CloseParen))
                    .map(|(ty, exprs)| Expr::Write(ty, exprs)))
                .or(FnCall::parser(expr).map(Expr::FnCall))
                .or(Ident::parser().map(Expr::Ident))
                .boxed();

            let unary = UnOp::parser()
                .repeated()
                .then(atom)
                .foldr(|op, expr| Expr::UnOp(op, Box::new(expr)));

            let bin_parsers = [
                BinOp::mul_parser().boxed(),
                BinOp::add_parser().boxed(),
                BinOp::cmp_parser().boxed(),
            ];

            let mut binary = unary.boxed();

            for op_parser in bin_parsers {
                binary = binary
                    .clone()
                    .then(op_parser.then(binary).repeated())
                    .foldl(|left, (op, right)| Expr::BinOp(Box::new(left), op, Box::new(right)))
                    .boxed();
            }

            binary
        })
    }
}

impl File {
    pub fn parser<'a>() -> Parser!['a, Self] {
        Stmt::parser(Expr::parser())
            .repeated()
            .then_ignore(end())
            .map(|stmts| File { stmts })
    }
}

impl FnArg {
    pub fn parser<'a>() -> Parser!['a, Self] {
        Ident::parser()
            .then_ignore(just(Token::Colon))
            .then(Type::parser())
            .map(|(name, ty)| FnArg { name, ty })
    }
}

impl FnCall {
    pub fn parser<'a>(expr: Parser!['a, Expr]) -> Parser!['a, Self] {
        Ident::parser()
            .then(
                expr.separated_by(just(Token::Comma))
                    .delimited_by(just(Token::OpenParen), just(Token::CloseParen)),
            )
            .map(|(name, args)| FnCall { name, args })
    }
}

impl FnDef {
    pub fn parser<'a>(expr: Parser!['a, Expr], stmt: Parser!['a, Stmt]) -> Parser!['a, Self] {
        just(Token::Def)
            .ignore_then(Ident::parser())
            .then_ignore(just(Token::Colon))
            .then(Type::parser())
            .then(
                FnArg::parser()
                    .separated_by(just(Token::Comma))
                    .delimited_by(just(Token::OpenParen), just(Token::CloseParen)),
            )
            .then_ignore(just(Token::Arrow))
            .then(expr.delimited_by(just(Token::Pipe), just(Token::Pipe)))
            .then(
                stmt.repeated()
                    .delimited_by(just(Token::OpenBracket), just(Token::CloseBracket)),
            )
            .map(|((((name, ret), args), limit), stmts)| FnDef {
                name,
                ret,
                args,
                limit: Box::new(limit),
                stmts,
            })
    }
}

impl Ident {
    pub fn parser<'a>() -> Parser!['a, Self] {
        filter_map(|span, tok| {
            if let Token::Ident(i) = tok {
                Ok(Ident(i.to_string()))
            } else {
                Err(Simple::expected_input_found(
                    span,
                    [Some(Token::Ident("..."))],
                    Some(tok),
                ))
            }
        })
    }
}

impl Literal {
    pub fn parser<'a>(expr: Parser!['a, Expr]) -> Parser!['a, Self] {
        filter_map(|span, tok| match tok {
            Token::Int(i) => Ok(Literal::Int(i128::from_str(i).unwrap())),
            Token::Float(f) => Ok(Literal::Float(f64::from_str(f).unwrap())),
            Token::Bit(b) => Ok(Literal::Bit(&b[0..1] == "1")),
            Token::Char(c) => Ok(Literal::Char(c.chars().nth(1).unwrap())),
            Token::Str(s) => Ok(Literal::CharArray(s.to_string())),
            _ => Err(Simple::expected_input_found(
                span,
                [
                    Some(Token::Int("...")),
                    Some(Token::Float("...")),
                    Some(Token::Bit("...")),
                    Some(Token::Char("...")),
                    Some(Token::Str("...")),
                ],
                Some(tok),
            )),
        })
        .or(expr
            .clone()
            .separated_by(just(Token::Comma))
            .delimited_by(just(Token::OpenBracket), just(Token::CloseBracket))
            .map(Literal::Array))
        .or(just(Token::Fn)
            .ignore_then(just(Token::Colon))
            .ignore_then(Type::parser())
            .then(
                FnArg::parser()
                    .separated_by(just(Token::Comma))
                    .delimited_by(just(Token::OpenParen), just(Token::CloseParen)),
            )
            .then_ignore(just(Token::Arrow))
            .then(
                expr.clone()
                    .delimited_by(just(Token::Pipe), just(Token::Pipe)),
            )
            .then(
                Stmt::parser(expr)
                    .repeated()
                    .delimited_by(just(Token::OpenBracket), just(Token::CloseBracket)),
            )
            .map(|(((ret, args), limit), stmts)| {
                Literal::Fn(FnDef {
                    name: Ident(String::from("<closure>")),
                    ret,
                    args,
                    limit: Box::new(limit),
                    stmts,
                })
            }))
    }
}

impl Stmt {
    pub fn parser<'a>(expr: Parser!['a, Expr]) -> Parser!['a, Self] {
        recursive(|stmt| {
            FnDef::parser(expr.clone(), stmt.clone())
                .map(Stmt::FnDef)
                .or(Assign::parser(expr.clone()).map(Stmt::Assign))
                .or(just(Token::Sync)
                    .ignore_then(
                        stmt.clone()
                            .repeated()
                            .delimited_by(just(Token::OpenCurly), just(Token::CloseCurly)),
                    )
                    .map(Stmt::Sync))
                .or(just(Token::Once)
                    .ignore_then(
                        stmt.repeated()
                            .delimited_by(just(Token::OpenCurly), just(Token::CloseCurly)),
                    )
                    .map(Stmt::Once))
                .or(just(Token::Type)
                    .ignore_then(Ident::parser())
                    .then_ignore(just(Token::Eq))
                    .then(Type::parser())
                    .then_ignore(just(Token::SemiColon))
                    .map(|(name, ty)| Stmt::TypeDef(name, ty)))
                .or(expr.map(Stmt::Expr).then_ignore(just(Token::SemiColon)))
        })
    }
}

impl Type {
    pub fn parser<'a>() -> Parser!['a, Self] {
        recursive(|ty| {
            Ident::parser().map(Type::Named)
            .or(ty
                .clone()
                .delimited_by(just(Token::OpenBracket), just(Token::CloseBracket))
                .map(|ty| Type::Array(Box::new(ty))))
            .or(just(Token::Fn)
                .ignore_then(just(Token::Colon))
                .ignore_then(ty.clone())
                .then(
                    ty.separated_by(just(Token::Comma))
                        .delimited_by(just(Token::OpenParen), just(Token::CloseParen)),
                )
                .map(|(ret, args)| Type::Fn(Box::new(ret), args)))
        })
    }
}

impl UnOp {
    pub fn parser<'a>() -> Parser!['a, Self] {
        just(Token::Bang).to(UnOp::Inv)
            .or(just(Token::Dash).to(UnOp::Neg))
    }
}

impl WriteTy {
    pub fn parser<'a>(expr: Parser!['a, Expr]) -> Parser!['a, Self] {
        just(Token::Ident("console"))
            .to(WriteTy::Console)
            .or(just(Token::Ident("error")).to(WriteTy::Error))
            .or(just(Token::Ident("honk")).to(WriteTy::RawFile))
            .or(expr.map(|expr| WriteTy::Other(Box::new(expr))))
    }
}
