use crate::interp::Env;
use super::{ValItem, Value, Fn, Result, Exception, Type, Op};

pub struct Array<'ip>(pub(crate) Vec<Value<'ip>>);

unsafe impl<'ip> ValItem<'ip> for Array<'ip> {
    fn allow_cast(ty: Type) -> Result<()> {
        if let Type::Array(_) = ty {
            Ok(())
        } else {
            Err(Exception::InvalidType(Type::Array(Box::new(Type::named("any"))), ty))
        }
    }

    fn clone(&self) -> Box<dyn ValItem<'ip> + 'ip> {
        Box::new(Array(self.0.clone()))
    }

    fn ty(&self, env: &mut Env<'_>) -> Type {
        let inner = self.0.get(0)
            .map_or_else(|| Type::named("null"), |v| v.ty());

        Type::Array(Box::new(inner))
    }

    fn get_field(&self, _name: &str) -> Option<Value<'ip>> {
        None
    }

    fn get_op(&self, _op: Op) -> Option<Fn<'ip>> {
        todo!()
    }
}
