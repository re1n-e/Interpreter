use crate::parse::Expr;
pub enum Stmt {
    expr(Expr),
    print(Expr),
}

