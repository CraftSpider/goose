use crate::interp::Env;
use super::{ValItem, Value, Fn, Result, Exception, Op, Type};

pub struct Null;

unsafe impl<'ip> ValItem<'ip> for Null {
    fn allow_cast(ty: Type) -> Result<()> {
        if ty == Type::named("null") {
            Ok(())
        } else {
            Err(Exception::InvalidType(Type::named("null"), ty))
        }
    }

    fn clone(&self) -> Box<dyn ValItem<'ip>> {
        Box::new(Null)
    }

    fn ty(&self, env: &mut Env<'_>) -> Type {
        Type::named("null")
    }

    fn get_field(&self, _name: &str) -> Option<Value<'ip>> {
        None
    }

    fn get_op(&self, op: Op) -> Option<Fn<'ip>> {
        todo!()
    }
}
