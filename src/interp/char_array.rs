use std::ops::Deref;
use crate::ast::Type;
use super::{ValItem, Op, Exception, Value, Fn, Result};

pub struct CharArray(String);

impl CharArray {
    pub fn new(str: String) -> CharArray {
        CharArray(str)
    }
}

unsafe impl<'ip> ValItem<'ip> for CharArray {
    fn allow_cast(ty: Type) -> Result<()> {
        if ty == Type::named("chararray") {
            Ok(())
        } else {
            Err(Exception::InvalidType(Type::named("chararray"), ty))
        }
    }

    fn clone(&self) -> Box<dyn ValItem<'ip> + 'ip> {
        Box::new(CharArray(self.0.clone()))
    }

    fn ty(&self) -> Type {
        Type::named("chararray")
    }

    fn get_field(&self, _name: &str) -> Option<Value<'ip>> {
        None
    }

    fn get_op(&self, op: Op) -> Option<Fn<'ip>> {
        todo!()
    }
}

impl Deref for CharArray {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
