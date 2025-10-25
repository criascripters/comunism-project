use std::collections::HashMap;
use std::fs;
use std::io::IsTerminal;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    mpsc, Arc, Mutex, OnceLock,
};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::env::{Env, FuncImpl, ScreenBuf, Value};
use crate::error::GambaError;
use crate::interpreter;

// relogio monotono p time
static START_INSTANT: OnceLock<Instant> = OnceLock::new();

// tui frame anterior para diff
static TUI_PREV: OnceLock<Mutex<Option<Arc<ScreenBuf>>>> = OnceLock::new();
fn tui_prev() -> &'static Mutex<Option<Arc<ScreenBuf>>> {
    TUI_PREV.get_or_init(|| Mutex::new(None))
}

fn char_from_str(s: &str) -> char {
    s.chars().next().unwrap_or(' ')
}

// screen helpers
fn screen_new_buf(w: usize, h: usize, fill: char) -> Arc<ScreenBuf> {
    let mut rows = Vec::with_capacity(h);
    for _ in 0..h {
        let mut row = Vec::with_capacity(w);
        row.resize(w, fill);
        rows.push(Arc::new(row));
    }
    Arc::new(ScreenBuf { w, h, rows })
}

fn screen_clone_replace_row(
    s: &Arc<ScreenBuf>,
    y: usize,
    new_row: Arc<Vec<char>>,
) -> Arc<ScreenBuf> {
    let mut rows = s.rows.clone();
    rows[y] = new_row;
    Arc::new(ScreenBuf {
        w: s.w,
        h: s.h,
        rows,
    })
}

fn screen_get_char(s: &ScreenBuf, x: usize, y: usize) -> Option<char> {
    if y >= s.h || x >= s.w {
        return None;
    }
    s.rows.get(y).and_then(|row| row.get(x)).cloned()
}

// tui raw helpers
fn ansi(s: &str) {
    print!("{}", s);
}
fn cursor_move(y1: usize, x1: usize) {
    print!("\x1b[{};{}H", y1, x1);
} // 1-based
fn hide_cursor() {
    ansi("\x1b[?25l");
}
fn show_cursor() {
    ansi("\x1b[?25h");
}
fn alt_screen_on() {
    ansi("\x1b[?1049h");
}
fn alt_screen_off() {
    ansi("\x1b[?1049l");
}
fn clear_all() {
    ansi("\x1b[2J\x1b[H");
}

fn render_full(s: &ScreenBuf) {
    clear_all();
    for (y, row) in s.rows.iter().enumerate() {
        cursor_move(y + 1, 1);
        // transforma vec<char> em string
        let line: String = row.iter().collect();
        print!("{}", line);
    }
    io::stdout().flush().ok();
}

fn render_diff(prev: &ScreenBuf, curr: &ScreenBuf) {
    if prev.w != curr.w || prev.h != curr.h {
        return render_full(curr);
    }
    for y in 0..curr.h {
        let a = &prev.rows[y];
        let b = &curr.rows[y];
        if Arc::ptr_eq(a, b) {
            continue;
        } // linha identica
          // diff simples por runs
        let (mut x, _in_run) = (0usize, false);
        while x < curr.w {
            let ca = a[x];
            let cb = b[x];
            if ca != cb {
                // inicia run
                let start = x;
                let mut end = x + 1;
                while end < curr.w && a[end] != b[end] {
                    end += 1;
                }
                cursor_move(y + 1, start + 1);
                let seg: String = b[start..end].iter().collect();
                print!("{}", seg);
                x = end;
                // in_run = false; // não usado
            } else {
                x += 1;
            }
        }
    }
    io::stdout().flush().ok();
}

use crate::{lexer, parser};

thread_local! {
    // semente nao-deterministica por thread, 0 indica ainda nn iniciado
    static RNG_STATE: std::cell::RefCell<u64> = std::cell::RefCell::new(0);
}

fn seed64() -> u64 {
    // mistura tempo + endereço de função para diversificar
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9e3779b97f4a7c15);
    let addr = seed64 as usize as u64;
    let mut x = t ^ addr.rotate_left(17) ^ (t.wrapping_mul(0x9e3779b97f4a7c15));
    if x == 0 {
        x = 0x9e3779b97f4a7c15;
    }
    x
}

// infra de tarefas
static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);

enum TaskState {
    Pending(mpsc::Receiver<Result<Value, GambaError>>),
    Done(Result<Value, GambaError>),
}

struct TaskEntry {
    state: TaskState,
}

static TASKS: OnceLock<Mutex<HashMap<u64, TaskEntry>>> = OnceLock::new();

fn tasks() -> &'static Mutex<HashMap<u64, TaskEntry>> {
    TASKS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn new_task<F>(job: F) -> Value
where
    F: Send + 'static + FnOnce() -> Result<Value, GambaError>,
{
    let (tx, rx) = mpsc::channel();
    let id = NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed);
    thread::spawn(move || {
        let res = job();
        let _ = tx.send(res);
    });
    tasks().lock().unwrap().insert(
        id,
        TaskEntry {
            state: TaskState::Pending(rx),
        },
    );
    Value::Task(id)
}

fn expect_task_in(name: &str, v: &Value) -> Result<u64, GambaError> {
    match v {
        Value::Task(id) => Ok(*id),
        _ => Err(GambaError::runtime(format!(
            "{}: esperado Task, mas recebeu {}",
            name,
            v.type_name()
        ))),
    }
}

// helpers para lidar com Handle com compat (aceita Number também)
fn expect_handle_in(name: &str, v: &Value) -> Result<u64, GambaError> {
    match v {
        Value::Handle(id) => Ok(*id),
        Value::Number(n) if *n >= 0.0 && n.fract() == 0.0 => Ok(*n as u64), // compat
        _ => Err(GambaError::runtime(format!(
            "{}: esperado Handle, mas recebeu {}",
            name,
            v.type_name()
        ))),
    }
}

// === loader de modulos com ciclo detectado e de-duplicado ===
enum ModuleEntry {
    Loading,
    Ready(Value),
}

static MODULES: OnceLock<Mutex<HashMap<String, ModuleEntry>>> = OnceLock::new();
fn modules() -> &'static Mutex<HashMap<String, ModuleEntry>> {
    MODULES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn resolve_path(base_dir_opt: Option<String>, path: &str) -> String {
    let p = Path::new(path);
    if p.is_absolute() {
        return p.to_string_lossy().to_string();
    }
    if let Some(base_dir) = base_dir_opt {
        let full = Path::new(&base_dir).join(p);
        return full.to_string_lossy().to_string();
    }
    p.to_string_lossy().to_string()
}

fn normalize_exports(v: Value) -> Value {
    match v {
        Value::Map(_) => v,
        other => {
            let mut m = HashMap::new();
            m.insert("default".to_string(), other);
            Value::Map(Arc::new(m))
        }
    }
}

fn load_module_from_path(abs: &str) -> Result<Value, GambaError> {
    // checa cache / ciclo
    {
        let mut guard = modules().lock().unwrap();
        match guard.get(abs) {
            Some(ModuleEntry::Ready(v)) => return Ok(v.clone()),
            Some(ModuleEntry::Loading) => {
                return Err(GambaError::runtime(format!(
                    "import/require: ciclo detectado em '{}'",
                    abs
                )));
            }
            None => {
                guard.insert(abs.to_string(), ModuleEntry::Loading);
            }
        }
    }

    // carrega/avalia fora do lock
    let src = fs::read_to_string(&abs).map_err(|e| GambaError::runtime(e.to_string()))?;
    let mut lx = crate::lexer::Lexer::new(&src);
    let tokens = lx.tokenize()?;
    let mut p = crate::parser::Parser::new(tokens);
    let program = p.parse_program()?;

    let mut rt = crate::env::Runtime::with_builtins();
    let dir = Path::new(&abs)
        .parent()
        .map(|d| d.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());
    rt.env
        .set("__file__".to_string(), Value::String(abs.to_string()))?;
    rt.env.set("__dir__".to_string(), Value::String(dir))?;
    let v = crate::interpreter::eval_program(&mut rt, program)?;
    let exports = normalize_exports(v);

    // finaliza cache
    let mut guard = modules().lock().unwrap();
    guard.insert(abs.to_string(), ModuleEntry::Ready(exports.clone()));
    Ok(exports)
}

fn rand_next() -> u64 {
    RNG_STATE.with(|s| {
        let mut x = *s.borrow();
        if x == 0 {
            x = seed64();
            *s.borrow_mut() = x;
        }
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
    F: 'static + Fn(&Env, &[Value]) -> Result<Value, GambaError> + Send + Sync,
{
    let func = Value::Func(FuncImpl::Builtin(std::sync::Arc::new(f)));
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
                Ok(Value::Map(Arc::new(out)))
            }
            Err(e) => {
                out.insert("ok".to_string(), Value::Bool(false));
                out.insert("error".to_string(), Value::String(format!("{}", e)));
                Ok(Value::Map(Arc::new(out)))
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
    builtin_many(env, "div", |_env, args| {
        arity("div", args, 2)?;
        let x = expect_number(&args[0])?;
        let y = expect_number(&args[1])?;
        if y == 0.0 {
            return Err(GambaError::runtime("div: divisão por zero"));
        }
        Ok(Value::Number(x / y))
    });
    builtin_many(env, "mod", |_env, args| {
        arity("mod", args, 2)?;
        let x = expect_number(&args[0])?;
        let y = expect_number(&args[1])?;
        if y == 0.0 {
            return Err(GambaError::runtime("mod: módulo por zero"));
        }
        Ok(Value::Number(x % y))
    });

    builtin_many(env, "eq", |_env, args| {
        if args.len() != 2 {
            return Err(GambaError::runtime(format!(
                "a função eq esperava 2 argumentos, mas recebeu {}",
                args.len()
            )));
        }
        Ok(Value::Bool(args[0].equals(&args[1])))
    });
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
        let xs = expect_list_arc_in("push", &args[0])?;
        let mut v = Vec::with_capacity(xs.len() + 1);
        v.extend(xs.iter().cloned());
        v.push(args[1].clone());
        Ok(Value::List(Arc::new(v)))
    });
    builtin_many(env, "range", |_env, args| {
        arity("range", args, 2)?;
        let (start, end) = expect_two_numbers(args)?;
        let (start_i, end_i) = (start as i64, end as i64);
        let mut v = Vec::new();
        if end_i >= start_i {
            for i in start_i..end_i {
                v.push(Value::Number(i as f64));
            }
        } else {
            for i in (end_i..start_i).rev() {
                v.push(Value::Number(i as f64));
            }
        }
        Ok(Value::List(Arc::new(v)))
    });
    builtin_many(env, "to_string", |_env, args| {
        arity("to_string", args, 1)?;
        Ok(Value::String(format!("{}", args[0])))
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
        let xs = expect_list_ref("rest", &args[0])?;
        if xs.len() <= 1 {
            Ok(Value::List(Arc::new(vec![])))
        } else {
            Ok(Value::List(Arc::new(xs[1..].to_vec())))
        }
    });

    builtin_many(env, "cons", |_env, args| {
        arity("cons", args, 2)?;
        let xs = expect_list_arc_in("cons", &args[1])?;
        let mut v = Vec::with_capacity(xs.len() + 1);
        v.push(args[0].clone());
        v.extend(xs.iter().cloned());
        Ok(Value::List(Arc::new(v)))
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
        Ok(Value::List(Arc::new(xs)))
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
        let xs = expect_list_ref("at", &args[0])?;
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
        let xs = expect_list_ref("flatten", &args[0])?;
        let mut result = Vec::new();
        for item in xs.iter().cloned() {
            match item {
                Value::List(inner) => result.extend(inner.iter().cloned()),
                other => result.push(other),
            }
        }
        Ok(Value::List(Arc::new(result)))
    });

    // map / dict (chaves string)
    builtin_many(env, "map_new", |_env, args| {
        arity("map_new", args, 0)?;
        Ok(Value::Map(Arc::new(HashMap::new())))
    });

    // pega com valor padrao: map_get_or(map, "k", default)
    builtin_many(env, "map_get_or", |_env, args| {
        arity("map_get_or", args, 3)?;
        let m = expect_map_ref("map_get_or", &args[0])?;
        let k = expect_string_in("map_get_or", &args[1])?;
        Ok(m.get(&k).cloned().unwrap_or(args[2].clone()))
    });

    builtin_many(env, "map_get", |_env, args| {
        arity("map_get", args, 2)?;
        let m = expect_map_ref("map_get", &args[0])?;
        let k = expect_string_in("map_get", &args[1])?;
        match m.get(&k) {
            Some(v) => Ok(v.clone()),
            None => Err(GambaError::runtime("map_get: chave não encontrada")),
        }
    });

    builtin_many(env, "map_has", |_env, args| {
        arity("map_has", args, 2)?;
        let m = expect_map_ref("map_has", &args[0])?;
        let k = expect_string_in("map_has", &args[1])?;
        Ok(Value::Bool(m.contains_key(&k)))
    });

    builtin_many(env, "map_set", |_env, args| {
        arity("map_set", args, 3)?;
        let m = expect_map_arc_in("map_set", &args[0])?;
        let k = expect_string_in("map_set", &args[1])?;
        let v = args[2].clone();
        let mut out = (*m).clone();
        out.insert(k, v);
        Ok(Value::Map(Arc::new(out)))
    });

    builtin_many(env, "map_keys", |_env, args| {
        arity("map_keys", args, 1)?;
        let m = expect_map_ref("map_keys", &args[0])?;
        let mut ks: Vec<String> = m.keys().cloned().collect();
        ks.sort();
        Ok(Value::List(Arc::new(
            ks.into_iter().map(Value::String).collect(),
        )))
    });

    builtin_many(env, "map_merge", |_env, args| {
        arity("map_merge", args, 2)?;
        let a = expect_map_arc_in("map_merge", &args[0])?;
        let b = expect_map_ref("map_merge", &args[1])?;
        let mut out = (*a).clone();
        for (k, v) in b {
            out.insert(k.clone(), v.clone());
        }
        Ok(Value::Map(Arc::new(out)))
    });

    // cria map a partir de lista de pares: [["k" v] ["x" y]]
    builtin_many(env, "map_from_pairs", |_env, args| {
        arity("map_from_pairs", args, 1)?;
        let pairs = expect_list_ref("map_from_pairs", &args[0])?;
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
        Ok(Value::Map(Arc::new(m)))
    });

    builtin_many(env, "screen_set", |_env, args| {
        arity("screen_set", args, 4)?;
        let screen = expect_list_arc_in("screen_set", &args[0])?;
        let x = expect_number_in("screen_set", &args[1])?;
        let y = expect_number_in("screen_set", &args[2])?;
        let ch = expect_string_in("screen_set", &args[3])?;

        if x < 0.0 || y < 0.0 || x.fract() != 0.0 || y.fract() != 0.0 {
            return Err(GambaError::runtime(
                "screen_set: coordenadas devem ser inteiros >= 0",
            ));
        }
        let (xi, yi) = (x as usize, y as usize);
        let rows = screen.as_ref();
        if yi >= rows.len() {
            return Err(GambaError::runtime(
                "screen_set: coordenada y fora do range",
            ));
        }

        let row_vec = match &rows[yi] {
            Value::List(row_arc) => (*(*row_arc)).clone(),
            _ => {
                return Err(GambaError::runtime(
                    "screen_set: screen deve ser lista de listas",
                ))
            }
        };
        if xi >= row_vec.len() {
            return Err(GambaError::runtime(
                "screen_set: coordenada x fora do range",
            ));
        }
        let mut new_row_vec = row_vec;
        new_row_vec[xi] = Value::String(ch);
        let new_row = Value::List(Arc::new(new_row_vec));

        let mut new_screen = rows.clone();
        new_screen[yi] = new_row;
        Ok(Value::List(Arc::new(new_screen)))
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
        Ok(Value::List(Arc::new(out)))
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
        Ok(Value::List(Arc::new(out)))
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

    // await(task): remove a task do mapa e espera fora do lock
    builtin_many(env, "await", |_env, args| {
        arity("await", args, 1)?;
        let id = expect_task_in("await", &args[0])?;

        enum Wait {
            Pending(mpsc::Receiver<Result<Value, GambaError>>),
            Done(Result<Value, GambaError>),
        }

        let wait = {
            let mut guard = tasks().lock().unwrap();
            match guard.remove(&id) {
                Some(TaskEntry {
                    state: TaskState::Pending(rx),
                }) => Wait::Pending(rx),
                Some(TaskEntry {
                    state: TaskState::Done(res),
                }) => Wait::Done(res),
                None => return Err(GambaError::runtime("await: task inválida")),
            }
        }; // lock liberado aqui

        match wait {
            Wait::Pending(rx) => match rx.recv() {
                Ok(Ok(v)) => Ok(v),
                Ok(Err(e)) => Err(e),
                Err(_) => Err(GambaError::runtime("await: task cancelada")),
            },
            Wait::Done(res) => res,
        }
    });

    // try_await(task): mesmo comportamento (bloqueia), mas em {ok: bool, ...}, removendo do mapa
    builtin_many(env, "try_await", |_env, args| {
        arity("try_await", args, 1)?;
        let id = expect_task_in("try_await", &args[0])?;
        let mut out = HashMap::new();

        let wait = {
            let mut guard = tasks().lock().unwrap();
            match guard.remove(&id) {
                Some(TaskEntry {
                    state: TaskState::Pending(rx),
                }) => Some(rx),
                Some(TaskEntry {
                    state: TaskState::Done(res),
                }) => {
                    match res {
                        Ok(v) => {
                            out.insert("ok".to_string(), Value::Bool(true));
                            out.insert("value".to_string(), v);
                        }
                        Err(e) => {
                            out.insert("ok".to_string(), Value::Bool(false));
                            out.insert("error".to_string(), Value::String(format!("{}", e)));
                        }
                    }
                    None
                }
                None => {
                    out.insert("ok".to_string(), Value::Bool(false));
                    out.insert(
                        "error".to_string(),
                        Value::String("try_await: task inválida".to_string()),
                    );
                    None
                }
            }
        };

        if let Some(rx) = wait {
            match rx.recv() {
                Ok(Ok(v)) => {
                    out.insert("ok".to_string(), Value::Bool(true));
                    out.insert("value".to_string(), v);
                }
                Ok(Err(e)) => {
                    out.insert("ok".to_string(), Value::Bool(false));
                    out.insert("error".to_string(), Value::String(format!("{}", e)));
                }
                Err(_) => {
                    out.insert("ok".to_string(), Value::Bool(false));
                    out.insert(
                        "error".to_string(),
                        Value::String("try_await: task cancelada".to_string()),
                    );
                }
            }
        }
        Ok(Value::Map(Arc::new(out)))
    });

    // poll(task): se completar, tambem remove do mapa (nao acumula)
    builtin_many(env, "poll", |_env, args| {
        arity("poll", args, 1)?;
        let id = expect_task_in("poll", &args[0])?;
        let mut out = HashMap::new();

        let mut remove = false;
        {
            let mut guard = tasks().lock().unwrap();
            let entry = guard
                .get_mut(&id)
                .ok_or_else(|| GambaError::runtime("poll: task inválida"))?;
            match &mut entry.state {
                TaskState::Pending(rx) => match rx.try_recv() {
                    Ok(res) => {
                        entry.state = TaskState::Done(res.clone());
                        out.insert("done".to_string(), Value::Bool(true));
                        match res {
                            Ok(v) => {
                                out.insert("value".to_string(), v);
                            }
                            Err(e) => {
                                out.insert("error".to_string(), Value::String(format!("{}", e)));
                            }
                        }
                        remove = true; // ja tem o resultado
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        out.insert("done".to_string(), Value::Bool(false));
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        entry.state = TaskState::Done(Err(GambaError::runtime("task cancelada")));
                        out.insert("done".to_string(), Value::Bool(true));
                        out.insert(
                            "error".to_string(),
                            Value::String("task cancelada".to_string()),
                        );
                        remove = true;
                    }
                },
                TaskState::Done(res) => {
                    out.insert("done".to_string(), Value::Bool(true));
                    match res {
                        Ok(v) => {
                            out.insert("value".to_string(), v.clone());
                        }
                        Err(e) => {
                            out.insert("error".to_string(), Value::String(format!("{}", e)));
                        }
                    }
                    remove = true;
                }
            }
            if remove {
                guard.remove(&id);
            }
        }
        Ok(Value::Map(Arc::new(out)))
    });

    // task_done(task): se ja foi removida (await anterior), considera done = true
    builtin_many(env, "task_done", |_env, args| {
        arity("task_done", args, 1)?;
        let id = expect_task_in("task_done", &args[0])?;
        let guard = tasks().lock().unwrap();
        let done = match guard.get(&id) {
            Some(TaskEntry {
                state: TaskState::Done(_),
            }) => true,
            Some(_) => false,
            None => true, // nao encontrado = já foi removida
        };
        Ok(Value::Bool(done))
    });

    // opcional: esquecer/soltar uma task (detach)
    builtin_many(env, "task_forget", |_env, args| {
        arity("task_forget", args, 1)?;
        let id = expect_task_in("task_forget", &args[0])?;
        tasks().lock().unwrap().remove(&id);
        Ok(Value::Unit)
    });

    // filesystem (sync)
    builtin_many(env, "read_file", |_env, args| {
        arity("read_file", args, 1)?;
        let path = expect_string_in("read_file", &args[0])?;
        let content = fs::read_to_string(&path).map_err(|e| GambaError::runtime(e.to_string()))?;
        Ok(Value::String(content))
    });

    builtin_many(env, "write_file", |_env, args| {
        arity("write_file", args, 2)?;
        let path = expect_string_in("write_file", &args[0])?;
        let data = expect_string_in("write_file", &args[1])?;
        fs::write(&path, data).map_err(|e| GambaError::runtime(e.to_string()))?;
        Ok(Value::Unit)
    });

    builtin_many(env, "append_file", |_env, args| {
        arity("append_file", args, 2)?;
        let path = expect_string_in("append_file", &args[0])?;
        let data = expect_string_in("append_file", &args[1])?;
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| GambaError::runtime(e.to_string()))?;
        f.write_all(data.as_bytes())
            .map_err(|e| GambaError::runtime(e.to_string()))?;
        Ok(Value::Unit)
    });

    builtin_many(env, "read_dir", |_env, args| {
        arity("read_dir", args, 1)?;
        let path = expect_string_in("read_dir", &args[0])?;
        let mut out = Vec::new();
        for entry in fs::read_dir(&path).map_err(|e| GambaError::runtime(e.to_string()))? {
            let e = entry.map_err(|e| GambaError::runtime(e.to_string()))?;
            let name = e.file_name().to_string_lossy().to_string();
            out.push(Value::String(name));
        }
        Ok(Value::List(Arc::new(out)))
    });

    builtin_many(env, "exists", |_env, args| {
        arity("exists", args, 1)?;
        let path = expect_string_in("exists", &args[0])?;
        Ok(Value::Bool(Path::new(&path).exists()))
    });

    builtin_many(env, "is_file", |_env, args| {
        arity("is_file", args, 1)?;
        let path = expect_string_in("is_file", &args[0])?;
        Ok(Value::Bool(Path::new(&path).is_file()))
    });

    builtin_many(env, "is_dir", |_env, args| {
        arity("is_dir", args, 1)?;
        let path = expect_string_in("is_dir", &args[0])?;
        Ok(Value::Bool(Path::new(&path).is_dir()))
    });

    builtin_many(env, "mkdir_all", |_env, args| {
        arity("mkdir_all", args, 1)?;
        let path = expect_string_in("mkdir_all", &args[0])?;
        fs::create_dir_all(&path).map_err(|e| GambaError::runtime(e.to_string()))?;
        Ok(Value::Unit)
    });

    builtin_many(env, "remove_file", |_env, args| {
        arity("remove_file", args, 1)?;
        let path = expect_string_in("remove_file", &args[0])?;
        fs::remove_file(&path).map_err(|e| GambaError::runtime(e.to_string()))?;
        Ok(Value::Unit)
    });

    builtin_many(env, "remove_dir_all", |_env, args| {
        arity("remove_dir_all", args, 1)?;
        let path = expect_string_in("remove_dir_all", &args[0])?;
        fs::remove_dir_all(&path).map_err(|e| GambaError::runtime(e.to_string()))?;
        Ok(Value::Unit)
    });

    builtin_many(env, "rename", |_env, args| {
        arity("rename", args, 2)?;
        let src = expect_string_in("rename", &args[0])?;
        let dst = expect_string_in("rename", &args[1])?;
        fs::rename(&src, &dst).map_err(|e| GambaError::runtime(e.to_string()))?;
        Ok(Value::Unit)
    });

    builtin_many(env, "copy_file", |_env, args| {
        arity("copy_file", args, 2)?;
        let src = expect_string_in("copy_file", &args[0])?;
        let dst = expect_string_in("copy_file", &args[1])?;
        fs::copy(&src, &dst).map_err(|e| GambaError::runtime(e.to_string()))?;
        Ok(Value::Unit)
    });

    builtin_many(env, "cwd", |_env, _args| {
        let p = std::env::current_dir().map_err(|e| GambaError::runtime(e.to_string()))?;
        Ok(Value::String(p.to_string_lossy().to_string()))
    });

    builtin_many(env, "set_cwd", |_env, args| {
        arity("set_cwd", args, 1)?;
        let p = expect_string_in("set_cwd", &args[0])?;
        std::env::set_current_dir(p).map_err(|e| GambaError::runtime(e.to_string()))?;
        Ok(Value::Unit)
    });

    // filesystem async
    builtin_many(env, "read_file_async", |_env, args| {
        arity("read_file_async", args, 1)?;
        let path = expect_string_in("read_file_async", &args[0])?;
        Ok(new_task(move || {
            let s = fs::read_to_string(&path).map_err(|e| GambaError::runtime(e.to_string()))?;
            Ok(Value::String(s))
        }))
    });

    builtin_many(env, "write_file_async", |_env, args| {
        arity("write_file_async", args, 2)?;
        let path = expect_string_in("write_file_async", &args[0])?;
        let data = expect_string_in("write_file_async", &args[1])?;
        Ok(new_task(move || {
            fs::write(&path, data).map_err(|e| GambaError::runtime(e.to_string()))?;
            Ok(Value::Unit)
        }))
    });

    // shell
    fn cmd_output_to_map(out: std::process::Output) -> Value {
        let mut m = HashMap::new();
        let status = out.status.code().unwrap_or(-1) as f64;
        m.insert("ok".to_string(), Value::Bool(out.status.success()));
        m.insert("status".to_string(), Value::Number(status));
        m.insert(
            "stdout".to_string(),
            Value::String(String::from_utf8_lossy(&out.stdout).to_string()),
        );
        m.insert(
            "stderr".to_string(),
            Value::String(String::from_utf8_lossy(&out.stderr).to_string()),
        );
        Value::Map(Arc::new(m))
    }

    builtin_many(env, "sh", |_env, args| {
        arity("sh", args, 1)?;
        let cmd = expect_string_in("sh", &args[0])?;
        let out = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .map_err(|e| GambaError::runtime(e.to_string()))?;
        Ok(cmd_output_to_map(out))
    });

    builtin_many(env, "sh_async", |_env, args| {
        arity("sh_async", args, 1)?;
        let cmd = expect_string_in("sh_async", &args[0])?;
        Ok(new_task(move || {
            let out = Command::new("sh")
                .arg("-c")
                .arg(&cmd)
                .output()
                .map_err(|e| GambaError::runtime(e.to_string()))?;
            Ok(cmd_output_to_map(out))
        }))
    });

    // shp(cmd, args_list)
    builtin_many(env, "shp", |_env, args| {
        arity("shp", args, 2)?;
        let cmd = expect_string_in("shp", &args[0])?;
        let xs = expect_list_in("shp", &args[1])?;
        let mut c = Command::new(&cmd);
        for a in xs {
            c.arg(expect_string(&a)?);
        }
        let out = c.output().map_err(|e| GambaError::runtime(e.to_string()))?;
        Ok(cmd_output_to_map(out))
    });

    builtin_many(env, "shp_async", |_env, args| {
        arity("shp_async", args, 2)?;
        let cmd = expect_string_in("shp_async", &args[0])?;
        let xs = expect_list_in("shp_async", &args[1])?;
        let vs: Vec<String> = xs
            .into_iter()
            .map(|v| expect_string(&v))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(new_task(move || {
            let mut c = Command::new(&cmd);
            for a in vs {
                c.arg(a);
            }
            let out = c.output().map_err(|e| GambaError::runtime(e.to_string()))?;
            Ok(cmd_output_to_map(out))
        }))
    });

    // modulos
    fn resolve_path(base_dir_opt: Option<String>, path: &str) -> String {
        let p = Path::new(path);
        if p.is_absolute() {
            return p.to_string_lossy().to_string();
        }
        if let Some(base_dir) = base_dir_opt {
            let full = Path::new(&base_dir).join(p);
            return full.to_string_lossy().to_string();
        }
        p.to_string_lossy().to_string()
    }

    fn normalize_exports(v: Value) -> Value {
        match v {
            Value::Map(_) => v,
            other => {
                let mut m = HashMap::new();
                m.insert("default".to_string(), other);
                Value::Map(Arc::new(m))
            }
        }
    }

    builtin_many(env, "import", |env, args| {
        if args.len() < 1 || args.len() > 2 {
            return Err(GambaError::runtime(
                "import: use 1 ou 2 argumentos (path, alias opcional)",
            ));
        }
        let rel = expect_string_in("import", &args[0])?;
        let base_dir = env.get("__dir__").and_then(|v| {
            if let Value::String(s) = v {
                Some(s)
            } else {
                None
            }
        });
        let abs = resolve_path(base_dir, &rel);
        let exports = load_module_from_path(&abs)?;

        if args.len() == 1 {
            let m = expect_map_ref("import", &exports)?;
            for (k, v) in m {
                env.set(k.clone(), v.clone())?;
            }
            Ok(Value::Unit)
        } else {
            let alias = expect_string_in("import", &args[1])?;
            env.set(alias, exports)?;
            Ok(Value::Unit)
        }
    });

    builtin_many(env, "require", |env, args| {
        arity("require", args, 1)?;
        let rel = expect_string_in("require", &args[0])?;
        let base_dir = env.get("__dir__").and_then(|v| {
            if let Value::String(s) = v {
                Some(s)
            } else {
                None
            }
        });
        let abs = resolve_path(base_dir, &rel);
        load_module_from_path(&abs)
    });

    // matematica extra
    builtin_many(env, "sqrt", |_env, args| {
        arity("sqrt", args, 1)?;
        Ok(Value::Number(expect_number_in("sqrt", &args[0])?.sqrt()))
    });
    builtin_many(env, "pow", |_env, args| {
        arity("pow", args, 2)?;
        Ok(Value::Number(
            expect_number(&args[0])?.powf(expect_number(&args[1])?),
        ))
    });
    builtin_many(env, "exp", |_env, args| {
        arity("exp", args, 1)?;
        Ok(Value::Number(expect_number_in("exp", &args[0])?.exp()))
    });
    builtin_many(env, "ln", |_env, args| {
        arity("ln", args, 1)?;
        Ok(Value::Number(expect_number_in("ln", &args[0])?.ln()))
    });
    builtin_many(env, "log10", |_env, args| {
        arity("log10", args, 1)?;
        Ok(Value::Number(expect_number_in("log10", &args[0])?.log10()))
    });
    builtin_many(env, "abs", |_env, args| {
        arity("abs", args, 1)?;
        Ok(Value::Number(expect_number_in("abs", &args[0])?.abs()))
    });
    builtin_many(env, "ceil", |_env, args| {
        arity("ceil", args, 1)?;
        Ok(Value::Number(expect_number_in("ceil", &args[0])?.ceil()))
    });
    builtin_many(env, "round", |_env, args| {
        arity("round", args, 1)?;
        Ok(Value::Number(expect_number_in("round", &args[0])?.round()))
    });
    builtin_many(env, "trunc", |_env, args| {
        arity("trunc", args, 1)?;
        Ok(Value::Number(expect_number_in("trunc", &args[0])?.trunc()))
    });
    builtin_many(env, "tan", |_env, args| {
        arity("tan", args, 1)?;
        Ok(Value::Number(expect_number_in("tan", &args[0])?.tan()))
    });
    builtin_many(env, "asin", |_env, args| {
        arity("asin", args, 1)?;
        Ok(Value::Number(expect_number_in("asin", &args[0])?.asin()))
    });
    builtin_many(env, "acos", |_env, args| {
        arity("acos", args, 1)?;
        Ok(Value::Number(expect_number_in("acos", &args[0])?.acos()))
    });
    builtin_many(env, "atan", |_env, args| {
        arity("atan", args, 1)?;
        Ok(Value::Number(expect_number_in("atan", &args[0])?.atan()))
    });
    builtin_many(env, "atan2", |_env, args| {
        arity("atan2", args, 2)?;
        Ok(Value::Number(
            expect_number(&args[0])?.atan2(expect_number(&args[1])?),
        ))
    });

    // clamp(x, lo, hi)
    builtin_many(env, "clamp", |_env, args| {
        arity("clamp", args, 3)?;
        let x = expect_number(&args[0])?;
        let lo = expect_number(&args[1])?;
        let hi = expect_number(&args[2])?;
        let res = if x < lo {
            lo
        } else if x > hi {
            hi
        } else {
            x
        };
        Ok(Value::Number(res))
    });

    // vetores
    fn zip_same_len<T>(a: &[T], b: &[T], name: &str) -> Result<(), GambaError> {
        if a.len() != b.len() {
            return Err(GambaError::runtime(format!(
                "{}: listas com tamanhos diferentes",
                name
            )));
        }
        Ok(())
    }

    builtin_many(env, "vec_add", |_env, args| {
        arity("vec_add", args, 2)?;
        let a = expect_num_list_in("vec_add", &args[0])?;
        let b = expect_num_list_in("vec_add", &args[1])?;
        zip_same_len(&a, &b, "vec_add")?;
        let mut result = Vec::with_capacity(a.len());
        for (x, y) in a.iter().zip(b.iter()) {
            result.push(Value::Number(x + y));
        }
        Ok(Value::List(Arc::new(result)))
    });
    builtin_many(env, "vec_sub", |_env, args| {
        arity("vec_sub", args, 2)?;
        let a = expect_num_list_in("vec_sub", &args[0])?;
        let b = expect_num_list_in("vec_sub", &args[1])?;
        zip_same_len(&a, &b, "vec_sub")?;
        let mut result = Vec::with_capacity(a.len());
        for (x, y) in a.iter().zip(b.iter()) {
            result.push(Value::Number(x - y));
        }
        Ok(Value::List(Arc::new(result)))
    });
    builtin_many(env, "vec_mul", |_env, args| {
        arity("vec_mul", args, 2)?;
        let a = expect_num_list_in("vec_mul", &args[0])?;
        let b = expect_num_list_in("vec_mul", &args[1])?;
        zip_same_len(&a, &b, "vec_mul")?;
        let mut result = Vec::with_capacity(a.len());
        for (x, y) in a.iter().zip(b.iter()) {
            result.push(Value::Number(x * y));
        }
        Ok(Value::List(Arc::new(result)))
    });
    builtin_many(env, "vec_div", |_env, args| {
        arity("vec_div", args, 2)?;
        let a = expect_num_list_in("vec_div", &args[0])?;
        let b = expect_num_list_in("vec_div", &args[1])?;
        zip_same_len(&a, &b, "vec_div")?;
        if b.iter().any(|&y| y == 0.0) {
            return Err(GambaError::runtime(
                "vec_div: divisão por zero em elemento do vetor",
            ));
        }
        let mut result = Vec::with_capacity(a.len());
        for (x, y) in a.iter().zip(b.iter()) {
            result.push(Value::Number(x / y));
        }
        Ok(Value::List(Arc::new(result)))
    });
    builtin_many(env, "vec_scale", |_env, args| {
        arity("vec_scale", args, 2)?;
        let a = expect_num_list_in("vec_scale", &args[0])?;
        let s = expect_number_in("vec_scale", &args[1])?;
        let mut result = Vec::with_capacity(a.len());
        for x in a.iter() {
            result.push(Value::Number(x * s));
        }
        Ok(Value::List(Arc::new(result)))
    });
    builtin_many(env, "vec_dot", |_env, args| {
        arity("vec_dot", args, 2)?;
        let a = expect_num_list_in("vec_dot", &args[0])?;
        let b = expect_num_list_in("vec_dot", &args[1])?;
        zip_same_len(&a, &b, "vec_dot")?;
        let mut acc = 0.0;
        for i in 0..a.len() {
            acc += a[i] * b[i];
        }
        Ok(Value::Number(acc))
    });
    builtin_many(env, "vec_sum", |_env, args| {
        arity("vec_sum", args, 1)?;
        let a = expect_num_list_in("vec_sum", &args[0])?;
        Ok(Value::Number(a.iter().sum()))
    });
    builtin_many(env, "vec_mean", |_env, args| {
        arity("vec_mean", args, 1)?;
        let a = expect_num_list_in("vec_mean", &args[0])?;
        if a.is_empty() {
            return Err(GambaError::runtime("vec_mean: lista vazia"));
        }
        Ok(Value::Number(a.iter().sum::<f64>() / (a.len() as f64)))
    });
    builtin_many(env, "vec_norm", |_env, args| {
        arity("vec_norm", args, 1)?;
        let a = expect_num_list_in("vec_norm", &args[0])?;
        Ok(Value::Number(a.iter().map(|x| x * x).sum::<f64>().sqrt()))
    });
    builtin_many(env, "vec_linspace", |_env, args| {
        arity("vec_linspace", args, 3)?;
        let start = expect_number(&args[0])?;
        let end = expect_number(&args[1])?;
        let n = expect_number(&args[2])?;
        if n <= 1.0 || n.fract() != 0.0 {
            return Err(GambaError::runtime("vec_linspace: n deve ser inteiro > 1"));
        }
        let n = n as usize;
        let step = (end - start) / ((n - 1) as f64);
        let mut v = Vec::with_capacity(n);
        for i in 0..n {
            v.push(Value::Number(start + (i as f64) * step));
        }
        Ok(Value::List(Arc::new(v)))
    });

    // par_map com chunking
    builtin_many(env, "par_map", |env, args| {
        arity("par_map", args, 2)?;
        let xs_ref = expect_list_ref("par_map", &args[0])?;
        let f_impl = expect_func(&args[1])?;
        let func_val = Value::Func(f_impl);

        let n = xs_ref.len();
        if n == 0 {
            return Ok(Value::List(Arc::new(vec![])));
        }

        let threads = std::thread::available_parallelism()
            .map(|x| x.get())
            .unwrap_or(4);
        let chunks = threads * 4;
        let chunk_size = (n + chunks - 1) / chunks;

        let (tx, rx) = mpsc::channel();
        for (chunk_idx, chunk) in xs_ref.chunks(chunk_size).enumerate() {
            let tx = tx.clone();
            let env_c = env.clone();
            let func_c = func_val.clone();
            let base = chunk_idx * chunk_size;
            let items: Vec<(usize, Value)> = chunk
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, v)| (base + i, v))
                .collect();

            crate::scheduler::pool().spawn(Box::new(move || {
                let mut out: Vec<(usize, Result<Value, GambaError>)> =
                    Vec::with_capacity(items.len());
                for (i, v) in items {
                    let r = interpreter::call_value(&env_c, func_c.clone(), vec![v]);
                    out.push((i, r));
                }
                let _ = tx.send(out);
            }));
        }
        drop(tx);

        let mut results: Vec<Option<Value>> = vec![None; n];
        for part in rx {
            for (i, r) in part {
                match r {
                    Ok(v) => results[i] = Some(v),
                    Err(e) => return Err(e),
                }
            }
        }
        Ok(Value::List(Arc::new(
            results.into_iter().map(|o| o.unwrap()).collect(),
        )))
    });

    // par_filter com chunking
    builtin_many(env, "par_filter", |env, args| {
        arity("par_filter", args, 2)?;
        let xs_ref = expect_list_ref("par_filter", &args[0])?;
        let f_impl = expect_func(&args[1])?;
        let func_val = Value::Func(f_impl);

        let n = xs_ref.len();
        if n == 0 {
            return Ok(Value::List(Arc::new(vec![])));
        }

        let threads = std::thread::available_parallelism()
            .map(|x| x.get())
            .unwrap_or(4);
        let chunks = threads * 4;
        let chunk_size = (n + chunks - 1) / chunks;

        let (tx, rx) = mpsc::channel();
        for (chunk_idx, chunk) in xs_ref.chunks(chunk_size).enumerate() {
            let tx = tx.clone();
            let env_c = env.clone();
            let func_c = func_val.clone();
            let base = chunk_idx * chunk_size;
            let items: Vec<(usize, Value)> = chunk
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, v)| (base + i, v))
                .collect();

            crate::scheduler::pool().spawn(Box::new(move || {
                let mut kept: Vec<(usize, Value)> = Vec::new();
                for (i, v) in items {
                    match interpreter::call_value(&env_c, func_c.clone(), vec![v.clone()]) {
                        Ok(rv) => match rv {
                            Value::Bool(true) => kept.push((i, v)),
                            Value::Bool(false) => {}
                            _ => {
                                let _ = tx.send(Err(GambaError::runtime(
                                    "par_filter: função deve retornar Bool",
                                )));
                                return;
                            }
                        },
                        Err(e) => {
                            let _ = tx.send(Err(e));
                            return;
                        }
                    }
                }
                let _ = tx.send(Ok(kept));
            }));
        }
        drop(tx);

        let mut kept_all: Vec<(usize, Value)> = Vec::new();
        for msg in rx {
            let mut part = msg?;
            kept_all.append(&mut part);
        }
        kept_all.sort_by_key(|(i, _)| *i);
        Ok(Value::List(Arc::new(
            kept_all.into_iter().map(|(_, v)| v).collect(),
        )))
    });

    // par_reduce: reduce paralelo com chunking
    builtin_many(env, "par_reduce", |env, args| {
        arity("par_reduce", args, 3)?;
        let xs = expect_list_in("par_reduce", &args[0])?;
        let init = args[1].clone();
        let f_impl = expect_func(&args[2])?;
        let func_val = Value::Func(f_impl.clone());

        let n = xs.len();
        if n == 0 {
            return Ok(init);
        }
        let threads = std::thread::available_parallelism()
            .map(|x| x.get())
            .unwrap_or(4);
        let chunks = threads * 4;
        let chunk_size = (n + chunks - 1) / chunks;
        if chunk_size == 0 {
            return Ok(init);
        }

        let (tx, rx) = mpsc::channel();
        for chunk in xs.chunks(chunk_size) {
            let tx = tx.clone();
            let env_c = env.clone();
            let func_c = func_val.clone();
            let items = chunk.to_vec();
            let init0 = init.clone();
            crate::scheduler::pool().spawn(Box::new(move || {
                let mut acc = init0;
                for v in items {
                    match interpreter::call_value(&env_c, func_c.clone(), vec![acc, v]) {
                        Ok(next) => acc = next,
                        Err(e) => {
                            let _ = tx.send(Err(e));
                            return;
                        }
                    }
                }
                let _ = tx.send(Ok(acc));
            }));
        }
        drop(tx);

        let mut acc = init.clone();
        for res in rx {
            let part = res?;
            acc = interpreter::call_value(env, Value::Func(f_impl.clone()), vec![acc, part])?;
        }
        Ok(acc)
    });

    // par_each: executa efeitos em paralelo
    builtin_many(env, "par_each", |env, args| {
        arity("par_each", args, 2)?;
        let xs = expect_list_in("par_each", &args[0])?;
        let f_impl = expect_func(&args[1])?;
        let func_val = Value::Func(f_impl);

        let (tx, rx) = mpsc::channel();
        let mut count = 0usize;
        for v in xs {
            count += 1;
            let tx = tx.clone();
            let env_c = env.clone();
            let func_c = func_val.clone();
            crate::scheduler::pool().spawn(Box::new(move || {
                let res = interpreter::call_value(&env_c, func_c, vec![v]);
                let _ = tx.send(res.map(|_| ()));
            }));
        }
        drop(tx);

        for _ in 0..count {
            rx.recv()
                .map_err(|_| GambaError::runtime("par_each: falha na coleta"))??;
        }
        Ok(Value::Unit)
    });

    // fork_join: executa multiplas funçoes em paralelo
    builtin_many(env, "fork_join", |env, args| {
        arity("fork_join", args, 1)?;
        let callables = expect_list_in("fork_join", &args[0])?;
        let n = callables.len();
        let (tx, rx) = mpsc::channel();

        for (i, c) in callables.into_iter().enumerate() {
            let func = expect_func(&c)?;
            let val = Value::Func(func);
            let tx = tx.clone();
            let env_c = env.clone();
            crate::scheduler::pool().spawn(Box::new(move || {
                let res = interpreter::call_value(&env_c, val, vec![]);
                let _ = tx.send((i, res));
            }));
        }
        drop(tx);

        let mut results: Vec<Option<Value>> = vec![None; n];
        for _ in 0..n {
            let (i, r) = rx
                .recv()
                .map_err(|_| GambaError::runtime("fork_join: falha"))?;
            results[i] = Some(r?);
        }
        Ok(Value::List(Arc::new(
            results.into_iter().map(|o| o.unwrap()).collect(),
        )))
    });

    // spawn/join no pool (nao cria thread por tarefa)
    static NEXT_HANDLE_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

    enum JoinState {
        Pending(mpsc::Receiver<Result<Value, GambaError>>),
        Done(Result<Value, GambaError>),
    }
    struct JoinEntry {
        state: JoinState,
    }

    static JOINS: std::sync::OnceLock<std::sync::Mutex<HashMap<u64, JoinEntry>>> =
        std::sync::OnceLock::new();
    fn joins() -> &'static std::sync::Mutex<HashMap<u64, JoinEntry>> {
        JOINS.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
    }

    fn spawn_in_pool<F>(job: F) -> u64
    where
        F: 'static + Send + FnOnce() -> Result<Value, GambaError>,
    {
        let (tx, rx) = mpsc::channel();
        crate::scheduler::pool().spawn(Box::new(move || {
            let r = job();
            let _ = tx.send(r);
        }));
        let id = NEXT_HANDLE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        joins().lock().unwrap().insert(
            id,
            JoinEntry {
                state: JoinState::Pending(rx),
            },
        );
        id
    }

    builtin_many(env, "spawn", |env, args| {
        if args.len() != 1 && args.len() != 2 {
            return Err(GambaError::runtime(
                "spawn: use spawn(fn) ou spawn(fn, [args...])",
            ));
        }
        let f_impl = expect_func(&args[0])?;
        let func_val = Value::Func(f_impl);
        let argv = if args.len() == 2 {
            expect_list_in("spawn", &args[1])?
        } else {
            vec![]
        };

        let env_c = env.clone();
        let id = spawn_in_pool(move || interpreter::call_value(&env_c, func_val, argv));
        // retorna Handle exato (sem perda com f64)
        Ok(Value::Handle(id))
    });

    // join unificado >:)
    // join(list, sep) -> String  (concatenaçao com separador)
    // join(handle)    -> Value   (aguarda handle de spawn)
    // mantem compat com scripts que ja usam join em ambos sentidos
    builtin_many(env, "join", |_env, args| {
        match args.len() {
            1 => {
                let id = expect_handle_in("join", &args[0])?;
                enum Wait {
                    Pending(mpsc::Receiver<Result<Value, GambaError>>),
                    Done(Result<Value, GambaError>),
                }
                let wait = {
                    let mut g = joins().lock().unwrap();
                    match g.remove(&id) {
                        Some(JoinEntry {
                            state: JoinState::Pending(rx),
                        }) => Wait::Pending(rx),
                        Some(JoinEntry {
                            state: JoinState::Done(r),
                        }) => Wait::Done(r),
                        None => return Err(GambaError::runtime("join: handle inválido")),
                    }
                }; // lock liberado
                match wait {
                    Wait::Pending(rx) => match rx.recv() {
                        Ok(Ok(v)) => Ok(v),
                        Ok(Err(e)) => Err(e),
                        Err(_) => Err(GambaError::runtime("join: cancelado")),
                    },
                    Wait::Done(r) => r,
                }
            }
            2 => {
                // join(list, sep)
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
            }
            _ => Err(GambaError::runtime(
                "join: use join(list, sep) ou join(handle)",
            )),
        }
    });

    // extras uteis: lines/unlines
    builtin_many(env, "lines", |_env, args| {
        arity("lines", args, 1)?;
        let s = expect_string_in("lines", &args[0])?;
        Ok(Value::List(Arc::new(
            s.lines().map(|l| Value::String(l.to_string())).collect(),
        )))
    });
    builtin_many(env, "unlines", |_env, args| {
        arity("unlines", args, 1)?;
        let xs = expect_list_in("unlines", &args[0])?;
        let mut out = String::new();
        for (i, v) in xs.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            out.push_str(&format!("{}", v));
        }
        Ok(Value::String(out))
    });

    static NEXT_CH_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

    struct RxWrap(std::sync::Mutex<mpsc::Receiver<Value>>);
    #[derive(Clone)]
    struct TxWrap(mpsc::Sender<Value>);

    static CH_TX: std::sync::OnceLock<std::sync::Mutex<HashMap<u64, TxWrap>>> =
        std::sync::OnceLock::new();
    static CH_RX: std::sync::OnceLock<std::sync::Mutex<HashMap<u64, std::sync::Arc<RxWrap>>>> =
        std::sync::OnceLock::new();

    fn ch_tx() -> &'static std::sync::Mutex<HashMap<u64, TxWrap>> {
        CH_TX.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
    }
    fn ch_rx() -> &'static std::sync::Mutex<HashMap<u64, std::sync::Arc<RxWrap>>> {
        CH_RX.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
    }

    fn clone_sendable(v: &Value) -> Result<Value, GambaError> {
        match v {
            Value::Func(_) => Err(GambaError::runtime(
                "não e permitido enviar funcoes pelo canal",
            )),
            Value::Task(_) => Err(GambaError::runtime(
                "não e permitido enviar task pelo canal",
            )),
            Value::List(xs) => {
                let mut out = Vec::with_capacity(xs.len());
                for it in xs.iter() {
                    out.push(clone_sendable(it)?);
                }
                Ok(Value::List(Arc::new(out)))
            }
            Value::Map(m) => {
                let mut out = HashMap::new();
                for (k, val) in m.iter() {
                    out.insert(k.clone(), clone_sendable(val)?);
                }
                Ok(Value::Map(Arc::new(out)))
            }
            Value::Screen(s) => Ok(Value::Screen(s.clone())),
            other => Ok(other.clone()),
        }
    }

    builtin_many(env, "chan_new", |_env, _args| {
        let (tx, rx) = mpsc::channel::<Value>();
        let tx_id = NEXT_CH_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let rx_id = NEXT_CH_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        ch_tx().lock().unwrap().insert(tx_id, TxWrap(tx));
        ch_rx().lock().unwrap().insert(
            rx_id,
            std::sync::Arc::new(RxWrap(std::sync::Mutex::new(rx))),
        );
        let mut m = HashMap::new();
        m.insert("tx".to_string(), Value::Number(tx_id as f64));
        m.insert("rx".to_string(), Value::Number(rx_id as f64));
        Ok(Value::Map(Arc::new(m)))
    });

    builtin_many(env, "chan_send", |_env, args| {
        arity("chan_send", args, 2)?;
        let tx_id = expect_number_in("chan_send", &args[0])? as u64;
        let val = clone_sendable(&args[1])?;
        let tx = ch_tx()
            .lock()
            .unwrap()
            .get(&tx_id)
            .cloned()
            .ok_or_else(|| GambaError::runtime("chan_send: tx inválido"))?;
        tx.0.send(val)
            .map_err(|_| GambaError::runtime("chan_send: receptor fechado"))?;
        Ok(Value::Unit)
    });

    builtin_many(env, "chan_recv", |_env, args| {
        arity("chan_recv", args, 1)?;
        let rx_id = expect_number_in("chan_recv", &args[0])? as u64;
        let rx = ch_rx()
            .lock()
            .unwrap()
            .get(&rx_id)
            .cloned()
            .ok_or_else(|| GambaError::runtime("chan_recv: rx inválido"))?;
        let v =
            rx.0.lock()
                .unwrap()
                .recv()
                .map_err(|_| GambaError::runtime("chan_recv: canal fechado"))?;
        Ok(v)
    });

    builtin_many(env, "chan_try_recv", |_env, args| {
        arity("chan_try_recv", args, 1)?;
        let rx_id = expect_number_in("chan_try_recv", &args[0])? as u64;
        let rx = ch_rx()
            .lock()
            .unwrap()
            .get(&rx_id)
            .cloned()
            .ok_or_else(|| GambaError::runtime("chan_try_recv: rx inválido"))?;
        let mut out = HashMap::new();
        match rx.0.lock().unwrap().try_recv() {
            Ok(v) => {
                out.insert("ok".to_string(), Value::Bool(true));
                out.insert("value".to_string(), v);
            }
            Err(mpsc::TryRecvError::Empty) => {
                out.insert("ok".to_string(), Value::Bool(false));
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                out.insert("ok".to_string(), Value::Bool(false));
            }
        }
        Ok(Value::Map(Arc::new(out)))
    });

    // fecha/remover ids dos mapas (sem erro se não existir)
    builtin_many(env, "chan_close_tx", |_env, args| {
        arity("chan_close_tx", args, 1)?;
        let tx_id = expect_number_in("chan_close_tx", &args[0])? as u64;
        ch_tx().lock().unwrap().remove(&tx_id);
        Ok(Value::Unit)
    });
    builtin_many(env, "chan_close_rx", |_env, args| {
        arity("chan_close_rx", args, 1)?;
        let rx_id = expect_number_in("chan_close_rx", &args[0])? as u64;
        ch_rx().lock().unwrap().remove(&rx_id);
        Ok(Value::Unit)
    });

    // tempo monotono
    builtin_many(env, "time", |_env, _args| {
        let start = *START_INSTANT.get_or_init(Instant::now);
        let secs = start.elapsed().as_secs_f64();
        Ok(Value::Number(secs))
    });

    // screen e tui

    // screen_new(w, h) ou screen_new(w, h, fill_char)
    builtin_many(env, "screen_new", |_env, args| {
        if args.len() != 2 && args.len() != 3 {
            return Err(GambaError::runtime(
                "screen_new: use 2 ou 3 args (w, h, fill opcional)",
            ));
        }
        let w = expect_number_in("screen_new", &args[0])?;
        let h = expect_number_in("screen_new", &args[1])?;
        if w <= 0.0 || h <= 0.0 || w.fract() != 0.0 || h.fract() != 0.0 {
            return Err(GambaError::runtime(
                "screen_new: w e h devem ser inteiros > 0",
            ));
        }
        let fill = if args.len() == 3 {
            char_from_str(&expect_string_in("screen_new", &args[2])?)
        } else {
            ' '
        };
        Ok(Value::Screen(screen_new_buf(w as usize, h as usize, fill)))
    });

    builtin_many(env, "screen_size", |_env, args| {
        arity("screen_size", args, 1)?;
        match &args[0] {
            Value::Screen(s) => Ok(Value::List(Arc::new(vec![
                Value::Number(s.w as f64),
                Value::Number(s.h as f64),
            ]))),
            _ => Err(GambaError::runtime("screen_size: esperado screen")),
        }
    });

    builtin_many(env, "screen_get", |_env, args| {
        arity("screen_get", args, 3)?;
        let (x, y) = (
            expect_number_in("screen_get", &args[1])?,
            expect_number_in("screen_get", &args[2])?,
        );
        match &args[0] {
            Value::Screen(s) => {
                if x < 0.0 || y < 0.0 || x.fract() != 0.0 || y.fract() != 0.0 {
                    return Err(GambaError::runtime("screen_get: coords inteiras >= 0"));
                }
                match screen_get_char(s, x as usize, y as usize) {
                    Some(c) => Ok(Value::String(c.to_string())),
                    None => Err(GambaError::runtime("screen_get: fora do range")),
                }
            }
            _ => Err(GambaError::runtime("screen_get: esperado screen")),
        }
    });

    // screen_set(screen, x, y, ch) cow por linha
    builtin_many(env, "screen_set", |_env, args| {
        arity("screen_set", args, 4)?;
        let (x, y) = (
            expect_number_in("screen_set", &args[1])?,
            expect_number_in("screen_set", &args[2])?,
        );
        let ch = char_from_str(&expect_string_in("screen_set", &args[3])?);
        if x < 0.0 || y < 0.0 || x.fract() != 0.0 || y.fract() != 0.0 {
            return Err(GambaError::runtime("screen_set: coords inteiras >= 0"));
        }
        match &args[0] {
            Value::Screen(s) => {
                let (xi, yi) = (x as usize, y as usize);
                if yi >= s.h || xi >= s.w {
                    return Err(GambaError::runtime("screen_set: fora do range"));
                }
                let mut new_row = s.rows[yi].to_vec();
                new_row[xi] = ch;
                Ok(Value::Screen(screen_clone_replace_row(
                    s,
                    yi,
                    Arc::new(new_row),
                )))
            }
            _ => Err(GambaError::runtime("screen_set: esperado screen")),
        }
    });

    // screen_edit(screen, [ [x y ch] ... ]) aplica varios edits com uma so copia por linha
    builtin_many(env, "screen_edit", |_env, args| {
        arity("screen_edit", args, 2)?;
        let edits = expect_list_ref("screen_edit", &args[1])?;
        let s = match &args[0] {
            Value::Screen(sc) => sc.clone(),
            _ => return Err(GambaError::runtime("screen_edit: esperado screen")),
        };

        // map de yi -> linha clonada
        let mut edited_rows: HashMap<usize, Vec<char>> = HashMap::new();

        for e in edits {
            let trip = expect_list_ref("screen_edit: cada edit", e)?;
            if trip.len() != 3 {
                return Err(GambaError::runtime(
                    "screen_edit: cada item deve ser [x y ch]",
                ));
            }
            let x = expect_number(&trip[0])?;
            let y = expect_number(&trip[1])?;
            let ch = char_from_str(&expect_string(&trip[2])?);
            if x < 0.0 || y < 0.0 || x.fract() != 0.0 || y.fract() != 0.0 {
                return Err(GambaError::runtime("screen_edit: coords inteiras >= 0"));
            }
            let (xi, yi) = (x as usize, y as usize);
            if yi >= s.h || xi >= s.w {
                continue; // fora ignora
            }
            let row = edited_rows.entry(yi).or_insert_with(|| s.rows[yi].to_vec());
            row[xi] = ch;
        }

        if edited_rows.is_empty() {
            return Ok(Value::Screen(s));
        }

        let mut rows = s.rows.clone();
        for (yi, row_vec) in edited_rows {
            rows[yi] = Arc::new(row_vec);
        }
        Ok(Value::Screen(Arc::new(ScreenBuf {
            w: s.w,
            h: s.h,
            rows,
        })))
    });

    // screen_blit_text(screen, x, y, text) escreve uma string (sem quebrar linha)
    builtin_many(env, "screen_blit_text", |_env, args| {
        arity("screen_blit_text", args, 4)?;
        let s = match &args[0] {
            Value::Screen(sc) => sc.clone(),
            _ => return Err(GambaError::runtime("screen_blit_text: esperado screen")),
        };
        let (x0, y0) = (
            expect_number_in("screen_blit_text", &args[1])?,
            expect_number_in("screen_blit_text", &args[2])?,
        );
        let text = expect_string_in("screen_blit_text", &args[3])?;
        if x0 < 0.0 || y0 < 0.0 || x0.fract() != 0.0 || y0.fract() != 0.0 {
            return Err(GambaError::runtime(
                "screen_blit_text: coords inteiras >= 0",
            ));
        }
        let (mut xi, yi) = (x0 as usize, y0 as usize);
        if yi >= s.h {
            return Ok(Value::Screen(s));
        }
        let mut row = s.rows[yi].to_vec();
        for ch in text.chars() {
            if xi >= s.w {
                break;
            }
            row[xi] = ch;
            xi += 1;
        }
        Ok(Value::Screen(screen_clone_replace_row(
            &s,
            yi,
            Arc::new(row),
        )))
    });

    // screen_to_string(screen) converte toda a tela em string unica com quebras
    builtin_many(env, "screen_to_string", |_env, args| {
        arity("screen_to_string", args, 1)?;
        match &args[0] {
            Value::Screen(s) => {
                let mut out = String::new();
                for (i, row) in s.rows.iter().enumerate() {
                    if i > 0 {
                        out.push('\n');
                    }
                    out.extend(row.iter());
                }
                Ok(Value::String(out))
            }
            _ => Err(GambaError::runtime("screen_to_string: esperado screen")),
        }
    });

    // tui begin/present/end
    builtin_many(env, "tui_begin", |_env, _args| {
        alt_screen_on();
        hide_cursor();
        clear_all();
        *tui_prev().lock().unwrap() = None;
        Ok(Value::Unit)
    });

    builtin_many(env, "tui_present", |_env, args| {
        arity("tui_present", args, 1)?;
        let s = match &args[0] {
            Value::Screen(sc) => sc.clone(),
            _ => return Err(GambaError::runtime("tui_present: esperado screen")),
        };
        let mut prev = tui_prev().lock().unwrap();
        if let Some(p) = prev.as_ref() {
            render_diff(p, &s);
        } else {
            render_full(&s);
        }
        *prev = Some(s);
        Ok(Value::Unit)
    });

    builtin_many(env, "tui_end", |_env, _args| {
        show_cursor();
        alt_screen_off();
        *tui_prev().lock().unwrap() = None;
        Ok(Value::Unit)
    });

    // math avancada
    builtin_many(env, "deg2rad", |_env, args| {
        arity("deg2rad", args, 1)?;
        Ok(Value::Number(
            expect_number_in("deg2rad", &args[0])? * std::f64::consts::PI / 180.0,
        ))
    });
    builtin_many(env, "rad2deg", |_env, args| {
        arity("rad2deg", args, 1)?;
        Ok(Value::Number(
            expect_number_in("rad2deg", &args[0])? * 180.0 / std::f64::consts::PI,
        ))
    });
    builtin_many(env, "hypot", |_env, args| {
        arity("hypot", args, 2)?;
        Ok(Value::Number(
            expect_number(&args[0])?.hypot(expect_number(&args[1])?),
        ))
    });
    builtin_many(env, "sinh", |_env, args| {
        arity("sinh", args, 1)?;
        Ok(Value::Number(expect_number_in("sinh", &args[0])?.sinh()))
    });
    builtin_many(env, "cosh", |_env, args| {
        arity("cosh", args, 1)?;
        Ok(Value::Number(expect_number_in("cosh", &args[0])?.cosh()))
    });
    builtin_many(env, "tanh", |_env, args| {
        arity("tanh", args, 1)?;
        Ok(Value::Number(expect_number_in("tanh", &args[0])?.tanh()))
    });
    builtin_many(env, "asinh", |_env, args| {
        arity("asinh", args, 1)?;
        Ok(Value::Number(expect_number_in("asinh", &args[0])?.asinh()))
    });
    builtin_many(env, "acosh", |_env, args| {
        arity("acosh", args, 1)?;
        Ok(Value::Number(expect_number_in("acosh", &args[0])?.acosh()))
    });
    builtin_many(env, "atanh", |_env, args| {
        arity("atanh", args, 1)?;
        Ok(Value::Number(expect_number_in("atanh", &args[0])?.atanh()))
    });

    // rand_norm() ~ n(0,1) via box-muller
    builtin_many(env, "rand_norm", |_env, _args| {
        let u1 = (rand_next() as f64 + 1.0) / (u64::MAX as f64 + 2.0);
        let u2 = (rand_next() as f64 + 1.0) / (u64::MAX as f64 + 2.0);
        let r = (-2.0 * u1.ln()).sqrt();
        let z = r * (2.0 * std::f64::consts::PI * u2).cos();
        Ok(Value::Number(z))
    });

    // vetores extra
    builtin_many(env, "vec_normalize", |_env, args| {
        arity("vec_normalize", args, 1)?;
        let a = expect_num_list_in("vec_normalize", &args[0])?;
        let norm = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm == 0.0 {
            return Err(GambaError::runtime("vec_normalize: vetor de norma zero"));
        }
        Ok(Value::List(Arc::new(
            a.iter().map(|x| Value::Number(x / norm)).collect(),
        )))
    });
    builtin_many(env, "vec_distance", |_env, args| {
        arity("vec_distance", args, 2)?;
        let a = expect_num_list_in("vec_distance", &args[0])?;
        let b = expect_num_list_in("vec_distance", &args[1])?;
        if a.len() != b.len() {
            return Err(GambaError::runtime("vec_distance: tamanhos diferentes"));
        }
        let mut acc = 0.0;
        for i in 0..a.len() {
            let d = a[i] - b[i];
            acc += d * d;
        }
        Ok(Value::Number(acc.sqrt()))
    });
    builtin_many(env, "vec_cross", |_env, args| {
        arity("vec_cross", args, 2)?;
        let a = expect_num_list_in("vec_cross", &args[0])?;
        let b = expect_num_list_in("vec_cross", &args[1])?;
        if a.len() != 3 || b.len() != 3 {
            return Err(GambaError::runtime("vec_cross: esperado vetores 3d"));
        }
        let (ax, ay, az) = (a[0], a[1], a[2]);
        let (bx, by, bz) = (b[0], b[1], b[2]);
        Ok(Value::List(Arc::new(vec![
            Value::Number(ay * bz - az * by),
            Value::Number(az * bx - ax * bz),
            Value::Number(ax * by - ay * bx),
        ])))
    });
    builtin_many(env, "vec_proj", |_env, args| {
        arity("vec_proj", args, 2)?;
        let a = expect_num_list_in("vec_proj", &args[0])?;
        let b = expect_num_list_in("vec_proj", &args[1])?;
        if a.len() != b.len() {
            return Err(GambaError::runtime("vec_proj: tamanhos diferentes"));
        }
        let dot_ab: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let dot_bb: f64 = b.iter().map(|y| y * y).sum();
        if dot_bb == 0.0 {
            return Err(GambaError::runtime("vec_proj: vetor-alvo de norma zero"));
        }
        let k = dot_ab / dot_bb;
        Ok(Value::List(Arc::new(
            b.iter().map(|y| Value::Number(k * y)).collect(),
        )))
    });
    builtin_many(env, "vec_min", |_env, args| {
        arity("vec_min", args, 2)?;
        let a = expect_num_list_in("vec_min", &args[0])?;
        let b = expect_num_list_in("vec_min", &args[1])?;
        if a.len() != b.len() {
            return Err(GambaError::runtime("vec_min: tamanhos diferentes"));
        }
        Ok(Value::List(Arc::new(
            a.iter()
                .zip(b.iter())
                .map(|(x, y)| Value::Number(x.min(*y)))
                .collect(),
        )))
    });
    builtin_many(env, "vec_max", |_env, args| {
        arity("vec_max", args, 2)?;
        let a = expect_num_list_in("vec_max", &args[0])?;
        let b = expect_num_list_in("vec_max", &args[1])?;
        if a.len() != b.len() {
            return Err(GambaError::runtime("vec_max: tamanhos diferentes"));
        }
        Ok(Value::List(Arc::new(
            a.iter()
                .zip(b.iter())
                .map(|(x, y)| Value::Number(x.max(*y)))
                .collect(),
        )))
    });
    builtin_many(env, "vec_clamp_each", |_env, args| {
        arity("vec_clamp_each", args, 3)?;
        let a = expect_num_list_in("vec_clamp_each", &args[0])?;
        let lo = expect_number_in("vec_clamp_each", &args[1])?;
        let hi = expect_number_in("vec_clamp_each", &args[2])?;
        Ok(Value::List(Arc::new(
            a.iter()
                .map(|x| {
                    let y = if *x < lo {
                        lo
                    } else if *x > hi {
                        hi
                    } else {
                        *x
                    };
                    Value::Number(y)
                })
                .collect(),
        )))
    });

    // hashmap extras
    builtin_many(env, "map_remove", |_env, args| {
        arity("map_remove", args, 2)?;
        let m = expect_map_arc_in("map_remove", &args[0])?;
        let k = expect_string_in("map_remove", &args[1])?;
        let mut out = (*m).clone();
        out.remove(&k);
        Ok(Value::Map(Arc::new(out)))
    });
    builtin_many(env, "map_values", |_env, args| {
        arity("map_values", args, 1)?;
        let m = expect_map_ref("map_values", &args[0])?;
        Ok(Value::List(Arc::new(m.values().cloned().collect())))
    });
    builtin_many(env, "map_entries", |_env, args| {
        arity("map_entries", args, 1)?;
        let m = expect_map_ref("map_entries", &args[0])?;
        let mut ks: Vec<_> = m.keys().cloned().collect();
        ks.sort();
        let mut out = Vec::with_capacity(ks.len());
        for k in ks {
            out.push(Value::List(Arc::new(vec![
                Value::String(k.clone()),
                m.get(&k).unwrap().clone(),
            ])));
        }
        Ok(Value::List(Arc::new(out)))
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
        Value::List(xs) => Ok((*xs).to_vec()),
        _ => Err(GambaError::runtime(format!(
            "esperado lista, mas recebeu {}",
            v.type_name()
        ))),
    }
}
fn expect_list_in(name: &str, v: &Value) -> Result<Vec<Value>, GambaError> {
    match v {
        Value::List(xs) => Ok((*xs).to_vec()),
        _ => Err(GambaError::runtime(format!(
            "{}: esperado List, mas recebeu {}. dica: chame {}(lista, ...) ou use pipe: xs |> {} ...",
            name,
            v.type_name(),
            name,
            name
        ))),
    }
}

// auxiliares de leitura por referencia (evitam clones quando nao precisa escrever)
fn expect_list_ref<'a>(name: &str, v: &'a Value) -> Result<&'a [Value], GambaError> {
    match v {
        Value::List(xs) => Ok(xs.as_slice()),
        _ => Err(GambaError::runtime(format!(
            "{}: esperado List, mas recebeu {}",
            name,
            v.type_name()
        ))),
    }
}
fn expect_map_ref<'a>(name: &str, v: &'a Value) -> Result<&'a HashMap<String, Value>, GambaError> {
    match v {
        Value::Map(m) => Ok(m),
        _ => Err(GambaError::runtime(format!(
            "{}: esperado Map/Dict, mas recebeu {}",
            name,
            v.type_name()
        ))),
    }
}
fn expect_list_arc_in(name: &str, v: &Value) -> Result<Arc<Vec<Value>>, GambaError> {
    match v {
        Value::List(xs) => Ok(xs.clone()),
        _ => Err(GambaError::runtime(format!(
            "{}: esperado List, mas recebeu {}",
            name,
            v.type_name()
        ))),
    }
}
fn expect_map_arc_in(name: &str, v: &Value) -> Result<Arc<HashMap<String, Value>>, GambaError> {
    match v {
        Value::Map(m) => Ok(m.clone()),
        _ => Err(GambaError::runtime(format!(
            "{}: esperado Map/Dict, mas recebeu {}",
            name,
            v.type_name()
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

fn expect_func(v: &Value) -> Result<FuncImpl, GambaError> {
    match v {
        Value::Func(f) => Ok(f.clone()),
        _ => Err(GambaError::runtime("esperado função")),
    }
}
fn expect_map(v: &Value) -> Result<HashMap<String, Value>, GambaError> {
    match v {
        Value::Map(m) => Ok((**m).clone()),
        _ => Err(GambaError::runtime("Esperado map/dict")),
    }
}
fn expect_map_in(name: &str, v: &Value) -> Result<HashMap<String, Value>, GambaError> {
    match v {
        Value::Map(m) => Ok((**m).clone()),
        _ => Err(GambaError::runtime(format!(
            "{}: esperado Map/Dict, mas recebeu {}",
            name,
            v.type_name()
        ))),
    }
}

fn expect_num_list_in(name: &str, v: &Value) -> Result<Vec<f64>, GambaError> {
    let xs = expect_list_in(name, v)?;
    let mut out = Vec::with_capacity(xs.len());
    for it in xs {
        match it {
            Value::Number(n) => out.push(n),
            _ => {
                return Err(GambaError::runtime(format!(
                    "{}: esperado lista de Number, mas encontrou {}",
                    name,
                    it.type_name()
                )))
            }
        }
    }
    Ok(out)
}
