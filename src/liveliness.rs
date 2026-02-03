use std::collections::HashSet;

use crate::{ expr::Expr, stmt::Stmt, value::Value };

pub struct LivelinessOptimiser {
    live_vars: HashSet<String>,
}

impl LivelinessOptimiser {
    pub fn new() -> Self {
        Self { live_vars: HashSet::new() }
    }

    pub fn optimise_tree(&mut self, stmts: Vec<Stmt>) -> Vec<Stmt> {
        let mut result = Vec::new();

        for stmt in stmts.into_iter().rev() {
            if let Some(res_stmt) = self.visit_stmt(stmt) {
                result.push(res_stmt);
            }
        }

        result.reverse();
        return result;
    }

    fn visit_stmt(&mut self, stmt: Stmt) -> Option<Stmt> {
        match stmt {
            Stmt::Assign { name, value } => {
                if !self.live_vars.contains(&name) && !self.has_side_effects(&*value) {
                    return None;
                }

                self.live_vars.remove(&name);

                self.visit_expr(&*value);

                Some(Stmt::Assign { name, value })
            }

            Stmt::Break | Stmt::Continue => Some(stmt),

            Stmt::Decr(name) => {
                self.live_vars.insert(name.clone());
                Some(Stmt::Decr(name))
            }

            Stmt::Expression(expr) => {
                self.visit_expr(&expr);
                Some(Stmt::Expression(expr))
            }

            Stmt::For { initializer, condition, step, body } => {
                let mut changed = true;
                let live_after_loop = self.live_vars.clone();
                let mut loop_in_vars = live_after_loop.clone();

                while changed {
                    let before_count = self.live_vars.len();
                    self.live_vars = loop_in_vars.clone();

                    self.visit_stmt(*step.clone());
                    self.optimise_tree(body.clone());
                    self.visit_expr(&condition);

                    loop_in_vars.extend(self.live_vars.iter().cloned());

                    if loop_in_vars.len() == before_count {
                        changed = false;
                    }
                }

                self.live_vars = loop_in_vars;

                let opt_step = self
                    .visit_stmt(*step)
                    .unwrap_or(Stmt::Expression(Expr::Literal(Value::Null)));

                let opt_body = self.optimise_tree(body);

                self.visit_expr(&condition);

                let opt_init = self
                    .visit_stmt(*initializer)
                    .unwrap_or(Stmt::Expression(Expr::Literal(Value::Null)));

                Some(Stmt::For {
                    initializer: Box::new(opt_init),
                    condition,
                    step: Box::new(opt_step),
                    body: opt_body,
                })
            }

            Stmt::Function { name, params, body } => {
                let outer_live_vars = self.live_vars.clone();

                self.live_vars = HashSet::new();

                let opt_body = self.optimise_tree(body);

                for param in params.iter() {
                    self.live_vars.remove(param);
                }

                self.live_vars.remove(&name);

                if !self.live_vars.is_empty() {
                    panic!("Undefined variables used: {:?}", self.live_vars);
                }

                self.live_vars = outer_live_vars;

                Some(Stmt::Function { name, params, body: opt_body })
            }

            Stmt::If { condition, then_branch, else_branch } => {
                // Basically output the live vars that can live via either branches
                let current_live = self.live_vars.clone();

                let opt_then = self.optimise_tree(then_branch);
                let then_live = self.live_vars.clone();

                self.live_vars = current_live;
                let opt_else = else_branch.map(|b| self.optimise_tree(b));
                let else_live = self.live_vars.clone();

                self.live_vars = then_live.union(&else_live).cloned().collect();

                self.visit_expr(&condition);

                Some(Stmt::If { condition, then_branch: opt_then, else_branch: opt_else })
            }

            Stmt::Incr(name) => {
                self.live_vars.insert(name.clone());
                Some(Stmt::Incr(name))
            }

            Stmt::Print(expr) => {
                self.visit_expr(&expr);
                Some(Stmt::Print(expr))
            }

            Stmt::Return(expr) => {
                if let Some(e) = &expr {
                    self.visit_expr(e);
                }
                Some(Stmt::Return(expr))
            }

            Stmt::Var { name, initializer } => {
                if !self.live_vars.contains(&name) {
                    if let Some(init_expr) = &initializer {
                        if !self.has_side_effects(init_expr) {
                            return None;
                        }
                    } else {
                        return None;
                    }
                }

                self.live_vars.remove(&name);

                if let Some(init_expr) = &initializer {
                    self.visit_expr(init_expr);
                }

                Some(Stmt::Var { name, initializer })
            }

            Stmt::While { condition, body } => {
                let mut changed = true;
                let loop_live_vars = self.live_vars.clone();
                let mut loop_in_vars = loop_live_vars.clone();

                while changed {
                    let before_count = loop_in_vars.len();

                    self.live_vars = loop_in_vars.clone();

                    self.optimise_tree(body.clone());
                    self.visit_expr(&condition);

                    loop_in_vars.extend(self.live_vars.iter().cloned());

                    if loop_in_vars.len() == before_count {
                        changed = false;
                    }
                }

                self.live_vars = loop_in_vars;
                self.visit_expr(&condition);

                Some(Stmt::While { condition, body: self.optimise_tree(body) })
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

            Expr::Grouping(expr) => {
                self.visit_expr(expr);
            }

            Expr::Index { list, index } => {
                self.live_vars.insert(list.clone());
                self.visit_expr(index);
            }

            Expr::List(list) => {
                for item in list {
                    self.visit_expr(item);
                }
            }

            Expr::ListMethodCall { object, arguments, .. } => {
                self.live_vars.insert(object.clone());

                for arg in arguments {
                    self.visit_expr(arg);
                }
            }

            Expr::Literal(_) => {}

            Expr::Logical { left, right, .. } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }

            Expr::Membership { left, right, .. } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }

            Expr::Slice { list, start, end } => {
                self.live_vars.insert(list.clone());
                if let Some(s) = start {
                    self.visit_expr(s);
                }
                if let Some(e) = end {
                    self.visit_expr(e);
                }
            }

            Expr::Unary { right, .. } => {
                self.visit_expr(right);
            }

            Expr::Var(name) => {
                self.live_vars.insert(name.clone());
            }
        }
    }

    fn has_side_effects(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Call { .. } | Expr::ListMethodCall { .. } => true,
            Expr::Binary { left, right, .. } =>
                self.has_side_effects(left) || self.has_side_effects(right),
            Expr::Literal(_) | Expr::Var(_) => false,
            _ => true,
        }
    }
}
