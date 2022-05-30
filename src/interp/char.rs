use crate::ast::{BinOp, Type};
use crate::interp::Exception;
use super::{ValItem, Value, Fn, Result};

pub struct Char(pub(crate) char);

unsafe impl<'ip> ValItem<'ip> for Char {
    fn allow_cast(ty: Type) -> Result<()> {
        if ty == Type::named("char") {
            Ok(())
        } else {
            Err(Exception::InvalidType(Type::named("char"), ty))
        }
    }

    fn clone(&self) -> Box<dyn ValItem<'ip> + 'ip> {
        Box::new(Char(self.0))
    }

    fn ty(&self) -> Type {
        Type::named("char")
    }

    fn get_field(&self, _name: &str) -> Option<Value<'ip>> {
        None
    }

    fn get_op(&self, op: BinOp) -> Option<Fn<'ip>> {
        todo!()
    }
}
