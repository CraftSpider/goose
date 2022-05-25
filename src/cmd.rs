
use std::path::PathBuf;
use clap::{Parser, ArgEnum};

#[derive(Copy, Clone, Debug, PartialEq)]
#[derive(ArgEnum)]
pub enum Emit {
    Tokens,
    Ast,
}

#[derive(Debug)]
#[derive(Parser)]
#[clap(version, about, long_about = None)]
pub struct Command {
    #[clap(parse(from_os_str))]
    pub(crate) file: PathBuf,
    #[clap(long, arg_enum, value_delimiter = ',', value_name = "EMIT")]
    emit: Vec<Emit>,
}

impl Command {
    pub fn should_emit(&self, e: Emit) -> bool {
        self.emit.contains(&e)
    }
}
