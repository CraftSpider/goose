use super::*;
use crate::interp::{BuiltinFn, Env, Exception, Result, Value};
use std::os::unix::io::{FromRawFd, RawFd};
use std::{fs, io, mem};

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
                    if let Some(old_val) = env.lookup_var(&self.ident) {
                        if old_val.ty() != val.ty() {
                            return Err(Exception::InvalidType(old_val.ty(), val.ty()));
                        }
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
            Expr::Write(ty, args) => {
                let (w, expr) = match ty {
                    WriteTy::Console => ("write_console", None),
                    WriteTy::Error => ("write_error", None),
                    WriteTy::RawFile => ("write_raw_file", None),
                    WriteTy::Other(expr) => ("write_io", Some(expr)),
                };

                let f = env
                    .lookup_var(w)
                    .ok_or_else(|| Exception::NameNotFound(Ident(w.to_string())))?
                    .clone();

                if let Value::Fn(f) = f {
                    let mut args = args
                        .iter()
                        .map(|val| val.interpret(env))
                        .collect::<Result<Vec<_>>>()?;

                    if let Some(expr) = expr {
                        args.insert(0, expr.interpret(env)?);
                    }

                    f.invoke(env, args)?;
                } else {
                    return Err(Exception::InvalidType(
                        Type::Fn(Box::new(Type::Null), vec![]),
                        f.ty(),
                    ));
                };

                Ok(Value::Null)
            }
            Expr::Literal(lit) => lit.interpret(env),
            Expr::Ident(i) => env
                .lookup_var(i)
                .cloned()
                .ok_or_else(|| Exception::NameNotFound(i.clone())),
            Expr::BinOp(left, mid, right) => {
                let lval = left.interpret(env)?;
                let rval = right.interpret(env)?;

                match mid {
                    BinOp::Eq => Ok(Value::Bit(lval == rval)),
                    BinOp::Neq => Ok(Value::Bit(lval != rval)),
                    BinOp::Add => lval.op_add(env, rval),
                    BinOp::Sub => lval.op_sub(env, rval),
                    BinOp::Mul => lval.op_mul(env, rval),
                    BinOp::Div => lval.op_div(env, rval),
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
            "write_console",
            Value::Fn(BuiltinFn::new(
                "write_console",
                Type::Null,
                vec![],
                |_env, args| {
                    let mut w = io::stdout();
                    for arg in args {
                        arg.write(&mut w)?;
                    }
                    Ok(Value::Null)
                },
            ).into()),
        );
        env.insert_var(
            "write_error",
            Value::Fn(BuiltinFn::new(
                "write_error",
                Type::Null,
                vec![],
                |_env, args| {
                    let mut w = io::stderr();
                    for arg in args {
                        arg.write(&mut w)?;
                    }
                    Ok(Value::Null)
                },
            ).into()),
        );
        env.insert_var(
            "write_raw_file",
            Value::Fn(BuiltinFn::new(
                "write_raw_file",
                Type::Null,
                vec![],
                |_env, args| {
                    let mut w = fs::File::options()
                        .write(true)
                        .create(true)
                        .append(true)
                        .open("honk")?;
                    for arg in args {
                        arg.write(&mut w)?;
                    }
                    Ok(Value::Null)
                },
            ).into()),
        );
        env.insert_var(
            "write_io",
            Value::Fn(BuiltinFn::new(
                "write_io",
                Type::Null,
                vec![],
                |_env, args| {
                    let (mut file, is_raw) = match &args[0] {
                        Value::Int(i) => (unsafe { fs::File::from_raw_fd(*i as RawFd) }, true),
                        Value::String(s) => (
                            fs::File::options()
                                .create(true)
                                .write(true)
                                .append(true)
                                .open(s)?,
                            false,
                        ),
                        _ => return Err(Exception::InvalidType(Type::CharArray, args[0].ty())),
                    };

                    for arg in &args[1..] {
                        arg.write(&mut file)?;
                    }

                    if is_raw {
                        mem::forget(file);
                    }

                    Ok(Value::Null)
                },
            ).into()),
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

            f.invoke(env, args)
        } else {
            Err(Exception::NameNotFound(self.name.clone()))
        }
    }
}

impl FnDef {
    pub fn define<'ip>(&'ip self, env: &mut Env<'ip>) -> Result<()> {
        env.insert_var(&self.name, Value::Fn(self.into()));
        Ok(())
    }

    pub fn invoke<'ip>(&'ip self, env: &mut Env<'ip>, args: Vec<Value<'ip>>) -> Result<Value<'ip>> {
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
            Literal::CharArray(s) => Ok(Value::String(s[1..s.len() - 1].to_string())),
            Literal::Bit(b) => Ok(Value::Bit(*b)),
            Literal::Fn(f) => Ok(Value::Fn(f.into())),
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
