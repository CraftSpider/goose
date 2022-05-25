
use core::ops::Deref;

mod parser;
mod interp;

#[derive(Clone, Debug)]
pub struct Assign {
    ty: AssignTy,
    ident: Ident,
    assign_op: AssignOp,
    val: Expr,
}

#[derive(Copy, Clone, Debug)]
pub enum AssignOp {
    Eq,
    PlusEq,
    SubEq,
    MulEq,
    DivEq,
}

#[derive(Copy, Clone, Debug)]
pub enum AssignTy {
    Unique,
    CarryOver,
    Default,
}

#[derive(Copy, Clone, Debug)]
pub enum BinOp {
    Eq,
    Neq,
    Add,
}

impl BinOp {
    pub fn pretty(&self) -> String {
        match self {
            BinOp::Eq => String::from("=="),
            BinOp::Neq => String::from("!="),
            BinOp::Add => String::from("+"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expr {
    FnCall(FnCall),
    Literal(Literal),
    Ident(Ident),
    BinOp(Box<Expr>, BinOp, Box<Expr>),
}

#[derive(Debug)]
pub struct File {
    stmts: Vec<Stmt>,
}

#[derive(Clone, Debug)]
pub struct FnArg {
    name: Ident,
    ty: Type,
}

#[derive(Clone, Debug)]
pub struct FnCall {
    name: Ident,
    args: Vec<Expr>,
}

#[derive(Clone, Debug)]
pub struct FnDef {
    name: Ident,
    ret: Type,
    args: Vec<FnArg>,
    limit: Box<Expr>,
    stmts: Vec<Stmt>,
}

impl FnDef {
    pub fn ret_ty(&self) -> &Type {
        &self.ret
    }

    pub fn arg_tys(&self) -> Vec<Type> {
        self.args.iter().map(|a| a.ty.clone()).collect()
    }
}

#[derive(Clone, Debug)]
pub struct Ident(String);

impl Deref for Ident {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub enum Literal {
    Int(i128),
    Float(f64),
    Char(char),
    CharArray(String),
    Bit(bool),
    Array(Box<Expr>),
}

#[derive(Clone, Debug)]
pub enum Stmt {
    FnDef(FnDef),
    Assign(Assign),
    Once(Vec<Stmt>),
    Sync(Vec<Stmt>),
    Expr(Expr),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Null,
    Int,
    Float,
    Char,
    CharArray,
    Bit,
    Array(Box<Type>),
    Fn(Box<Type>, Vec<Type>),
}

impl Type {
    pub fn pretty(&self) -> String {
        match self {
            Type::Null => String::from("null"),
            Type::Int => String::from("int"),
            Type::Float => String::from("float"),
            Type::Char => String::from("char"),
            Type::CharArray => String::from("chararray"),
            Type::Bit => String::from("bit"),
            Type::Array(inner) => format!("[{}]", inner.pretty()),
            Type::Fn(ret, args) => format!(
                "fn: {} ({})",
                ret.pretty(),
                args.iter().map(|ty| ty.pretty()).intersperse(String::from(" ")).collect::<String>(),
            )
        }
    }
}
