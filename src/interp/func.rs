use crate::ast::{BinOp, FnDef, Type};
use super::{Value, ValItem, BuiltinFn, Env, Result, Exception};

#[derive(Clone, Debug)]
pub enum Fn<'ip> {
    User(&'ip FnDef),
    Builtin(BuiltinFn),
}

impl<'ip> Fn<'ip> {
    pub fn name(&self) -> &str {
        match self {
            Fn::User(fd) => fd.name(),
            Fn::Builtin(b) => &*b.name,
        }
    }

    pub fn ret_ty(&self) -> &Type {
        match self {
            Fn::User(fd) => fd.ret_ty(),
            Fn::Builtin(b) => &b.ret,
        }
    }

    pub fn arg_tys(&self) -> Vec<Type> {
        match self {
            Fn::User(fd) => fd.arg_tys(),
            Fn::Builtin(b) => b.args.clone(),
        }
    }

    pub fn invoke(&self, env: &mut Env<'ip>, args: Vec<Value<'ip>>) -> Result<Value<'ip>> {
        match self {
            Fn::User(fd) => fd.invoke(env, args),
            Fn::Builtin(b) => b.invoke(env, &args),
        }
    }
}

unsafe impl<'ip> ValItem<'ip> for Fn<'ip> {
    fn allow_cast(ty: Type) -> Result<()> {
        if let Type::Fn(_, _) = ty {
            Ok(())
        } else {
            Err(Exception::InvalidType(Type::Fn(Box::new(Type::named("any")), vec![]), ty))
        }
    }

    fn clone(&self) -> Box<dyn ValItem<'ip>> {
        Box::new(Clone::clone(self))
    }

    fn ty(&self) -> Type {
        Type::Fn(Box::new(Clone::clone(self.ret_ty())), self.arg_tys())
    }

    fn get_field(&self, _name: &str) -> Option<Value<'ip>> {
        // TODO: Maybe functions have name/type fields?
        None
    }

    fn get_op(&self, op: BinOp) -> Option<Fn<'ip>> {
        todo!()
    }
}

impl<'ip> From<&'ip FnDef> for Fn<'ip> {
    fn from(fd: &'ip FnDef) -> Self {
        Fn::User(fd)
    }
}

impl From<BuiltinFn> for Fn<'_> {
    fn from(b: BuiltinFn) -> Self {
        Fn::Builtin(b)
    }
}
