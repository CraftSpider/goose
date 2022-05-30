use crate::ast::Type;
use super::{ValItem, Value, Fn, Result, Op, Exception};

pub struct Char(char);

impl Char {
    pub fn new(c: char) -> Char {
        Char(c)
    }
}

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

    fn get_op(&self, op: Op) -> Option<Fn<'ip>> {
        todo!()
    }
}
