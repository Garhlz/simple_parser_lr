use crate::symbol::{NonTerminal, Symbol, Terminal};
use std::fmt;

/// 产生式：一条文法规则。
///
/// 约定：
/// - `rhs.is_empty()` 表示 ε
/// - `production_id` 直接使用 `Grammar::productions` 中的下标
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Production {
    pub lhs: NonTerminal,
    pub rhs: Vec<Symbol>,
}

/// 文法：产生式集合 + 开始符号。
#[derive(Debug, Clone)]
pub struct Grammar {
    pub name: String,
    pub start: NonTerminal,
    pub productions: Vec<Production>,
}

impl Production {
    pub fn new(lhs: NonTerminal, rhs: Vec<Symbol>) -> Self {
        Self { lhs, rhs }
    }

    pub fn rhs_len(&self) -> usize {
        self.rhs.len()
    }
}

impl Grammar {
    /// Simple 左递归文法，用于 LR(0)、SLR(1)、LR(1) 实验。
    ///
    /// 注意：
    /// - 该文法保留直接左递归；
    /// - 不包含 LL(1) 改造后的 tail 非终结符；
    /// - `productions[0]` 是拓广产生式 `Program' -> Program`。
    pub fn simple_lr() -> Self {
        let productions = vec![
            // 0. Program' → Program
            prod(NonTerminal::AugmentedStart, vec![nt(NonTerminal::Program)]),
            // 1. Program → StmtList
            prod(NonTerminal::Program, vec![nt(NonTerminal::StmtList)]),
            // 2-3. StmtList → StmtList Statement | Statement
            prod(
                NonTerminal::StmtList,
                vec![nt(NonTerminal::StmtList), nt(NonTerminal::Statement)],
            ),
            prod(NonTerminal::StmtList, vec![nt(NonTerminal::Statement)]),
            // 4-7. Statement → SimpleStmt ; | IfStmt | WhileStmt | Block
            prod(
                NonTerminal::Statement,
                vec![nt(NonTerminal::SimpleStmt), t(Terminal::Semicolon)],
            ),
            prod(NonTerminal::Statement, vec![nt(NonTerminal::IfStmt)]),
            prod(NonTerminal::Statement, vec![nt(NonTerminal::WhileStmt)]),
            prod(NonTerminal::Statement, vec![nt(NonTerminal::Block)]),
            // 8-9. Block → { StmtList } | { }
            prod(
                NonTerminal::Block,
                vec![
                    t(Terminal::LBrace),
                    nt(NonTerminal::StmtList),
                    t(Terminal::RBrace),
                ],
            ),
            prod(
                NonTerminal::Block,
                vec![t(Terminal::LBrace), t(Terminal::RBrace)],
            ),
            // 10-11. SimpleStmt → DeclStmt | AssignStmt
            prod(NonTerminal::SimpleStmt, vec![nt(NonTerminal::DeclStmt)]),
            prod(NonTerminal::SimpleStmt, vec![nt(NonTerminal::AssignStmt)]),
            // 12. DeclStmt → let Variable DeclInit
            prod(
                NonTerminal::DeclStmt,
                vec![
                    t(Terminal::Let),
                    nt(NonTerminal::Variable),
                    nt(NonTerminal::DeclInit),
                ],
            ),
            // 13-14. DeclInit → = Expr | ε
            prod(
                NonTerminal::DeclInit,
                vec![t(Terminal::Assign), nt(NonTerminal::Expr)],
            ),
            prod(NonTerminal::DeclInit, vec![]),
            // 15. AssignStmt → Variable = Expr
            prod(
                NonTerminal::AssignStmt,
                vec![
                    nt(NonTerminal::Variable),
                    t(Terminal::Assign),
                    nt(NonTerminal::Expr),
                ],
            ),
            // 16. IfStmt → if ( BoolExpr ) Block ElsePart
            prod(
                NonTerminal::IfStmt,
                vec![
                    t(Terminal::If),
                    t(Terminal::LParen),
                    nt(NonTerminal::BoolExpr),
                    t(Terminal::RParen),
                    nt(NonTerminal::Block),
                    nt(NonTerminal::ElsePart),
                ],
            ),
            // 17-18. ElsePart → else ElseBody | ε
            prod(
                NonTerminal::ElsePart,
                vec![t(Terminal::Else), nt(NonTerminal::ElseBody)],
            ),
            prod(NonTerminal::ElsePart, vec![]),
            // 19-20. ElseBody → Block | IfStmt
            prod(NonTerminal::ElseBody, vec![nt(NonTerminal::Block)]),
            prod(NonTerminal::ElseBody, vec![nt(NonTerminal::IfStmt)]),
            // 21. WhileStmt → while ( BoolExpr ) Block
            prod(
                NonTerminal::WhileStmt,
                vec![
                    t(Terminal::While),
                    t(Terminal::LParen),
                    nt(NonTerminal::BoolExpr),
                    t(Terminal::RParen),
                    nt(NonTerminal::Block),
                ],
            ),
            // 22-24. Expr → Expr + Term | Expr - Term | Term
            prod(
                NonTerminal::Expr,
                vec![
                    nt(NonTerminal::Expr),
                    t(Terminal::Plus),
                    nt(NonTerminal::Term),
                ],
            ),
            prod(
                NonTerminal::Expr,
                vec![
                    nt(NonTerminal::Expr),
                    t(Terminal::Minus),
                    nt(NonTerminal::Term),
                ],
            ),
            prod(NonTerminal::Expr, vec![nt(NonTerminal::Term)]),
            // 25-27. Term → Term * Factor | Term / Factor | Factor
            prod(
                NonTerminal::Term,
                vec![
                    nt(NonTerminal::Term),
                    t(Terminal::Star),
                    nt(NonTerminal::Factor),
                ],
            ),
            prod(
                NonTerminal::Term,
                vec![
                    nt(NonTerminal::Term),
                    t(Terminal::Slash),
                    nt(NonTerminal::Factor),
                ],
            ),
            prod(NonTerminal::Term, vec![nt(NonTerminal::Factor)]),
            // 28-30. Factor → ( Expr ) | Variable | num
            prod(
                NonTerminal::Factor,
                vec![
                    t(Terminal::LParen),
                    nt(NonTerminal::Expr),
                    t(Terminal::RParen),
                ],
            ),
            prod(NonTerminal::Factor, vec![nt(NonTerminal::Variable)]),
            prod(NonTerminal::Factor, vec![t(Terminal::Num)]),
            // 31-32. BoolExpr → BoolExpr or BoolAnd | BoolAnd
            prod(
                NonTerminal::BoolExpr,
                vec![
                    nt(NonTerminal::BoolExpr),
                    t(Terminal::Or),
                    nt(NonTerminal::BoolAnd),
                ],
            ),
            prod(NonTerminal::BoolExpr, vec![nt(NonTerminal::BoolAnd)]),
            // 33-34. BoolAnd → BoolAnd and BoolNot | BoolNot
            prod(
                NonTerminal::BoolAnd,
                vec![
                    nt(NonTerminal::BoolAnd),
                    t(Terminal::And),
                    nt(NonTerminal::BoolNot),
                ],
            ),
            prod(NonTerminal::BoolAnd, vec![nt(NonTerminal::BoolNot)]),
            // 35-36. BoolNot → not BoolNot | BoolAtom
            prod(
                NonTerminal::BoolNot,
                vec![t(Terminal::Not), nt(NonTerminal::BoolNot)],
            ),
            prod(NonTerminal::BoolNot, vec![nt(NonTerminal::BoolAtom)]),
            // 37-40. BoolAtom → ( BoolExpr ) | true | false | Relation
            prod(
                NonTerminal::BoolAtom,
                vec![
                    t(Terminal::LParen),
                    nt(NonTerminal::BoolExpr),
                    t(Terminal::RParen),
                ],
            ),
            prod(NonTerminal::BoolAtom, vec![t(Terminal::True)]),
            prod(NonTerminal::BoolAtom, vec![t(Terminal::False)]),
            prod(NonTerminal::BoolAtom, vec![nt(NonTerminal::Relation)]),
            // 41-42. Relation → Variable RelOp Expr | num RelOp Expr
            prod(
                NonTerminal::Relation,
                vec![
                    nt(NonTerminal::Variable),
                    nt(NonTerminal::RelOp),
                    nt(NonTerminal::Expr),
                ],
            ),
            prod(
                NonTerminal::Relation,
                vec![
                    t(Terminal::Num),
                    nt(NonTerminal::RelOp),
                    nt(NonTerminal::Expr),
                ],
            ),
            // 43-48. RelOp → == | != | < | <= | > | >=
            prod(NonTerminal::RelOp, vec![t(Terminal::Eq)]),
            prod(NonTerminal::RelOp, vec![t(Terminal::Ne)]),
            prod(NonTerminal::RelOp, vec![t(Terminal::Lt)]),
            prod(NonTerminal::RelOp, vec![t(Terminal::Le)]),
            prod(NonTerminal::RelOp, vec![t(Terminal::Gt)]),
            prod(NonTerminal::RelOp, vec![t(Terminal::Ge)]),
            // 49. Variable → id
            prod(NonTerminal::Variable, vec![t(Terminal::Id)]),
        ];

        Self {
            name: "Simple LR 左递归文法".into(),
            start: NonTerminal::AugmentedStart,
            productions,
        }
    }

    pub fn production(&self, id: usize) -> Option<&Production> {
        self.productions.get(id)
    }

    pub fn productions_for(&self, lhs: NonTerminal) -> impl Iterator<Item = (usize, &Production)> {
        self.productions
            .iter()
            .enumerate()
            .filter(move |(_, p)| p.lhs == lhs)
    }

}

fn t(terminal: Terminal) -> Symbol {
    Symbol::Terminal(terminal)
}

fn nt(non_terminal: NonTerminal) -> Symbol {
    Symbol::NonTerminal(non_terminal)
}

fn prod(lhs: NonTerminal, rhs: Vec<Symbol>) -> Production {
    Production::new(lhs, rhs)
}

impl fmt::Display for Production {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} →", self.lhs)?;

        if self.rhs.is_empty() {
            write!(f, " ε")?;
        } else {
            for sym in &self.rhs {
                write!(f, " {sym}")?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for Grammar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "文法: {}", self.name)?;
        writeln!(f, "开始符号: {}", self.start)?;
        writeln!(f)?;

        for (i, p) in self.productions.iter().enumerate() {
            writeln!(f, "({i}) {p}")?;
        }

        Ok(())
    }
}
