use core::ops::Deref;

mod interp;
mod parser;

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
    Sub,
    Mul,
    Div,
}

impl BinOp {
    pub fn pretty(&self) -> String {
        match self {
            BinOp::Eq => String::from("=="),
            BinOp::Neq => String::from("!="),
            BinOp::Add => String::from("+"),
            BinOp::Sub => String::from("-"),
            BinOp::Mul => String::from("*"),
            BinOp::Div => String::from("/"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expr {
    FnCall(FnCall),
    Write(WriteTy, Vec<Expr>),
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
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ret_ty(&self) -> &Type {
        &self.ret
    }

    pub fn arg_tys(&self) -> Vec<Type> {
        self.args.iter().map(|a| a.ty.clone()).collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ident(pub(crate) String);

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
    Fn(FnDef),
    Array(Vec<Expr>),
}

#[derive(Clone, Debug)]
pub enum Stmt {
    FnDef(FnDef),
    Assign(Assign),
    Once(Vec<Stmt>),
    Sync(Vec<Stmt>),
    Expr(Expr),
    TypeDef(Ident, Type),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Named(Ident),
    Array(Box<Type>),
    Fn(Box<Type>, Vec<Type>),
}

impl Type {
    pub(crate) fn named(name: &str) -> Type {
        Type::Named(Ident(name.to_string()))
    }

    pub fn pretty(&self) -> String {
        match self {
            Type::Named(name) => String::from(&**name),
            Type::Array(inner) => format!("[{}]", inner.pretty()),
            Type::Fn(ret, args) => format!(
                "fn: {} ({})",
                ret.pretty(),
                args.iter()
                    .map(|ty| ty.pretty())
                    .intersperse(String::from(" "))
                    .collect::<String>(),
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub enum WriteTy {
    Console,
    Error,
    RawFile,
    Other(Box<Expr>),
}
