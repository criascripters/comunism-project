#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub items: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    String(String),
    Bool(bool),
    List(Vec<Expr>),
    Ident(String),

    // a :: expr
    Let {
        name: String,
        expr: Box<Expr>,
    },

    // when cond { then } { else }
    When {
        cond: Box<Expr>,
        then_branch: Block,
        else_branch: Block,
    },

    // fn(a b){ body }
    Lambda {
        params: Vec<String>,
        body: Block,
    },

    // f x y z
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },

    // (e)
    Group(Box<Expr>),

    // { ... }
    Block(Block),

    // binarios
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryMinus(Box<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}
