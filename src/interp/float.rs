use crate::interp::Env;
use super::{ValItem, Value, Fn, Op, Exception, Result, Type};

pub struct Float(f64);

impl Float {
    pub fn new(f: f64) -> Float {
        Float(f)
    }

    pub fn val(&self) -> f64 {
        self.0
    }
}

unsafe impl<'ip> ValItem<'ip> for Float {
    fn allow_cast(ty: Type) -> Result<()> {
        if ty == Type::named("float") {
            Ok(())
        } else {
            Err(Exception::InvalidType(Type::named("float"), ty))
        }
    }

    fn clone(&self) -> Box<dyn ValItem<'ip> + 'ip> {
        Box::new(Float(self.0))
    }

    fn ty(&self, env: &mut Env<'_>) -> Type {
        Type::named("float")
    }

    fn get_field(&self, _name: &str) -> Option<Value<'ip>> {
        None
    }

    fn get_op(&self, op: Op) -> Option<Fn<'ip>> {
        todo!()
    }
}
