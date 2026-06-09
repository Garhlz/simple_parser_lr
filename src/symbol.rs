use std::fmt;

/// 终结符：由 lexer 产生，也作为 ACTION 表的列。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Terminal {
    Id,
    Num,

    Let,
    If,
    Else,
    While,

    True,
    False,
    And,
    Or,
    Not,

    Plus,
    Minus,
    Star,
    Slash,

    Assign,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    LParen,
    RParen,
    LBrace,
    RBrace,

    Semicolon,

    /// 输入结束符，显示为 `#`
    End,
}

/// 非终结符：作为产生式左部，也作为 GOTO 表的列。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NonTerminal {
    /// 拓广开始符号：Program' -> Program
    AugmentedStart,

    Program,
    StmtList,
    Statement,
    Block,

    SimpleStmt,
    DeclStmt,
    DeclInit,
    AssignStmt,

    IfStmt,
    ElsePart,
    ElseBody,
    WhileStmt,

    Expr,
    Term,
    Factor,

    BoolExpr,
    BoolAnd,
    BoolNot,
    BoolAtom,

    Relation,
    RelOp,

    Variable,
}

/// 通用文法符号：终结符、非终结符、ε。
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Symbol {
    Terminal(Terminal),
    NonTerminal(NonTerminal),
    Epsilon,
}

impl Terminal {
    pub const fn all() -> &'static [Terminal] {
        &[
            Terminal::Id,
            Terminal::Num,
            Terminal::Let,
            Terminal::If,
            Terminal::Else,
            Terminal::While,
            Terminal::True,
            Terminal::False,
            Terminal::And,
            Terminal::Or,
            Terminal::Not,
            Terminal::Plus,
            Terminal::Minus,
            Terminal::Star,
            Terminal::Slash,
            Terminal::Assign,
            Terminal::Eq,
            Terminal::Ne,
            Terminal::Lt,
            Terminal::Le,
            Terminal::Gt,
            Terminal::Ge,
            Terminal::LParen,
            Terminal::RParen,
            Terminal::LBrace,
            Terminal::RBrace,
            Terminal::Semicolon,
            Terminal::End,
        ]
    }
}

impl NonTerminal {
    pub const fn all() -> &'static [NonTerminal] {
        &[
            NonTerminal::AugmentedStart,
            NonTerminal::Program,
            NonTerminal::StmtList,
            NonTerminal::Statement,
            NonTerminal::Block,
            NonTerminal::SimpleStmt,
            NonTerminal::DeclStmt,
            NonTerminal::DeclInit,
            NonTerminal::AssignStmt,
            NonTerminal::IfStmt,
            NonTerminal::ElsePart,
            NonTerminal::ElseBody,
            NonTerminal::WhileStmt,
            NonTerminal::Expr,
            NonTerminal::Term,
            NonTerminal::Factor,
            NonTerminal::BoolExpr,
            NonTerminal::BoolAnd,
            NonTerminal::BoolNot,
            NonTerminal::BoolAtom,
            NonTerminal::Relation,
            NonTerminal::RelOp,
            NonTerminal::Variable,
        ]
    }
}

impl From<Terminal> for Symbol {
    fn from(value: Terminal) -> Self {
        Symbol::Terminal(value)
    }
}

impl From<NonTerminal> for Symbol {
    fn from(value: NonTerminal) -> Self {
        Symbol::NonTerminal(value)
    }
}

impl fmt::Display for Terminal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Terminal::Id => "id",
            Terminal::Num => "num",

            Terminal::Let => "let",
            Terminal::If => "if",
            Terminal::Else => "else",
            Terminal::While => "while",

            Terminal::True => "true",
            Terminal::False => "false",
            Terminal::And => "and",
            Terminal::Or => "or",
            Terminal::Not => "not",

            Terminal::Plus => "+",
            Terminal::Minus => "-",
            Terminal::Star => "*",
            Terminal::Slash => "/",

            Terminal::Assign => "=",
            Terminal::Eq => "==",
            Terminal::Ne => "!=",
            Terminal::Lt => "<",
            Terminal::Le => "<=",
            Terminal::Gt => ">",
            Terminal::Ge => ">=",

            Terminal::LParen => "(",
            Terminal::RParen => ")",
            Terminal::LBrace => "{",
            Terminal::RBrace => "}",

            Terminal::Semicolon => ";",

            Terminal::End => "#",
        };

        write!(f, "{text}")
    }
}

impl fmt::Display for NonTerminal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            NonTerminal::AugmentedStart => "Program'",
            NonTerminal::Program => "Program",
            NonTerminal::StmtList => "StmtList",
            NonTerminal::Statement => "Statement",
            NonTerminal::Block => "Block",
            NonTerminal::SimpleStmt => "SimpleStmt",
            NonTerminal::DeclStmt => "DeclStmt",
            NonTerminal::DeclInit => "DeclInit",
            NonTerminal::AssignStmt => "AssignStmt",
            NonTerminal::IfStmt => "IfStmt",
            NonTerminal::ElsePart => "ElsePart",
            NonTerminal::ElseBody => "ElseBody",
            NonTerminal::WhileStmt => "WhileStmt",
            NonTerminal::Expr => "Expr",
            NonTerminal::Term => "Term",
            NonTerminal::Factor => "Factor",
            NonTerminal::BoolExpr => "BoolExpr",
            NonTerminal::BoolAnd => "BoolAnd",
            NonTerminal::BoolNot => "BoolNot",
            NonTerminal::BoolAtom => "BoolAtom",
            NonTerminal::Relation => "Relation",
            NonTerminal::RelOp => "RelOp",
            NonTerminal::Variable => "Variable",
        };

        write!(f, "{text}")
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Symbol::Terminal(t) => write!(f, "{t}"),
            Symbol::NonTerminal(nt) => write!(f, "{nt}"),
            Symbol::Epsilon => write!(f, "ε"),
        }
    }
}
