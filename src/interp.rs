use core::fmt;
use std::collections::HashMap;
use std::io;
use std::io::Error;
use std::ptr::NonNull;

mod array;
mod null;
mod func;
mod int;
mod ty;
mod bit;
mod float;
mod char;
mod char_array;

pub use array::Array;
pub use int::Int;
pub use char_array::CharArray;
pub use bit::Bit;
pub use func::Fn;
pub use self::char::Char;
pub use float::Float;

use crate::ast::{BinOp, Ident, Type};

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

#[derive(Debug, Default)]
pub struct Env<'ip> {
    sync: bool,
    first_iter: bool,
    value_stack: Vec<HashMap<String, Value<'ip>>>,
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
    }
}

pub unsafe trait ValItem<'ip>: 'ip {
    fn allow_cast(ty: Type) -> Result<()>
    where
        Self: Sized;

    fn clone(&self) -> Box<dyn ValItem<'ip> + 'ip>;
    fn ty(&self) -> Type;
    fn write(&self, w: &mut dyn io::Write) -> io::Result<()> {
        #![allow(unused_variables)]
        Ok(())
    }
    fn get_field(&self, name: &str) -> Option<Value<'ip>>;
    fn get_op(&self, op: BinOp) -> Option<Fn<'ip>>;
}

impl<'ip> dyn ValItem<'ip> {
    fn downcast<T: ValItem<'ip>>(&self) -> Result<&T> {
        T::allow_cast(self.ty())
            .map(|_| {
                let ptr = NonNull::from(self)
                    .cast::<T>();

                unsafe { ptr.as_ref() }
            })
    }
}

pub struct Value<'ip> {
    data: Box<dyn ValItem<'ip> + 'ip>,
}

impl Clone for Value<'_> {
    fn clone(&self) -> Self {
        Value { data: self.data.clone() }
    }
}

impl fmt::Debug for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = Vec::<u8>::new();
        self.data.write(&mut s)
            .unwrap();
        let s = String::from_utf8_lossy(&s);

        f.debug_struct("Value")
            .field("data", &s)
            .finish()
    }
}

impl<'ip> Value<'ip> {
    pub(crate) fn new<T: ValItem<'ip> + 'ip>(item: T) -> Value<'ip> {
        Value { data: Box::new(item) }
    }

    pub fn null() -> Value<'ip> {
        Self::new(null::Null)
    }

    pub fn downcast<T: ValItem<'ip> + 'ip>(&self) -> Result<&T> {
        self.data.downcast()
    }

    pub fn ty(&self) -> Type {
        self.data.ty()
    }

    pub fn write<W: io::Write>(&self, w: &mut W) -> Result<()> {
        self.data.write(w)?;
        /*match self {
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
            Value::Type(t) => write!(w, "<type {}>", t.pretty())?,
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
        };*/
        Ok(())
    }

    pub fn get_op(&self, op: BinOp) -> Option<Fn<'ip>> {
        // TODO: Handle fallback to inverting Eq/Neq
        self.data.get_op(op)
    }
}
