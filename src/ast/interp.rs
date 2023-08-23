use super::*;
use crate::interp::{
    BuiltinFn, Env, Exception, Result, Value, Int, CharArray, Fn, Bit, Float, Char, Array, Op, Type
};

use std::os::unix::io::{FromRawFd, RawFd};
use std::{fs, io, mem};

impl Assign {
    pub fn interpret<'ip>(&'ip self, env: &mut Env<'ip>) -> Result<Value<'ip>> {
        let null = Value::null();
        let val = self.val.interpret(env)?;

        let out = match self.assign_op {
            AssignOp::Eq => match self.ty {
                AssignTy::Unique => env.insert_var(&self.ident, val),
                AssignTy::CarryOver => {
                    if env.is_first_iter() {
                        env.insert_var(&self.ident, val)
                    } else {
                        &null
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
            AssignOp::PlusEq | AssignOp::SubEq | AssignOp::MulEq | AssignOp::DivEq => {
                let op = match self.assign_op {
                    AssignOp::PlusEq => Op::Add,
                    AssignOp::SubEq => Op::Sub,
                    AssignOp::MulEq => Op::Mul,
                    AssignOp::DivEq => Op::Div,
                    _ => unreachable!(),
                };

                let old_val = env
                    .lookup_var(&self.ident)
                    .ok_or_else(|| Exception::NameNotFound(self.ident.clone()))?
                    .clone();

                let new_val = old_val
                    .get_op(op)
                    .ok_or_else(|| Exception::InvalidOp(op, old_val.ty(), Some(val.ty())))?
                    .invoke(env, vec![old_val, val])?;

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
                    WriteTy::RawFile => ("write_honk", None),
                    WriteTy::Other(expr) => ("write_io", Some(expr)),
                };

                let f = env
                    .lookup_var(w)
                    .ok_or_else(|| Exception::NameNotFound(Ident(w.to_string())))?
                    .clone();

                let f = f.downcast::<Fn<'_>>()?;

                let mut args = args
                    .iter()
                    .map(|val| val.interpret(env))
                    .collect::<Result<Vec<_>>>()?;

                if let Some(expr) = expr {
                    args.insert(0, expr.interpret(env)?);
                }

                f.invoke(env, args)?;

                Ok(Value::null())
            }
            Expr::Literal(lit) => lit.interpret(env),
            Expr::Ident(i) => env
                .lookup_var(i)
                .cloned()
                .ok_or_else(|| Exception::NameNotFound(i.clone())),
            &Expr::UnOp(op, ref expr) => {
                let op = op.into();
                let rval = expr.interpret(env)?;


                rval.get_op(op)
                    .ok_or_else(|| Exception::InvalidOp(op, rval.ty(), None))?
                    .invoke(env, vec![rval])
            }
            &Expr::BinOp(ref left, op, ref right) => {
                let op = op.into();
                let lval = left.interpret(env)?;
                let rval = right.interpret(env)?;

                lval.get_op(op)
                    .ok_or_else(|| Exception::InvalidOp(op, lval.ty(), Some(rval.ty())))?
                    .invoke(env, vec![lval, rval])
            }
        }
    }
}

impl File {
    pub fn interpret<'ip>(&'ip self, env: &mut Env<'ip>) -> Result<()> {
        // Push the global variables scope
        env.push_scope();
        // Push global variables
        env.insert_var("int", Value::new(Type::named("int")));
        // Push global functions
        env.insert_var(
            "write_console",
            Value::new::<Fn<'_>>(BuiltinFn::new(
                "write_console",
                Type::named("null"),
                vec![],
                |_env, args| {
                    let mut w = io::stdout();
                    for arg in args {
                        arg.write(&mut w)?;
                    }
                    Ok(Value::null())
                },
            ).into()),
        );
        env.insert_var(
            "write_error",
            Value::new::<Fn<'_>>(BuiltinFn::new(
                "write_error",
                Type::named("null"),
                vec![],
                |_env, args| {
                    let mut w = io::stderr();
                    for arg in args {
                        arg.write(&mut w)?;
                    }
                    Ok(Value::null())
                },
            ).into()),
        );
        env.insert_var(
            "write_honk",
            Value::new::<Fn<'_>>(BuiltinFn::new(
                "write_honk",
                Type::named("null"),
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
                    Ok(Value::null())
                },
            ).into()),
        );
        env.insert_var(
            "write_io",
            Value::new::<Fn<'_>>(BuiltinFn::new(
                "write_io",
                Type::named("null"),
                vec![],
                |_env, args: &[Value<'_>]| {
                    let (mut file, is_raw) = if let Ok(i) = args[0].downcast::<Int>() {
                        (unsafe { fs::File::from_raw_fd(**i as RawFd) }, true)
                    } else if let Ok(s) = args[0].downcast::<CharArray>() {
                        (
                            fs::File::options()
                                .create(true)
                                .write(true)
                                .append(true)
                                .open(&**s)?,
                            false,
                        )
                    } else {
                        return Err(Exception::InvalidType(Type::named("chararray"), args[0].ty()));
                    };

                    for arg in &args[1..] {
                        arg.write(&mut file)?;
                    }

                    if is_raw {
                        mem::forget(file);
                    }

                    Ok(Value::null())
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
        let val = env.lookup_var(&self.name).cloned()
            .ok_or_else(|| Exception::NameNotFound(self.name.clone()))?;

        let f = val.downcast::<Fn<'_>>()?;

        let args = self
            .args
            .iter()
            .map(|expr| expr.interpret(env))
            .collect::<Result<_>>()?;

        f.invoke(env, args)
    }
}

impl FnDef {
    pub fn define<'ip>(&'ip self, env: &mut Env<'ip>) -> Result<()> {
        env.insert_var(&self.name, Value::new::<Fn<'_>>(self.into()));
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
                let lim = self.limit.interpret(env)
                    .map(|v| v.downcast::<Bit>().map(Bit::val).unwrap_or(false) == true)
                    .unwrap_or(false);

                if lim {
                    return if self.ret == Type::named("null") {
                        Ok(Value::null())
                    } else {
                        Err(Exception::InvalidType(self.ret.clone(), Type::named("null")))
                    };
                }
            }

            for stmt in &self.stmts {
                let val = stmt.interpret(env)?;
                let lim = self.limit.interpret(env)
                    .map(|v| v.downcast::<Bit>().map(Bit::val).unwrap_or(false) == true)
                    .unwrap_or(false);

                if lim {
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
            Literal::Int(i) => Ok(Value::new(Int::new(*i))),
            Literal::Float(f) => Ok(Value::new(Float::new(*f))),
            Literal::Char(c) => Ok(Value::new(Char::new(*c))),
            Literal::CharArray(s) => Ok(Value::new(CharArray::new(s[1..s.len() - 1].to_string()))),
            Literal::Bit(b) => Ok(Value::new(Bit::new(*b))),
            Literal::Fn(f) => Ok(Value::new::<Fn<'_>>(f.into())),
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

                Ok(Value::new(Array(vals)))
            }
        }
    }
}

impl Stmt {
    pub fn interpret<'ip>(&'ip self, env: &mut Env<'ip>) -> Result<Value<'ip>> {
        match self {
            Stmt::FnDef(def) => def.define(env).map(|_| Value::null()),
            Stmt::Assign(assign) => assign.interpret(env),
            Stmt::Sync(sync) => {
                env.set_sync(true);
                for stmt in sync {
                    stmt.interpret(env)?;
                }
                Ok(Value::null())
            }
            Stmt::Once(once) => {
                if env.is_first_iter() {
                    for stmt in once {
                        stmt.interpret(env)?;
                    }
                }
                Ok(Value::null())
            }
            Stmt::Expr(expr) => Ok(expr.interpret(env)?),
            Stmt::TypeDef(name, ty) => {
                env.insert_var(name, Value::new(ty.clone()));
                Ok(Value::null())
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
