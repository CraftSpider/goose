use crate::ast::Type;
use super::{ValItem, Value, Fn, Result, Exception, Op};

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

    fn ty(&self) -> Type {
        Type::named("null")
    }

    fn get_field(&self, _name: &str) -> Option<Value<'ip>> {
        None
    }

    fn get_op(&self, op: Op) -> Option<Fn<'ip>> {
        todo!()
    }
}
