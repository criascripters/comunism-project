use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{self, Display};
use std::rc::Rc;

use crate::ast::Block;
use crate::error::GambaError;

#[derive(Clone)]
pub enum FuncImpl {
    Builtin(Rc<dyn Fn(&Env, &[Value]) -> Result<Value, GambaError>>),
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
    List(Vec<Value>),
    Func(FuncImpl),
    Map(std::collections::HashMap<String, Value>),
    Unit,
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
                for v in xs {
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
        }
    }
}

#[derive(Clone)]
pub struct Env(Rc<EnvInner>);

struct EnvInner {
    parent: Option<Env>,
    values: RefCell<HashMap<String, Value>>,
}

impl Env {
    pub fn new() -> Env {
        Env(Rc::new(EnvInner {
            parent: None,
            values: RefCell::new(HashMap::new()),
        }))
    }

    pub fn child(&self) -> Env {
        Env(Rc::new(EnvInner {
            parent: Some(self.clone()),
            values: RefCell::new(HashMap::new()),
        }))
    }

    // permite rebind no mesmo escopo (estilo rust let-shadow), reduz atrito no REPL e scripts
    pub fn set(&self, name: String, val: Value) -> Result<(), GambaError> {
        self.0.values.borrow_mut().insert(name, val);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.0.values.borrow().get(name) {
            return Some(v.clone());
        }
        if let Some(ref p) = self.0.parent {
            return p.get(name);
        }
        None
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
