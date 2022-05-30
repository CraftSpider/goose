use std::ops::Deref;
use crate::ast::{BinOp, Type};
use crate::interp::Exception;
use super::{ValItem, Value, Fn, Result};

pub struct CharArray(pub(crate) String);

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

    fn get_op(&self, op: BinOp) -> Option<Fn<'ip>> {
        todo!()
    }
}

impl Deref for CharArray {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
