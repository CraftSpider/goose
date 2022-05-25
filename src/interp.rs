use core::fmt;
use std::collections::HashMap;
use std::io;
use std::io::Error;

use crate::ast::{BinOp, FnDef, Ident, Type};

pub type Result<T> = core::result::Result<T, Exception>;

#[derive(Debug)]
pub enum Exception {
    InvalidType(Type, Type),
    InvalidOp(Type, BinOp, Type),
    NameNotFound(Ident),
    Io,
}

impl fmt::Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Exception::InvalidType(expected, actual) => {
                write!(
                    f,
                    "Expected type `{}`, got type `{}`",
                    expected.pretty(),
                    actual.pretty()
                )
            }
            Exception::InvalidOp(left, op, right) => {
                write!(
                    f,
                    "Attempted to invoke operator {} on invalid types. Left: `{}`, Right: `{}`",
                    op.pretty(),
                    left.pretty(),
                    right.pretty(),
                )
            }
            Exception::NameNotFound(name) => {
                write!(f, "Attempted to access invalid identifier {}", &**name)
            }
            Exception::Io => {
                write!(f, "IO operation failed")
            }
        }
    }
}

impl From<io::Error> for Exception {
    fn from(_: Error) -> Self {
        Exception::Io
    }
}

#[derive(Clone)]
pub struct BuiltinFn {
    name: Ident,
    ret: Type,
    args: Vec<Type>,
    handler: for<'ip> fn(env: &mut Env<'ip>, args: &[Value<'ip>]) -> Result<Value<'ip>>,
}

impl fmt::Debug for BuiltinFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BuiltinFn")
            .field("ret", &self.ret)
            .field("args", &self.args)
            .field("handler", &(self.handler as *const ()))
            .finish()
    }
}

impl BuiltinFn {
    pub fn new(
        name: &str,
        ret: Type,
        args: Vec<Type>,
        handler: for<'ip> fn(env: &mut Env<'ip>, args: &[Value<'ip>]) -> Result<Value<'ip>>,
    ) -> BuiltinFn {
        BuiltinFn {
            name: Ident(String::from(name)),
            ret,
            args,
            handler,
        }
    }

    pub fn invoke<'ip>(&self, env: &mut Env<'ip>, args: &[Value<'ip>]) -> Result<Value<'ip>> {
        (self.handler)(env, args)
    }
}

#[derive(Default)]
pub struct Env<'ip> {
    sync: bool,
    first_iter: bool,
    value_stack: Vec<HashMap<String, Value<'ip>>>,
    ty_stack: Vec<HashMap<String, Type>>,
}

impl<'ip> Env<'ip> {
    pub fn set_sync(&mut self, sync: bool) {
        self.sync = sync;
    }

    pub fn set_first_iter(&mut self, first: bool) {
        self.first_iter = first;
    }

    pub fn is_first_iter(&self) -> bool {
        self.first_iter
    }

    pub fn push_scope(&mut self) {
        self.value_stack.push(HashMap::new());
        self.ty_stack.push(HashMap::new());
    }

    pub fn lookup_var(&mut self, var: &str) -> Option<&Value<'ip>> {
        let scope = self
            .value_stack
            .iter()
            .rev()
            .find(|scope| scope.contains_key(var));

        scope.and_then(|scope| scope.get(var))
    }

    pub fn insert_var(&mut self, name: &str, value: Value<'ip>) -> &Value<'ip> {
        let scope = self.value_stack.last_mut().unwrap();
        scope.insert(name.to_string(), value);
        scope.get(name).unwrap()
    }

    pub fn pop_scope(&mut self) {
        self.value_stack.pop();
        self.ty_stack.pop();
    }

    pub fn insert_ty(&mut self, name: &str, ty: Type) -> &Type {
        let scope = self.ty_stack.last_mut().unwrap();
        scope.insert(name.to_string(), ty);
        scope.get(name).unwrap()
    }
}

// TODO: This should instead probably be a type + some data. This will make it more complicated,
//       but allows types to be defined in terms of goose code in the future
#[derive(Clone, Debug)]
pub enum Value<'ip> {
    Null,
    Int(i128),
    Float(f64),
    Bit(bool),
    Char(char),
    String(String),
    Array(Vec<Value<'ip>>),
    Fn(&'ip FnDef),
    Builtin(BuiltinFn),
}

impl<'ip> Value<'ip> {
    pub fn ty(&self) -> Type {
        match self {
            Value::Null => Type::Null,
            Value::Int(_) => Type::Int,
            Value::Float(_) => Type::Float,
            Value::Bit(_) => Type::Bit,
            Value::Char(_) => Type::Char,
            Value::String(_) => Type::CharArray,
            Value::Array(arr) => {
                let inner = if let Some(val) = arr.first() {
                    val.ty()
                } else {
                    Type::Null
                };
                Type::Array(Box::new(inner))
            }
            Value::Fn(def) => Type::Fn(Box::new(def.ret_ty().clone()), def.arg_tys()),
            Value::Builtin(def) => Type::Fn(Box::new(def.ret.clone()), def.args.clone()),
        }
    }

    pub fn write<W: io::Write>(&self, w: &mut W) -> Result<()> {
        match self {
            Value::Null => write!(w, "null")?,
            Value::Int(i) => write!(w, "{}", i)?,
            Value::Float(f) => write!(w, "{}", f)?,
            Value::Bit(b) => {
                if *b {
                    write!(w, "1b")?
                } else {
                    write!(w, "0b")?
                }
            }
            Value::Char(c) => write!(w, "{}", c)?,
            Value::String(s) => write!(w, "{}", s)?,
            Value::Array(a) => {
                write!(w, "[")?;
                for (idx, item) in a.iter().enumerate() {
                    if idx != 0 {
                        write!(w, ", ")?;
                    }
                    item.write(w)?;
                }
                write!(w, "]")?;
            }
            Value::Fn(f) => write!(w, "<fn {}>", f.name())?,
            Value::Builtin(b) => write!(w, "<builtin {}>", &*b.name)?,
        };
        Ok(())
    }

    pub fn op_add(&self, _env: &mut Env<'ip>, val: Value<'ip>) -> Result<Value<'ip>> {
        match (self, val) {
            (&Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (&Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (a, b) => Err(Exception::InvalidOp(a.ty(), BinOp::Add, b.ty())),
        }
    }

    pub fn op_sub(&self, _env: &mut Env<'ip>, val: Value<'ip>) -> Result<Value<'ip>> {
        match (self, val) {
            (&Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (&Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (a, b) => Err(Exception::InvalidOp(a.ty(), BinOp::Add, b.ty())),
        }
    }

    pub fn op_mul(&self, _env: &mut Env<'ip>, val: Value<'ip>) -> Result<Value<'ip>> {
        match (self, val) {
            (&Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (&Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (a, b) => Err(Exception::InvalidOp(a.ty(), BinOp::Add, b.ty())),
        }
    }

    pub fn op_div(&self, _env: &mut Env<'ip>, val: Value<'ip>) -> Result<Value<'ip>> {
        match (self, val) {
            (&Value::Int(a), Value::Int(b)) => Ok(Value::Int(a / b)),
            (&Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
            (a, b) => Err(Exception::InvalidOp(a.ty(), BinOp::Add, b.ty())),
        }
    }
}

impl<'ip> PartialEq for Value<'ip> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(left), Value::Int(right)) => left == right,
            (Value::Float(left), Value::Float(right)) => left == right,
            (Value::Fn(left), Value::Fn(right)) => std::ptr::eq(*left, *right),
            _ => false,
        }
    }
}
