use std::collections::HashMap;

use crate::{ expr::Expr, stmt::Stmt };

enum ReturnStatus {
    Returns,
    Continues,
}

pub struct Semantics {
    sts: Vec<HashMap<String, bool>>, // Symbol table stack
    loop_depth: usize,
}

impl Semantics {
    pub fn new() -> Self {
        return Self {
            sts: vec![HashMap::<String, bool>::new()],
            loop_depth: 0,
        };
    }

    pub fn run(&mut self, ast: &Vec<Stmt>) {
        for stmt in ast {
            self.visit_stmt(stmt);
        }
    }

    fn is_defined(&self, name: &str) -> bool {
        for scope in self.sts.iter().rev() {
            if scope.contains_key(name) {
                return true;
            }
        }
        return false;
    }

    fn enter_scope(&mut self) {
        self.sts.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.sts.pop().expect("Tried to exit global scope");
    }

    fn visit_block(&mut self, body: &Vec<Stmt>) -> ReturnStatus {
        for stmt in body {
            let status = self.visit_stmt(stmt);
            if let ReturnStatus::Returns = status {
                let idx = body
                    .iter()
                    .position(|s| s == stmt)
                    .unwrap();
                if idx + 1 < body.len() {
                    panic!("Unreachable code after return");
                }
                return ReturnStatus::Returns;
            }
        }
        return ReturnStatus::Continues;
    }

    fn visit_stmt(&mut self, stmt: &Stmt) -> ReturnStatus {
        match stmt {
            Stmt::Assign { name, value } => {
                self.visit_expr(*&value);
                if !self.is_defined(&name) {
                    panic!("Cannot assign value to non-existent variable");
                }
                ReturnStatus::Continues
            }
            Stmt::Break => {
                if self.loop_depth == 0 {
                    panic!("Break used outside of a loop");
                }
                ReturnStatus::Returns
            }
            Stmt::Continue => {
                if self.loop_depth == 0 {
                    panic!("Continue used outside of a loop");
                }
                ReturnStatus::Returns
            }
            Stmt::Decr { name } => {
                if !self.is_defined(&name) {
                    panic!("Undefined variable being decremented");
                }
                ReturnStatus::Continues
            }
            Stmt::Expression { expression } => {
                self.visit_expr(expression);
                ReturnStatus::Continues
            }
            Stmt::For { initializer, condition, step, body } => {
                self.visit_stmt(*&initializer);
                self.visit_expr(condition);
                self.visit_stmt(*&step);

                self.loop_depth += 1;
                self.enter_scope();
                let status = self.visit_block(body);
                self.exit_scope();
                self.loop_depth -= 1;
                status
            }
            Stmt::Function { name, params, body } => {
                let scope = self.sts.last_mut().expect("No scope available");
                if scope.contains_key(name) {
                    panic!("Function redefined");
                }
                scope.insert(name.clone(), true);

                self.enter_scope();
                for param in params {
                    let scope = self.sts.last_mut().expect("No scope available");
                    if scope.contains_key(param) {
                        panic!("Parameter redeclared");
                    }
                    scope.insert(param.clone(), true);
                }

                let status = self.visit_block(body);
                self.exit_scope();

                status
            }
            Stmt::If { condition, then_branch, else_branch } => {
                self.visit_expr(condition);

                self.enter_scope();
                let then_status = self.visit_block(then_branch);
                self.exit_scope();

                let else_status = if let Some(branch) = else_branch {
                    self.enter_scope();
                    let status = self.visit_block(branch);
                    self.exit_scope();
                    status
                } else {
                    ReturnStatus::Continues
                };

                match (then_status, else_status) {
                    (ReturnStatus::Returns, ReturnStatus::Returns) => ReturnStatus::Returns,
                    _ => ReturnStatus::Continues,
                }
            }
            Stmt::Incr { name } => {
                if !self.is_defined(&name) {
                    panic!("Undefined variable being incremented");
                }
                ReturnStatus::Continues
            }
            Stmt::Print { expression } => {
                self.visit_expr(expression);
                ReturnStatus::Continues
            }
            Stmt::Return { value } => {
                if let Some(expr) = value {
                    self.visit_expr(expr);
                }
                ReturnStatus::Returns
            }
            Stmt::Var { name, initializer } => {
                if let Some(e) = initializer {
                    self.visit_expr(e);
                }
                let scope = self.sts.last_mut().expect("No scope available");
                if scope.contains_key(name) {
                    panic!("Variable redefined in the same scope");
                }
                scope.insert(name.clone(), true);
                ReturnStatus::Continues
            }
            Stmt::While { condition, body } => {
                self.visit_expr(condition);

                self.loop_depth += 1;
                self.enter_scope();
                let status = self.visit_block(body);
                self.exit_scope();
                self.loop_depth -= 1;
                status
            }
        }
    }
    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Binary { left, right, .. } => {
                self.visit_expr(*&left);
                self.visit_expr(*&right);
            }
            Expr::Call { callee, arguments } => {
                self.visit_expr(*&callee);
                for arg in arguments {
                    self.visit_expr(arg);
                }
            }
            Expr::Grouping { expression } => self.visit_expr(*&expression),
            Expr::Index { list, index } => {
                self.visit_expr(*&index);
                if !self.is_defined(&list) {
                    panic!("Undefined list being indexed")
                }
            }
            Expr::List { items } => {
                for item in items {
                    self.visit_expr(item);
                }
            }
            Expr::ListMethodCall { object, call } => {
                self.visit_expr(*&call);
                if !self.is_defined(&object) {
                    panic!("Method called on undefined list")
                }
            }
            Expr::Literal { .. } => {}
            Expr::Logical { left, right, .. } => {
                self.visit_expr(*&left);
                self.visit_expr(*&right);
            }
            Expr::Membership { left, right, .. } => {
                self.visit_expr(*&left);
                self.visit_expr(*&right);
            }
            Expr::Slice { list, start, end } => {
                if let Some(e) = start {
                    self.visit_expr(*&e);
                }
                if let Some(e) = end {
                    self.visit_expr(*&e);
                }

                if !self.is_defined(&list) {
                    panic!("Undefined list being sliced")
                }
            }
            Expr::Unary { right, .. } => self.visit_expr(*&right),
            Expr::Var { name } => {
                if !self.is_defined(&name) {
                    println!("{name}");
                    panic!("Undefined variable in expression")
                }
            }
        }
    }
}
