// use crate::ast::Type;
use super::{ValItem, Value, Fn, Exception, Result, Op, Env};
use once_cell::sync::OnceCell;

#[derive(Clone, Debug)]
pub struct Type {
    id: u64,
    name: String,
}

impl Type {
    pub(super) fn new(id: u64, name: String) -> Type {
        Type {
            id,
            name,
        }
    }

    pub fn new_type<'ip>(env: &mut Env<'ip>, name: &str) -> Type {
        env.new_ty(name)
    }
}

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

    fn ty(&self, env: &mut Env<'_>) -> Type {
        static TY: OnceCell<Type> = OnceCell::new();

        Clone::clone(TY.get_or_init(|| env.new_ty("int")))
    }

    fn get_field(&self, _name: &str) -> Option<Value<'ip>> {
        // TODO: Support fields
        None
    }

    fn get_op(&self, op: Op) -> Option<Fn<'ip>> {
        todo!()
    }
}
