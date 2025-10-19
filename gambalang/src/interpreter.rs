use crate::ast::*;
use crate::env::{Env, FuncImpl, Lambda, Runtime, Value};
use crate::error::GambaError;

pub fn eval_program(rt: &mut Runtime, p: Program) -> Result<Value, GambaError> {
    let mut last = Value::Unit;
    for e in p.items {
        last = eval_expr(&rt.env, e)?;
    }
    Ok(last)
}

// avalia um Program diretamente em um ambiente existente (util para builtin eval)
pub fn eval_program_in_env(env: &Env, p: Program) -> Result<Value, GambaError> {
    let mut last = Value::Unit;
    for e in p.items {
        last = eval_expr(env, e)?;
    }
    Ok(last)
}

fn truthy(v: &Value) -> Result<bool, GambaError> {
    match v {
        Value::Bool(b) => Ok(*b),
        _ => Err(GambaError::runtime("when: condição deve ser bool")),
    }
}

// suporte a tail-call: resultado de avaliar algo em posição de cauda
enum TailAction {
    Return(Value),
    TailCall { func: Value, args: Vec<Value> },
}

// avalia uma expressao em posição de cauda, permitindo trampolin quando for chamada
fn eval_tail(env: &Env, e: Expr) -> Result<TailAction, GambaError> {
    match e {
        // desagrupa (when ( ... ) { ... } etc.)
        Expr::Group(inner) => eval_tail(env, *inner),
        // bloco em cauda: cria escopo filho como de costume e mantem cauda no último item
        Expr::Block(b) => {
            let child = env.child();
            let mut items = b.items;
            if items.is_empty() {
                return Ok(TailAction::Return(Value::Unit));
            }
            let last = items.pop().unwrap();
            for it in items {
                eval_expr(&child, it)?;
            }
            eval_tail(&child, last)
        }
        // when em cauda: a cauda fica dentro do ramo escolhido
        Expr::When {
            cond,
            then_branch,
            else_branch,
        } => {
            let c = eval_expr(env, *cond)?;
            if truthy(&c)? {
                eval_tail(env, Expr::Block(then_branch))
            } else {
                eval_tail(env, Expr::Block(else_branch))
            }
        }
        // chamada em cauda: nao executa agora; retorna TailCall para o trampolim do call
        Expr::Call { func, args } => {
            let fval = eval_expr(env, *func)?;
            let mut evaled_args = Vec::with_capacity(args.len());
            for a in args {
                evaled_args.push(eval_expr(env, a)?);
            }
            Ok(TailAction::TailCall {
                func: fval,
                args: evaled_args,
            })
        }
        // qualquer outra coisa: avalia normalmente e retorna
        other => Ok(TailAction::Return(eval_expr(env, other)?)),
    }
}

fn eval_block(env: &Env, b: Block) -> Result<Value, GambaError> {
    let child = env.child();
    let mut last = Value::Unit;
    for e in b.items {
        last = eval_expr(&child, e)?;
    }
    Ok(last)
}

pub fn eval_expr(env: &Env, e: Expr) -> Result<Value, GambaError> {
    match e {
        Expr::Number(n) => Ok(Value::Number(n)),
        Expr::String(s) => Ok(Value::String(s)),
        Expr::Bool(b) => Ok(Value::Bool(b)),
        Expr::List(items) => {
            let mut vs = Vec::with_capacity(items.len());
            for it in items {
                vs.push(eval_expr(env, it)?);
            }
            Ok(Value::List(vs))
        }
        Expr::Ident(name) => env
            .get(&name)
            .ok_or_else(|| GambaError::runtime(format!("nome não encontrado: {}", name))),
        Expr::Let { name, expr } => {
            let v = eval_expr(env, *expr)?;
            env.set(name, v.clone())?;
            Ok(v)
        }
        Expr::When {
            cond,
            then_branch,
            else_branch,
        } => {
            let c = eval_expr(env, *cond)?;
            if truthy(&c)? {
                eval_block(env, then_branch)
            } else {
                eval_block(env, else_branch)
            }
        }
        Expr::Lambda { params, body } => Ok(Value::Func(FuncImpl::Lambda(Lambda {
            params,
            body,
            env: env.clone(),
        }))),
        Expr::Call { func, args } => {
            let fval = eval_expr(env, *func)?;
            let mut arg_vals = Vec::with_capacity(args.len());
            for a in args {
                arg_vals.push(eval_expr(env, a)?);
            }
            call(env, fval, arg_vals)
        }
        Expr::Group(inner) => eval_expr(env, *inner),
        Expr::Block(b) => eval_block(env, b),
        Expr::Binary { op, left, right } => {
            let l = eval_expr(env, *left)?;
            let r = eval_expr(env, *right)?;
            match (op, l, r) {
                (BinaryOp::Add, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                // concatenação de strings com +
                (BinaryOp::Add, Value::String(a), v) => Ok(Value::String(format!("{}{}", a, v))),
                (BinaryOp::Add, v, Value::String(b)) => Ok(Value::String(format!("{}{}", v, b))),
                (BinaryOp::Sub, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
                (BinaryOp::Mul, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
                (BinaryOp::Div, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a / b)),
                (BinaryOp::Mod, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a % b)),
                _ => Err(GambaError::runtime(
                    "operação binária inválida para tipos fornecidos",
                )),
            }
        }
        Expr::UnaryMinus(inner) => {
            let v = eval_expr(env, *inner)?;
            match v {
                Value::Number(n) => Ok(Value::Number(-n)),
                _ => Err(GambaError::runtime("operador - unário espera número")),
            }
        }
    }
}

fn call(env: &Env, fval: Value, args: Vec<Value>) -> Result<Value, GambaError> {
    let mut current_func = fval;
    let mut current_args = args;

    loop {
        match current_func {
            Value::Func(FuncImpl::Builtin(f)) => {
                // builtin: executa e retorna (nenhum crescimento de pilha)
                return f(env, &current_args);
            }
            Value::Func(FuncImpl::Lambda(l)) => {
                if l.params.len() != current_args.len() {
                    return Err(GambaError::runtime(format!(
                        "função esperava {} args, recebeu {}",
                        l.params.len(),
                        current_args.len()
                    )));
                }
                let frame = l.env.child();
                for (p, v) in l.params.iter().zip(current_args.iter()) {
                    frame.set(p.clone(), v.clone())?;
                }

                // avalia o corpo: tudo menos o último de forma normal
                let mut items = l.body.items.clone();
                if items.is_empty() {
                    return Ok(Value::Unit);
                }
                let last = items.pop().unwrap();
                for e in items {
                    eval_expr(&frame, e)?;
                }

                // última expressão em posição de cauda: pode devolver TailCall para trampolim
                match eval_tail(&frame, last)? {
                    TailAction::Return(v) => return Ok(v),
                    TailAction::TailCall { func, args } => {
                        // troca alvo e args e itera, sem crescer a pilha
                        current_func = func;
                        current_args = args;
                        continue;
                    }
                }
            }
            other => {
                return Err(GambaError::runtime(format!(
                    "tentativa de chamar um valor não-função (tipo {} = {}). \
Dica: o lado direito de |> deve ser uma função (nome, chamada ou lambda). \
Exemplos válidos: xs |> map(fn x {{ ... }}),  x |> f(a),  x |> (fn v {{ ... }}).",
                    other.type_name(),
                    other
                )));
            }
        }
    }
}

pub fn call_value(env: &Env, fval: Value, args: Vec<Value>) -> Result<Value, GambaError> {
    call(env, fval, args)
}
