#![feature(iter_intersperse)]

use std::process::ExitCode;

mod token;
mod ast;
mod parser;
mod interp;
mod cmd;

fn main() -> ExitCode {
    use token::tokenize;
    use parser::parse;
    use interp::Env;
    use cmd::{Command, Emit};
    use clap::Parser;

    let args = Command::parse();

    let file = &args.file;

    let file = match std::fs::read_to_string(file) {
        Ok(file) => file,
        Err(e) => {
            println!("Couldn't read provided file: {}", e);
            return ExitCode::FAILURE
        }
    };

    let tokens = tokenize(&file);

    if args.should_emit(Emit::Tokens) {
        println!("{:?}", tokens.iter().map(|(tok, _)| tok).collect::<Vec<_>>());
    }

    let ast = match parse(&tokens) {
        Ok(ast) => ast,
        Err(errs) => {
            for err in errs {
                println!("Parse Failure: {}", err);
                println!("at {}", &file[err.span().start.saturating_sub(10)..usize::min(err.span().end + 10, file.len())])
            };
            return ExitCode::FAILURE;
        }
    };

    if args.should_emit(Emit::Ast) {
        println!("{:#?}", ast);
    }

    let mut ctx = Env::default();
    if let Err(e) = ast.interpret(&mut ctx) {
        println!("Exception: {}", e);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
