use super::*;
use crate::interp::{BuiltinFn, Env, Exception, Result, Value};

impl Assign {
    pub fn interpret<'ip>(&'ip self, env: &mut Env<'ip>) -> Result<Value<'ip>> {
        let val = self.val.interpret(env)?;

        let out = match self.assign_op {
            AssignOp::Eq => match self.ty {
                AssignTy::Unique => env.insert_var(&self.ident, val),
                AssignTy::CarryOver => {
                    if env.is_first_iter() {
                        env.insert_var(&self.ident, val)
                    } else {
                        &Value::Null
                    }
                }
                AssignTy::Default => {
                    if env.lookup_var(&self.ident).is_some() {
                        env.insert_var(&self.ident, val)
                    } else {
                        return Err(Exception::NameNotFound(self.ident.clone()));
                    }
                }
            },
            AssignOp::PlusEq => {
                let old_val = env
                    .lookup_var(&self.ident)
                    .ok_or_else(|| Exception::NameNotFound(self.ident.clone()))?
                    .clone();

                let new_val = old_val.op_add(env, val)?;

                env.insert_var(&self.ident, new_val)
            }
            AssignOp::SubEq => {
                let old_val = env
                    .lookup_var(&self.ident)
                    .ok_or_else(|| Exception::NameNotFound(self.ident.clone()))?
                    .clone();

                let new_val = old_val.op_sub(env, val)?;

                env.insert_var(&self.ident, new_val)
            }
            AssignOp::MulEq => {
                let old_val = env
                    .lookup_var(&self.ident)
                    .ok_or_else(|| Exception::NameNotFound(self.ident.clone()))?
                    .clone();

                let new_val = old_val.op_mul(env, val)?;

                env.insert_var(&self.ident, new_val)
            }
            AssignOp::DivEq => {
                let old_val = env
                    .lookup_var(&self.ident)
                    .ok_or_else(|| Exception::NameNotFound(self.ident.clone()))?
                    .clone();

                let new_val = old_val.op_div(env, val)?;

                env.insert_var(&self.ident, new_val)
            }
        };

        Ok(out.clone())
    }
}

impl Expr {
    pub fn interpret<'ip>(&'ip self, env: &mut Env<'ip>) -> Result<Value<'ip>> {
        match self {
            Expr::FnCall(call) => call.interpret(env),
            Expr::Literal(lit) => lit.interpret(env),
            Expr::Ident(i) => env
                .lookup_var(&i)
                .cloned()
                .ok_or_else(|| Exception::NameNotFound(i.clone())),
            Expr::BinOp(left, mid, right) => {
                let lval = left.interpret(env)?;
                let rval = right.interpret(env)?;

                match mid {
                    BinOp::Eq => Ok(Value::Bit(lval == rval)),
                    BinOp::Neq => Ok(Value::Bit(lval != rval)),
                    BinOp::Add => lval.op_add(env, rval),
                }
            }
        }
    }
}

impl File {
    pub fn interpret<'ip>(&'ip self, env: &mut Env<'ip>) -> Result<()> {
        // Push the global variables scope
        env.push_scope();
        // Push global functions
        env.insert_var(
            "print",
            Value::Builtin(BuiltinFn::new(Type::Null, vec![], |_env, args| {
                for (idx, arg) in args.iter().enumerate() {
                    if idx != 0 {
                        print!(" ");
                    }
                    print!("{:?}", arg);
                }
                println!();
                Ok(Value::Null)
            })),
        );

        for stmt in &self.stmts {
            stmt.interpret(env)?;
        }
        Ok(())
    }
}

impl FnCall {
    pub fn interpret<'ip>(&'ip self, env: &mut Env<'ip>) -> Result<Value<'ip>> {
        let val = env.lookup_var(&self.name).cloned();

        if let Some(Value::Fn(f)) = val {
            let args = self
                .args
                .iter()
                .map(|expr| expr.interpret(env))
                .collect::<Result<_>>()?;

            f.call(env, args)
        } else if let Some(Value::Builtin(b)) = val {
            let args = self
                .args
                .iter()
                .map(|expr| expr.interpret(env))
                .collect::<Result<Vec<_>>>()?;

            b.invoke(env, &args)
        } else {
            Err(Exception::NameNotFound(self.name.clone()))
        }
    }
}

impl FnDef {
    pub fn define<'ip>(&'ip self, env: &mut Env<'ip>) -> Result<()> {
        env.insert_var(&self.name, Value::Fn(self));
        Ok(())
    }

    pub fn call<'ip>(&'ip self, env: &mut Env<'ip>, args: Vec<Value<'ip>>) -> Result<Value<'ip>> {
        env.push_scope();

        if self.args.len() != args.len() {
            panic!("Invalid arg lengths")
        }

        for (arg, val) in self.args.iter().zip(args.iter()) {
            arg.ty.validate(env, val)?;
            env.insert_var(&arg.name, val.clone());
        }

        env.set_first_iter(true);
        loop {
            if self.stmts.is_empty() {
                if let Ok(Value::Bit(true)) = self.limit.interpret(env) {
                    return if self.ret == Type::Null {
                        Ok(Value::Null)
                    } else {
                        Err(Exception::InvalidType(self.ret.clone(), Type::Null))
                    };
                }
            }

            for stmt in &self.stmts {
                let val = stmt.interpret(env)?;

                if let Ok(Value::Bit(true)) = self.limit.interpret(env) {
                    env.pop_scope();
                    return if val.ty() != self.ret {
                        Err(Exception::InvalidType(self.ret.clone(), val.ty()))
                    } else {
                        Ok(val)
                    };
                }
            }
            env.set_first_iter(false);
        }
    }
}

impl Literal {
    pub fn interpret<'ip>(&'ip self, env: &mut Env<'ip>) -> Result<Value<'ip>> {
        match self {
            Literal::Int(i) => Ok(Value::Int(*i)),
            Literal::Float(f) => Ok(Value::Float(*f)),
            Literal::Char(c) => Ok(Value::Char(*c)),
            Literal::CharArray(s) => Ok(Value::String(s.clone())),
            Literal::Bit(b) => Ok(Value::Bit(*b)),
            Literal::Fn(f) => Ok(Value::Fn(f)),
            Literal::Array(a) => {
                let vals = a
                    .iter()
                    .map(|expr| expr.interpret(env))
                    .collect::<Result<Vec<_>>>()?;

                if let Some(val) = vals.first() {
                    let first_ty = val.ty();
                    for i in vals.iter().skip(1) {
                        if i.ty() != first_ty {
                            return Err(Exception::InvalidType(first_ty, i.ty()));
                        }
                    }
                }

                Ok(Value::Array(vals))
            }
        }
    }
}

impl Stmt {
    pub fn interpret<'ip>(&'ip self, env: &mut Env<'ip>) -> Result<Value<'ip>> {
        match self {
            Stmt::FnDef(def) => def.define(env).map(|_| Value::Null),
            Stmt::Assign(assign) => assign.interpret(env),
            Stmt::Sync(sync) => {
                env.set_sync(true);
                for stmt in sync {
                    stmt.interpret(env)?;
                }
                Ok(Value::Null)
            }
            Stmt::Once(once) => {
                if env.is_first_iter() {
                    for stmt in once {
                        stmt.interpret(env)?;
                    }
                }
                Ok(Value::Null)
            }
            Stmt::Expr(expr) => Ok(expr.interpret(env)?),
            Stmt::TypeDef(name, ty) => {
                env.insert_ty(name, ty.clone());
                Ok(Value::Null)
            }
        }
    }
}

impl Type {
    pub fn validate<'ip>(&'ip self, _env: &mut Env<'ip>, val: &Value<'ip>) -> Result<()> {
        if *self == val.ty() {
            Ok(())
        } else {
            Err(Exception::InvalidType(self.clone(), val.ty()))
        }
    }
}
