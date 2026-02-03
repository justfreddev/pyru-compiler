use std::collections::HashMap;

use crate::{ expr::Expr, stmt::Stmt, value::Value };

enum ReturnStatus {
    Returns,
    Continues,
}

#[derive(Clone)]
pub struct SymbolInfo {
    declared: bool,
    assigned: bool,
    pub constant: Option<Value>,
}

pub struct SemanticAnalyser {
    sts: Vec<HashMap<String, SymbolInfo>>, // Symbol table stack
    loop_depth: usize,
}

impl SemanticAnalyser {
    pub fn new() -> Self {
        return Self {
            sts: vec![HashMap::<String, SymbolInfo>::new()],
            loop_depth: 0,
        };
    }

    pub fn run(&mut self, ast: &Vec<Stmt>) {
        for stmt in ast {
            self.visit_stmt(stmt);
        }
    }

    fn is_declared(&self, name: &str) -> bool {
        for scope in self.sts.iter().rev() {
            if let Some(SymbolInfo { declared, .. }) = scope.get(name) {
                if *declared {
                    return true;
                }
            }
        }
        return false;
    }

    fn is_assigned(&self, name: &str) -> bool {
        for scope in self.sts.iter().rev() {
            if let Some(SymbolInfo { assigned, .. }) = scope.get(name) {
                if *assigned {
                    return true;
                }
            }
        }
        return false;
    }

    fn assign(&mut self, name: &str, value: Option<Value>) {
        for scope in self.sts.iter_mut().rev() {
            if let Some(info) = scope.get_mut(name) {
                info.assigned = true;
                info.constant = value;
                return;
            }
        }
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
            if let (_, ReturnStatus::Returns) = status {
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

    fn visit_stmt(&mut self, stmt: &Stmt) -> (Stmt, ReturnStatus) {
        match stmt {
            Stmt::Assign { name, value } => {
                if !self.is_declared(name) {
                    panic!("Cannot assign value to non-existent variable");
                }
                self.assign(name, None);
                (Stmt::Assign { name: name.clone(), value: value.clone() }, ReturnStatus::Continues)
            }

            Stmt::Decr(name) => {
                if !(self.is_declared(name) && self.is_assigned(name)) {
                    panic!("Undefined variable being decremented");
                }
                (Stmt::Decr(name.clone()), ReturnStatus::Continues)
            }

            Stmt::Expression(expr) => {
                self.visit_expr(expr);
                (Stmt::Expression(expr.clone()), ReturnStatus::Continues)
            }

            Stmt::For { initializer, condition, step, body } => {
                self.visit_stmt(&*initializer);
                let before_loop = self.sts.clone();
                self.loop_depth += 1;
                {
                    self.sts = before_loop.clone();
                    self.enter_scope();
                    self.visit_expr(condition);
                    self.visit_block(body);
                    self.visit_stmt(&*step);
                    self.exit_scope();
                }
                self.loop_depth -= 1;
                self.sts = before_loop;
                (
                    Stmt::For {
                        initializer: initializer.clone(),
                        condition: condition.clone(),
                        step: step.clone(),
                        body: body.clone(),
                    },
                    ReturnStatus::Continues,
                )
            }

            Stmt::Function { name, params, body } => {
                let scope = self.sts.last_mut().expect("No scope available");
                if scope.contains_key(name) {
                    panic!("Function redefined");
                }
                scope.insert(name.clone(), SymbolInfo {
                    declared: true,
                    assigned: true,
                    constant: None,
                });

                self.enter_scope();
                for param in params {
                    let scope = self.sts.last_mut().expect("No scope available");
                    if scope.contains_key(param) {
                        panic!("Parameter redeclared");
                    }
                    scope.insert(param.clone(), SymbolInfo {
                        declared: true,
                        assigned: true,
                        constant: None,
                    });
                }
                let status = self.visit_block(body);
                self.exit_scope();

                (
                    Stmt::Function {
                        name: name.clone(),
                        params: params.clone(),
                        body: body.clone(),
                    },
                    status,
                )
            }

            Stmt::If { condition, then_branch, else_branch } => {
                self.visit_expr(condition);
                let before_if = self.sts.clone();

                let then_status;
                let then_state;
                {
                    self.sts = before_if.clone();
                    self.enter_scope();
                    then_status = self.visit_block(then_branch);
                    self.exit_scope();
                    then_state = self.sts.clone();
                }

                let else_state;
                let else_status = if let Some(branch) = else_branch {
                    self.sts = before_if.clone();
                    self.enter_scope();
                    let status = self.visit_block(branch);
                    self.exit_scope();
                    else_state = self.sts.clone();
                    status
                } else {
                    else_state = before_if.clone();
                    ReturnStatus::Continues
                };

                self.sts = before_if;
                for (i, scope) in self.sts.iter_mut().enumerate() {
                    let then_scope = &then_state[i];
                    let else_scope = &else_state[i];
                    for (name, sym) in scope.iter_mut() {
                        sym.assigned =
                            then_scope.get(name).map_or(false, |s| s.assigned) &&
                            else_scope.get(name).map_or(false, |s| s.assigned);
                    }
                }

                let status = match (then_status, else_status) {
                    (ReturnStatus::Returns, ReturnStatus::Returns) => ReturnStatus::Returns,
                    _ => ReturnStatus::Continues,
                };

                (
                    Stmt::If {
                        condition: condition.clone(),
                        then_branch: then_branch.clone(),
                        else_branch: else_branch.clone(),
                    },
                    status,
                )
            }

            Stmt::Incr(name) => {
                if !(self.is_declared(name) && self.is_assigned(name)) {
                    panic!("Undefined variable being incremented");
                }
                (Stmt::Incr(name.clone()), ReturnStatus::Continues)
            }

            Stmt::Print(expr) => {
                self.visit_expr(expr);
                (Stmt::Print(expr.clone()), ReturnStatus::Continues)
            }

            Stmt::Return(value) => {
                if let Some(expr) = value {
                    self.visit_expr(expr);
                    (Stmt::Return(Some(expr.clone())), ReturnStatus::Returns)
                } else {
                    (Stmt::Return(None), ReturnStatus::Returns)
                }
            }

            Stmt::Var { name, initializer } => {
                let assigned = initializer.is_some();
                let scope = self.sts.last_mut().expect("No scope available");
                if scope.contains_key(name) {
                    panic!("Variable redefined in the same scope");
                }
                scope.insert(name.clone(), SymbolInfo { declared: true, assigned, constant: None });
                (
                    Stmt::Var { name: name.clone(), initializer: initializer.clone() },
                    ReturnStatus::Continues,
                )
            }

            Stmt::While { condition, body } => {
                self.visit_expr(condition);
                let before_loop = self.sts.clone();
                self.loop_depth += 1;
                {
                    self.sts = before_loop.clone();
                    self.enter_scope();
                    self.visit_block(body);
                    self.exit_scope();
                }
                self.loop_depth -= 1;
                self.sts = before_loop;
                (
                    Stmt::While { condition: condition.clone(), body: body.clone() },
                    ReturnStatus::Continues,
                )
            }

            Stmt::Break => {
                if self.loop_depth == 0 {
                    panic!("Break used outside of a loop");
                }
                (Stmt::Break, ReturnStatus::Returns)
            }

            Stmt::Continue => {
                if self.loop_depth == 0 {
                    panic!("Continue used outside of a loop");
                }
                (Stmt::Continue, ReturnStatus::Returns)
            }
        }
    }
    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Binary { left, right, .. } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }

            Expr::Call { callee, arguments } => {
                self.visit_expr(callee);
                for arg in arguments {
                    self.visit_expr(arg);
                }
            }

            Expr::Grouping(e) => self.visit_expr(e),

            Expr::Index { list, index } => {
                self.visit_expr(index);
                if !self.is_assigned(list) {
                    panic!("Unassigned list being indexed");
                }
            }

            Expr::List(items) => {
                for item in items {
                    self.visit_expr(item);
                }
            }

            Expr::ListMethodCall { object, method_name, arguments } => {
                for arg in arguments {
                    self.visit_expr(arg);
                }
                const LIST_METHODS: &[&str] = &[
                    "index",
                    "insertAt",
                    "len",
                    "pop",
                    "push",
                    "remove",
                    "sort",
                ];
                if !LIST_METHODS.contains(&method_name.as_str()) {
                    panic!("Unknown list method: {}", method_name);
                }
                if !self.is_assigned(object) {
                    panic!("Method called on unassigned list");
                }
            }

            Expr::Literal { .. } => {}

            Expr::Logical { left, right, .. } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }

            Expr::Membership { left, right, .. } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }

            Expr::Slice { list, start, end } => {
                if let Some(e) = start {
                    self.visit_expr(e);
                }
                if let Some(e) = end {
                    self.visit_expr(e);
                }
                if !self.is_assigned(list) {
                    panic!("Undefined list being sliced");
                }
            }

            Expr::Unary { right, .. } => self.visit_expr(right),

            Expr::Var(name) => {
                if !self.is_assigned(name) {
                    panic!("Unassigned variable in expression");
                }
            }
        }
    }
}
