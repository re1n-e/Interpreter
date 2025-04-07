use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    evaluate::Evaluate,
    lexer::Token,
    parse::{Expr, Stmt},
};

pub struct Resolver {
    evaluate: Rc<RefCell<Evaluate>>,
    scopes: Vec<HashMap<String, bool>>,
}

impl Resolver {
    pub fn new(evaluate: Rc<RefCell<Evaluate>>) -> Self {
        Resolver {
            evaluate,
            scopes: Vec::new(),
        }
    }

    fn visit_block_stmt(&mut self, stmt: &Stmt) {
        self.begin_scope();
        match stmt {
            Stmt::Block(statements) => (),
            _ => (),
        }
        self.end_scope();
    }

    fn visit_expression_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expression(expr) => self.resolve_single_expr(expr),
            _ => (),
        }
    }

    fn visit_function_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Function(name, _, _) => {
                self.declare(name);
                self.define(name);
            }
            _ => (),
        }
        self.resolve_function(stmt);
    }

    fn visit_if_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::If(condition, then_branch, else_branch) => {
                self.resolve_single_expr(condition);
                self.resolve_single_stmt(then_branch);
                let else_branch = else_branch.clone();
                if let Some(stmt) = *else_branch {
                    self.resolve_single_stmt(&stmt);
                }
            }
            _ => (),
        }
    }

    fn visit_print_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Print(expression) => self.resolve_single_expr(expression),
            _ => (),
        }
    }

    fn visit_return_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Return(_, value) => {
                if !matches!(value, &Expr::Null) {
                    self.resolve_single_expr(value);
                }
            }
            _ => (),
        }
    }

    fn visit_var_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Var(name, initializer) => {
                self.declare(name);
                if !matches!(initializer, &Expr::Null) {
                    self.resolve_single_expr(initializer);
                }
                self.define(name);
            }
            _ => (),
        }
    }

    fn visit_while_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::While(condition, body) => {
                self.resolve_single_expr(condition);
                self.resolve_single_stmt(body);
            }
            _ => (),
        }
    }

    fn visit_assign_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Assign { name, value } => {
                self.resolve_single_expr(value);
                self.resolve_local(expr, name);
            }
            _ => (),
        }
    }

    fn visit_binary_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let _ = operator;
                self.resolve_single_expr(left);
                self.resolve_single_expr(right);
            }
            _ => (),
        }
    }

    fn visit_call_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let _ = paren;
                self.resolve_single_expr(&callee);
                for args in arguments {
                    self.resolve_single_expr(args);
                }
            }
            _ => (),
        }
    }

    fn visit_grouping_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Grouping { expression } => self.resolve_single_expr(&expression),
            _ => (),
        }
    }

    fn visit_literal_expr(&mut self, _expr: &Expr) {}

    fn visit_logical_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let _ = operator;
                self.resolve_single_expr(left);
                self.resolve_single_expr(right);
            }
            _ => (),
        }
    }

    fn visit_unary_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Unary { operator, right } => {
                let _ = operator;
                self.resolve_single_expr(right);
            }
            _ => (),
        }
    }

    fn visit_variable_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Variable { name } => {
                if !self.scopes.is_empty()
                    && self.scopes.last().unwrap().get(&name.lexeme) == Some(&false)
                {
                    eprintln!("Can't read local variable in its own initializer.");
                    std::process::exit(70);
                }
                self.resolve_local(expr, name);
            }
            _ => (),
        }
    }

    fn resolve(&mut self, stmts: &Vec<Stmt>) {
        for stmt in stmts {
            self.resolve_single_stmt(stmt);
        }
    }

    fn resolve_single_stmt(&mut self, stmt: &Stmt) {}

    fn resolve_single_expr(&mut self, expr: &Expr) {}

    fn resolve_function(&mut self, stmt: &Stmt) {
        self.begin_scope();
        match stmt {
            Stmt::Function(_, params, body) => {
                for param in params {
                    self.declare(param);
                    self.define(param);
                }
                self.resolve(body);
            }
            _ => (),
        }
        self.end_scope();
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), false);
        }
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), true);
        }
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        for i in self.scopes.len() - 1..0 {
            if self.scopes.get(i).unwrap().contains_key(&name.lexeme) {
                self.evaluate
                    .borrow_mut()
                    .resolve(expr, self.scopes.len() - 1);
            }
        }
    }
}
