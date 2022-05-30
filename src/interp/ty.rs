use crate::ast::Type;
use super::{ValItem, Value, Fn, Exception, Result, Op};

unsafe impl<'ip> ValItem<'ip> for Type {
    fn allow_cast(ty: Type) -> Result<()> {
        if ty == Type::named("type") {
            Ok(())
        } else {
            Err(Exception::InvalidType(Type::named("type"), ty))
        }
    }

    fn clone(&self) -> Box<dyn ValItem<'ip>> {
        Box::new(Clone::clone(self))
    }

    fn ty(&self) -> Type {
        Type::named("type")
    }

    fn get_field(&self, _name: &str) -> Option<Value<'ip>> {
        // TODO: Support fields
        None
    }

    fn get_op(&self, op: Op) -> Option<Fn<'ip>> {
        todo!()
    }
}
