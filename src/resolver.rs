use std::collections::HashMap;

use crate::evaluate::Evaluate;
use crate::lexer::Token;
use crate::parse::Expr;
use crate::parse::Stmt;

pub struct Resolver {
    interpreter: Evaluate,
    scopes: Vec<HashMap<String, bool>>,
}

impl Resolver {
    pub fn new(interpreter: Evaluate) -> Self {
        Resolver {
            interpreter,
            scopes: Vec::new(),
        }
    }

    fn visit_block_stmt(&mut self, stmt: Stmt) {
        self.begin_scope();
        match stmt {
            Stmt::Block(statements) => self.resolve_stmt(statements),
            _ => (),
        }
        self.end_scope();
    }

    fn visit_expression_stmt(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Expression(expr) => self.resolve_expr(&expr),
            _ => (),
        }
    }

    fn visit_if_stmt(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::If(condition, then_branch, else_branch) =>
            {
                self.resolve_expr(&condition);
                self.resolve_stmt(&then_branch);
            },
            _ => (),
        }
    }

    fn visit_var_stmt(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Var(name, initializer) => {
                self.declare(name.clone());
                self.resolve_expr(&initializer);
                self.define(name);
            }
            _ => (),
        }
    }

    fn visit_assign_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Assign { name, value } => {
                self.resolve_expr(value);
                self.resolve_local(expr, name.clone());
            }
            _ => (),
        }
    }

    fn visit_function_stmt(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Function(name, params, body) => {
                self.declare(name.clone());
                self.define(name.clone());
                self.resolve_function(&params, &body);
            }
            _ => (),
        }
    }

    fn resolve_function(&mut self, params: &Vec<Token>, body: &Vec<Stmt>) {
        self.begin_scope();
        for param in params {
            self.declare(param.clone());
            self.define(param.clone());
        }
        self.resolve_stmt(body.clone());
        self.end_scope();
    }

    fn visit_variable_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Variable { name } => {
                let n = self.scopes.len() - 1;
                if !self.scopes.is_empty() && self.scopes[n].get(&name.lexeme) == Some(&false) {
                    eprintln!("To be done");
                    std::process::exit(70);
                }
            }
            _ => (),
        }
    }

    fn resolve_local(&mut self, expr: &Expr, name: Token) {
        for i in self.scopes.len() - 1..0 {
            if self.scopes[i].contains_key(&name.lexeme) {
                let n = self.scopes.len();
                self.interpreter.resolve(expr, n - 1 - i);
            }
        }
    }

    fn define(&mut self, name: Token) {
        if self.scopes.is_empty() {
            return;
        }
        let n = self.scopes.len() - 1;
        self.scopes[n].insert(name.lexeme, true);
    }

    fn declare(&mut self, name: Token) {
        if self.scopes.is_empty() {
            return;
        }
        let n = self.scopes.len() - 1;
        self.scopes[n].insert(name.lexeme, false);
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn resolve_stmt(&mut self, statements: Vec<Stmt>) {
        self.interpreter.define_globals();
        for stmt in statements {
            let _ = self.interpreter.execute(stmt, false);
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        let _ = self.interpreter.visit_expression_stmt(&expr);
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }
}
