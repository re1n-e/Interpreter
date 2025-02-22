use crate::parse::Expr;
use crate::lexer::Token;
pub enum Stmt {
    exprStmt {
        left: Box<Expr>,
        delimiter: Token,
    },
    printStmt {
        stmt: String,
        eval: Expr,
        delimiter: Token,
    }
}