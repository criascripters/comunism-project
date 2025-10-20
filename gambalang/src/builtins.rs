use std::collections::HashMap;
use std::io::IsTerminal;
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::env::{Env, FuncImpl, Value};
use crate::error::GambaError;
use crate::interpreter;
use crate::{lexer, parser};

thread_local! {
    static RNG_STATE: std::cell::RefCell<u64> = std::cell::RefCell::new(0x123456789abcdef);
}

fn rand_next() -> u64 {
    RNG_STATE.with(|s| {
        let mut x = *s.borrow();
        // xorshift64*
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        *s.borrow_mut() = x;
        x.wrapping_mul(2685821657736338717)
    })
}

fn builtin_many<F>(env: &Env, name: &str, f: F)
where
    F: 'static + Fn(&Env, &[Value]) -> Result<Value, GambaError>,
{
    let func = Value::Func(FuncImpl::Builtin(std::rc::Rc::new(f)));
    env.set(name.to_string(), func).unwrap();
}

fn arity(name: &str, args: &[Value], expected: usize) -> Result<(), GambaError> {
    if args.len() != expected {
        return Err(GambaError::runtime(format!(
            "a função {} esperava {} argumentos, mas recebeu {}",
            name,
            expected,
            args.len()
        )));
    }
    Ok(())
}

pub fn install_builtins(env: &Env) {
    // I/O
    builtin_many(env, "print", |_env, args| {
        arity("print", args, 1)?;
        print!("{}", args[0]);
        Ok(Value::Unit)
    });
    builtin_many(env, "println", |_env, args| {
        arity("println", args, 1)?;
        println!("{}", args[0]);
        Ok(Value::Unit)
    });
    builtin_many(env, "flush", |_env, _args| {
        io::stdout()
            .flush()
            .map_err(|e| GambaError::runtime(e.to_string()))?;
        Ok(Value::Unit)
    });
    builtin_many(env, "clear", |_env, _args| {
        print!("\x1b[2J\x1b[H");
        Ok(Value::Unit)
    });
    builtin_many(env, "input", |_env, args| {
        if args.len() > 1 {
            return Err(GambaError::runtime(
                "input espera 0 ou 1 argumento (prompt opcional)",
            ));
        }
        if args.len() == 1 {
            print!("{}", args[0]);
            io::stdout().flush().ok();
        }
        // evita travar em CI/ambiente nao interativo
        if !io::stdin().is_terminal() {
            return Ok(Value::String(String::new()));
        }
        let mut buf = String::new();
        io::stdin()
            .read_line(&mut buf)
            .map_err(|e| GambaError::runtime(e.to_string()))?;
        if buf.ends_with('\n') {
            buf.pop();
            if buf.ends_with('\r') {
                buf.pop();
            }
        }
        Ok(Value::String(buf))
    });
    builtin_many(env, "sleep_ms", |_env, args| {
        arity("sleep_ms", args, 1)?;
        let ms = expect_number_in("sleep_ms", &args[0])?;
        thread::sleep(Duration::from_millis(ms as u64));
        Ok(Value::Unit)
    });
    builtin_many(env, "now_ms", |_env, _args| {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        Ok(Value::Number(now.as_millis() as f64))
    });

    // eval: avalia uma string de código gamba no ambiente atual
    builtin_many(env, "eval", |env, args| {
        arity("eval", args, 1)?;
        let src = expect_string_in("eval", &args[0])?;
        let mut lx = lexer::Lexer::new(&src);
        let tokens = lx.tokenize()?;
        let mut p = parser::Parser::new(tokens);
        let program = p.parse_program()?;
        interpreter::eval_program_in_env(env, program)
    });

    // try_eval: avalia código e captura erros; retorna um map {ok: bool, value: v} ou {ok: false, error: "msg"}
    builtin_many(env, "try_eval", |env, args| {
        arity("try_eval", args, 1)?;
        let src = expect_string_in("try_eval", &args[0])?;
        let mut out = HashMap::new();
        let res = (|| {
            let mut lx = lexer::Lexer::new(&src);
            let tokens = lx.tokenize()?;
            let mut p = parser::Parser::new(tokens);
            let program = p.parse_program()?;
            interpreter::eval_program_in_env(env, program)
        })();
        match res {
            Ok(v) => {
                out.insert("ok".to_string(), Value::Bool(true));
                out.insert("value".to_string(), v);
                Ok(Value::Map(out))
            }
            Err(e) => {
                out.insert("ok".to_string(), Value::Bool(false));
                out.insert("error".to_string(), Value::String(format!("{}", e)));
                Ok(Value::Map(out))
            }
        }
    });

    // erro explícito (útil p/ cancelar loops como forever)
    builtin_many(env, "error", |_env, args| {
        arity("error", args, 1)?;
        let msg = expect_string_in("error", &args[0])?;
        Err(GambaError::runtime(msg))
    });

    // encerra o processo (opcional, mas prático em CLIs)
    builtin_many(env, "exit", |_env, args| {
        if args.len() == 0 {
            std::process::exit(0);
        } else if args.len() == 1 {
            let code = expect_number(&args[0])? as i32;
            std::process::exit(code);
        } else {
            Err(GambaError::runtime(
                "exit: usa 0 ou 1 argumento (código opcional)",
            ))
        }
    });

    // numericos
    builtin_many(env, "add", |_env, a| bin_num("add", a, |x, y| x + y));
    builtin_many(env, "sub", |_env, a| bin_num("sub", a, |x, y| x - y));
    builtin_many(env, "mult", |_env, a| bin_num("mult", a, |x, y| x * y));
    builtin_many(env, "div", |_env, a| bin_num("div", a, |x, y| x / y));
    builtin_many(env, "mod", |_env, a| bin_num("mod", a, |x, y| x % y));

    builtin_many(env, "eq", |_env, args| Ok(Value::Bool(equals(&args)?)));
    builtin_many(env, "gt", |_env, args| {
        let (x, y) = expect_two_numbers(args)?;
        Ok(Value::Bool(x > y))
    });
    builtin_many(env, "lt", |_env, args| {
        let (x, y) = expect_two_numbers(args)?;
        Ok(Value::Bool(x < y))
    });
    builtin_many(env, "ge", |_env, args| {
        let (x, y) = expect_two_numbers(args)?;
        Ok(Value::Bool(x >= y))
    });
    builtin_many(env, "le", |_env, args| {
        let (x, y) = expect_two_numbers(args)?;
        Ok(Value::Bool(x <= y))
    });
    builtin_many(env, "not", |_env, args| {
        arity("not", args, 1)?;
        Ok(Value::Bool(expect_bool(&args[0])? == false))
    });
    builtin_many(env, "and", |_env, args| {
        arity("and", args, 2)?;
        Ok(Value::Bool(
            expect_bool(&args[0])? && expect_bool(&args[1])?,
        ))
    });
    builtin_many(env, "or", |_env, args| {
        arity("or", args, 2)?;
        Ok(Value::Bool(
            expect_bool(&args[0])? || expect_bool(&args[1])?,
        ))
    });

    // random
    builtin_many(env, "rand", |_env, _args| {
        let r = rand_next();
        let x = (r >> 11) as f64 / ((1u64 << 53) as f64); // [0,1)
        Ok(Value::Number(x))
    });
    builtin_many(env, "rand_int", |_env, args| {
        arity("rand_int", args, 1)?;
        let max = expect_number_in("rand_int", &args[0])? as i64;
        if max <= 0 {
            return Err(GambaError::runtime("rand_int: max deve ser > 0"));
        }
        let r = (rand_next() % (max as u64)) as i64;
        Ok(Value::Number(r as f64))
    });

    // coleções
    builtin_many(env, "len", |_env, args| {
        arity("len", args, 1)?;
        Ok(Value::Number(match &args[0] {
            Value::List(xs) => xs.len() as f64,
            Value::String(s) => s.chars().count() as f64,
            Value::Map(m) => m.len() as f64,
            _ => return Err(GambaError::runtime("len: esperado List, String ou Map")),
        }))
    });
    builtin_many(env, "push", |_env, args| {
        arity("push", args, 2)?;
        let mut xs = expect_list_in("push", &args[0])?;
        xs.push(args[1].clone());
        Ok(Value::List(xs))
    });
    builtin_many(env, "range", |_env, args| {
        arity("range", args, 2)?;
        let (start, end) = expect_two_numbers(args)?;
        let start_i = start as i64;
        let end_i = end as i64;
        if end_i >= start_i {
            let mut v = Vec::new();
            for i in start_i..end_i {
                v.push(Value::Number(i as f64));
            }
            Ok(Value::List(v))
        } else {
            let mut v = Vec::new();
            for i in (end_i..start_i).rev() {
                v.push(Value::Number(i as f64));
            }
            Ok(Value::List(v))
        }
    });
    builtin_many(env, "to_string", |_env, args| {
        arity("to_string", args, 1)?;
        Ok(Value::String(format!("{}", args[0])))
    });
    builtin_many(env, "join", |_env, args| {
        arity("join", args, 2)?;
        let xs = expect_list_in("join", &args[0])?;
        let sep = expect_string_in("join", &args[1])?;
        let mut out = String::new();
        for (i, v) in xs.iter().enumerate() {
            if i > 0 {
                out.push_str(&sep);
            }
            out.push_str(&format!("{}", v));
        }
        Ok(Value::String(out))
    });

    // listas basicas
    builtin_many(env, "first", |_env, args| {
        arity("first", args, 1)?;
        let xs = expect_list_in("first", &args[0])?;
        if xs.is_empty() {
            return Err(GambaError::runtime("first: lista vazia"));
        }
        Ok(xs[0].clone())
    });

    builtin_many(env, "rest", |_env, args| {
        arity("rest", args, 1)?;
        let xs = expect_list_in("rest", &args[0])?;
        if xs.len() <= 1 {
            Ok(Value::List(vec![]))
        } else {
            Ok(Value::List(xs[1..].to_vec()))
        }
    });

    builtin_many(env, "cons", |_env, args| {
        arity("cons", args, 2)?;
        let mut xs = expect_list_in("cons", &args[1])?;
        xs.insert(0, args[0].clone());
        Ok(Value::List(xs))
    });

    // strings
    builtin_many(env, "char_at", |_env, args| {
        arity("char_at", args, 2)?;
        let s = expect_string_in("char_at", &args[0])?;
        let idx = expect_number_in("char_at", &args[1])?;
        if idx < 0.0 || idx.fract() != 0.0 {
            return Err(GambaError::runtime("char_at: índice deve ser inteiro >= 0"));
        }
        let i = idx as usize;
        match s.chars().nth(i) {
            Some(c) => Ok(Value::String(c.to_string())),
            None => Ok(Value::String(String::new())), // fora do range => ""
        }
    });

    builtin_many(env, "string_to_list", |_env, args| {
        arity("string_to_list", args, 1)?;
        let s = expect_string_in("string_to_list", &args[0])?;
        let xs: Vec<Value> = s.chars().map(|c| Value::String(c.to_string())).collect();
        Ok(Value::List(xs))
    });

    builtin_many(env, "string_to_number", |_env, args| {
        arity("string_to_number", args, 1)?;
        let s = expect_string_in("string_to_number", &args[0])?;
        match s.trim().parse::<f64>() {
            Ok(n) => Ok(Value::Number(n)),
            Err(_) => Err(GambaError::runtime("string_to_number: string inválida")),
        }
    });

    builtin_many(env, "at", |_env, args| {
        arity("at", args, 2)?;
        let xs = expect_list_in("at", &args[0])?;
        let idx = expect_number_in("at", &args[1])?;
        if idx < 0.0 || idx.fract() != 0.0 {
            return Err(GambaError::runtime("at: índice deve ser inteiro >= 0"));
        }
        let i = idx as usize;
        if i >= xs.len() {
            return Err(GambaError::runtime("at: índice fora do range"));
        }
        Ok(xs[i].clone())
    });

    // funções matemáticas
    builtin_many(env, "cos", |_env, args| {
        arity("cos", args, 1)?;
        let x = expect_number_in("cos", &args[0])?;
        Ok(Value::Number(x.cos()))
    });

    builtin_many(env, "sin", |_env, args| {
        arity("sin", args, 1)?;
        let x = expect_number_in("sin", &args[0])?;
        Ok(Value::Number(x.sin()))
    });

    builtin_many(env, "floor", |_env, args| {
        arity("floor", args, 1)?;
        let x = expect_number_in("floor", &args[0])?;
        Ok(Value::Number(x.floor()))
    });

    builtin_many(env, "flatten", |_env, args| {
        arity("flatten", args, 1)?;
        let xs = expect_list_in("flatten", &args[0])?;
        let mut result = Vec::new();
        for item in xs {
            match item {
                Value::List(inner) => {
                    result.extend(inner);
                }
                _ => {
                    result.push(item);
                }
            }
        }
        Ok(Value::List(result))
    });

    // map / dict (chaves string)
    builtin_many(env, "map_new", |_env, args| {
        arity("map_new", args, 0)?;
        Ok(Value::Map(HashMap::new()))
    });

    // pega com valor padrao: map_get_or(map, "k", default)
    builtin_many(env, "map_get_or", |_env, args| {
        arity("map_get_or", args, 3)?;
        let m = expect_map_in("map_get_or", &args[0])?;
        let k = expect_string_in("map_get_or", &args[1])?;
        Ok(m.get(&k).cloned().unwrap_or(args[2].clone()))
    });

    builtin_many(env, "map_get", |_env, args| {
        arity("map_get", args, 2)?;
        let m = expect_map_in("map_get", &args[0])?;
        let k = expect_string_in("map_get", &args[1])?;
        match m.get(&k) {
            Some(v) => Ok(v.clone()),
            None => Err(GambaError::runtime("map_get: chave não encontrada")),
        }
    });

    builtin_many(env, "map_has", |_env, args| {
        arity("map_has", args, 2)?;
        let m = expect_map_in("map_has", &args[0])?;
        let k = expect_string_in("map_has", &args[1])?;
        Ok(Value::Bool(m.contains_key(&k)))
    });

    builtin_many(env, "map_set", |_env, args| {
        arity("map_set", args, 3)?;
        let mut m = expect_map_in("map_set", &args[0])?;
        let k = expect_string_in("map_set", &args[1])?;
        let v = args[2].clone();
        m.insert(k, v);
        Ok(Value::Map(m))
    });

    builtin_many(env, "map_keys", |_env, args| {
        arity("map_keys", args, 1)?;
        let m = expect_map(&args[0])?;
        let mut ks: Vec<String> = m.keys().cloned().collect();
        ks.sort();
        Ok(Value::List(ks.into_iter().map(Value::String).collect()))
    });

    builtin_many(env, "map_merge", |_env, args| {
        arity("map_merge", args, 2)?;
        let mut a = expect_map(&args[0])?;
        let b = expect_map(&args[1])?;
        for (k, v) in b {
            a.insert(k, v);
        }
        Ok(Value::Map(a))
    });

    // cria map a partir de lista de pares: [["k" v] ["x" y]]
    builtin_many(env, "map_from_pairs", |_env, args| {
        arity("map_from_pairs", args, 1)?;
        let pairs = expect_list_in("map_from_pairs", &args[0])?;
        let mut m = HashMap::new();
        for p in pairs {
            match p {
                Value::List(items) if items.len() == 2 => {
                    let key = expect_string_in("map_from_pairs", &items[0])?;
                    let val = items[1].clone();
                    m.insert(key, val);
                }
                _ => {
                    return Err(GambaError::runtime(
                        "map_from_pairs: esperado lista de [key value]",
                    ));
                }
            }
        }
        Ok(Value::Map(m))
    });

    builtin_many(env, "screen_set", |_env, args| {
        arity("screen_set", args, 4)?;
        let mut screen = expect_list_in("screen_set", &args[0])?;
        let x = expect_number_in("screen_set", &args[1])?;
        let y = expect_number_in("screen_set", &args[2])?;
        let char = expect_string_in("screen_set", &args[3])?;

        if x < 0.0 || y < 0.0 || x.fract() != 0.0 || y.fract() != 0.0 {
            return Err(GambaError::runtime(
                "screen_set: coordenadas devem ser inteiros >= 0",
            ));
        }

        let xi = x as usize;
        let yi = y as usize;

        if yi >= screen.len() {
            return Err(GambaError::runtime(
                "screen_set: coordenada y fora do range",
            ));
        }

        match &mut screen[yi] {
            Value::List(row) => {
                if xi >= row.len() {
                    return Err(GambaError::runtime(
                        "screen_set: coordenada x fora do range",
                    ));
                }
                row[xi] = Value::String(char);
            }
            _ => {
                return Err(GambaError::runtime(
                    "screen_set: screen deve ser lista de listas",
                ))
            }
        }

        Ok(Value::List(screen))
    });

    // alta ordem: map/filter/reduce executam lambdas por interpreter
    builtin_many(env, "map", |env, args| {
        arity("map", args, 2)?;
        let xs = expect_list_in("map", &args[0])?;
        let f_impl = expect_func(&args[1])?;
        let f_val = Value::Func(f_impl);
        let mut out = Vec::with_capacity(xs.len());
        for x in xs {
            let r = interpreter::call_value(env, f_val.clone(), vec![x])?;
            out.push(r);
        }
        Ok(Value::List(out))
    });

    builtin_many(env, "filter", |env, args| {
        arity("filter", args, 2)?;
        let xs = expect_list_in("filter", &args[0])?;
        let f_impl = expect_func(&args[1])?;
        let f_val = Value::Func(f_impl);
        let mut out = Vec::new();
        for x in xs {
            let keep = interpreter::call_value(env, f_val.clone(), vec![x.clone()])?;
            if expect_bool(&keep)? {
                out.push(x);
            }
        }
        Ok(Value::List(out))
    });

    builtin_many(env, "reduce", |env, args| {
        arity("reduce", args, 3)?;
        let xs = expect_list_in("reduce", &args[0])?;
        let mut acc = args[1].clone();
        let f_impl = expect_func(&args[2])?;
        let f_val = Value::Func(f_impl);
        for x in xs {
            acc = interpreter::call_value(env, f_val.clone(), vec![acc, x])?;
        }
        Ok(acc)
    });

    // repeat: aplica f ao valor n vezes. uso: x |> repeat(f, n)  ou repeat(x, f, n)
    builtin_many(env, "repeat", |env, args| {
        arity("repeat", args, 3)?;
        let mut acc = args[0].clone();
        let f_impl = expect_func(&args[1])?;
        let f_val = Value::Func(f_impl);
        let n = expect_number_in("repeat", &args[2])?;
        if n < 0.0 || n.fract() != 0.0 {
            return Err(GambaError::runtime(format!(
                "a função repeat esperava um inteiro >= 0 como 3º argumento, mas recebeu {}",
                args[2].type_name()
            )));
        }
        let times = n as usize;
        for _ in 0..times {
            acc = interpreter::call_value(env, f_val.clone(), vec![acc])?;
        }
        Ok(acc)
    });

    // each: itera pela lista e aplica a função (pra side-effects), retorna Unit
    // Uso: xs |> each(fn x { println(x) })
    builtin_many(env, "each", |env, args| {
        arity("each", args, 2)?;
        let xs = expect_list_in("each", &args[0])?;
        let f_impl = expect_func(&args[1])?;
        let f_val = Value::Func(f_impl);
        for x in xs {
            let _ = interpreter::call_value(env, f_val.clone(), vec![x])?;
        }
        Ok(Value::Unit)
    });

    // forever: laço infinito controlado em rust (nao cresce a pilha)
    // uso: forever(estado_inicial, fn s { ...; novo_estado })
    builtin_many(env, "forever", |env, args| {
        arity("forever", args, 2)?;
        let mut state = args[0].clone();
        let f_impl = expect_func(&args[1])?;
        let f_val = Value::Func(f_impl);
        loop {
            state = interpreter::call_value(env, f_val.clone(), vec![state])?;
        }
    });
}

fn expect_number(v: &Value) -> Result<f64, GambaError> {
    match v {
        Value::Number(n) => Ok(*n),
        _ => Err(GambaError::runtime(format!(
            "esperado número, mas recebeu {}",
            v.type_name()
        ))),
    }
}

fn expect_number_in(name: &str, v: &Value) -> Result<f64, GambaError> {
    match v {
        Value::Number(n) => Ok(*n),
        _ => Err(GambaError::runtime(format!(
            "{}: esperado Number, mas recebeu {}",
            name,
            v.type_name()
        ))),
    }
}
fn expect_bool(v: &Value) -> Result<bool, GambaError> {
    match v {
        Value::Bool(b) => Ok(*b),
        _ => Err(GambaError::runtime(format!(
            "esperado bool, mas recebeu {}",
            v.type_name()
        ))),
    }
}
fn expect_bool_in(name: &str, v: &Value) -> Result<bool, GambaError> {
    match v {
        Value::Bool(b) => Ok(*b),
        _ => Err(GambaError::runtime(format!(
            "{}: esperado Bool, mas recebeu {}",
            name,
            v.type_name()
        ))),
    }
}
fn expect_string(v: &Value) -> Result<String, GambaError> {
    match v {
        Value::String(s) => Ok(s.clone()),
        _ => Err(GambaError::runtime(format!(
            "esperado string, mas recebeu {}",
            v.type_name()
        ))),
    }
}
fn expect_string_in(name: &str, v: &Value) -> Result<String, GambaError> {
    match v {
        Value::String(s) => Ok(s.clone()),
        _ => Err(GambaError::runtime(format!(
            "{}: esperado String, mas recebeu {}",
            name,
            v.type_name()
        ))),
    }
}
fn expect_list(v: &Value) -> Result<Vec<Value>, GambaError> {
    match v {
        Value::List(xs) => Ok(xs.clone()),
        _ => Err(GambaError::runtime(format!(
            "esperado lista, mas recebeu {}",
            v.type_name()
        ))),
    }
}
fn expect_list_in(name: &str, v: &Value) -> Result<Vec<Value>, GambaError> {
    match v {
        Value::List(xs) => Ok(xs.clone()),
        _ => Err(GambaError::runtime(format!(
            "{}: esperado List, mas recebeu {}. dica: chame {}(lista, ...) ou use pipe: xs |> {} ...",
            name,
            v.type_name(),
            name,
            name
        ))),
    }
}
fn expect_two_numbers(args: &[Value]) -> Result<(f64, f64), GambaError> {
    if args.len() != 2 {
        return Err(GambaError::runtime(format!(
            "esta função esperava 2 argumentos, mas recebeu {}",
            args.len()
        )));
    }
    Ok((expect_number(&args[0])?, expect_number(&args[1])?))
}
fn bin_num<F>(name: &str, args: &[Value], op: F) -> Result<Value, GambaError>
where
    F: Fn(f64, f64) -> f64,
{
    if args.len() != 2 {
        return Err(GambaError::runtime(format!(
            "a função {} esperava 2 argumentos, mas recebeu {}",
            name,
            args.len()
        )));
    }
    Ok(Value::Number(op(
        expect_number(&args[0])?,
        expect_number(&args[1])?,
    )))
}

fn equals(args: &[Value]) -> Result<bool, GambaError> {
    if args.len() != 2 {
        return Err(GambaError::runtime(format!(
            "a função eq esperava 2 argumentos, mas recebeu {}",
            args.len()
        )));
    }
    Ok(eq_value(&args[0], &args[1]))
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

fn expect_func(v: &Value) -> Result<FuncImpl, GambaError> {
    match v {
        Value::Func(f) => Ok(f.clone()),
        _ => Err(GambaError::runtime("esperado função")),
    }
}
fn expect_map(v: &Value) -> Result<HashMap<String, Value>, GambaError> {
    match v {
        Value::Map(m) => Ok(m.clone()),
        _ => Err(GambaError::runtime("Esperado map/dict")),
    }
}
fn expect_map_in(name: &str, v: &Value) -> Result<HashMap<String, Value>, GambaError> {
    match v {
        Value::Map(m) => Ok(m.clone()),
        _ => Err(GambaError::runtime(format!(
            "{}: esperado Map/Dict, mas recebeu {}",
            name,
            v.type_name()
        ))),
    }
}
