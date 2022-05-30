use crate::ast::{BinOp, Type};
use crate::interp::{BuiltinFn, Exception};
use super::{Value, ValItem, Fn, Result};

pub struct Bit(bool);

impl Bit {
    pub fn new(b: bool) -> Bit {
        Bit(b)
    }

    pub fn val(&self) -> bool {
        self.0
    }
}

unsafe impl<'ip> ValItem<'ip> for Bit {
    fn allow_cast(ty: Type) -> Result<()> {
        if ty == Type::named("bit") {
            Ok(())
        } else {
            Err(Exception::InvalidType(Type::named("bit"), ty))
        }
    }

    fn clone(&self) -> Box<dyn ValItem<'ip> + 'ip> {
        Box::new(Bit(self.0))
    }

    fn ty(&self) -> Type {
        Type::named("bit")
    }

    fn get_field(&self, _name: &str) -> Option<Value<'ip>> {
        None
    }

    fn get_op(&self, op: BinOp) -> Option<Fn<'ip>> {
        match op {
            BinOp::Eq => Some(BuiltinFn::new(
                "bit_eq",
                Type::named("bit"),
                vec![Type::named("bit"), Type::named("bit")],
                |_env, args| {
                    if args.len() != 2 {
                        panic!("{:?}", args);
                    }
                    let a = args[0].downcast::<Bit>()?;
                    let b = args[0].downcast::<Bit>()?;

                    Ok(Value::new(Bit(a.0 == b.0)))
                }
            ).into()),
            _ => None,
        }
    }
}
