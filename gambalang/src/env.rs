use std::collections::HashMap;
use std::fmt::{self, Display};
use std::sync::{Arc, RwLock};

use crate::ast::Block;
use crate::error::GambaError;

// buffer interno do screen linhas imutaveis cow por linha
#[derive(Debug)]
pub struct ScreenBuf {
    pub w: usize,
    pub h: usize,
    pub rows: Vec<Arc<Vec<char>>>, // row-major cada linha compartilha internamente
}

#[derive(Clone)]
pub enum FuncImpl {
    Builtin(Arc<dyn Fn(&Env, &[Value]) -> Result<Value, GambaError> + Send + Sync>),
    Lambda(Lambda),
}

#[derive(Clone)]
pub struct Lambda {
    pub params: Vec<String>,
    pub body: Block,
    pub env: Env, // captura léxica
}

#[derive(Clone)]
pub enum Value {
    Number(f64),
    Bool(bool),
    String(String),
    List(Arc<Vec<Value>>),
    Func(FuncImpl),
    Map(Arc<HashMap<String, Value>>),
    Unit,

    // tarefas async (globais por id)
    Task(u64),

    // handle de join (pool) com precisao exata (era f64)
    Handle(u64),

    // tela de tui persistente
    Screen(Arc<ScreenBuf>),
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "Number",
            Value::Bool(_) => "Bool",
            Value::String(_) => "String",
            Value::List(_) => "List",
            Value::Func(_) => "Function",
            Value::Map(_) => "Map",
            Value::Unit => "Unit",
            Value::Task(_) => "Task",
            Value::Handle(_) => "Handle",
            Value::Screen(_) => "Screen",
        }
    }

    // centraliza igualdade estrutural pra reutilizar em interpreter/builtins
    pub fn equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Number(x), Value::Number(y)) => x == y,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::String(x), Value::String(y)) => x == y,
            (Value::Unit, Value::Unit) => true,
            (Value::List(xs), Value::List(ys)) => {
                xs.len() == ys.len() && xs.iter().zip(ys.iter()).all(|(u, v)| u.equals(v))
            }
            (Value::Map(mx), Value::Map(my)) => {
                mx.len() == my.len()
                    && mx
                        .iter()
                        .all(|(k, vx)| my.get(k).map_or(false, |vy| vx.equals(vy)))
            }
            // compara por identidade numérica dos identificadores
            (Value::Task(a), Value::Task(b)) => a == b,
            (Value::Handle(a), Value::Handle(b)) => a == b,
            // igualdade estrutural de tela
            (Value::Screen(a), Value::Screen(b)) => {
                if a.w != b.w || a.h != b.h {
                    return false;
                }
                // rapido tenta pointer eq de linhas
                if Arc::ptr_eq(a, b) {
                    return true;
                }
                for (ra, rb) in a.rows.iter().zip(b.rows.iter()) {
                    if ra.len() != rb.len() {
                        return false;
                    }
                    if !ra.iter().zip(rb.iter()).all(|(ca, cb)| ca == cb) {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::Bool(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "{}", s),
            Value::List(xs) => {
                write!(f, "[")?;
                let mut first = true;
                for v in xs.as_ref() {
                    if !first {
                        write!(f, " ")?;
                    }
                    first = false;
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Func(_) => write!(f, "<fn>"),
            Value::Map(m) => {
                let mut keys: Vec<_> = m.keys().cloned().collect();
                keys.sort();
                write!(f, "{{")?;
                for (i, k) in keys.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}: {}", k, m.get(k).unwrap())?;
                }
                write!(f, "}}")
            }
            Value::Unit => write!(f, "()"),
            Value::Task(id) => write!(f, "<task {}>", id),
            Value::Handle(id) => write!(f, "<handle {}>", id),
            Value::Screen(s) => write!(f, "<screen {}x{}>", s.w, s.h),
        }
    }
}

#[derive(Clone)]
pub struct Env(Arc<EnvInner>);

struct EnvInner {
    parent: Option<Env>,
    values: RwLock<HashMap<String, Value>>,
}

impl Env {
    pub fn new() -> Env {
        Env(Arc::new(EnvInner {
            parent: None,
            values: RwLock::new(HashMap::new()),
        }))
    }

    pub fn child(&self) -> Env {
        Env(Arc::new(EnvInner {
            parent: Some(self.clone()),
            values: RwLock::new(HashMap::new()),
        }))
    }

    // permite rebind no mesmo escopo (estilo rust let-shadow), reduz atrito no REPL e scripts
    pub fn set(&self, name: String, val: Value) -> Result<(), GambaError> {
        self.0.values.write().unwrap().insert(name, val);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.0.values.read().unwrap().get(name) {
            return Some(v.clone());
        }
        if let Some(ref p) = self.0.parent {
            return p.get(name);
        }
        None
    }

    // verifica se um nome existe no escopo atual (nao olha parent)
    // util pra decidir se um let é primeira definição (predeclare) ou rebind
    pub fn contains_local(&self, name: &str) -> bool {
        self.0.values.read().unwrap().contains_key(name)
    }

    // pre declara um nome com Unit no escopo atual (pra let rec/forward ref)
    // seguro e O(1)
    pub fn predeclare(&self, name: String) {
        self.0.values.write().unwrap().insert(name, Value::Unit);
    }

    // procura um nome no escopo atual ou em algum ancestral
    pub fn contains_any(&self, name: &str) -> bool {
        if self.0.values.read().unwrap().contains_key(name) {
            return true;
        }
        if let Some(ref p) = self.0.parent {
            return p.contains_any(name);
        }
        false
    }

    // define um valor no escopo onde o nome já existe (subindo a cadeia)
    // se não existir em lugar nenhum, define no escopo atual
    pub fn set_deep(&self, name: String, val: Value) -> Result<(), GambaError> {
        if self.0.values.read().unwrap().contains_key(&name) {
            self.0.values.write().unwrap().insert(name, val);
            return Ok(());
        }
        if let Some(ref p) = self.0.parent {
            // tenta setar no pai; se não existir no pai, continuará subindo
            return p.set_deep(name, val);
        }
        // não existe em nenhum lugar (segurança): set no local
        self.set(name, val)
    }
}

pub struct Runtime {
    pub env: Env,
}

impl Runtime {
    pub fn new() -> Self {
        Self { env: Env::new() }
    }
    pub fn with_builtins() -> Self {
        let rt = Self::new();
        crate::builtins::install_builtins(&rt.env);
        rt
    }
}
