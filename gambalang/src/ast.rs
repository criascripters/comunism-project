#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub items: Vec<Expr>,
}

#[derive(Debug, Clone, Copy)]
pub enum CallStyle {
    Parens,   // f(a, b)
    Juxta,    // f a b
    Pipe,     // x |> f(a)  ou  x |> f a
    Internal, // chamadas geradas pelo runtime/builtins (sem origem lexical direta)
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
        // posição do call site; ajuda a apontar erros de runtime na chamada
        call_line: usize,
        call_col: usize,
        style: CallStyle,
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

    UnaryNot(Box<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Gt,
    Ge,
    Lt,
    Le,
    Eq,

    And, // &&
    Or,  // ||
}
