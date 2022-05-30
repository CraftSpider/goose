use std::io;
use std::io::Write;
use std::ops::Deref;
use crate::ast::{BinOp, Type};
use crate::interp::Bit;
use super::{ValItem, Value, Fn, Result, BuiltinFn, Exception};

#[derive(Clone)]
pub struct Int(i128);

impl Int {
    pub fn new(val: i128) -> Int {
        Int(val)
    }

    pub fn val(&self) -> i128 {
        self.0
    }
}

unsafe impl<'ip> ValItem<'ip> for Int {
    fn allow_cast(ty: Type) -> Result<()> {
        if ty == Type::named("int") {
            Ok(())
        } else {
            Err(Exception::InvalidType(Type::named("int"), ty))
        }
    }

    fn clone(&self) -> Box<dyn ValItem<'ip>> {
        Box::new(Int(self.0))
    }

    fn ty(&self) -> Type {
        Type::named("int")
    }

    fn write(&self, w: &mut dyn Write) -> io::Result<()> {
        write!(w, "{}", self.0)
    }

    fn get_field(&self, _: &str) -> Option<Value<'ip>> {
        None
    }

    fn get_op(&self, op: BinOp) -> Option<Fn<'ip>> {
        match op {
            BinOp::Eq => Some(BuiltinFn::new(
                "int_eq",
                Type::named("bit"),
                vec![Type::named("int"), Type::named("int")],
                |_env, args| {
                    if args.len() != 2 {
                        panic!("{:?}", args);
                    }

                    let a = args[0].downcast::<Int>()?;
                    let b = args[1].downcast::<Int>()?;

                    Ok(Value::new(Bit::new(a.0 == b.0)))
                }
            ).into()),
            BinOp::Neq => Some(BuiltinFn::new(
                "int_eq",
                Type::named("bit"),
                vec![Type::named("int"), Type::named("int")],
                |_env, args| {
                    if args.len() != 2 {
                        panic!("{:?}", args);
                    }

                    let a = args[0].downcast::<Int>()?;
                    let b = args[1].downcast::<Int>()?;

                    Ok(Value::new(Bit::new(a.0 != b.0)))
                }
            ).into()),
            BinOp::Add => Some(BuiltinFn::new(
                "int_add",
                Type::named("int"),
                vec![Type::named("int"), Type::named("int")],
                |_env, args| {
                    if args.len() != 2 {
                        panic!("{:?}", args);
                    }

                    let a = args[0].downcast::<Int>()?;
                    let b = args[1].downcast::<Int>()?;

                    Ok(Value::new(Int(a.0 + b.0)))
                }
            ).into()),
            _ => None,
        }
    }
}

impl Deref for Int {
    type Target = i128;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
