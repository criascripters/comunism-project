use crate::ast::*;
use crate::env::{Env, FuncImpl, Lambda, Runtime, Value};
use crate::error::{GambaError, Suggestion};

pub fn eval_program(rt: &mut Runtime, p: Program) -> Result<Value, GambaError> {
    let mut last = Value::Unit;
    for e in p.items {
        last = eval_expr(&rt.env, e)?;
    }
    Ok(last)
}

// avalia um Program diretamente em um ambiente existente (util pra builtin eval)
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
    TailCall {
        func: Value,
        args: Vec<Value>,
        style: Option<CallStyle>,
    },
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
        // chamada em cauda: nao executa agora; retorna TailCall pra o trampolim do call
        Expr::Call {
            func, args, style, ..
        } => {
            let fval = eval_expr(env, *func)?;
            let mut evaled_args = Vec::with_capacity(args.len());
            for a in args {
                evaled_args.push(eval_expr(env, a)?);
            }
            Ok(TailAction::TailCall {
                func: fval,
                args: evaled_args,
                style: Some(style),
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
        Expr::Ident(name) => env.get(&name).ok_or_else(|| {
            GambaError::runtime(format!("nome não encontrado: '{}'", name))
                .add_suggestion(Suggestion::new(
                    "verifique se você definiu este nome com '::' antes de usá-lo",
                ))
                .add_suggestion(Suggestion::new("nomes são case-sensitive"))
        }),
        Expr::Let { name, expr } => {
            // semântica de let:
            // - se o nome ainda nao existe no escopo atual: pré-declara (Unit) antes de avaliar o RHS
            //   isso deixa "let rec" o padrão pra definições novas (funções recursivas ficam naturais)
            // - se ja existe: é rebind (não pré-declara); o RHS enxerga o valor anterior
            let is_rebind = env.contains_local(&name);
            if !is_rebind {
                env.predeclare(name.clone());
            }
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
        Expr::Call {
            func,
            args,
            call_line,
            call_col,
            style,
        } => {
            let fval = eval_expr(env, *func)?;
            let mut arg_vals = Vec::with_capacity(args.len());
            for a in args {
                arg_vals.push(eval_expr(env, a)?);
            }
            call(env, fval, arg_vals, Some(style)).map_err(|e| e.with_pos(call_line, call_col))
        }
        Expr::Group(inner) => eval_expr(env, *inner),
        Expr::Block(b) => eval_block(env, b),
        Expr::Binary { op, left, right } => {
            let l = eval_expr(env, *left)?;
            let r = eval_expr(env, *right)?;
            match (op, l.clone(), r.clone()) {
                (BinaryOp::Add, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                // concatenação de strings com +
                (BinaryOp::Add, Value::String(a), v) => Ok(Value::String(format!("{}{}", a, v))),
                (BinaryOp::Add, v, Value::String(b)) => Ok(Value::String(format!("{}{}", v, b))),
                (BinaryOp::Sub, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
                (BinaryOp::Mul, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
                (BinaryOp::Div, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a / b)),
                (BinaryOp::Mod, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a % b)),
                (BinaryOp::Gt, Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a > b)),
                (BinaryOp::Ge, Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a >= b)),
                (BinaryOp::Eq, a, b) => Ok(Value::Bool(eq_value(&a, &b))),
                _ => Err(GambaError::runtime(format!(
                    "operação inválida: tipos incompatíveis para o operador {:?} entre {} e {}",
                    op,
                    l.clone().type_name(),
                    r.clone().type_name()
                ))),
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

fn eq_value(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Unit, Value::Unit) => true,
        (Value::List(xs), Value::List(ys)) => {
            xs.len() == ys.len() && xs.iter().zip(ys.iter()).all(|(u, v)| eq_value(u, v))
        }
        (Value::Map(mx), Value::Map(my)) => {
            if mx.len() != my.len() {
                return false;
            }
            mx.iter()
                .all(|(k, vx)| my.get(k).map_or(false, |vy| eq_value(vx, vy)))
        }
        _ => false,
    }
}

fn call(
    env: &Env,
    fval: Value,
    args: Vec<Value>,
    style: Option<CallStyle>,
) -> Result<Value, GambaError> {
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
                        "a função anônima esperava {} argumentos, mas recebeu {}",
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

                // ultima expressão em posição de cauda: pode devolver TailCall pra trampolim
                match eval_tail(&frame, last)? {
                    TailAction::Return(v) => return Ok(v),
                    TailAction::TailCall { func, args, style } => {
                        // troca alvo e args e itera, sem crescer a pilha
                        current_func = func;
                        current_args = args;
                        // o estilo já foi capturado anteriormente, continuamos com o mesmo loop
                        continue;
                    }
                }
            }
            other => {
                return Err(nonfunc_call_error(style, &other));
            }
        }
    }
}

fn nonfunc_call_error(style: Option<CallStyle>, callee: &Value) -> GambaError {
    let hint = match style.unwrap_or(CallStyle::Internal) {
        CallStyle::Pipe => {
            "após |> o alvo deve ser uma função (nome, chamada ou lambda). \
Exemplos: xs |> map(fn x { ... }),  x |> f(a),  x |> (fn v { ... })."
        }
        CallStyle::Juxta => {
            "chamada por justaposição: o alvo não é uma função. \
Dica: justaposição só vale quando o alvo é um nome de função, uma lambda ou uma chamada. \
Se você quis apenas compor expressões, use parênteses: \
ex.: (width - 2) - len(title). Se quis chamar, garanta que o alvo é função."
        }
        CallStyle::Parens => {
            "o identificador/expressão usado como função não avalia para função. \
Dica: verifique se o nome não foi sobrescrito por um número/string/map."
        }
        CallStyle::Internal => "valor chamado não é função.",
    };

    GambaError::runtime(format!(
        "tentativa de chamar um valor não-função ({} = {}). {}",
        callee.type_name(),
        callee,
        hint
    ))
}

pub fn call_value(env: &Env, fval: Value, args: Vec<Value>) -> Result<Value, GambaError> {
    // chamadas internas (map/reduce/each/etc.) não têm "estilo" lexical
    call(env, fval, args, Some(CallStyle::Internal))
}
